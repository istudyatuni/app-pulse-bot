use common::{types::SourceId, UnixDateTime};

use crate::{models, IgnoreNotFound};

use super::{Result, DB, SOURCE_TABLE};

impl DB {
    /// Add source, ignore if it's already exists. Always updates description
    pub async fn add_source_or_ignore(&self, name: &str, description: &str) -> Result<()> {
        // used to check failed constraint
        const SOURCE_NAME_UNIQ_ERR: &str = "UNIQUE constraint failed: source.name";

        log::debug!("adding source {name}");
        let res = sqlx::query(&format!(
            "insert into {SOURCE_TABLE} (source_id, name, description)
             values ((select max(source_id) from {SOURCE_TABLE}) + 1, ?, ?)"
        ))
        .bind(name)
        .bind(description)
        .execute(&self.pool)
        .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(e) = e.as_database_error() {
                    if e.is_unique_violation() && e.message() == SOURCE_NAME_UNIQ_ERR {
                        log::debug!("source {name} exists, ignoring");
                        return self.save_source_description(name, description).await;
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
    /// Update source description. Use source's name because it's unique anyway
    pub async fn save_source_description(&self, source_name: &str, description: &str) -> Result<()> {
        log::debug!("save source description: {description}");
        sqlx::query(&format!(
            "update {SOURCE_TABLE}
             set description = ?
             where name = ?"
        ))
        .bind(description)
        .bind(source_name)
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
    pub async fn get_sources(&self) -> Result<Vec<models::Source>> {
        log::debug!("select sources");
        Ok(
            sqlx::query_as::<_, models::Source>(&format!("select * from {SOURCE_TABLE} order by name"))
                .fetch_all(&self.pool)
                .await?,
        )
    }
}
