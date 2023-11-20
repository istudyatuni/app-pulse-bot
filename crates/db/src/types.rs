use std::fmt::Display;

use teloxide::types::{ChatId as TgChatId, UserId as TgUserId};

pub type Id = i64;

#[derive(Debug, Default, Clone, Copy)]
pub struct UserId(pub u64);

impl Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Default, Clone, Copy)]
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
    i64 => ChatId: v => Self(v),

    ChatId => UserId: v => Self(v.0 as _),
    UserId => ChatId: v => Self(v.0 as _),

    ChatId => TgChatId: v => Self(v.0),
    TgChatId => ChatId: v => Self(v.0),

    UserId => TgUserId: v => Self(v.0),
    TgUserId => UserId: v => Self(v.0),

    UserId => TgChatId: v => Self(v.0 as _),
    TgChatId => UserId: v => Self(v.0 as _),

    ChatId => Id: v => v.0,
    Id => UserId: v => Self(v as _),
    UserId => Id: v => v.0 as _,
);
