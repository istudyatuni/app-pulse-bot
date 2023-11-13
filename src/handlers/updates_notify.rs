use anyhow::Result;
use teloxide::prelude::*;
use tokio::sync::mpsc::Receiver;

use crate::{
    db::{models::ShouldNotify, DB},
    sources::Update,
    tg::KeyboardBuilder,
    IGNORE_TOKEN, NOTIFY_TOKEN,
};

pub(crate) async fn start_updates_notify_job(bot: Bot, db: DB, mut rx: Receiver<Vec<Update>>) {
    log::debug!("starting listen for updates");
    // todo: graceful shutdown for updates
    while let Some(updates) = rx.recv().await {
        for update in updates {
            log::debug!("got update for {}", update.app_id());
            let users = match db.select_users().await {
                Ok(v) => v,
                Err(e) => {
                    log::error!("failed to select users: {e}");
                    continue;
                }
            };
            for user in users {
                let user_id = user.user_id();
                let chat_id = user_id.into();
                let app_id = update.app_id();
                match db.should_notify_user(user_id, app_id).await {
                    Ok(s) => match s {
                        ShouldNotify::Unspecified => {
                            send_suggest_update(bot.clone(), chat_id, &update)
                                .await
                                .log_on_error()
                                .await
                        }
                        ShouldNotify::Notify => {
                            send_update(bot.clone(), chat_id, &update)
                                .await
                                .log_on_error()
                                .await;
                        }
                        ShouldNotify::Ignore => {
                            log::debug!("ignoring update {app_id} for user {user_id}")
                        }
                    },
                    Err(e) => log::error!("failed to check, if should notify user {user_id}: {e}"),
                };
            }
        }
    }
}

async fn send_suggest_update(bot: Bot, chat_id: ChatId, update: &Update) -> Result<()> {
    let mut text = vec!["New app to track updates\n".to_string()];
    if let Some(description) = update.description() {
        text.push(format!("\n{description}\n"));
    }
    if let Some(url) = update.description_link() {
        text.push(url.to_string());
    } else if let Some(url) = update.update_link() {
        text.push(url.to_string());
    }

    let app_id = update.app_id();
    let mut keyboard = KeyboardBuilder::new()
        .row()
        .callback("Notify", format!("{app_id}:{NOTIFY_TOKEN}"))
        .callback("Ignore", format!("{app_id}:{IGNORE_TOKEN}"));

    if let Some(url) = update.update_link() {
        keyboard = keyboard.row().url("See update", url.clone());
    }

    bot.send_message(chat_id, text.join(""))
        .reply_markup(keyboard.build_reply_inline_keyboard())
        .await?;
    Ok(())
}

async fn send_update(bot: Bot, chat_id: ChatId, update: &Update) -> Result<()> {
    let app_id = update.app_id();
    let mut text = vec![format!("New update for {app_id}\n")];
    if let Some(url) = update.update_link() {
        text.push(url.to_string());
    } else if let Some(url) = update.description_link() {
        text.push(url.to_string());
    }

    let keyboard = KeyboardBuilder::new()
        .row()
        .callback("Ignore", format!("{app_id}:{IGNORE_TOKEN}"));
    bot.send_message(chat_id, text.join(""))
        .reply_markup(keyboard.build_reply_inline_keyboard())
        .await?;
    Ok(())
}
