use std::fmt::Display;

use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Number, Value};
use teloxide::types::{ChatId as TgChatId, UserId as TgUserId};

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(transparent)]
pub(crate) struct UserId(pub(crate) u64);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<TgUserId> for UserId {
    fn from(value: TgUserId) -> Self {
        Self(value.0)
    }
}

impl From<UserId> for TgUserId {
    fn from(value: UserId) -> Self {
        Self(value.0)
    }
}

impl From<UserId> for TgChatId {
    fn from(value: UserId) -> Self {
        Self(value.0 as _)
    }
}

impl From<UserId> for ChatId {
    fn from(value: UserId) -> Self {
        Self(value.0 as _)
    }
}

impl From<UserId> for Id {
    fn from(value: UserId) -> Self {
        Self::Number(value.0 as _)
    }
}

impl From<UserId> for Value {
    fn from(value: UserId) -> Self {
        Self::Number(Number::Int(value.0 as _))
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(transparent)]
pub(crate) struct ChatId(pub(crate) i64);

impl Display for ChatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<TgChatId> for ChatId {
    fn from(value: TgChatId) -> Self {
        Self(value.0)
    }
}

impl From<TgChatId> for UserId {
    fn from(value: TgChatId) -> Self {
        Self(value.0 as _)
    }
}

impl From<ChatId> for TgChatId {
    fn from(value: ChatId) -> Self {
        Self(value.0)
    }
}

impl From<ChatId> for UserId {
    fn from(value: ChatId) -> Self {
        Self(value.0 as _)
    }
}

impl From<ChatId> for Id {
    fn from(value: ChatId) -> Self {
        Self::Number(value.0)
    }
}
