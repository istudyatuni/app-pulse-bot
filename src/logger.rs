use std::thread;

use log::{Level, Metadata, Record};
use simplelog::SharedLogger;
use tokio::sync::mpsc::Sender;

use crate::TG_LOG_ENABLED;

#[derive(Debug)]
pub(crate) struct TgLogger {
    sender: Sender<String>,
    config: Config,
}

impl TgLogger {
    pub(crate) fn new(sx: Sender<String>, config: Config) -> Box<Self> {
        let s = Self { sender: sx, config };
        Box::new(s)
    }
    fn is_should_ignore(&self, msg: &str) -> bool {
        self.config.ignore.iter().any(|pat| msg.contains(pat))
    }
}

impl log::Log for TgLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        TG_LOG_ENABLED && metadata.level() <= Level::Error
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let text = record.args().to_string();
            if self.is_should_ignore(&text) {
                return;
            }

            let mut msg = format!("[ERROR] {text}\n        at {}", record.target());
            if let Some(file) = record.file() {
                msg += &format!(": {file}");
                if let Some(line) = record.line() {
                    msg += &format!(":{line}");
                }
            }
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
        log::LevelFilter::Error
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
