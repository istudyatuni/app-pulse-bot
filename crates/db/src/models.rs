use sqlx::{sqlite::SqliteRow, Row};

use super::types::{Id, UserId};

#[derive(Debug, Default, sqlx::FromRow)]
pub struct User {
    user_id: Id,
    lang: String,
}

impl User {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id: user_id.into(),
            lang: i18n::DEFAULT_USER_LANG.to_string(),
        }
    }
    pub fn user_id(&self) -> Id {
        self.user_id
    }
    pub fn lang(&self) -> &str {
        self.lang.as_str()
    }
}

/*#[derive(Debug)]
struct App {
    id: Id,
    app_id: String,
}*/

#[derive(Debug, Default)]
pub struct UserUpdate {
    user_id: Id,
    app_id: String,
    should_notify: ShouldNotify,
}

impl UserUpdate {
    pub fn new(user_id: Id, app_id: &str, should_notify: ShouldNotify) -> Self {
        Self {
            user_id,
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

#[derive(Debug, Default, Clone, Copy)]
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
