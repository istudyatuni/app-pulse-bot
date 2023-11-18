#![allow(unused)]

use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};

use crate::{tr, IGNORE_TOKEN, NOTIFY_TOKEN};

const BELL_MSG: &str = "🔔";
const NO_BELL_MSG: &str = "🔕";

#[derive(Debug, Default)]
pub(crate) struct KeyboardBuilder {
    keys: Vec<Vec<InlineKeyboardButton>>,
    current_row: usize,
    state: KeyboardBuilderState,
}

impl KeyboardBuilder {
    fn with_rows_capacity(cap: usize) -> Self {
        Self {
            keys: Vec::with_capacity(cap),
            ..Self::default()
        }
    }
    fn row(mut self) -> Self {
        if !self.keys.is_empty() {
            self.current_row += 1;
        }
        self.keys.push(vec![]);
        self
    }
    fn callback<T, D>(mut self, text: T, data: D) -> Self
    where
        T: Into<String>,
        D: Into<String>,
    {
        if self.keys.len() <= self.current_row {
            log::error!(
                "index {} out of bounds for new callback ({}, {}) for keyboard",
                self.current_row,
                text.into(),
                data.into(),
            );
            self.state = KeyboardBuilderState::Invalid;
            return self;
        }
        self.keys[self.current_row].push(InlineKeyboardButton::callback(text, data));
        self
    }
    fn url<T>(mut self, text: T, url: Url) -> Self
    where
        T: Into<String>,
    {
        if self.keys.len() <= self.current_row {
            log::error!(
                "index {} out of bounds for new url ({}, {url}) for keyboard",
                self.current_row,
                text.into(),
            );
            self.state = KeyboardBuilderState::Invalid;
            return self;
        }
        self.keys[self.current_row].push(InlineKeyboardButton::url(text, url));
        self
    }
    fn build_reply_markup(mut self) -> ReplyMarkup {
        ReplyMarkup::InlineKeyboard(self.build_inline_keyboard_markup())
    }
    fn build_inline_keyboard_markup(mut self) -> InlineKeyboardMarkup {
        if let KeyboardBuilderState::Invalid = self.state {
            log::error!("failed to build reply_inline_keyboard, dropping");
            self.keys = vec![];
        }
        InlineKeyboardMarkup::new(self.keys)
    }
}

#[derive(Debug, Default)]
enum KeyboardBuilderState {
    #[default]
    Valid,
    Invalid,
}

pub(crate) struct Keyboards;

impl Keyboards {
    fn update_keyboard(
        app_id: &str,
        url: Option<Url>,
        kind: NewAppKeyboardKind,
        lang: &str,
    ) -> KeyboardBuilder {
        let mut keyboard = match kind {
            NewAppKeyboardKind::Both => KeyboardBuilder::with_rows_capacity(2)
                .row()
                .callback(tr!(notify_button, lang), format!("{app_id}:{NOTIFY_TOKEN}"))
                .callback(tr!(ignore_button, lang), format!("{app_id}:{IGNORE_TOKEN}"))
                .row(),
            NewAppKeyboardKind::NotifyEnabled => KeyboardBuilder::with_rows_capacity(1)
                .row()
                .callback(BELL_MSG, format!("{app_id}:{IGNORE_TOKEN}")),
            NewAppKeyboardKind::NotifyDisabled => KeyboardBuilder::with_rows_capacity(1)
                .row()
                .callback(NO_BELL_MSG, format!("{app_id}:{NOTIFY_TOKEN}")),
        };

        if let Some(url) = url {
            keyboard = keyboard.url(tr!(see_update_button, lang), url.clone());
        }
        keyboard
    }
    pub(crate) fn update(
        app_id: &str,
        url: Option<Url>,
        kind: NewAppKeyboardKind,
        lang: &str,
    ) -> ReplyMarkup {
        Self::update_keyboard(app_id, url, kind, lang).build_reply_markup()
    }
    pub(crate) fn update_as_inline_keyboard(
        app_id: &str,
        url: Option<Url>,
        kind: NewAppKeyboardKind,
        lang: &str,
    ) -> InlineKeyboardMarkup {
        Self::update_keyboard(app_id, url, kind, lang).build_inline_keyboard_markup()
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
                        Btn::callback(NOTIFY_MSG, "test:notify"),
                        Btn::callback(IGNORE_MSG, "test:ignore"),
                    ],
                    vec![update_btn.clone()],
                ],
            ),
            (
                Keyboards::update(APP_ID, Some(url.clone()), Kind::NotifyEnabled, USER_LANG),
                vec![vec![
                    Btn::callback(BELL_MSG, "test:ignore"),
                    update_btn.clone(),
                ]],
            ),
            (
                Keyboards::update(APP_ID, Some(url.clone()), Kind::NotifyDisabled, USER_LANG),
                vec![vec![
                    Btn::callback(NO_BELL_MSG, "test:notify"),
                    update_btn.clone(),
                ]],
            ),
        ];
        for (res, expected) in table {
            assert_eq!(res, Reply::InlineKeyboard(Markup::new(expected)));
        }
    }
}
