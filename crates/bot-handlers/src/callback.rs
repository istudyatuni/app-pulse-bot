use anyhow::Result;

use crate::keyboards::LanguagesKeyboardToken;
use crate::{IGNORE_TOKEN, NOTIFY_FLAG, NOTIFY_TOKEN, SET_LANG_FLAG};

use db::models::ShouldNotify;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum Callback {
    Notify {
        app_id: String,
        should_notify: ShouldNotify,
    },
    SetLang {
        lang: String,
        token: LanguagesKeyboardToken,
    },
}

impl TryFrom<&str> for Callback {
    type Error = CallbackParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
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

                let should_notify = match should_notify {
                    NOTIFY_TOKEN => ShouldNotify::Notify,
                    IGNORE_TOKEN => ShouldNotify::Ignore,
                    _ => {
                        return Err(CallbackParseError::InvalidToken);
                    }
                };
                Callback::Notify {
                    app_id,
                    should_notify,
                }
            }
            SET_LANG_FLAG => {
                if data.len() != 3 {
                    return Err(CallbackParseError::InvalidCallback);
                }

                let (token, lang) = (data[1].try_into().ok(), data[2].to_string());
                let Some(token) = token else {
                    return Err(CallbackParseError::InvalidToken);
                };
                Callback::SetLang { lang, token }
            }
            _ => return Err(CallbackParseError::UnknownCallbackType),
        };
        Ok(res)
    }
}

impl TryFrom<&String> for Callback {
    type Error = CallbackParseError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum CallbackParseError {
    InvalidCallback,
    InvalidToken,
    UnknownCallbackType,
}

#[cfg(test)]
impl Callback {
    fn notify(app_id: &str, should_notify: ShouldNotify) -> Self {
        Self::Notify {
            app_id: app_id.to_string(),
            should_notify,
        }
    }
    fn set_lang(token: LanguagesKeyboardToken, lang: &str) -> Self {
        Self::SetLang {
            lang: lang.to_string(),
            token,
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
                Ok(Callback::set_lang(LanguagesKeyboardToken::Start, "en")),
            ),
            (
                format!("{SET_LANG_FLAG}:settings:en"),
                Ok(Callback::set_lang(LanguagesKeyboardToken::Settings, "en")),
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
