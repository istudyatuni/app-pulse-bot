use anyhow::Result;

use db::models::ShouldNotify;

use crate::{
    keyboards::LanguagesKeyboardKind, PayloadData, PayloadLayout, PayloadParseError,
    CALLBACK_VERSION,
};

// flags is at the start of message: {flag}:{payload}
const NOTIFY_FLAG: &str = "notify";
const SET_LANG_FLAG: &str = "lang";

// payload tokens: {notify-flag}:{app-id}:{token}
const IGNORE_TOKEN: &str = "ignore";
const NOTIFY_TOKEN: &str = "notify";

const NOTIFY_CALLBACK_LAYOUT: PayloadLayout = PayloadLayout::new(3, Some(1));
const SETLANG_CALLBACK_LAYOUT: PayloadLayout = PayloadLayout::new(3, None);

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

    // todo: cache
    fn to_payload(&self) -> String {
        match self {
            Self::Notify {
                app_id,
                should_notify,
            } => NOTIFY_CALLBACK_LAYOUT
                .make_payload(vec![
                    NOTIFY_FLAG,
                    app_id,
                    should_notify.to_payload().as_str(),
                ])
                .inspect_err(|e| log::error!("invalid notify callback is created: {e}"))
                .unwrap_or_default(),
            Self::SetLang { lang, kind } => SETLANG_CALLBACK_LAYOUT
                .make_payload(vec![SET_LANG_FLAG, kind.to_payload().as_str(), lang])
                .inspect_err(|e| log::error!("invalid notify callback is created: {e}"))
                .unwrap_or_default(),
        }
    }
    fn try_from_payload(value: &str) -> Result<Self, Self::Error> {
        let data: Vec<_> = value.split(':').collect();
        if data.is_empty() {
            return Err(CallbackParseError::InvalidCallback);
        }

        // handle old callbacks, without version
        let version: Option<u8> = data[0].parse().ok();
        if version.is_none_or(|v| v < CALLBACK_VERSION) {
            return Err(CallbackParseError::OutdatedCallback);
        }

        // todo: probably do not hardcode this?
        let res = match data[1] {
            NOTIFY_FLAG => {
                let data = NOTIFY_CALLBACK_LAYOUT.parse_payload(value)?;
                Callback::Notify {
                    app_id: data[1].clone(),
                    should_notify: ShouldNotify::try_from_payload(&data[2])?,
                }
            }
            SET_LANG_FLAG => {
                let data = SETLANG_CALLBACK_LAYOUT.parse_payload(value)?;
                let Some(kind) = LanguagesKeyboardKind::try_from_payload(&data[1]).ok() else {
                    return Err(CallbackParseError::InvalidToken);
                };
                Callback::SetLang {
                    lang: data[2].to_string(),
                    kind,
                }
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
    OutdatedCallback,
    UnknownCallbackType,
}

impl From<PayloadParseError> for CallbackParseError {
    fn from(value: PayloadParseError) -> Self {
        match value {
            PayloadParseError::InvalidSize => Self::InvalidCallback,
        }
    }
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
        const V: u8 = CALLBACK_VERSION;
        let app_id = "some-app";
        let strange_app_id = "some-app:name";

        let table = vec![
            (
                format!("{V}:{NOTIFY_FLAG}:{app_id}:{NOTIFY_TOKEN}"),
                Ok(Callback::notify(app_id, ShouldNotify::Notify)),
            ),
            (
                format!("{V}:{NOTIFY_FLAG}:{app_id}:{IGNORE_TOKEN}"),
                Ok(Callback::notify(app_id, ShouldNotify::Ignore)),
            ),
            (
                format!("{V}:{NOTIFY_FLAG}:{strange_app_id}:{IGNORE_TOKEN}"),
                Ok(Callback::notify(strange_app_id, ShouldNotify::Ignore)),
            ),
            (
                format!("{V}:{SET_LANG_FLAG}:start:en"),
                Ok(Callback::set_lang(LanguagesKeyboardKind::Start, "en")),
            ),
            (
                format!("{V}:{SET_LANG_FLAG}:settings:en"),
                Ok(Callback::set_lang(LanguagesKeyboardKind::Settings, "en")),
            ),
            (
                format!("{V}:{SET_LANG_FLAG}:starta:en"),
                Err(CallbackParseError::InvalidToken),
            ),
            (
                format!("{V}:{NOTIFY_FLAG}:{app_id}:asdf"),
                Err(CallbackParseError::InvalidToken),
            ),
            (
                format!("{V}:{NOTIFY_FLAG}:{app_id}"),
                Err(CallbackParseError::InvalidCallback),
            ),
            (
                format!("{V}:{NOTIFY_FLAG}:{strange_app_id}"),
                Err(CallbackParseError::InvalidToken),
            ),
            (
                format!("{NOTIFY_FLAG}:{app_id}:{NOTIFY_TOKEN}"),
                Err(CallbackParseError::OutdatedCallback),
            ),
        ];
        for (input, expected) in table {
            let res = Callback::try_from(&input);
            assert_eq!(res, expected, "Callback::try_from({input})");
        }
    }
}
