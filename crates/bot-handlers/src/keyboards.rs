#![allow(unused)]

use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};

use crate::{IGNORE_TOKEN, NOTIFY_TOKEN};

const NOTIFY_MSG: &str = "Notify";
const IGNORE_MSG: &str = "Ignore";
const BELL_MSG: &str = "🔔";
const NO_BELL_MSG: &str = "🔕";
const SEE_UPDATE_MSG: &str = "See update";

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
    pub(crate) fn update_keyboard(
        app_id: &str,
        url: Option<Url>,
        kind: NewAppKeyboardKind,
    ) -> KeyboardBuilder {
        let mut keyboard = match kind {
            NewAppKeyboardKind::Both => KeyboardBuilder::with_rows_capacity(2)
                .row()
                .callback(NOTIFY_MSG, format!("{app_id}:{NOTIFY_TOKEN}"))
                .callback(IGNORE_MSG, format!("{app_id}:{IGNORE_TOKEN}"))
                .row(),
            NewAppKeyboardKind::NotifyEnabled => KeyboardBuilder::with_rows_capacity(1)
                .row()
                .callback(BELL_MSG, format!("{app_id}:{IGNORE_TOKEN}")),
            NewAppKeyboardKind::NotifyDisabled => KeyboardBuilder::with_rows_capacity(1)
                .row()
                .callback(NO_BELL_MSG, format!("{app_id}:{NOTIFY_TOKEN}")),
        };

        if let Some(url) = url {
            keyboard = keyboard.url(SEE_UPDATE_MSG, url.clone());
        }
        keyboard
    }
    pub(crate) fn update(app_id: &str, url: Option<Url>, kind: NewAppKeyboardKind) -> ReplyMarkup {
        Self::update_keyboard(app_id, url, kind).build_reply_markup()
    }
    pub(crate) fn update_as_inline_keyboard(
        app_id: &str,
        url: Option<Url>,
        kind: NewAppKeyboardKind,
    ) -> InlineKeyboardMarkup {
        Self::update_keyboard(app_id, url, kind).build_inline_keyboard_markup()
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

    const APP_ID: &str = "test";

    #[test]
    fn test_new_app_keyboard() {
        let url = Url::parse("http://example.com/update").unwrap();
        let update_btn = Btn::url(SEE_UPDATE_MSG, url.clone());
        let table = vec![
            (
                Keyboards::update(APP_ID, Some(url.clone()), Kind::Both),
                vec![
                    vec![
                        Btn::callback(NOTIFY_MSG, "test:notify"),
                        Btn::callback(IGNORE_MSG, "test:ignore"),
                    ],
                    vec![update_btn.clone()],
                ],
            ),
            (
                Keyboards::update(APP_ID, Some(url.clone()), Kind::NotifyEnabled),
                vec![vec![
                    Btn::callback(BELL_MSG, "test:ignore"),
                    update_btn.clone(),
                ]],
            ),
            (
                Keyboards::update(APP_ID, Some(url.clone()), Kind::NotifyDisabled),
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