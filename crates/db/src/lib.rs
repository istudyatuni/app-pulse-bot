use sqlx::{pool::PoolConnection, sqlite::SqliteConnectOptions, Sqlite, SqlitePool};
use sqlx_migrator::{Migrate, Migrator, Plan};

use common::types::Id;

mod app;
mod migrations;
pub mod models;
mod source;
mod user;

#[cfg(test)]
mod tests;

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
        let pool = SqlitePool::connect_with(SqliteConnectOptions::new().filename(path).create_if_missing(true)).await?;

        Self::migrate(pool.acquire().await?).await?;

        sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await?;

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
        Ok(
            sqlx::query_as::<_, models::fetch::Count>(&format!("select count(*) as count {sql_predicate}"))
                .fetch_one(&self.pool)
                .await?
                .count,
        )
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
