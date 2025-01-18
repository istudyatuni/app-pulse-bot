use common::{
    types::SourceId,
    UnixDateTime,
};

use crate::{models, IgnoreNotFound};

use super::{Result, DB, SOURCE_TABLE};

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
            },
        }
    }
    pub async fn save_source_updated_at(&self, source_id: SourceId, last_updated_at: UnixDateTime) -> Result<()> {
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
    pub async fn get_source_id_by_source_name(&self, source_name: &str) -> Result<Option<SourceId>> {
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
    /*pub async fn get_source(&self, source_id: SourceId) -> Result<Option<models::Source>> {
        log::debug!("select source last_updated_at");
        Ok(sqlx::query_as::<_, models::Source>(&format!(
            "select * from {SOURCE_TABLE}
             where source_id = ?"
        ))
        .bind(source_id)
        .fetch_optional(&self.pool)
        .await?)
    }*/
}
