use std::fmt::Display;

use db::models::ShouldNotify;
use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};

use crate::{callback::Callback, tr, PayloadData};

const BELL_MSG: &str = "🔔";
const NO_BELL_MSG: &str = "🔕";

#[derive(Debug, Default)]
pub(crate) struct KeyboardBuilder {
    keys: Vec<InlineKeyboardButton>,
    columns: usize,
}

impl KeyboardBuilder {
    fn with_layout(rows_capacity: usize, columns: usize) -> Self {
        Self {
            keys: Vec::with_capacity(rows_capacity * columns),
            columns,
        }
    }
    fn callback<T, D>(mut self, text: T, data: D) -> Self
    where
        T: Into<String>,
        D: Into<String>,
    {
        self.keys.push(InlineKeyboardButton::callback(text, data));
        self
    }
    fn url<T>(mut self, text: T, url: Url) -> Self
    where
        T: Into<String>,
    {
        self.keys.push(InlineKeyboardButton::url(text, url));
        self
    }
}

impl From<KeyboardBuilder> for ReplyMarkup {
    fn from(value: KeyboardBuilder) -> Self {
        Self::InlineKeyboard(value.into())
    }
}

impl From<KeyboardBuilder> for InlineKeyboardMarkup {
    fn from(value: KeyboardBuilder) -> Self {
        Self::new(value.keys.chunks(value.columns).map(|row| row.to_owned()))
    }
}

pub(crate) struct Keyboards;

impl Keyboards {
    pub(crate) fn update(
        app_id: &str,
        url: Option<Url>,
        kind: NewAppKeyboardKind,
        lang: &str,
    ) -> KeyboardBuilder {
        let keyboard = match kind {
            NewAppKeyboardKind::Both => KeyboardBuilder::with_layout(2, 2)
                .callback(
                    tr!(notify_button, lang),
                    Callback::notify(app_id, ShouldNotify::Notify).to_payload(),
                )
                .callback(
                    tr!(ignore_button, lang),
                    Callback::notify(app_id, ShouldNotify::Ignore).to_payload(),
                ),
            NewAppKeyboardKind::NotifyEnabled => KeyboardBuilder::with_layout(1, 2).callback(
                BELL_MSG,
                Callback::notify(app_id, ShouldNotify::Ignore).to_payload(),
            ),
            NewAppKeyboardKind::NotifyDisabled => KeyboardBuilder::with_layout(1, 2).callback(
                NO_BELL_MSG,
                Callback::notify(app_id, ShouldNotify::Notify).to_payload(),
            ),
        };

        if let Some(url) = url {
            keyboard.url(tr!(see_update_button, lang), url.clone())
        } else {
            keyboard
        }
    }
    pub(crate) fn languages(kind: LanguagesKeyboardKind) -> KeyboardBuilder {
        const LANGS_IN_ROW: usize = 3;
        let langs: Vec<&'static str> = i18n::Localize::languages();
        let mut keyboard =
            KeyboardBuilder::with_layout(langs.len() / LANGS_IN_ROW + 1, LANGS_IN_ROW);
        for lang in langs {
            keyboard = keyboard.callback(
                tr!(lang_name, lang),
                Callback::set_lang(kind, lang).to_payload(),
            );
        }

        keyboard
    }
}

#[derive(Debug)]
pub(crate) enum NewAppKeyboardKind {
    /// Show both buttons
    Both,
    /// Notifications enabled, show "bell" icon
    NotifyEnabled,
    /// Notifications disabled, show "no-bell" icon
    NotifyDisabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LanguagesKeyboardKind {
    /// Keyboard was requested when user pressed /start
    Start,
    /// Keyboard was requested when user pressed /settings
    Settings,
}

impl Display for LanguagesKeyboardKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Start => "start",
            Self::Settings => "settings",
        };
        s.fmt(f)
    }
}

impl TryFrom<&str> for LanguagesKeyboardKind {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let s = match value {
            "start" => Some(Self::Start),
            "settings" => Some(Self::Settings),
            _ => None,
        };
        s.ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use teloxide::types::{
        InlineKeyboardButton as Btn, InlineKeyboardMarkup as Markup, ReplyMarkup as Reply,
    };

    use super::NewAppKeyboardKind as Kind;
    use super::*;

    const NOTIFY_MSG: &str = "Notify";
    const IGNORE_MSG: &str = "Ignore";
    const SEE_UPDATE_MSG: &str = "See update";

    const APP_ID: &str = "test";
    const USER_LANG: &str = "en";

    #[test]
    fn test_new_app_keyboard() {
        let url = Url::parse("http://example.com/update").unwrap();
        let update_btn = Btn::url(SEE_UPDATE_MSG, url.clone());
        let table = vec![
            (
                Keyboards::update(APP_ID, Some(url.clone()), Kind::Both, USER_LANG),
                vec![
                    vec![
                        Btn::callback(NOTIFY_MSG, "notify:test:notify"),
                        Btn::callback(IGNORE_MSG, "notify:test:ignore"),
                    ],
                    vec![update_btn.clone()],
                ],
            ),
            (
                Keyboards::update(APP_ID, Some(url.clone()), Kind::NotifyEnabled, USER_LANG),
                vec![vec![
                    Btn::callback(BELL_MSG, "notify:test:ignore"),
                    update_btn.clone(),
                ]],
            ),
            (
                Keyboards::update(APP_ID, Some(url.clone()), Kind::NotifyDisabled, USER_LANG),
                vec![vec![
                    Btn::callback(NO_BELL_MSG, "notify:test:notify"),
                    update_btn.clone(),
                ]],
            ),
        ];
        for (res, expected) in table {
            let res: ReplyMarkup = res.into();
            assert_eq!(res, Reply::InlineKeyboard(Markup::new(expected)));
        }
    }
}
