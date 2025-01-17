use anyhow::Result;

use common::types::{AppId, SourceId};
use db::models::ShouldNotify;

use crate::{
    keyboards::{ChangeSubscribeAction, LanguagesKeyboardKind},
    PayloadData, PayloadLayout, PayloadParseError, CALLBACK_VERSION,
};

// flags is at the start of message: {flag}:{payload}
const NOTIFY_FLAG: &str = "notify";
const SHOW_SOURCE_FLAG: &str = "source";
const CHANGE_SUBSCRIBE_FLAG: &str = "sub";
const SET_LANG_FLAG: &str = "lang";

// payload tokens: {notify-flag}:{app-id}:{token}
const IGNORE_TOKEN: &str = "ignore";
const NOTIFY_TOKEN: &str = "notify";

const NOTIFY_CALLBACK_LAYOUT: PayloadLayout = PayloadLayout::new(4, None);
const SHOW_SOURCE_CALLBACK_LAYOUT: PayloadLayout = PayloadLayout::new(2, None);
const CHANGE_SUBSCRIBE_CALLBACK_LAYOUT: PayloadLayout = PayloadLayout::new(3, None);
const SETLANG_CALLBACK_LAYOUT: PayloadLayout = PayloadLayout::new(3, None);

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub(crate) enum Callback {
    /// Notify about app updates
    Notify {
        source_id: SourceId,
        app_id: AppId,
        should_notify: ShouldNotify,
    },
    /// Show information about source and button to subscribe/unsubscribe
    ShowSource { source_id: SourceId },
    /// Subscribe/unsubscribe to source
    ChangeSubscribe {
        source_id: SourceId,
        action: ChangeSubscribeAction,
    },
    /// Change user language
    SetLang { lang: String, kind: LanguagesKeyboardKind },
}

impl PayloadData for Callback {
    type Error = CallbackParseError;

    // todo: cache
    fn to_payload(&self) -> String {
        match self {
            Self::Notify {
                source_id,
                app_id,
                should_notify,
            } => NOTIFY_CALLBACK_LAYOUT
                .make_payload(vec![
                    NOTIFY_FLAG,
                    &source_id.to_string(),
                    &app_id.to_string(),
                    should_notify.to_payload().as_str(),
                ])
                .inspect_err(|e| log::error!("invalid notify callback is created: {e}"))
                .unwrap_or_default(),
            Callback::ShowSource { source_id } => SHOW_SOURCE_CALLBACK_LAYOUT
                .make_payload(vec![SHOW_SOURCE_FLAG, &source_id.to_string()])
                .inspect_err(|e| log::error!("invalid show_source callback is created: {e}"))
                .unwrap_or_default(),
            Callback::ChangeSubscribe { source_id, action } => CHANGE_SUBSCRIBE_CALLBACK_LAYOUT
                .make_payload(vec![
                    CHANGE_SUBSCRIBE_FLAG,
                    action.to_payload().as_str(),
                    &source_id.to_string(),
                ])
                .inspect_err(|e| log::error!("invalid change_subscribe callback is created: {e}"))
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
                let [source_id, app_id, should_notify] = &data[1..=3] else {
                    log::error!("invalid number of elements in notify callback");
                    return Err(CallbackParseError::InvalidCallback);
                };
                Callback::Notify {
                    source_id: source_id
                        .parse()
                        .inspect_err(|e| log::error!("failed to parse source_id in notify callback: {e}"))
                        .map_err(|_| CallbackParseError::InvalidCallback)?,
                    app_id: app_id
                        .parse()
                        .inspect_err(|e| log::error!("failed to parse app_id in notify callback: {e}"))
                        .map_err(|_| CallbackParseError::InvalidCallback)?,
                    should_notify: ShouldNotify::try_from_payload(should_notify)
                        .inspect_err(|_| log::error!("failed to parse should_notify"))?,
                }
            },
            SHOW_SOURCE_FLAG => {
                let data = SHOW_SOURCE_CALLBACK_LAYOUT.parse_payload(value)?;
                let [source_id] = &data[1..=1] else {
                    log::error!("invalid number of elements in show_source callback");
                    return Err(CallbackParseError::InvalidCallback);
                };
                Callback::ShowSource {
                    source_id: source_id
                        .parse()
                        .inspect_err(|e| log::error!("failed to parse source_id in show_source callback: {e}"))
                        .map_err(|_| CallbackParseError::InvalidCallback)?,
                }
            },
            CHANGE_SUBSCRIBE_FLAG => {
                let data = CHANGE_SUBSCRIBE_CALLBACK_LAYOUT.parse_payload(value)?;
                let [action, source_id] = &data[1..=2] else {
                    log::error!("invalid number of elements in change_subscribe callback");
                    return Err(CallbackParseError::InvalidCallback);
                };
                Callback::ChangeSubscribe {
                    source_id: source_id
                        .parse()
                        .inspect_err(|e| log::error!("failed to parse source_id in change_subscribe callback: {e}"))
                        .map_err(|_| CallbackParseError::InvalidCallback)?,
                    action: ChangeSubscribeAction::try_from_payload(action)
                        .inspect_err(|_| log::error!("failed to parse action in change_subscribe callback"))
                        .map_err(|_| CallbackParseError::InvalidToken)?,
                }
            },
            SET_LANG_FLAG => {
                let data = SETLANG_CALLBACK_LAYOUT.parse_payload(value)?;
                let [kind, lang] = &data[1..=2] else {
                    log::error!("invalid number of elements in set_lang callback");
                    return Err(CallbackParseError::InvalidCallback);
                };
                Callback::SetLang {
                    lang: lang.to_string(),
                    kind: LanguagesKeyboardKind::try_from_payload(kind)
                        .inspect_err(|_| log::error!("failed to parse kind in set_lang callback"))
                        .map_err(|_| CallbackParseError::InvalidToken)?,
                }
            },
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
    pub(crate) fn notify(source_id: SourceId, app_id: AppId, should_notify: ShouldNotify) -> Self {
        Self::Notify {
            source_id,
            app_id,
            should_notify,
        }
    }
    pub(crate) fn show_source(source_id: SourceId) -> Self {
        Self::ShowSource { source_id }
    }
    #[cfg_attr(not(test), expect(unused))]
    pub(crate) fn change_subscribe(source_id: SourceId, action: ChangeSubscribeAction) -> Self {
        Self::ChangeSubscribe { source_id, action }
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
        common::init_logger();

        const V: u8 = CALLBACK_VERSION;
        let source_id = 2.into();
        let app_id = 1.into();

        let cb = |s: String| format!("{V}:{s}");

        let table = vec![
            // notify
            (
                cb(format!("{NOTIFY_FLAG}:{source_id}:{app_id}:{NOTIFY_TOKEN}")),
                Ok(Callback::notify(source_id, app_id, ShouldNotify::Notify)),
            ),
            (
                cb(format!("{NOTIFY_FLAG}:{source_id}:{app_id}:{IGNORE_TOKEN}")),
                Ok(Callback::notify(source_id, app_id, ShouldNotify::Ignore)),
            ),
            (
                cb(format!("{NOTIFY_FLAG}:{source_id}:{app_id}:asdf")),
                Err(CallbackParseError::InvalidToken),
            ),
            (
                cb(format!("{NOTIFY_FLAG}:{app_id}")),
                Err(CallbackParseError::InvalidCallback),
            ),
            // show_source
            (
                cb(format!("{SHOW_SOURCE_FLAG}:{source_id}")),
                Ok(Callback::show_source(source_id)),
            ),
            (
                cb(format!("{SHOW_SOURCE_FLAG}:asdf")),
                Err(CallbackParseError::InvalidCallback),
            ),
            // change_subscribe
            (
                cb(format!("{CHANGE_SUBSCRIBE_FLAG}:sub:{source_id}")),
                Ok(Callback::change_subscribe(source_id, ChangeSubscribeAction::Subscribe)),
            ),
            (
                cb(format!("{CHANGE_SUBSCRIBE_FLAG}:unsub:{source_id}")),
                Ok(Callback::change_subscribe(
                    source_id,
                    ChangeSubscribeAction::Unsubscribe,
                )),
            ),
            (
                cb(format!("{CHANGE_SUBSCRIBE_FLAG}:asdf:{source_id}")),
                Err(CallbackParseError::InvalidToken),
            ),
            // set_lang
            (
                cb(format!("{SET_LANG_FLAG}:start:en")),
                Ok(Callback::set_lang(LanguagesKeyboardKind::Start, "en")),
            ),
            (
                cb(format!("{SET_LANG_FLAG}:settings:en")),
                Ok(Callback::set_lang(LanguagesKeyboardKind::Settings, "en")),
            ),
            (
                cb(format!("{SET_LANG_FLAG}:starta:en")),
                Err(CallbackParseError::InvalidToken),
            ),
            // other
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
