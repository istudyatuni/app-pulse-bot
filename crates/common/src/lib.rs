use chrono::Utc;

pub type UnixDateTime = i64;

pub struct DateTime;

impl DateTime {
    pub fn now() -> UnixDateTime {
        Utc::now().timestamp()
    }
}
