use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

use async_trait::async_trait;

use crate::extractor::tg::{
    fetch_public_channel, Document, KeyboardButton, Media, Message, ReplyInlineMarkupRow,
    ReplyMarkup,
};
use crate::*;

const CHANNEL_NAME: &str = "alexstranniklite";

pub struct Source {
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

    async fn get_updates(&self) -> super::UpdatesList {
        if self.wait_remains().is_some() {
            return super::UpdatesList::default();
        }

        log::debug!("fetching updates for {CHANNEL_NAME}");
        let msgs = match fetch_public_channel(CHANNEL_NAME).await {
            Ok(v) => v,
            Err(e) => {
                log::error!("failed to fetch: {e}");
                return super::UpdatesList::default();
            }
        };

        let mut last_update = 0;

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
                            .app_id(get_app_id(upd))
                            .description_link(&format!("{channel_link}{}", msg.id))
                            .update_link(&format!("{channel_link}{}", upd.id))
                            .build(),
                    );

                    if last_update != 0 {
                        last_update = upd.date;
                    }
                }
                msg_with_update = None;
            }
        }
        super::UpdatesList {
            updates,
            last_update,
        }
    }

    fn reset_timer(&self) {
        self.reset_timer()
    }
}

fn is_description(msg: &Message) -> bool {
    has_button(msg, "DOWNLOAD 🛡")
}

fn is_update(msg: &Message) -> bool {
    has_discuss_button(msg) || has_apk_attachment(msg)
}

fn has_discuss_button(msg: &Message) -> bool {
    has_button(msg, "DISCUSS ✅")
}

fn has_apk_attachment(msg: &Message) -> bool {
    if let Some(Media::messageMediaDocument {
        document: Document::document { ref mime_type },
    }) = msg.media
    {
        if mime_type == "application/vnd.android.package-archive" {
            return true;
        }
    }
    false
}

fn has_button(msg: &Message, text: &str) -> bool {
    let Some(ref r) = msg.reply_markup else {
        return false;
    };
    if let ReplyMarkup::replyInlineMarkup { ref rows } = r {
        for row in rows {
            if let ReplyInlineMarkupRow::keyboardButtonRow { buttons } = row {
                for button in buttons {
                    if let KeyboardButton::keyboardButtonUrl { text: t, .. } = button {
                        if t == text {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Only works for update message type
///
/// - `"<strong>app</strong> 1.2.3 <strong>arm7</strong>"` -> `"app"`
/// - `"<a ...><strong>app</strong></a> 1.2.3 <strong>arm7</strong>"` -> `"app"`
fn get_app_id(msg: &Message) -> String {
    let msg: &str = &msg.message;
    if msg.starts_with("<a") {
        msg.split("<strong>")
            .skip(1)
            .take(1)
            .collect::<String>()
            .split("</strong>")
            .take(1)
            .collect()
    } else {
        if !msg.starts_with("<strong>") {
            log::error!("got unknown message when parsing app_id: {msg}");
        }
        let s = msg.split(' ').take(1).collect::<String>();
        let s = s.strip_prefix("<strong>").unwrap_or(&s);
        s.strip_suffix("</strong>").unwrap_or(s).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_id() {
        let table = &[
            ("<strong>app</strong> 1.2.3 <strong>arm7</strong>", "app"),
            ("<a href=\"mts.music\" target=\"_blank\" rel=\"nofollow\"><strong>app.text</strong></a> 9.19.0", "app.text"),
        ];
        for (msg, expected) in table {
            assert_eq!(
                get_app_id(&Message {
                    message: msg.to_string(),
                    ..Default::default()
                }),
                expected.to_string()
            );
        }
    }
}
