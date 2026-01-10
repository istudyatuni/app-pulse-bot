use teloxide::{prelude::Requester, types::ChatFullInfoKind, Bot};

use common::LogError;
use db::DB;

pub async fn run_collect_user_names_job(bot: Bot, db: DB) -> Result<(), UsersCollectError> {
    for u in db.select_all_users().await? {
        let chat = bot.get_chat(u.tg_user_id()).await?;
        if let ChatFullInfoKind::Private(chat) = chat.kind {
            if let Some(name) = get_chat_name(chat.first_name.as_deref(), chat.last_name.as_deref())
            {
                db.save_user_name(u.user_id(), &name).await.log_error();
            }
            if let Some(username) = chat.username {
                db.save_user_username(u.user_id(), &username)
                    .await
                    .log_error();
            }
        } else {
            log::error!(code = false; "saved chat {} is not private", u.display());
        }
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum UsersCollectError {
    #[error(transparent)]
    DbError(#[from] db::Error),
    #[error(transparent)]
    RequestError(#[from] teloxide::RequestError),
}

pub(crate) fn get_chat_name(first_name: Option<&str>, last_name: Option<&str>) -> Option<String> {
    match (&first_name, &last_name) {
        (Some(first), Some(last)) => Some(format!("{first} {last}")),
        (Some(name), None) | (None, Some(name)) => Some(name.to_string()),
        (None, None) => None,
    }
}
