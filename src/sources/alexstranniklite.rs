use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

use async_trait::async_trait;

use super::extractor::tg::{
    fetch_public_channel, KeyboardButton, Message, ReplyInlineMarkupRow, ReplyMarkup,
};
use super::*;

const CHANNEL_NAME: &str = "alexstranniklite";

pub(crate) struct Source {
    timeout: Duration,
    // probably RwLock is wrong
    timer: RwLock<Instant>,
}

impl Source {
    fn elapsed(&self) -> Duration {
        self.timer
            .read()
            .expect("failed to read from timer: RwLock<Instant>")
            .elapsed()
    }
    fn reset_timer(&self) {
        let mut t = self
            .timer
            .write()
            .expect("failed to write timer: RwLock<Instant>: already blocked");
        *t = Instant::now()
    }
}

#[async_trait]
impl UpdateSource for Source {
    fn new() -> Self {
        Self::with_timeout(TG_SOURCE_TIMEOUT)
    }

    fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            timer: RwLock::new(Instant::now() - timeout),
        }
    }

    fn wait_remains(&self) -> Option<Duration> {
        if self.elapsed() < self.timeout {
            Some(self.timeout - self.elapsed())
        } else {
            None
        }
    }

    async fn get_updates(&self) -> Vec<super::Update> {
        if self.wait_remains().is_some() {
            return vec![];
        }

        log::debug!("fetching updates for {CHANNEL_NAME}");
        let msgs = match fetch_public_channel(CHANNEL_NAME).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("failed to fetch: {e}");
                return vec![];
            }
        };

        // Seaching 2 messages: update, then description, only in this case saving update
        let channel_link = format!("https://t.me/{CHANNEL_NAME}/");
        let mut updates = vec![];
        let mut msg_with_update = None;
        for msg in msgs {
            if is_update(&msg) {
                msg_with_update = Some(msg);
                continue;
            }
            if let Some(upd) = &msg_with_update {
                if is_description(&msg) {
                    updates.push(
                        Update::builder()
                            .app_id(get_app_id(&upd))
                            .description_link(&format!("{channel_link}{}", msg.id))
                            .update_link(&format!("{channel_link}{}", upd.id))
                            .build(),
                    )
                }
                msg_with_update = None;
            }
        }
        updates
    }

    fn reset_timer(&self) {
        self.reset_timer()
    }
}

fn is_description(msg: &Message) -> bool {
    let Some(ref r) = msg.reply_markup else {
        return false;
    };
    match r {
        ReplyMarkup::replyInlineMarkup { ref rows } => match rows.as_slice() {
            [_, _, ReplyInlineMarkupRow::keyboardButtonRow { buttons }] => match buttons.as_slice()
            {
                [KeyboardButton::keyboardButtonUrl { text, .. }] => text == "DOWNLOAD 🛡",
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

fn is_update(msg: &Message) -> bool {
    let Some(ref r) = msg.reply_markup else {
        return false;
    };
    match r {
        ReplyMarkup::replyInlineMarkup { ref rows } => match rows.as_slice() {
            [ReplyInlineMarkupRow::keyboardButtonRow { buttons }] => match buttons.as_slice() {
                [KeyboardButton::keyboardButtonUrl { text, .. }] => text == "DISCUSS ✅",
                _ => false,
            },
            _ => false,
        },
        _ => false,
    }
}

/// Only works for update message type
///
/// `"<strong>app</strong> 1.2.3 <strong>arm7</strong>"` -> `"app"`
fn get_app_id(msg: &Message) -> String {
    let s = msg.message.split(" ").take(1).collect::<String>();
    let s = s.strip_prefix("<strong>").unwrap_or(&s);
    s.strip_suffix("</strong>").unwrap_or(&s).to_string()
}
