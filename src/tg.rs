#![allow(unused)]

use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};

#[derive(Debug, Default)]
pub(crate) struct KeyboardBuilder {
    keys: Vec<Vec<InlineKeyboardButton>>,
    current_row: usize,
    state: KeyboardBuilderState,
}

impl KeyboardBuilder {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn with_rows_capacity(cap: usize) -> Self {
        Self {
            keys: Vec::with_capacity(cap),
            ..Self::default()
        }
    }
    pub(crate) fn row(mut self) -> Self {
        if !self.keys.is_empty() {
            self.current_row += 1;
        }
        self.keys.push(vec![]);
        self
    }
    pub(crate) fn callback<T, D>(mut self, text: T, data: D) -> Self
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
    pub(crate) fn url<T>(mut self, text: T, url: Url) -> Self
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
    pub(crate) fn build_reply_inline_keyboard(mut self) -> ReplyMarkup {
        if let KeyboardBuilderState::Invalid = self.state {
            log::error!("failed to build reply_inline_keyboard, dropping");
            self.keys = vec![];
        }
        ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(self.keys))
    }
}

#[derive(Debug, Default)]
enum KeyboardBuilderState {
    #[default]
    Valid,
    Invalid,
}
