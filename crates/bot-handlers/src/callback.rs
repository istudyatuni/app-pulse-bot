use anyhow::Result;

use crate::keyboards::LanguagesKeyboardKind;
use crate::PayloadData;

use db::models::ShouldNotify;

// flags is at the start of message: {flag}:{payload}
const NOTIFY_FLAG: &str = "notify";
const SET_LANG_FLAG: &str = "lang";

// payload tokens: {notify-flag}:{app-id}:{token}
const IGNORE_TOKEN: &str = "ignore";
const NOTIFY_TOKEN: &str = "notify";

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum Callback {
    Notify {
        app_id: String,
        should_notify: ShouldNotify,
    },
    SetLang {
        lang: String,
        kind: LanguagesKeyboardKind,
    },
}

impl PayloadData for Callback {
    type Error = CallbackParseError;

    fn to_payload(&self) -> String {
        match self {
            Self::Notify {
                app_id,
                should_notify,
            } => format!("{NOTIFY_FLAG}:{app_id}:{}", should_notify.to_payload()),
            Self::SetLang { lang, kind } => {
                format!("{SET_LANG_FLAG}:{}:{lang}", kind.to_payload())
            }
        }
    }
    fn try_from_payload(value: &str) -> Result<Self, Self::Error> {
        let data: Vec<_> = value.split(':').collect();
        let res = match data[0] {
            NOTIFY_FLAG => {
                if data.len() < 3 {
                    return Err(CallbackParseError::InvalidCallback);
                }

                let (app_id, should_notify) = if data.len() == 3 {
                    (data[1].to_string(), data[2])
                } else {
                    // if data[1] contains ':'
                    (data[1..data.len() - 1].join(":"), data[data.len() - 1])
                };

                Callback::Notify {
                    app_id,
                    should_notify: ShouldNotify::try_from_payload(should_notify)?,
                }
            }
            SET_LANG_FLAG => {
                if data.len() != 3 {
                    return Err(CallbackParseError::InvalidCallback);
                }

                let (kind, lang) = (
                    LanguagesKeyboardKind::try_from_payload(data[1]).ok(),
                    data[2].to_string(),
                );
                let Some(kind) = kind else {
                    return Err(CallbackParseError::InvalidToken);
                };
                Callback::SetLang { lang, kind }
            }
            _ => return Err(CallbackParseError::UnknownCallbackType),
        };
        Ok(res)
    }
}

impl TryFrom<&str> for Callback {
    type Error = CallbackParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from_payload(value)
    }
}

impl TryFrom<&String> for Callback {
    type Error = CallbackParseError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from_payload(value.as_str())
    }
}

impl PayloadData for ShouldNotify {
    type Error = CallbackParseError;

    fn to_payload(&self) -> String {
        match self {
            Self::Notify => NOTIFY_TOKEN.to_string(),
            Self::Ignore => IGNORE_TOKEN.to_string(),
        }
    }

    fn try_from_payload(payload: &str) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        match payload {
            NOTIFY_TOKEN => Ok(Self::Notify),
            IGNORE_TOKEN => Ok(Self::Ignore),
            _ => Err(CallbackParseError::InvalidToken),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum CallbackParseError {
    InvalidCallback,
    InvalidToken,
    UnknownCallbackType,
}

impl Callback {
    pub(crate) fn notify(app_id: &str, should_notify: ShouldNotify) -> Self {
        Self::Notify {
            app_id: app_id.to_string(),
            should_notify,
        }
    }
    pub(crate) fn set_lang(kind: LanguagesKeyboardKind, lang: &str) -> Self {
        Self::SetLang {
            lang: lang.to_string(),
            kind,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_from_str() {
        let app_id = "some-app";
        let strange_app_id = "some-app:name";
        let table = vec![
            (
                format!("{NOTIFY_FLAG}:{app_id}:{NOTIFY_TOKEN}"),
                Ok(Callback::notify(app_id, ShouldNotify::Notify)),
            ),
            (
                format!("{NOTIFY_FLAG}:{app_id}:{IGNORE_TOKEN}"),
                Ok(Callback::notify(app_id, ShouldNotify::Ignore)),
            ),
            (
                format!("{NOTIFY_FLAG}:{strange_app_id}:{IGNORE_TOKEN}"),
                Ok(Callback::notify(strange_app_id, ShouldNotify::Ignore)),
            ),
            (
                format!("{SET_LANG_FLAG}:start:en"),
                Ok(Callback::set_lang(LanguagesKeyboardKind::Start, "en")),
            ),
            (
                format!("{SET_LANG_FLAG}:settings:en"),
                Ok(Callback::set_lang(LanguagesKeyboardKind::Settings, "en")),
            ),
            (
                format!("{SET_LANG_FLAG}:starta:en"),
                Err(CallbackParseError::InvalidToken),
            ),
            (
                format!("{NOTIFY_FLAG}:{app_id}:asdf"),
                Err(CallbackParseError::InvalidToken),
            ),
            (
                format!("{NOTIFY_FLAG}:{app_id}"),
                Err(CallbackParseError::InvalidCallback),
            ),
            (
                format!("{NOTIFY_FLAG}:{strange_app_id}"),
                Err(CallbackParseError::InvalidToken),
            ),
        ];
        for (input, expected) in table {
            assert_eq!(Callback::try_from(&input), expected);
        }
    }
}
