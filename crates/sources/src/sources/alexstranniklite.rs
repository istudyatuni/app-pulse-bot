use std::time::Duration;

use async_trait::async_trait;
use timer::Timer;

use crate::extractor::tg::{
    fetch_public_channel, Document, KeyboardButton, Media, Message, ReplyInlineMarkupRow,
    ReplyMarkup,
};
use crate::*;

const CHANNEL_NAME: &str = "alexstranniklite";

pub struct Source {
    timer: Timer,
}

impl Source {
    async fn get_updates_list(&self) -> super::UpdatesList {
        if self.wait_remains().is_some() {
            return super::UpdatesList::default();
        }

        let msgs = match fetch_public_channel(CHANNEL_NAME).await {
            Ok(v) => v,
            Err(_) => return super::UpdatesList::default(),
        };

        let mut last_update = None;

        // Seaching 2 messages: update, then maybe message with apk only, then
        // description, only in this case saving update. Possible sequences:
        //
        // 1. update (apk) with name - description
        // 2. update (apk) with name - apk - description
        // 3. apk - update (apk) with name - description
        //
        // case 3 does not require special handling
        let channel_link = format!("https://t.me/{CHANNEL_NAME}/");
        let mut updates = vec![];
        let mut msg_with_update = None;
        for msg in msgs {
            if is_update(&msg) {
                msg_with_update = Some(msg);
                continue;
            }
            if let Some(update) = &msg_with_update {
                if is_description(&msg) {
                    // handle case 1
                    updates.push(
                        Update::builder()
                            .app_id(get_app_id(update))
                            .description_link(&format!("{channel_link}{}", msg.id))
                            .update_link(&format!("{channel_link}{}", update.id))
                            .update_time(update.date)
                            .build(),
                    );

                    last_update = last_update.or(Some(update.date));
                } else if has_apk_attachment(&msg) {
                    // handle case 2
                    continue;
                }
                msg_with_update = None;
            }
        }
        super::UpdatesList {
            updates,
            last_update: last_update.unwrap_or_default(),
        }
    }
}

#[async_trait]
impl UpdateSource for Source {
    // such type to use log_error when creating source
    type InitError = &'static str;

    fn with_timeout(timeout: Duration) -> Result<Self, Self::InitError> {
        Ok(Self {
            timer: Timer::new(timeout),
        })
    }

    fn wait_remains(&self) -> Option<Duration> {
        self.timer.elapsed_remains()
    }

    fn reset_timer(&self) {
        self.timer.reset()
    }
}

#[async_trait]
impl UpdateSourceList for Source {
    async fn get_updates(&self) -> super::UpdatesList {
        self.get_updates_list().await
    }
}

fn is_description(msg: &Message) -> bool {
    has_button(msg, "DOWNLOAD 🛡")
}

/// If this is a message with APK and has message (with app id)
fn is_update(msg: &Message) -> bool {
    (has_discuss_button(msg) || has_apk_attachment(msg)) && !msg.message.is_empty()
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
/// - `"<b>app</b> 1.2.3 <b>arm7</b>"` -> `"app"`
/// - `"<a ...><b>app</b></a> 1.2.3 <b>arm7</b>"` -> `"app"`
fn get_app_id(msg: &Message) -> String {
    let msg: &str = &msg.message;
    if msg.starts_with("<a") {
        msg.split("<b>")
            .skip(1)
            .take(1)
            .collect::<String>()
            .split("</b>")
            .take(1)
            .collect()
    } else {
        if !msg.starts_with("<b>") {
            log::error!("got unknown message when parsing app_id: {msg}");
        }
        let s = msg.split(' ').take(1).collect::<String>();
        let s = s.strip_prefix("<b>").unwrap_or(&s);
        s.strip_suffix("</b>").unwrap_or(s).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_app_id() {
        let table = &[
            ("<b>app</b> 1.2.3 <b>arm7</b>", "app"),
            ("<a href=\"mts.music\" target=\"_blank\" rel=\"nofollow\"><b>app.text</b></a> 9.19.0", "app.text"),
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
