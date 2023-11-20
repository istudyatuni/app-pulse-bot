use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub mod models;
mod types;

use types::{Id, UserId};

const USER_TABLE: &str = "user";
const USER_UPDATE_TABLE: &str = "user_update";
const USER_SUBSCRIBE_TABLE: &str = "user_subscribe";
// const SOURCE_TABLE: &str = "source";

// Temporary, while there is only one source
const SOURCE_ID: Id = 1;

#[derive(Debug, Clone)]
pub struct DB {
    pool: SqlitePool,
}

impl DB {
    pub async fn init(path: &str) -> Result<Self> {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true),
        )
        .await?;
        sqlx::migrate!("../../migrations").run(&pool).await?;

        Ok(Self { pool })
    }
    pub async fn save_user(&self, user_id: UserId) -> Result<()> {
        log::debug!("saving user {user_id}");
        let user = models::User::new(user_id);
        sqlx::query(&format!(
            "insert into {USER_TABLE} (user_id, lang) values (?, ?)"
        ))
        .bind(user.user_id())
        .bind(user.lang())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn select_user(&self, user_id: UserId) -> Result<Option<models::User>> {
        log::debug!("select user {user_id}");
        let id: Id = user_id.into();
        let res = sqlx::query_as::<_, models::User>(&format!(
            "select * from {USER_TABLE}
             where user_id = ?"
        ))
        .bind(id)
        .fetch_one(&self.pool)
        .await;

        match res {
            Ok(u) => Ok(Some(u)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    pub async fn select_users(&self) -> Result<Vec<models::User>> {
        log::debug!("select users");
        Ok(
            sqlx::query_as::<_, models::User>(&format!("select * from {USER_TABLE}"))
                .fetch_all(&self.pool)
                .await?,
        )
    }
    pub async fn save_should_notify_user(
        &self,
        user_id: UserId,
        app_id: &str,
        should_notify: models::ShouldNotify,
    ) -> Result<()> {
        log::debug!("saving user {user_id} should_notify: {should_notify:?}");
        let update = models::UserUpdate::new(user_id.into(), app_id, should_notify);

        // on conflict: https://sqlite.org/lang_upsert.html
        // "excluded." means "get new value"
        sqlx::query(&format!(
            "insert into {USER_UPDATE_TABLE}
             (user_id, source_id, app_id, should_notify)
             values (?, ?, ?, ?)
             on conflict(user_id, source_id, app_id)
             do update set should_notify=excluded.should_notify"
        ))
        .bind(update.user_id())
        .bind(SOURCE_ID)
        .bind(update.app_id())
        .bind(update.should_notify().to_db())
        .execute(&self.pool)
        .await?;

        log::debug!("user preference saved");
        Ok(())
    }
    pub async fn save_user_lang(&self, user_id: UserId, lang: &str) -> Result<()> {
        let id: Id = user_id.into();
        sqlx::query(&format!(
            "update {USER_TABLE}
             set lang = ?
             where user_id = ?"
        ))
        .bind(lang)
        .bind(id)
        .execute(&self.pool)
        .await?;
        log::debug!("user lang updated");
        Ok(())
    }
    pub async fn save_user_subscribed(&self, user_id: UserId, subscribed: bool) -> Result<()> {
        log::debug!("saving user {user_id} subscribe: {subscribed}");
        let update = models::UserSubscribe::new(user_id.into(), subscribed);

        sqlx::query(&format!(
            "insert into {USER_SUBSCRIBE_TABLE}
             (user_id, source_id, subscribed)
             values (?, ?, ?)
             on conflict(user_id, source_id)
             do update set subscribed=excluded.subscribed"
        ))
        .bind(update.user_id())
        .bind(SOURCE_ID)
        .bind(update.subscribed())
        .execute(&self.pool)
        .await?;

        log::debug!("user subscribe saved");
        Ok(())
    }
    pub async fn should_notify_user(
        &self,
        user_id: UserId,
        app_id: &str,
    ) -> Result<models::ShouldNotify> {
        log::debug!("getting user preference");
        let id: Id = user_id.into();
        let update = sqlx::query_as::<_, models::ShouldNotify>(&format!(
            "select should_notify
             from {USER_UPDATE_TABLE}
             where user_id = ? and source_id = ? and app_id = ?"
        ))
        .bind(id)
        .bind(SOURCE_ID)
        .bind(app_id)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or_default();
        Ok(update)
    }
}
