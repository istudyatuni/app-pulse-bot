use anyhow::Result;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub mod models;
pub mod types;

use common::UnixDateTime;

use types::{Id, UserId};

const USER_TABLE: &str = "user";
const USER_UPDATE_TABLE: &str = "user_update";
const USER_SUBSCRIBE_TABLE: &str = "user_subscribe";
const APP_TABLE: &str = "app";
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
    pub async fn add_user(&self, user_id: impl Into<UserId>) -> Result<()> {
        self.add_user_impl(models::User::new(user_id.into())).await
    }
    pub async fn add_user_with_lang(
        &self,
        user_id: impl Into<UserId>,
        lang: impl Into<String>,
    ) -> Result<()> {
        self.add_user_impl(models::User::new_with_lang(user_id.into(), lang))
            .await
    }
    async fn add_user_impl(&self, user: models::User) -> Result<()> {
        log::debug!("saving user {}", user.user_id());
        sqlx::query(&format!(
            "insert into {USER_TABLE} (user_id, lang, last_version_notified) values (?, ?, ?)"
        ))
        .bind(user.user_id())
        .bind(user.lang())
        .bind(common::version())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn select_user(&self, user_id: impl Into<UserId>) -> Result<Option<models::User>> {
        let user_id = user_id.into();
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
    pub async fn select_users_to_notify(&self, app_id: &str) -> Result<Vec<models::User>> {
        log::debug!("select subscribed users");
        Ok(sqlx::query_as::<_, models::User>(&format!(
            "select u.*
             from {USER_TABLE} u
             join {USER_SUBSCRIBE_TABLE} us on u.user_id = us.user_id
             join {SOURCE_TABLE} s on us.source_id = s.source_id
             join {APP_TABLE} a on a.source_id = s.source_id
             where us.subscribed = true
               and s.source_id = ?
               and a.app_id = ?
               and a.last_updated_at > u.last_notified_at",
        ))
        .bind(SOURCE_ID)
        .bind(app_id)
        .fetch_all(&self.pool)
        .await?)
    }
    /// Select users, not yet notified about bot update
    pub async fn select_users_to_notify_about_bot_update(&self) -> Result<Vec<models::User>> {
        self.select_users_to_notify_about_bot_update_impl(common::version())
            .await
    }
    async fn select_users_to_notify_about_bot_update_impl(
        &self,
        version: u32,
    ) -> Result<Vec<models::User>> {
        log::debug!("select users to notify about bot update: version {version}");
        Ok(sqlx::query_as::<_, models::User>(&format!(
            "select *
             from {USER_TABLE}
             where last_version_notified < ?",
        ))
        .bind(version)
        .fetch_all(&self.pool)
        .await?)
    }
    pub async fn save_should_notify_user(
        &self,
        user_id: impl Into<UserId>,
        app_id: &str,
        should_notify: models::ShouldNotify,
    ) -> Result<()> {
        let user_id = user_id.into();
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
    pub async fn save_user_lang(&self, user_id: impl Into<UserId>, lang: &str) -> Result<()> {
        let id: Id = user_id.into().into();
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
    pub async fn save_user_subscribed(
        &self,
        user_id: impl Into<UserId>,
        subscribed: bool,
    ) -> Result<()> {
        let user_id = user_id.into();
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
    /// Set `last_notified_at` for all users, subscribed to source
    pub async fn save_all_users_last_notified(&self, last_notified_at: UnixDateTime) -> Result<()> {
        log::debug!("saving all users last_notified_at: {last_notified_at}");

        sqlx::query(&format!(
            "update {USER_TABLE}
               set last_notified_at = ?
             from {USER_SUBSCRIBE_TABLE} us
             where us.source_id = ?
               and us.subscribed = true",
        ))
        .bind(last_notified_at)
        .bind(SOURCE_ID)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    pub async fn save_user_version_notified(&self, user_id: impl Into<UserId>) -> Result<()> {
        self.save_user_version_notified_impl(user_id, common::version())
            .await
    }
    async fn save_user_version_notified_impl(
        &self,
        user_id: impl Into<UserId>,
        version: u32,
    ) -> Result<()> {
        let user_id = user_id.into();
        log::debug!("saving user {user_id} version notified");
        let user = models::User::new(user_id);
        sqlx::query(&format!(
            "update {USER_TABLE}
             set last_version_notified = ?
             where user_id = ?"
        ))
        .bind(version)
        .bind(user.user_id())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    pub async fn should_notify_user(
        &self,
        user_id: impl Into<UserId>,
        app_id: &str,
    ) -> Result<models::ShouldNotify> {
        let user_id = user_id.into();
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

#[cfg(test)]
impl DB {
    pub async fn save_user_last_notified(
        &self,
        user_id: impl Into<UserId>,
        last_notified_at: UnixDateTime,
    ) -> Result<()> {
        let user_id = user_id.into();
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
}

// App
impl DB {
    /// Add new app, if there is already exists app with
    /// (`app_id`, `source_id`), update `last_updated_at`
    pub async fn add_or_update_app(
        &self,
        app_id: &str,
        name: &str,
        last_updated_at: UnixDateTime,
    ) -> Result<()> {
        log::debug!("saving app {app_id}");
        let app = models::App::new(app_id, SOURCE_ID, name, last_updated_at);
        sqlx::query(&format!(
            "insert into {APP_TABLE}
             (app_id, source_id, name, last_updated_at)
             values (?, ?, ?, ?)
             on conflict(app_id, source_id)
             do update set last_updated_at=excluded.last_updated_at"
        ))
        .bind(app.app_id())
        .bind(app.source_id())
        .bind(app.name())
        .bind(app.last_updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use std::ops::RangeFrom;

    use super::*;

    struct Timer {
        iter: Box<dyn Iterator<Item = i64>>,
    }

    impl Timer {
        fn new() -> Self {
            Self {
                iter: Box::new(RangeFrom { start: 0i64 }),
            }
        }
        fn skip(&mut self, count: u32) {
            for _ in 0..count {
                self.iter.next();
            }
        }
        fn next<T: From<i64>>(&mut self) -> T {
            self.iter.next().unwrap().into()
        }
    }

    async fn prepare_db_timer(test_name: &str) -> Result<DB> {
        let file = format!("../../target/{test_name}.db");

        let _ = tokio::fs::remove_file(&file).await;
        let db = DB::init(&file).await?;

        Ok(db)
    }

    #[tokio::test]
    async fn test_select_users_to_notify() -> Result<()> {
        const APP_ID: &str = "test";

        let db = prepare_db_timer("test_select_users_to_notify").await?;
        let mut timer = Timer::new();
        timer.skip(1);

        db.add_or_update_app(APP_ID, "", timer.next()).await?;

        // there are 2 users
        for u in [1, 2] {
            db.add_user(u).await?;
            db.save_user_subscribed(u, true).await?;
        }

        // source updated before one of users was notified
        db.save_source_updated_at(timer.next()).await?;
        db.save_user_last_notified(1, timer.next()).await?;

        let users = db.select_users_to_notify(APP_ID).await?;
        assert_eq!(users.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_no_select_users_to_notify() -> Result<()> {
        const APP_ID: &str = "test";

        let db = prepare_db_timer("test_no_select_users_to_notify").await?;
        let mut timer = Timer::new();
        timer.skip(1);

        db.add_or_update_app(APP_ID, "", timer.next()).await?;

        // there is one user
        db.add_user(1).await?;
        db.save_user_subscribed(1, true).await?;

        // source updated before user was notified
        db.save_source_updated_at(timer.next()).await?;
        db.save_user_last_notified(1, timer.next()).await?;

        let users = db.select_users_to_notify(APP_ID).await?;
        assert!(users.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_select_users_to_notify_about_bot_update() -> Result<()> {
        let db = prepare_db_timer("test_select_users_to_notify_about_bot_update").await?;
        let mut timer = Timer::new();
        timer.skip(1);

        // there is one user
        db.add_user(1).await?;
        db.save_user_version_notified_impl(1, 0).await?;

        for v in 1..20 {
            let users = db
                .select_users_to_notify_about_bot_update_impl(v + 1)
                .await?;
            assert_eq!(users.len(), 1, "notify about version {}", v + 1);
            db.save_user_version_notified_impl(1, v).await?;
        }

        Ok(())
    }
}
