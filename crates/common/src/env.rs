use std::sync::LazyLock;

use dotenvy_macro::dotenv;

pub(crate) static VERSION: LazyLock<u32> =
    LazyLock::new(|| env!("BOT_VERSION").to_string().parse().expect("invalid bot version"));

/// Get bot version
pub fn version() -> u32 {
    *VERSION
}

pub(crate) static ADMIN_CHAT_ID: LazyLock<Option<i64>> = LazyLock::new(|| {
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

pub fn is_admin_chat_id(id: i64) -> bool {
    admin_chat_id().is_some_and(|i| i == id)
}

pub const NIXPKGS_PASS: &str = dotenv!("NIXPKGS_PASS");
