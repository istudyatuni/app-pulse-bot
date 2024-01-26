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

pub const TG_LOG_TARGET: &str = "tg-notify";

pub type UnixDateTime = i64;

pub struct DateTime;

impl DateTime {
    pub fn now() -> UnixDateTime {
        Utc::now().timestamp()
    }
}
