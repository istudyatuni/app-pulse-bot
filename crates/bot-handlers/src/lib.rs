mod bot_admin_messages;
mod bot_callback;
mod bot_messages;
mod callback;
mod commands;
mod keyboards;
mod updates_notify;
mod user;
mod utils;

// flags is at the start of message: {flag}:{payload}
const NOTIFY_FLAG: &str = "notify";
const SET_LANG_FLAG: &str = "lang";

// payload tokens: {notify-flag}:{app-id}:{token}
const IGNORE_TOKEN: &str = "ignore";
const NOTIFY_TOKEN: &str = "notify";

pub use bot_admin_messages::admin_command_handler;
pub use bot_callback::callback_handler;
pub use bot_messages::{command_handler, message_handler};
pub use commands::{AdminCommand, Command};
pub use updates_notify::start_updates_notify_job;
pub use user::run_collect_user_names_job;

pub(crate) use i18n::{tr, DEFAULT_USER_LANG};
