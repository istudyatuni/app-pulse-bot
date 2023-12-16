use chrono::Utc;

pub type UnixDateTime = i64;

pub struct DateTime;

impl DateTime {
    pub fn now() -> UnixDateTime {
        Utc::now().timestamp()
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
