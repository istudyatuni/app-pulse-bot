use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

use super::types::UserId;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    user_id: UserId,
    lang: String,
}

impl User {
    pub fn new(user_id: UserId) -> Self {
        Self {
            user_id,
            lang: i18n::DEFAULT_USER_LANG.to_string(),
            ..Default::default()
        }
    }
    pub fn user_id(&self) -> UserId {
        self.user_id
    }
    pub fn lang(&self) -> &str {
        self.lang.as_str()
    }
}

/*#[derive(Debug, Serialize, Deserialize)]
struct App {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    app_id: String,
}*/

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    user_id: UserId,
    app_id: String,
    should_notify: ShouldNotify,
}

impl UserUpdate {
    pub fn new(user_id: UserId, app_id: &str, should_notify: ShouldNotify) -> Self {
        Self {
            user_id,
            app_id: app_id.to_string(),
            should_notify,
            ..Default::default()
        }
    }
    pub fn should_notify(&self) -> ShouldNotify {
        self.should_notify
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ShouldNotify {
    #[default]
    Unspecified,
    Notify,
    Ignore,
}
