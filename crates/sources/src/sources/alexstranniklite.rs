use std::time::Duration;

use async_trait::async_trait;

use db::DB;

use crate::extractor::tg::{
    fetch_public_channel, Document, KeyboardButton, Media, Message, ReplyInlineMarkupRow,
    ReplyMarkup,
};
use crate::*;
use timer::Timer;

const CHANNEL_NAME: &str = "alexstranniklite";

pub struct Source {
    timer: Timer,
    db: DB,
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
                    let app_name = get_app_name(update);
                    let app_id = match self.db.get_app_id_by_app_name(&app_name).await {
                        Ok(Some(id)) => id,
                        Ok(None) => {
                            log::error!("app by app_name ({app_name}) not found");
                            msg_with_update.take();
                            continue;
                        }
                        Err(e) => {
                            log::error!("failed to get app_id by app_name ({app_name}): {e}");
                            msg_with_update.take();
                            continue;
                        }
                    };

                    updates.push(
                        Update::builder()
                            .app_id(app_id)
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
                msg_with_update.take();
            }
        }
        super::UpdatesList {
            updates,
            last_update: last_update.unwrap_or_default(),
        }
    }
}

impl UpdateSource for Source {
    // such type to use log_error when creating source
    type InitError = &'static str;

    fn name() -> &'static str {
        "tg@alexstranniklite"
    }

    fn with_timeout(timeout: Duration, db: DB) -> Result<Self, Self::InitError> {
        Ok(Self {
            timer: Timer::new(timeout),
            db,
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
/// - `"<strong>app</strong> 1.2.3 <strong>arm7</strong>"` -> `"app"`
/// - `"<a ...><strong>app</strong></a> 1.2.3 <strong>arm7</strong>"` -> `"app"`
fn get_app_name(msg: &Message) -> String {
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
                get_app_name(&Message {
                    message: msg.to_string(),
                    ..Default::default()
                }),
                expected.to_string()
            );
        }
    }
}
