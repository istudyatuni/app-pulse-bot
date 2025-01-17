use std::ops::RangeFrom;

use common::types::{Id, SourceId};
use models::ShouldNotify;

use super::*;

struct Timer {
    iter: Box<dyn Iterator<Item = i64>>,
}

impl Timer {
    fn new() -> Self {
        Self {
            iter: Box::new(RangeFrom { start: 0 }),
        }
    }
    fn skip(&mut self, count: u32) {
        for _ in 0..count {
            self.iter.next();
        }
    }
    fn next<T: From<i64>>(&mut self) -> T {
        self.iter.next().unwrap().into()
    }
}

async fn prepare() -> Result<DB> {
    common::init_logger();

    const DIR: &str = "target/test-db";
    const REL_PATH: &str = "../..";
    std::fs::create_dir_all(DIR).unwrap();
    let path_fmt = |id| format!("{REL_PATH}/{DIR}/{id}.db");

    // in hope that no single test can call this at the same time
    let mut id = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let mut file = path_fmt(id);
    while std::fs::exists(&file).unwrap() {
        id += 1;
        file = path_fmt(id);
    }

    log::debug!("using db at {DIR}/{id}.db");
    DB::init(&file).await
}

#[tokio::test]
async fn test_select_users_to_notify() -> Result<()> {
    common::init_logger();

    const SOURCE_ID: SourceId = SourceId::new(1);

    let db = prepare().await?;
    let mut timer = Timer::new();
    timer.skip(1);

    let app_id = db.add_app(SOURCE_ID, "").await?;
    db.save_app_last_updated_at(app_id, timer.next()).await?;

    // there are 2 users
    for u in [1, 2] {
        db.add_user_simple(u).await?;
        db.save_user_subscribed_to_source(u, SOURCE_ID, true).await?;
    }

    // source updated before one of users was notified
    db.save_source_updated_at(SOURCE_ID, timer.next()).await?;
    db.save_user_last_notified(1, timer.next()).await?;

    let users = db.select_users_to_notify(SOURCE_ID, app_id).await?;
    assert_eq!(users.len(), 1);

    Ok(())
}

#[tokio::test]
async fn test_no_select_users_to_notify() -> Result<()> {
    const SOURCE_ID: SourceId = SourceId::new(1);

    let db = prepare().await?;
    let mut timer = Timer::new();
    timer.skip(1);

    // todo: seems that app in db is not required, and result of
    // select_users_to_notify is still empty
    let app_id = db.add_app(SOURCE_ID, "").await?;
    db.save_app_last_updated_at(app_id, timer.next()).await?;

    // there is one user
    db.add_user_simple(1).await?;
    db.save_user_subscribed_to_source(1, SOURCE_ID, true).await?;

    // source updated before user was notified
    db.save_source_updated_at(SOURCE_ID, timer.next()).await?;
    db.save_user_last_notified(1, timer.next()).await?;

    let users = db.select_users_to_notify(SOURCE_ID, app_id).await?;
    assert!(users.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_select_users_to_notify_about_bot_update() -> Result<()> {
    let db = prepare().await?;
    let mut timer = Timer::new();
    timer.skip(1);

    // there is one user
    db.add_user_simple(1).await?;
    db.save_user_version_notified_impl(1, 0).await?;

    for v in 1..20 {
        let users = db.select_users_to_notify_about_bot_update_impl(v + 1).await?;
        assert_eq!(users.len(), 1, "notify about version {}", v + 1);
        db.save_user_version_notified_impl(1, v).await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_select_apps_to_check_updates_empty() -> Result<()> {
    const SOURCE_ID: SourceId = SourceId::new(1);
    const USER_ID: Id = 1;

    let db = prepare().await?;
    let mut timer = Timer::new();
    timer.skip(1);

    let app_id = db.add_app(SOURCE_ID, "").await?;

    // there is one user
    db.add_user_simple(USER_ID).await?;
    db.save_user_subscribed_to_source(USER_ID, SOURCE_ID, false).await?;
    db.save_should_notify_user(USER_ID, app_id, ShouldNotify::Notify)
        .await?;

    let apps = db.get_apps_to_check_updates(SOURCE_ID).await?;
    assert!(apps.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_select_apps_to_check_updates_empty_user_blocked() -> Result<()> {
    const SOURCE_ID: SourceId = SourceId::new(1);
    const USER_ID: Id = 1;

    let db = prepare().await?;
    let mut timer = Timer::new();
    timer.skip(1);

    let app_id = db.add_app(SOURCE_ID, "").await?;

    // there is one user
    db.add_user_simple(USER_ID).await?;
    db.save_user_bot_blocked(USER_ID, true).await?;
    db.save_user_subscribed_to_source(USER_ID, SOURCE_ID, true).await?;
    db.save_should_notify_user(USER_ID, app_id, ShouldNotify::Notify)
        .await?;

    let apps = db.get_apps_to_check_updates(SOURCE_ID).await?;
    assert!(apps.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_select_apps_to_check_updates() -> Result<()> {
    const SOURCE_ID: SourceId = SourceId::new(1);
    const USER_ID: Id = 1;

    let db = prepare().await?;
    let mut timer = Timer::new();
    timer.skip(1);

    // there is one user
    db.add_user_simple(USER_ID).await?;
    db.save_user_subscribed_to_source(USER_ID, SOURCE_ID, true).await?;

    let app_id = db.add_app(SOURCE_ID, "").await?;
    db.save_should_notify_user(USER_ID, app_id, ShouldNotify::Notify)
        .await?;

    let apps = db.get_apps_to_check_updates(SOURCE_ID).await?;
    assert_eq!(apps.len(), 1);

    Ok(())
}
