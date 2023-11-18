mod bot_callback;
mod bot_messages;
mod keyboards;
mod updates_notify;

const NOTIFY_TOKEN: &str = "notify";
const IGNORE_TOKEN: &str = "ignore";

pub use bot_callback::callback_handler;
pub use bot_messages::{message_handler, Command};
pub use updates_notify::start_updates_notify_job;

use interly::localize;

#[localize]
pub(crate) struct Localize;

// temporary
pub(crate) const USER_LANG: &str = "en";
