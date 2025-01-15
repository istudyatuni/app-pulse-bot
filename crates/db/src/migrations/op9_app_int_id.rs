use std::collections::HashMap;

use sqlx_migrator::error::Error;
use sqlx_migrator::operation::Operation;

use common::UnixDateTime;

use crate::{models::ShouldNotify, types::Id, APP_TABLE, USER_UPDATE_TABLE};

/// Use `int` for `app_id`
///
/// Migrate `app` table
///
/// - remove `name`
/// - rename `app_id` to `name`
/// - create `app_id` `int`
/// - fill `app_id` with increasing numbers
///
/// Migrate other tables
///
/// - `user_update.app_id`
pub(crate) struct Operation9AppIntId;

#[derive(Debug, sqlx::FromRow)]
struct AppOld {
    app_id: String,
    source_id: Id,
    last_updated_at: UnixDateTime,
}

#[derive(Debug, sqlx::FromRow)]
struct UserUpdateOld {
    user_id: Id,
    source_id: Id,
    app_id: String,
    should_notify: Option<ShouldNotify>,
}

#[async_trait::async_trait]
impl Operation<sqlx::Sqlite> for Operation9AppIntId {
    async fn up(&self, connection: &mut sqlx::SqliteConnection) -> Result<(), Error> {
        const APP_TMP: &str = "app_";
        const USER_UPDATE_TMP: &str = "user_update_";

        // migrate app

        sqlx::query(&format!("alter table {APP_TABLE} rename to {APP_TMP}"))
            .execute(&mut *connection)
            .await?;
        // todo: app_id is now enough for primary key
        sqlx::query(&format!(
            "create table {APP_TABLE} (
             app_id int not null,
             source_id int not null,
             name text,
             last_updated_at int default 0, -- unix time

             primary key (app_id, source_id)
        )"
        ))
        .execute(&mut *connection)
        .await?;

        let apps = sqlx::query_as::<_, AppOld>(&format!("select * from {APP_TMP}"))
            .fetch_all(&mut *connection)
            .await?;
        let app_id_map: HashMap<String, Id> = apps
            .iter()
            .map(|a| a.app_id.clone())
            .zip(1 as Id..)
            .collect();

        for app in apps {
            sqlx::query(&format!(
                "insert into {APP_TABLE}
                 (app_id, source_id, name, last_updated_at)
                 values (?, ?, ?, ?)"
            ))
            .bind(app_id_map.get(&app.app_id).expect("app_id should be known"))
            .bind(app.source_id)
            .bind(&app.app_id)
            .bind(app.last_updated_at)
            .execute(&mut *connection)
            .await?;
        }
        sqlx::query(&format!("drop table {APP_TMP}"))
            .execute(&mut *connection)
            .await?;

        // migrate user_update

        sqlx::query(&format!(
            "alter table {USER_UPDATE_TABLE} rename to {USER_UPDATE_TMP}"
        ))
        .execute(&mut *connection)
        .await?;
        sqlx::query(&format!(
            "create table {USER_UPDATE_TABLE} (
             user_id int not null,
             source_id int not null,
             app_id int not null,
             should_notify int not null, -- bool

             primary key (user_id, source_id, app_id)
        )"
        ))
        .execute(&mut *connection)
        .await?;
        let updates =
            sqlx::query_as::<_, UserUpdateOld>(&format!("select * from {USER_UPDATE_TMP}"))
                .fetch_all(&mut *connection)
                .await?;

        for update in updates {
            sqlx::query(&format!(
                "insert into {USER_UPDATE_TABLE}
                 (user_id, source_id, app_id, should_notify)
                 values (?, ?, ?, ?)"
            ))
            .bind(update.user_id)
            .bind(update.source_id)
            .bind(
                app_id_map
                    .get(&update.app_id)
                    .expect("app_id should be known"),
            )
            .bind(update.should_notify)
            .execute(&mut *connection)
            .await?;
        }
        sqlx::query(&format!("drop table {USER_UPDATE_TMP}"))
            .execute(&mut *connection)
            .await?;

        Ok(())
    }
    async fn down(&self, connection: &mut sqlx::SqliteConnection) -> Result<(), Error> {
        sqlx::query("DROP TABLE sample").execute(connection).await?;
        Ok(())
    }
}
