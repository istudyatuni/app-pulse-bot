use crate::{
    keyboards::LanguagesKeyboardToken, IGNORE_TOKEN, NOTIFY_FLAG, NOTIFY_TOKEN, SET_LANG_FLAG,
};

use db::models::ShouldNotify;

#[derive(Debug)]
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
                    (data[1..data.len()].join(":"), data[data.len() - 1])
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
            _ => {
                return Err(CallbackParseError::UnknowkCallbackType);
            }
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

pub(crate) enum CallbackParseError {
    InvalidCallback,
    InvalidToken,
    UnknowkCallbackType,
}
