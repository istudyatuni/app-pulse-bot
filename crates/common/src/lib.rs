use std::{fmt::Display, sync::LazyLock};

use chrono::Utc;
use dotenvy_macro::dotenv;

static VERSION: LazyLock<u32> = LazyLock::new(|| {
    env!("BOT_VERSION")
        .to_string()
        .parse()
        .expect("invalid bot version")
});

/// Get bot version
pub fn version() -> u32 {
    *VERSION
}

static ADMIN_CHAT_ID: LazyLock<Option<i64>> = LazyLock::new(|| {
    let s = dotenv!("ADMIN_CHAT_ID");
    if s.is_empty() {
        None
    } else {
        Some(s.parse().expect("invalid ADMIN_CHAT_ID"))
    }
});

pub fn admin_chat_id() -> Option<i64> {
    *ADMIN_CHAT_ID
}

pub type UnixDateTime = i64;

pub struct DateTime;

impl DateTime {
    pub fn now() -> UnixDateTime {
        Utc::now().timestamp()
    }
}

pub trait LogError {
    fn log_error(&self);
    fn log_error_with_msg(&self, msg: &str);
    fn eprint_error_with_msg(&self, msg: &str);
}

impl<T, E> LogError for Result<T, E>
where
    E: Display,
{
    fn log_error(&self) {
        if let Err(e) = self {
            log::error!("{e}")
        }
    }
    fn log_error_with_msg(&self, msg: &str) {
        if let Err(e) = self {
            log::error!("{msg}: {e}")
        }
    }
    fn eprint_error_with_msg(&self, msg: &str) {
        if let Err(e) = self {
            eprintln!("{msg}: {e}")
        }
    }
}
