use anyhow::Result;
use teloxide::prelude::*;
use tokio::sync::mpsc::Receiver;

use common::{DateTime, LogError};
use db::{models::ShouldNotify, DB};
use sources::{Update, UpdatesList};

use crate::keyboards::{Keyboards, NewAppKeyboardKind};
use crate::tr;

pub async fn start_updates_notify_job(bot: Bot, db: DB, mut rx: Receiver<UpdatesList>) {
    notify_bot_update(bot.clone(), db.clone())
        .await
        .log_error_msg("failed to notify about bot update");

    log::debug!("starting listen for updates");
    // todo: graceful shutdown for updates
    while let Some(updates) = rx.recv().await {
        log::debug!("got {} updates", updates.count());
        db.save_source_updated_at(updates.last_update)
            .await
            .log_error_msg("failed to save source last_updated_at");

        for update in updates.updates {
            let app_id = update.app_id();
            log::debug!("got update for app {}", app_id);

            if let Err(e) = db.add_or_update_app(app_id, "", update.update_time()).await {
                log::error!("failed to add app: {e}");
                continue;
            }

            let users = match db.select_users_to_notify(app_id).await {
                Ok(v) => v,
                Err(e) => {
                    log::error!("failed to select users: {e}");
                    continue;
                }
            };
            log::debug!("sending app '{app_id}' update to {} users", users.len());

            for user in &users {
                let user_id = user.user_id();
                let chat_id = ChatId(user_id);
                let lang = user.lang();
                let res = match db.should_notify_user(user_id, app_id).await {
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
                if let Err(e) = res {
                    match e {
                        UpdateError::BotBlocked(chat_id) => handle_bot_blocked(&db, chat_id).await,
                        UpdateError::RequestError(ref e) => {
                            log::error!("error from update notifier: {e}")
                        }
                    }
                }
            }
        }

        db.save_all_users_last_notified(DateTime::now())
            .await
            .log_error_msg("failed to save all users last_notified_at");
    }
}

async fn send_suggest_update(
    bot: Bot,
    chat_id: ChatId,
    update: &Update,
    lang: &str,
) -> Result<(), UpdateError> {
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
        .await
        .map_bot_blocked_error(chat_id)
}

async fn send_update(
    bot: Bot,
    chat_id: ChatId,
    update: &Update,
    lang: &str,
) -> Result<(), UpdateError> {
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
        .await
        .map_bot_blocked_error(chat_id)
}

async fn notify_bot_update(bot: Bot, db: DB) -> Result<()> {
    let users = db.select_users_to_notify_about_bot_update().await?;
    log::debug!("sending bot update notification to {} users", users.len());

    let mut failed = (0, users.len());
    let mut errors = vec![];
    for u in users {
        let user_id = u.user_id();
        let chat_id = ChatId(user_id);
        let lang = u.lang();
        let text = crate::utils::escape(tr!(bot_updated, lang));
        if let Err(e) = bot
            .send_message(chat_id, text)
            .parse_mode(teloxide::types::ParseMode::MarkdownV2)
            .await
            .map_bot_blocked_error(chat_id)
        {
            match e {
                UpdateError::BotBlocked(chat_id) => {
                    handle_bot_blocked(&db, chat_id).await;
                }
                UpdateError::RequestError(e) => {
                    failed.0 += 1;
                    errors.push(e.to_string());
                }
            }
        } else if let Err(e) = db.save_user_version_notified(user_id).await {
            log::error!("failed to save user {user_id} notified: {e}");
        }
    }
    if failed.0 > 0 {
        // one formatted error message from telegram is approximately 200-400
        // symbols, so 5 chunks each 400 symbols is 2k symbols, while message
        // limit is 4096 symbols, so it's pretty appropriate constant
        for (i, e) in errors.chunks(5).enumerate() {
            let e = e.join("\n\n");
            log::error!(
                "failed to send bot update notification to {}/{} users. errors ({i}):\n\n{e}",
                failed.0,
                failed.1
            );
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum UpdateError {
    #[error("bot blocked by user {0}")]
    BotBlocked(ChatId),

    #[error(transparent)]
    RequestError(#[from] teloxide::RequestError),
}

trait MapBotBlockedError {
    fn map_bot_blocked_error(self, chat_id: ChatId) -> Result<(), UpdateError>;
}

impl<R> MapBotBlockedError for Result<R, teloxide::RequestError> {
    fn map_bot_blocked_error(self, chat_id: ChatId) -> Result<(), UpdateError> {
        match self {
            Ok(_) => Ok(()),
            Err(e) => match e {
                teloxide::RequestError::Api(teloxide::ApiError::BotBlocked) => {
                    Err(UpdateError::BotBlocked(chat_id))
                }
                _ => Err(e.into()),
            },
        }
    }
}

/// Save that user blocked bot
async fn handle_bot_blocked(db: &DB, chat_id: ChatId) {
    log::info!(tg = true; "bot blocked by user {chat_id}");
    db.save_user_bot_blocked(chat_id, true)
        .await
        .log_error_msg("failed to save user bot_blocked");
}
