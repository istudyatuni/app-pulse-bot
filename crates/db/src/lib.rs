use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub mod models;
pub mod types;

use common::UnixDateTime;

use types::{Id, UserId};

const USER_TABLE: &str = "user";
const USER_UPDATE_TABLE: &str = "user_update";
const USER_SUBSCRIBE_TABLE: &str = "user_subscribe";
const SOURCE_TABLE: &str = "source";

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
}

// User
impl DB {
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
    /// Select subscribed and not yet notified users for specific source
    pub async fn select_users_to_notify(&self) -> Result<Vec<models::User>> {
        log::debug!("select subscribed users");
        Ok(sqlx::query_as::<_, models::User>(&format!(
            "select u.*
             from {USER_TABLE} u
             join {USER_SUBSCRIBE_TABLE} us
               on u.user_id = us.user_id
             join {SOURCE_TABLE} s
               on us.source_id = s.source_id
             where us.subscribed = true
               and s.last_updated_at > u.last_notified_at
               and s.source_id = ?",
        ))
        .bind(SOURCE_ID)
        .fetch_all(&self.pool)
        .await?)
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
        let update = models::UserSubscribe::new(user_id, subscribed);

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
    pub async fn save_user_last_notified(
        &self,
        user_id: UserId,
        last_notified_at: UnixDateTime,
    ) -> Result<()> {
        log::debug!("saving user {user_id} last_notified_at: {last_notified_at}");
        let user_id: Id = user_id.into();

        sqlx::query(&format!(
            "update {USER_TABLE}
             set last_notified_at = ?
             where user_id = ?",
        ))
        .bind(last_notified_at)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        log::debug!("user last_notified_at saved");
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

// Source
impl DB {
    pub async fn save_source_updated_at(&self, last_updated_at: UnixDateTime) -> Result<()> {
        log::debug!("save source last_updated_at: {last_updated_at}");
        sqlx::query(&format!(
            "update {SOURCE_TABLE}
             set last_updated_at = ?
             where source_id = ?"
        ))
        .bind(last_updated_at)
        .bind(SOURCE_ID)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn get_source_updated_at(&self) -> Result<UnixDateTime> {
        log::debug!("select source last_updated_at");
        let res = sqlx::query_as::<_, models::Source>(&format!(
            "select last_updated_at
             from {SOURCE_TABLE}
             where source_id = ?"
        ))
        .bind(SOURCE_ID)
        .fetch_optional(&self.pool)
        .await?;

        if res.is_none() {
            log::error!("source not found when selecting last_updated_at");
        }
        Ok(res.map(|s| s.last_updated_at()).unwrap_or_default())
    }
}
