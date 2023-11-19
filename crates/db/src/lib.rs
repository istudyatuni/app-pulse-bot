use anyhow::Result;
use camino::Utf8PathBuf;
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::opt::PatchOp;
use surrealdb::sql::Value;
use surrealdb::Surreal;

pub mod models;
mod types;

use types::UserId;

const USER_TABLE: &str = "user";
const USER_UPDATE_TABLE: &str = "user_update";

const SCHEMA: &str = r#"
DEFINE TABLE user SCHEMAFULL;

DEFINE FIELD user_id ON TABLE user TYPE int;
DEFINE FIELD lang ON TABLE user TYPE string;
DEFINE INDEX user_id_index ON TABLE user COLUMNS user_id UNIQUE;

--

DEFINE TABLE user_update SCHEMAFULL;

DEFINE FIELD user_id ON TABLE user_update TYPE int;
DEFINE FIELD app_id ON TABLE user_update TYPE string;
DEFINE FIELD should_notify ON TABLE user_update TYPE string;

-- DEFINE INDEX user_app_id_index ON TABLE user_update COLUMNS user_id, app_id UNIQUE;
"#;

#[derive(Debug, Clone)]
pub struct DB {
    conn: Surreal<Db>,
}

impl DB {
    pub async fn init(path: &str) -> Result<Self> {
        let path = Utf8PathBuf::from(".")
            .canonicalize_utf8()
            .expect("failed to find absolute path of db")
            .join(path);
        let db = Surreal::new::<RocksDb>(path.as_str()).await?;
        db.use_ns("dev").use_db("dev").await?;

        db.query(SCHEMA).await?;

        Ok(Self { conn: db })
    }
    pub async fn save_user(&self, user_id: UserId) -> Result<()> {
        log::debug!("saving user {user_id}");
        let _: Option<models::User> = self
            .conn
            .create((USER_TABLE, user_id))
            .content(models::User::new(user_id))
            .await?;
        Ok(())
    }
    pub async fn select_user(&self, user_id: UserId) -> Result<Option<models::User>> {
        log::debug!("select user {user_id}");
        Ok(self.conn.select((USER_TABLE, user_id)).await?)
    }
    pub async fn select_users(&self) -> Result<Vec<models::User>> {
        log::info!("select users");
        Ok(self.conn.select(USER_TABLE).await?)
    }
    pub async fn save_should_notify_user(
        &self,
        user_id: UserId,
        app_id: &str,
        should_notify: models::ShouldNotify,
    ) -> Result<()> {
        let _: Option<models::UserUpdate> = match self
            .conn
            .create((USER_UPDATE_TABLE, make_user_update_id(user_id, app_id)))
            .content(models::UserUpdate::new(user_id, app_id, should_notify))
            .await
        {
            Ok(res) => res,
            Err(surrealdb::Error::Db(surrealdb::error::Db::RecordExists { .. })) => {
                self.update_should_notify_user(user_id, app_id, should_notify)
                    .await?;
                return Ok(());
            }
            e => e?,
        };
        log::debug!("user preference saved");
        Ok(())
    }
    pub async fn save_user_lang(&self, user_id: UserId, lang: &str) -> Result<()> {
        let _: Option<models::User> = self
            .conn
            .update((USER_TABLE, user_id))
            .patch(PatchOp::replace("/lang", lang))
            .await?;
        log::debug!("user lang updated");
        Ok(())
    }
    async fn update_should_notify_user(
        &self,
        user_id: UserId,
        app_id: &str,
        should_notify: models::ShouldNotify,
    ) -> Result<()> {
        let _: Option<models::UserUpdate> = self
            .conn
            .update((USER_UPDATE_TABLE, make_user_update_id(user_id, app_id)))
            .patch(PatchOp::replace("/should_notify", should_notify))
            .await?;
        log::debug!("user preference updated");
        Ok(())
    }
    pub async fn should_notify_user(
        &self,
        user_id: UserId,
        app_id: &str,
    ) -> Result<models::ShouldNotify> {
        log::debug!("getting user preference");
        let user_update: Option<models::UserUpdate> = self
            .conn
            .select((USER_UPDATE_TABLE, make_user_update_id(user_id, app_id)))
            .await?;
        Ok(user_update.map(|u| u.should_notify()).unwrap_or_default())
    }
    #[allow(unused)]
    pub fn add_app(&self, name: &str, source_id: &str) {
        log::debug!("saving app {name}");
    }
}

fn make_user_update_id(user_id: UserId, app_id: &str) -> Vec<Value> {
    vec![user_id.into(), app_id.into()]
}
