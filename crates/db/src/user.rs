use common::{
    types::{Id, UserId},
    UnixDateTime,
};

use crate::{models, IgnoreNotFound};

use super::{Result, APP_TABLE, DB, SOURCE_TABLE, USER_SUBSCRIBE_TABLE, USER_TABLE, USER_UPDATE_TABLE};

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
    /// Select subscribed and not yet notified users for specific source. Does
    /// not check last_updated_version
    pub async fn select_users_to_notify(&self, source_id: Id, app_id: Id) -> Result<Vec<models::User>> {
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
               and (
                 a.last_updated_at = 0
                 or a.last_updated_at > u.last_notified_at
               )",
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
    pub(crate) async fn select_users_to_notify_about_bot_update_impl(&self, version: u32) -> Result<Vec<models::User>> {
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
    pub async fn save_user_username(&self, user_id: impl Into<UserId>, username: &str) -> Result<()> {
        self.save_user_string_table(user_id, "username", username).await
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
        source_id: Id,
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
        .bind(source_id)
        .bind(update.subscribed())
        .execute(&self.pool)
        .await?;

        log::debug!("user subscribe saved");
        Ok(())
    }
    /// Set `last_notified_at` for all users, subscribed to source
    pub async fn save_all_users_last_notified(&self, source_id: Id, last_notified_at: UnixDateTime) -> Result<()> {
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
        self.save_user_string_table(user_id, "last_version_notified", &common::version().to_string())
            .await
    }
    pub async fn save_user_bot_blocked(&self, user_id: impl Into<UserId>, blocked: bool) -> Result<()> {
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
    pub(crate) async fn save_user_version_notified_impl(&self, user_id: impl Into<UserId>, version: u32) -> Result<()> {
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
