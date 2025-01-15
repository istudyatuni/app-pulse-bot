use teloxide::types::{ChatId as TgChatId, Recipient, UserId as TgUserId};

pub type Id = i64;

#[derive(Debug, Default, Clone, Copy)]
pub struct UserId(pub u64);

#[derive(Debug, Default, Clone, Copy)]
pub struct ChatId(pub i64);

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

    ChatId => Recipient: v => Recipient::Id(v.into()),
    UserId => Recipient: v => Recipient::Id(v.into()),

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

/// Simple [`std::fmt::Display`] implementation
macro_rules! display {
    ($($from:ty : $self:ident => $value:expr),* $(,)?) => {
        $(impl ::std::fmt::Display for $from {
            fn fmt(&$self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                $value.fmt(f)
            }
        })*
    };
}

display!(
    ChatId: self => self.0,
    UserId: self => self.0,
);
