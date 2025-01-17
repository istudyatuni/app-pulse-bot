use common::{types::Id, UnixDateTime};

use crate::{models, IgnoreNotFound};

use super::{Result, APP_TABLE, DB, USER_SUBSCRIBE_TABLE, USER_TABLE, USER_UPDATE_TABLE};

impl DB {
    pub async fn get_app(&self, app_id: Id) -> Result<Option<models::App>> {
        log::debug!("select app {app_id}");
        let res = sqlx::query_as::<_, models::App>(&format!(
            "select * from {APP_TABLE}
             where app_id = ?"
        ))
        .bind(app_id)
        .fetch_one(&self.pool)
        .await;

        Ok(res.ignore_not_found()?)
    }
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
        last_updated_version: &str,
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
