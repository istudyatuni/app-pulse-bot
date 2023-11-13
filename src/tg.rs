#![allow(unused)]

use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup};

#[derive(Debug)]
pub(crate) struct KeyboardBuilder {
    keys: Vec<Vec<InlineKeyboardButton>>,
    current_row: usize,
}

impl KeyboardBuilder {
    pub(crate) fn new() -> Self {
        Self {
            keys: vec![],
            current_row: 0,
        }
    }
    pub(crate) fn row(mut self) -> Self {
        if !self.keys.is_empty() {
            self.current_row += 1;
        } else {
            self.keys.push(vec![]);
        }
        self
    }
    pub(crate) fn callback<T, D>(mut self, text: T, data: D) -> Self
    where
        T: Into<String>,
        D: Into<String>,
    {
        self.keys[self.current_row].push(InlineKeyboardButton::callback(text, data));
        self
    }
    pub(crate) fn url<T>(mut self, text: T, url: Url) -> Self
    where
        T: Into<String>,
    {
        self.keys[self.current_row].push(InlineKeyboardButton::url(text, url));
        self
    }
    pub(crate) fn build_reply_inline_keyboard(self) -> ReplyMarkup {
        ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(self.keys))
    }
}
