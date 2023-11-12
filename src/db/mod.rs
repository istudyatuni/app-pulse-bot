#![allow(unused)]

#[derive(Debug, Clone)]
pub(crate) struct DB;

impl DB {
    pub(crate) fn new() -> Self {
        Self
    }
    pub(crate) fn save_user(&self, user_id: i64) {
        log::info!("saving user {user_id}");
    }
    pub(crate) fn should_notify_user(&self, user_id: i64, app_id: i64) -> ShouldNotify {
        log::info!("getting user preference");
        ShouldNotify::Unspecified
    }
    pub(crate) fn add_app(&self, name: &str, source_id: &str) {
        log::info!("saving app {name}");
    }
}

#[derive(Debug)]
pub(crate) enum ShouldNotify {
    Unspecified,
    Notify,
    Ignore,
}
