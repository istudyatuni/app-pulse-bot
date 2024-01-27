use std::thread;

use log::{kv::Key, Level, Metadata, Record};
use simplelog::SharedLogger;
use tokio::sync::mpsc::Sender;

use crate::{handlers::tg_logs::LogMessage, TG_LOG_ENABLED};

/// By default only error logs are sent. If [`crate::TG_LOG_ENABLED`] is
/// false, do not send anything
///
/// If called with "tg" key like `log::info!(tg = true; "..")`, will send not
/// only ERROR log.
///
/// - All error logs will be wrapped in markdown block with `[ERROR]` appended
///
/// If "tg" is set to `true`:
///
/// - All info logs will be sent as is
/// - All warn logs will contain `Warning: ` before message
///
/// All debug and trace messages are filtered.
#[derive(Debug)]
pub(crate) struct TgLogger {
    sender: Sender<LogMessage>,
    config: Config,
}

impl TgLogger {
    pub(crate) fn new(sx: Sender<LogMessage>, config: Config) -> Box<Self> {
        let s = Self { sender: sx, config };
        Box::new(s)
    }
}

impl log::Log for TgLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        TG_LOG_ENABLED && metadata.level() <= Level::Error
    }

    fn log(&self, record: &Record) {
        // search key "tg"
        let should_always_send = record
            .key_values()
            .get(Key::from_str("tg"))
            .is_some_and(|v| v.to_bool().is_some_and(|v| v));

        if self.enabled(record.metadata()) || should_always_send {
            let text = record.args().to_string();
            if self.config.is_should_ignore(&text) {
                return;
            }

            let level = record.metadata().level();
            let msg = if should_always_send {
                if level > Level::Error {
                    LogMessage::simple(text)
                } else {
                    LogMessage::simple_with_level(text, level)
                }
            } else {
                LogMessage::log_error(text, record.target(), record.file(), record.line())
            };
            thread::scope(|s| {
                s.spawn(|| {
                    let _ = self.sender.blocking_send(msg);
                });
            });
        }
    }

    fn flush(&self) {}
}

impl SharedLogger for TgLogger {
    fn level(&self) -> log::LevelFilter {
        log::LevelFilter::Info
    }

    fn config(&self) -> Option<&simplelog::Config> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn log::Log> {
        Box::new(*self)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Config {
    ignore: Vec<String>,
}

impl Config {
    fn is_should_ignore(&self, msg: &str) -> bool {
        self.ignore.iter().any(|pat| msg.contains(pat))
    }
}

#[derive(Debug, Clone)]
pub struct ConfigBuilder(Config);

impl ConfigBuilder {
    pub fn new() -> Self {
        Self(Config::default())
    }
    pub fn add_ignore(&mut self, s: &str) -> &mut Self {
        self.0.ignore.push(s.to_string());
        self
    }
    pub fn build(&mut self) -> Config {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_ignore() {
        let conf = ConfigBuilder::new().add_ignore("test").build();
        assert!(conf.is_should_ignore("test - ignored"));
        assert!(!conf.is_should_ignore("not ignored"));
    }
}
