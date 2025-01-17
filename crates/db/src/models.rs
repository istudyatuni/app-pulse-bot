use sqlx::{sqlite::SqliteRow, Row};

use common::{
    markdown::user_mention,
    types::{AppId, Id, SourceId, UserId},
    DateTime, UnixDateTime,
};

#[derive(Debug, Default, sqlx::FromRow, bon::Builder)]
pub struct User {
    /// User ID
    user_id: Id,

    /// Username
    username: Option<String>,

    /// First name + last name
    name: Option<String>,

    /// Language, selected by user
    #[builder(default = i18n::DEFAULT_USER_LANG.to_string())]
    lang: String,

    /// When user was last notified
    #[builder(default = DateTime::now())]
    last_notified_at: UnixDateTime,

    /// Is bot blocked by user
    #[builder(default)]
    bot_blocked: bool,
}

impl User {
    #[cfg(test)]
    pub fn new(user_id: UserId) -> Self {
        Self::builder().user_id(user_id.into()).build()
    }
    pub fn user_id(&self) -> Id {
        self.user_id
    }
    pub fn tg_user_id(&self) -> UserId {
        self.user_id.into()
    }
    pub fn lang(&self) -> &str {
        self.lang.as_str()
    }
    pub fn last_notified_at(&self) -> UnixDateTime {
        self.last_notified_at
    }
    pub fn bot_blocked(&self) -> bool {
        self.bot_blocked
    }
    /// Display user name. Can contain link to user, which is only works
    /// inside inline links, so message should be set to markdown
    pub fn display(&self) -> String {
        match (&self.username, &self.name) {
            (Some(username), Some(name)) => format!("@{username} ({name})"),
            (Some(username), None) => format!("@{username}"),
            (None, Some(name)) => user_mention(self.tg_user_id().into(), name),
            (None, None) => user_mention(self.tg_user_id().into(), &self.user_id.to_string()),
        }
    }
}

#[derive(Debug, Default)]
pub struct UserUpdate {
    user_id: Id,
    app_id: AppId,
    should_notify: Option<ShouldNotify>,
}

impl UserUpdate {
    pub fn new(user_id: Id, app_id: AppId, should_notify: Option<ShouldNotify>) -> Self {
        Self {
            user_id,
            app_id,
            should_notify,
        }
    }
    pub fn user_id(&self) -> Id {
        self.user_id
    }
    pub fn app_id(&self) -> AppId {
        self.app_id
    }
    pub fn should_notify(&self) -> Option<ShouldNotify> {
        self.should_notify
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[repr(u8)]
pub enum ShouldNotify {
    Ignore = 0,
    Notify,
}

impl sqlx::FromRow<'_, SqliteRow> for ShouldNotify {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let res: u8 = row.try_get("should_notify")?;
        if res == 0 {
            Ok(Self::Ignore)
        } else {
            Ok(Self::Notify)
        }
    }
}

#[derive(Debug, Default)]
pub struct UserSubscribe {
    user_id: Id,
    #[expect(unused)]
    source_id: SourceId,
    subscribed: bool,
}

impl UserSubscribe {
    pub fn new(user_id: UserId, subscribed: bool) -> Self {
        Self {
            user_id: user_id.into(),
            source_id: 0.into(),
            subscribed,
        }
    }
    pub fn user_id(&self) -> Id {
        self.user_id
    }
    pub fn subscribed(&self) -> bool {
        self.subscribed
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct App {
    app_id: AppId,
    source_id: SourceId,
    name: String,
    last_updated_at: UnixDateTime,
    last_updated_version: Option<String>,
}

impl App {
    pub fn app_id(&self) -> AppId {
        self.app_id
    }
    pub fn source_id(&self) -> SourceId {
        self.source_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn last_updated_at(&self) -> UnixDateTime {
        self.last_updated_at
    }
    pub fn last_updated_version(&self) -> Option<&str> {
        self.last_updated_version.as_deref()
    }
}

#[allow(unused)]
#[derive(Debug, sqlx::FromRow)]
pub struct Source {
    source_id: SourceId,
    name: String,
    last_updated_at: UnixDateTime,
}

impl Source {
    pub fn last_updated_at(&self) -> UnixDateTime {
        self.last_updated_at
    }
}

#[derive(Debug)]
pub struct Stats {
    pub apps: u32,
    pub sources: u32,
    pub users: u32,
    pub blocked_users: u32,
}

/// Struct helpers for extracting partial structs
pub mod fetch {
    #[derive(sqlx::FromRow)]
    pub(crate) struct SourceId {
        pub source_id: common::types::SourceId,
    }

    #[derive(sqlx::FromRow)]
    pub(crate) struct AppId {
        pub app_id: common::types::AppId,
    }

    #[derive(sqlx::FromRow)]
    pub(crate) struct Count {
        pub count: u32,
    }
}
