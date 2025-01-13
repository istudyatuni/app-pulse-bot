use sqlx::{sqlite::SqliteRow, Row};
use teloxide::utils::markdown::user_mention;

use common::{DateTime, UnixDateTime};

use crate::{
    types::{Id, UserId},
    SOURCE_ID,
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
    #[allow(unused)]
    source_id: Id,
    app_id: String,
    should_notify: Option<ShouldNotify>,
}

impl UserUpdate {
    pub fn new(user_id: Id, app_id: &str, should_notify: Option<ShouldNotify>) -> Self {
        Self {
            user_id,
            source_id: SOURCE_ID,
            app_id: app_id.to_string(),
            should_notify,
        }
    }
    pub fn user_id(&self) -> Id {
        self.user_id
    }
    pub fn app_id(&self) -> &str {
        self.app_id.as_str()
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
    #[allow(unused)]
    source_id: Id,
    subscribed: bool,
}

impl UserSubscribe {
    pub fn new(user_id: UserId, subscribed: bool) -> Self {
        Self {
            user_id: user_id.into(),
            source_id: SOURCE_ID,
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
    app_id: String,
    source_id: Id,
    name: String,
    last_updated_at: UnixDateTime,
}

impl App {
    pub fn new(app_id: &str, source_id: Id, name: &str, last_updated_at: UnixDateTime) -> Self {
        Self {
            app_id: app_id.to_string(),
            source_id,
            name: name.to_string(),
            last_updated_at,
        }
    }
    pub fn app_id(&self) -> &str {
        &self.app_id
    }
    pub fn source_id(&self) -> Id {
        self.source_id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn last_updated_at(&self) -> UnixDateTime {
        self.last_updated_at
    }
}

#[allow(unused)]
#[derive(Debug, sqlx::FromRow)]
pub struct Source {
    source_id: Id,
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
    use crate::types::Id;

    #[derive(sqlx::FromRow)]
    pub(crate) struct FetchSourceId {
        pub source_id: Id,
    }

    #[derive(sqlx::FromRow)]
    pub(crate) struct FetchCount {
        pub count: u32,
    }
}
