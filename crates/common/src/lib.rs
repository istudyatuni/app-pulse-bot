use std::fmt::Display;

use chrono::Utc;
use lazy_static::lazy_static;

lazy_static! {
    static ref VERSION: u32 = env!("BOT_VERSION")
        .to_string()
        .parse()
        .expect("invalid bot version");
}

/// Get bot version
pub fn version() -> u32 {
    *VERSION
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
