use chrono::Utc;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub type UnixDateTime = i64;

pub struct DateTime;

impl DateTime {
    pub fn now() -> UnixDateTime {
        Utc::now().timestamp()
    }
}
