#![allow(unused)]

use std::path::PathBuf;

use anyhow::Result;
use camino::Utf8PathBuf;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::sql::Id;
use surrealdb::Surreal;

pub(crate) mod models;

const USER_TABLE: &str = "user";
const USER_UPDATE_TABLE: &str = "user_update";

const SCHEMA: &str = r#"
DEFINE TABLE user SCHEMAFULL;

DEFINE FIELD user_id ON TABLE user TYPE int;
DEFINE INDEX user_id_index ON TABLE user COLUMNS user_id UNIQUE;

--

DEFINE TABLE user_update SCHEMAFULL;

DEFINE FIELD user_id ON TABLE user_update TYPE int;
DEFINE INDEX user_id_index ON TABLE user_update COLUMNS user_id UNIQUE;

DEFINE FIELD app_id ON TABLE user_update TYPE string;
DEFINE INDEX app_id_index ON TABLE user_update COLUMNS app_id UNIQUE;
DEFINE FIELD should_notify ON TABLE user_update TYPE string;
"#;

#[derive(Debug, Clone)]
pub(crate) struct DB {
    conn: Surreal<Db>,
}

impl DB {
    pub(crate) async fn init(path: &str) -> Result<Self> {
        let path = Utf8PathBuf::from(".")
            .canonicalize_utf8()
            .expect("failed to find absolute path of db")
            .join(path);
        let db = Surreal::new::<RocksDb>(path.as_str()).await?;
        db.use_ns("dev").use_db("dev").await?;

        db.query(SCHEMA).await?;

        Ok(Self { conn: db })
    }
    pub(crate) async fn save_user(&self, user_id: i64) -> Result<()> {
        log::debug!("saving user {user_id}");
        let _: Option<models::User> = self
            .conn
            .create((USER_TABLE, user_id))
            .content(models::User::new(user_id))
            .await?;
        Ok(())
    }
    pub(crate) async fn select_user(&self, user_id: i64) -> Result<Option<models::User>> {
        log::debug!("select user {user_id}");
        Ok(self.conn.select((USER_TABLE, user_id)).await?)
    }
    pub(crate) async fn select_users(&self) -> Result<Vec<models::User>> {
        log::info!("select users");
        Ok(self.conn.select(USER_TABLE).await?)
    }
    pub(crate) async fn should_notify_user(
        &self,
        user_id: i64,
        app_id: &str,
    ) -> models::ShouldNotify {
        log::debug!("getting user preference");
        models::ShouldNotify::Unspecified
    }
    pub(crate) fn add_app(&self, name: &str, source_id: &str) {
        log::debug!("saving app {name}");
    }
}
