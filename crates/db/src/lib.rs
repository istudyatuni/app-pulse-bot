use sqlx::{pool::PoolConnection, sqlite::SqliteConnectOptions, Sqlite, SqlitePool};
use sqlx_migrator::{Migrate, Migrator, Plan};

use common::{
    types::{Id, UserId},
    UnixDateTime,
};

mod migrations;
pub mod models;

const USER_TABLE: &str = "user";
const USER_UPDATE_TABLE: &str = "user_update";
const USER_SUBSCRIBE_TABLE: &str = "user_subscribe";
const APP_TABLE: &str = "app";
const SOURCE_TABLE: &str = "source";

const SOURCE_ID: Id = 1;

// todo: add variant for NoRowsAffected
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to run query: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("failed to run migrations: {0}")]
    Migrate(#[from] sqlx_migrator::Error),
}

type Result<T, E = Error> = std::result::Result<T, E>;

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

        Self::migrate(pool.acquire().await?).await?;

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await?;

        Ok(Self { pool })
    }
    async fn migrate(mut conn: PoolConnection<Sqlite>) -> Result<()> {
        // actually run fake migrations in tests
        #[cfg(not(test))]
        const FAKE_AS_FAKE: bool = true;
        #[cfg(test)]
        const FAKE_AS_FAKE: bool = false;

        // run old migrations
        let mut migrator = Migrator::default();
        migrations::register_fake_migrations(&mut migrator);
        let plan = Plan::apply_all().fake(FAKE_AS_FAKE);
        migrator.run(&mut conn, &plan).await?;

        // run migrations
        let mut migrator = Migrator::default();
        migrations::register_migrations(&mut migrator);
        let plan = Plan::apply_all();
        migrator.run(&mut conn, &plan).await?;

        Ok(())
    }
}

// User
impl DB {
    pub async fn add_user(&self, user: models::User) -> Result<()> {
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

        Ok(res.ignore_not_found()?)
    }
    /// Select subscribed and not yet notified users for specific source
    pub async fn select_users_to_notify(
        &self,
        source_id: Id,
        app_id: Id,
    ) -> Result<Vec<models::User>> {
        log::debug!("select subscribed users");
        Ok(sqlx::query_as::<_, models::User>(&format!(
            "select u.*
             from {USER_TABLE} u
             join {USER_SUBSCRIBE_TABLE} us on u.user_id = us.user_id
             join {SOURCE_TABLE} s on us.source_id = s.source_id
             join {APP_TABLE} a on a.source_id = s.source_id
             where us.subscribed = true
               and u.bot_blocked = false
               and s.source_id = ?
               and a.app_id = ?
               and a.last_updated_at > u.last_notified_at",
        ))
        .bind(source_id)
        .bind(app_id)
        .fetch_all(&self.pool)
        .await?)
    }
    /// Select all users
    pub async fn select_all_users(&self) -> Result<Vec<models::User>> {
        log::debug!("select all users");
        Ok(
            sqlx::query_as::<_, models::User>(&format!("select * from {USER_TABLE}"))
                .fetch_all(&self.pool)
                .await?,
        )
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
             where last_version_notified < ?
               and bot_blocked = false",
        ))
        .bind(version)
        .fetch_all(&self.pool)
        .await?)
    }
    pub async fn save_should_notify_user(
        &self,
        user_id: impl Into<UserId>,
        source_id: Id,
        app_id: Id,
        should_notify: models::ShouldNotify,
    ) -> Result<()> {
        let user_id = user_id.into();
        log::debug!("saving user {user_id} should_notify: {should_notify:?}");
        let update = models::UserUpdate::new(user_id.into(), app_id, Some(should_notify));

        sqlx::query(&format!(
            "insert into {USER_UPDATE_TABLE}
             (user_id, source_id, app_id, should_notify)
             values (?, ?, ?, ?)
             on conflict(user_id, source_id, app_id)
             do update set should_notify=excluded.should_notify"
        ))
        .bind(update.user_id())
        .bind(source_id)
        .bind(update.app_id())
        .bind(update.should_notify())
        .execute(&self.pool)
        .await?;

        log::debug!("user preference saved");
        Ok(())
    }
    pub async fn save_user_lang(&self, user_id: impl Into<UserId>, lang: &str) -> Result<()> {
        self.save_user_string_table(user_id, "lang", lang).await
    }
    pub async fn save_user_username(
        &self,
        user_id: impl Into<UserId>,
        username: &str,
    ) -> Result<()> {
        self.save_user_string_table(user_id, "username", username)
            .await
    }
    pub async fn save_user_name(&self, user_id: impl Into<UserId>, name: &str) -> Result<()> {
        self.save_user_string_table(user_id, "name", name).await
    }
    async fn save_user_string_table(
        &self,
        user_id: impl Into<UserId>,
        user_table_column: &str,
        value: &str,
    ) -> Result<()> {
        let id: Id = user_id.into().into();
        sqlx::query(&format!(
            "update {USER_TABLE}
             set {user_table_column} = ?
             where user_id = ?"
        ))
        .bind(value)
        .bind(id)
        .execute(&self.pool)
        .await?;

        log::debug!("user {user_table_column} updated");
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
    pub async fn save_all_users_last_notified(
        &self,
        source_id: Id,
        last_notified_at: UnixDateTime,
    ) -> Result<()> {
        log::debug!("saving all users last_notified_at: {last_notified_at}");

        sqlx::query(&format!(
            "update {USER_TABLE}
               set last_notified_at = ?
             from {USER_SUBSCRIBE_TABLE} us
             where us.source_id = ?
               and us.subscribed = true",
        ))
        .bind(last_notified_at)
        .bind(source_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    pub async fn save_user_version_notified(&self, user_id: impl Into<UserId>) -> Result<()> {
        self.save_user_string_table(
            user_id,
            "last_version_notified",
            &common::version().to_string(),
        )
        .await
    }
    pub async fn save_user_bot_blocked(
        &self,
        user_id: impl Into<UserId>,
        blocked: bool,
    ) -> Result<()> {
        let id: Id = user_id.into().into();
        log::debug!("saving user {id} bot_blocked: {blocked}");
        sqlx::query(&format!(
            "update {USER_TABLE}
             set bot_blocked = ?
             where user_id = ?"
        ))
        .bind(blocked)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    pub async fn should_notify_user(
        &self,
        user_id: impl Into<UserId>,
        source_id: Id,
        app_id: Id,
    ) -> Result<Option<models::ShouldNotify>> {
        log::debug!("getting user preference");
        let id: Id = user_id.into().into();
        let update = sqlx::query_as::<_, models::ShouldNotify>(&format!(
            "select should_notify
             from {USER_UPDATE_TABLE}
             where user_id = ? and source_id = ? and app_id = ?"
        ))
        .bind(id)
        .bind(source_id)
        .bind(app_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(update)
    }
}

#[cfg(test)]
impl DB {
    pub async fn add_user_simple(&self, user_id: impl Into<UserId>) -> Result<()> {
        self.add_user(models::User::new(user_id.into())).await
    }
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
    async fn save_user_version_notified_impl(
        &self,
        user_id: impl Into<UserId>,
        version: u32,
    ) -> Result<()> {
        let user_id: Id = user_id.into().into();
        sqlx::query(&format!(
            "update {USER_TABLE}
             set last_version_notified = ?
             where user_id = ?"
        ))
        .bind(version)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// App
impl DB {
    pub async fn save_app_last_updated_at(
        &self,
        source_id: Id,
        app_id: Id,
        last_updated_at: UnixDateTime,
    ) -> Result<()> {
        log::debug!("update last_updated_at for app {app_id}");
        sqlx::query(&format!(
            "update {APP_TABLE}
             set last_updated_at = ?
             where source_id = ? and app_id = ?"
        ))
        .bind(last_updated_at)
        .bind(source_id)
        .bind(app_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    pub async fn save_app_last_updated_version(
        &self,
        source_id: Id,
        app_id: Id,
        last_updated_version: String,
    ) -> Result<()> {
        log::debug!("update last_updated_at for app {app_id}");
        sqlx::query(&format!(
            "update {APP_TABLE}
             set last_updated_version = ?
             where source_id = ? and app_id = ?"
        ))
        .bind(last_updated_version)
        .bind(source_id)
        .bind(app_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    /// Returns id of new app. `Ok(None)` means app already exists
    pub async fn add_app(&self, source_id: Id, name: &str) -> Result<Id> {
        log::debug!("adding app {name} from source {source_id}");
        let res = sqlx::query_as::<_, models::fetch::AppId>(&format!(
            "insert or ignore into {APP_TABLE}
             (app_id, source_id, name)
             values ((select coalesce(max(app_id), 0) from {APP_TABLE}) + 1, ?, ?)
             returning app_id"
        ))
        .bind(source_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(res.app_id)
    }
    pub async fn get_app_name_by_app_id(&self, app_id: Id) -> Result<Option<String>> {
        log::debug!("select app_name from app {app_id}");
        let res = sqlx::query_as::<_, models::fetch::Name>(&format!(
            "select name from {APP_TABLE}
             where app_id = ?"
        ))
        .bind(app_id)
        .fetch_one(&self.pool)
        .await;

        Ok(res.ignore_not_found()?.map(|r| r.name))
    }
    pub async fn get_app_id(&self, source_id: Id, app_name: &str) -> Result<Option<Id>> {
        log::debug!("select app_id from app {app_name}");
        let res = sqlx::query_as::<_, models::fetch::AppId>(&format!(
            "select app_id from {APP_TABLE}
             where source_id = ? and name = ?"
        ))
        .bind(source_id)
        .bind(app_name)
        .fetch_one(&self.pool)
        .await;

        Ok(res.ignore_not_found()?.map(|r| r.app_id))
    }
    /// Return list of apps from specified source when at least 1 user subsribed to this source
    pub async fn get_apps_to_check_updates(&self, source_id: Id) -> Result<Vec<models::App>> {
        log::debug!("select apps by source_id {source_id}");
        Ok(sqlx::query_as::<_, models::App>(&format!(
            "select a.* from {APP_TABLE} a
             join {USER_SUBSCRIBE_TABLE} us on us.source_id = a.source_id
             join {USER_TABLE} u on us.user_id = u.user_id
             where a.app_id in (
               select app_id from {USER_UPDATE_TABLE}
               where source_id = ?
                 and should_notify = true
             )
             and a.source_id = ?
             and us.subscribed = true
             and u.bot_blocked = false"
        ))
        .bind(source_id)
        .bind(source_id)
        .fetch_all(&self.pool)
        .await?)
    }
}

// Source
impl DB {
    pub async fn add_source_or_ignore(&self, name: &str) -> Result<()> {
        // used to check failed constraint
        const SOURCE_NAME_UNIQ_ERR: &str = "UNIQUE constraint failed: source.name";

        log::debug!("adding source {name}");
        let res = sqlx::query(&format!(
            "insert into {SOURCE_TABLE} (source_id, name)
             values ((select max(source_id) from {SOURCE_TABLE}) + 1, ?)"
        ))
        .bind(name)
        .execute(&self.pool)
        .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(e) = e.as_database_error() {
                    if e.is_unique_violation() && e.message() == SOURCE_NAME_UNIQ_ERR {
                        log::debug!("source {name} exists, ignoring");
                        return Ok(());
                    }
                }
                Err(e.into())
            }
        }
    }
    pub async fn save_source_updated_at(
        &self,
        source_id: Id,
        last_updated_at: UnixDateTime,
    ) -> Result<()> {
        log::debug!("save source last_updated_at: {last_updated_at}");
        sqlx::query(&format!(
            "update {SOURCE_TABLE}
             set last_updated_at = ?
             where source_id = ?"
        ))
        .bind(last_updated_at)
        .bind(source_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
    pub async fn get_source_id_by_app_id(&self, app_id: Id) -> Result<Option<Id>> {
        log::debug!("select source_id from app by app_id {app_id}");
        let res = sqlx::query_as::<_, models::fetch::SourceId>(&format!(
            "select source_id from {APP_TABLE}
             where app_id = ?"
        ))
        .bind(app_id)
        .fetch_one(&self.pool)
        .await;

        Ok(res.ignore_not_found()?.map(|r| r.source_id))
    }
    pub async fn get_source_id_by_source_name(&self, source_name: &str) -> Result<Option<Id>> {
        log::debug!("select source_id source name {source_name}");
        let res = sqlx::query_as::<_, models::fetch::SourceId>(&format!(
            "select source_id from {SOURCE_TABLE}
             where name = ?"
        ))
        .bind(source_name)
        .fetch_one(&self.pool)
        .await;

        Ok(res.ignore_not_found()?.map(|r| r.source_id))
    }
    pub async fn get_source_updated_at(&self, source_id: Id) -> Result<UnixDateTime> {
        log::debug!("select source last_updated_at");
        let res = sqlx::query_as::<_, models::Source>(&format!(
            "select last_updated_at
             from {SOURCE_TABLE}
             where source_id = ?"
        ))
        .bind(source_id)
        .fetch_optional(&self.pool)
        .await?;

        if res.is_none() {
            log::error!("source not found when selecting last_updated_at");
        }
        Ok(res.map(|s| s.last_updated_at()).unwrap_or_default())
    }
}

// Stats
impl DB {
    pub async fn load_stats(&self) -> Result<models::Stats> {
        Ok(models::Stats {
            apps: self.load_count(&format!("from {APP_TABLE}")).await?,
            sources: self.load_count(&format!("from {SOURCE_TABLE}")).await?,
            users: self.load_count(&format!("from {USER_TABLE}")).await?,
            blocked_users: self
                .load_count(&format!("from {USER_TABLE} u where u.bot_blocked = true"))
                .await?,
        })
    }
    async fn load_count(&self, sql_predicate: &str) -> Result<u32> {
        Ok(sqlx::query_as::<_, models::fetch::Count>(&format!(
            "select count(*) as count {sql_predicate}"
        ))
        .fetch_one(&self.pool)
        .await?
        .count)
    }
}

trait IgnoreNotFound<T> {
    type Error;

    fn ignore_not_found(self) -> Result<Option<T>, Self::Error>;
}

impl<T> IgnoreNotFound<T> for std::result::Result<T, sqlx::Error> {
    type Error = sqlx::Error;

    fn ignore_not_found(self) -> std::result::Result<Option<T>, Self::Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::RangeFrom;

    use models::ShouldNotify;

    use super::*;

    struct Timer {
        iter: Box<dyn Iterator<Item = i64>>,
    }

    impl Timer {
        fn new() -> Self {
            Self {
                iter: Box::new(RangeFrom { start: 0 }),
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

    async fn prepare(test_name: &str) -> Result<DB> {
        let file = format!("../../target/{test_name}.db");

        let _ = tokio::fs::remove_file(&file).await;
        let db = DB::init(&file).await?;

        // init after migrations
        common::init_logger();

        Ok(db)
    }

    #[tokio::test]
    async fn test_select_users_to_notify() -> Result<()> {
        common::init_logger();

        const SOURCE_ID: Id = 1;

        let db = prepare("test_select_users_to_notify").await?;
        let mut timer = Timer::new();
        timer.skip(1);

        let app_id = db.add_app(SOURCE_ID, "").await?;
        db.save_app_last_updated_at(SOURCE_ID, app_id, timer.next()).await?;

        // there are 2 users
        for u in [1, 2] {
            db.add_user_simple(u).await?;
            db.save_user_subscribed(u, true).await?;
        }

        // source updated before one of users was notified
        db.save_source_updated_at(SOURCE_ID, timer.next()).await?;
        db.save_user_last_notified(1, timer.next()).await?;

        let users = db.select_users_to_notify(SOURCE_ID, app_id).await?;
        assert_eq!(users.len(), 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_no_select_users_to_notify() -> Result<()> {
        const SOURCE_ID: Id = 1;

        let db = prepare("test_no_select_users_to_notify").await?;
        let mut timer = Timer::new();
        timer.skip(1);

        // todo: seems that app in db is not required, and result of select_users_to_notify is still
        // empty
        let app_id = db.add_app(SOURCE_ID, "").await?;
        db.save_app_last_updated_at(SOURCE_ID, app_id, timer.next()).await?;

        // there is one user
        db.add_user_simple(1).await?;
        db.save_user_subscribed(1, true).await?;

        // source updated before user was notified
        db.save_source_updated_at(SOURCE_ID, timer.next()).await?;
        db.save_user_last_notified(1, timer.next()).await?;

        let users = db.select_users_to_notify(SOURCE_ID, app_id).await?;
        assert!(users.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_select_users_to_notify_about_bot_update() -> Result<()> {
        let db = prepare("test_select_users_to_notify_about_bot_update").await?;
        let mut timer = Timer::new();
        timer.skip(1);

        // there is one user
        db.add_user_simple(1).await?;
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

    #[tokio::test]
    async fn test_select_apps_to_check_updates_empty() -> Result<()> {
        const SOURCE_ID: Id = 1;
        const USER_ID: Id = 1;

        let db = prepare("test_select_apps_to_check_updates_empty").await?;
        let mut timer = Timer::new();
        timer.skip(1);

        let app_id = db.add_app(SOURCE_ID, "").await?;

        // there is one user
        db.add_user_simple(USER_ID).await?;
        db.save_user_subscribed(USER_ID, false).await?;
        db.save_should_notify_user(USER_ID, SOURCE_ID, app_id, ShouldNotify::Notify)
            .await?;

        let apps = db.get_apps_to_check_updates(SOURCE_ID).await?;
        assert!(apps.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_select_apps_to_check_updates_empty_user_blocked() -> Result<()> {
        const SOURCE_ID: Id = 1;
        const USER_ID: Id = 1;

        let db = prepare("test_select_apps_to_check_updates_empty_user_blocked").await?;
        let mut timer = Timer::new();
        timer.skip(1);

        let app_id = db.add_app(SOURCE_ID, "").await?;

        // there is one user
        db.add_user_simple(USER_ID).await?;
        db.save_user_bot_blocked(USER_ID, true).await?;
        db.save_user_subscribed(USER_ID, true).await?;
        db.save_should_notify_user(USER_ID, SOURCE_ID, app_id, ShouldNotify::Notify)
            .await?;

        let apps = db.get_apps_to_check_updates(SOURCE_ID).await?;
        assert!(apps.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_select_apps_to_check_updates() -> Result<()> {
        const SOURCE_ID: Id = 1;
        const USER_ID: Id = 1;

        let db = prepare("test_select_apps_to_check_updates").await?;
        let mut timer = Timer::new();
        timer.skip(1);

        // there is one user
        db.add_user_simple(USER_ID).await?;
        db.save_user_subscribed(USER_ID, true).await?;

        let app_id = db.add_app(SOURCE_ID, "").await?;
        db.save_should_notify_user(USER_ID, SOURCE_ID, app_id, ShouldNotify::Notify)
            .await?;

        let apps = db.get_apps_to_check_updates(SOURCE_ID).await?;
        assert_eq!(apps.len(), 1);

        Ok(())
    }
}
