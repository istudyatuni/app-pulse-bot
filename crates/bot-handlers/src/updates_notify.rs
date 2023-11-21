use anyhow::Result;
use teloxide::prelude::*;
use tokio::sync::mpsc::Receiver;

use common::DateTime;
use db::{models::ShouldNotify, DB};
use sources::{Update, UpdatesList};

use crate::keyboards::{Keyboards, NewAppKeyboardKind};
use crate::tr;

pub async fn start_updates_notify_job(bot: Bot, db: DB, mut rx: Receiver<UpdatesList>) {
    log::debug!("starting listen for updates");
    // todo: graceful shutdown for updates
    while let Some(updates) = rx.recv().await {
        log::debug!("got {} updates", updates.count());
        match db.save_source_updated_at(updates.last_update).await {
            Ok(()) => (),
            Err(e) => log::error!("failed to save source last_updated_at: {e}"),
        }

        let users = match db.select_users_to_notify().await {
            Ok(v) => v,
            Err(e) => {
                log::error!("failed to select users: {e}");
                continue;
            }
        };
        log::debug!("sending to {} users", users.len());

        for update in updates.updates {
            log::debug!("got update for {}", update.app_id());
            for user in &users {
                let user_id = user.user_id();
                let chat_id = ChatId(user_id);
                let app_id = update.app_id();
                let lang = user.lang();
                let f = match db.should_notify_user(user_id.into(), app_id).await {
                    Ok(s) => match s {
                        ShouldNotify::Unspecified => {
                            send_suggest_update(bot.clone(), chat_id, &update, lang).await
                        }
                        ShouldNotify::Notify => {
                            send_update(bot.clone(), chat_id, &update, lang).await
                        }
                        ShouldNotify::Ignore => {
                            log::debug!("ignoring update {app_id} for user {user_id}");
                            continue;
                        }
                    },
                    Err(e) => {
                        log::error!("failed to check, if should notify user {user_id}: {e}");
                        continue;
                    }
                };
                f.log_on_error().await;
            }
        }

        let now = DateTime::now();
        for u in users {
            match db
                .save_user_last_notified(u.user_id().into(), now)
                .await
            {
                Ok(()) => (),
                Err(e) => log::error!("failed to save user last_notified_at: {e}"),
            }
        }
    }
}

async fn send_suggest_update(bot: Bot, chat_id: ChatId, update: &Update, lang: &str) -> Result<()> {
    let mut text = vec![tr!(new_app_msg, lang) + "\n"];
    if let Some(description) = update.description() {
        text.push(format!("\n{description}\n"));
    }
    if let Some(url) = update.description_link() {
        text.push(url.to_string());
    } else if let Some(url) = update.update_link() {
        text.push(url.to_string());
    }

    bot.send_message(chat_id, text.join(""))
        .reply_markup(Keyboards::update(
            update.app_id(),
            update.update_link().clone(),
            NewAppKeyboardKind::Both,
            lang,
        ))
        .await?;
    Ok(())
}

async fn send_update(bot: Bot, chat_id: ChatId, update: &Update, lang: &str) -> Result<()> {
    let app_id = update.app_id();
    let mut text = vec![tr!(new_update_msg, lang, app_id) + "\n"];
    if let Some(url) = update.update_link() {
        text.push(url.to_string());
    } else if let Some(url) = update.description_link() {
        text.push(url.to_string());
    }

    bot.send_message(chat_id, text.join(""))
        .reply_markup(Keyboards::update(
            app_id,
            update.update_link().clone(),
            NewAppKeyboardKind::NotifyEnabled,
            lang,
        ))
        .await?;
    Ok(())
}
