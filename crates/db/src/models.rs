use sqlx::{sqlite::SqliteRow, Row};

use common::{DateTime, UnixDateTime};

use crate::SOURCE_ID;

use super::types::{Id, UserId};

#[derive(Debug, Default, sqlx::FromRow)]
pub struct User {
    user_id: Id,
    lang: String,
    last_notified_at: UnixDateTime,
    bot_blocked: bool,
}

impl User {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id: user_id.into(),
            lang: i18n::DEFAULT_USER_LANG.to_string(),
            last_notified_at: DateTime::now(),
            bot_blocked: false,
        }
    }
    pub fn new_with_lang(user_id: UserId, lang: impl Into<String>) -> Self {
        Self {
            user_id: user_id.into(),
            lang: lang.into(),
            last_notified_at: DateTime::now(),
            bot_blocked: false,
        }
    }
    pub fn user_id(&self) -> Id {
        self.user_id
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
}

#[derive(Debug, Default)]
pub struct UserUpdate {
    user_id: Id,
    #[allow(unused)]
    source_id: Id,
    app_id: String,
    should_notify: ShouldNotify,
}

impl UserUpdate {
    pub fn new(user_id: Id, app_id: &str, should_notify: ShouldNotify) -> Self {
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
    pub fn should_notify(&self) -> ShouldNotify {
        self.should_notify
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ShouldNotify {
    #[default]
    Unspecified,
    Notify,
    Ignore,
}

impl ShouldNotify {
    pub fn to_db(&self) -> Option<bool> {
        match self {
            Self::Unspecified => None,
            Self::Notify => Some(true),
            Self::Ignore => Some(false),
        }
    }
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
