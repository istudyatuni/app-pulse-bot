use std::fmt::Display;

use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Number, Value};
use teloxide::types::{ChatId as TgChatId, UserId as TgUserId};

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(transparent)]
pub struct UserId(pub u64);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
#[serde(transparent)]
pub struct ChatId(pub i64);

impl Display for ChatId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! cast {
    ($($from:ty => $to:ty : $value:ident => $convert:expr),* $(,)?) => {
        $(impl From<$from> for $to {
            fn from($value: $from) -> Self {
                $convert
            }
        })*
    };
}

cast!(
    ChatId => UserId: v => Self(v.0 as _),
    UserId => ChatId: v => Self(v.0 as _),

    ChatId => TgChatId: v => Self(v.0),
    TgChatId => ChatId: v => Self(v.0),

    UserId => TgUserId: v => Self(v.0),
    TgUserId => UserId: v => Self(v.0),

    UserId => TgChatId: v => Self(v.0 as _),
    TgChatId => UserId: v => Self(v.0 as _),

    ChatId => Id: v => Self::Number(v.0),
    UserId => Id: v => Self::Number(v.0 as _),
    UserId => Value: v => Self::Number(Number::Int(v.0 as _)),
);
