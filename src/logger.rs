use std::thread;

use log::{Level, Metadata, Record};
use simplelog::SharedLogger;
use tokio::sync::mpsc::Sender;

use crate::TG_LOG_ENABLED;

#[derive(Debug)]
pub(crate) struct TgLogger {
    sender: Sender<String>,
}

impl TgLogger {
    pub(crate) fn new(sx: Sender<String>) -> Box<Self> {
        let s = Self { sender: sx };
        Box::new(s)
    }
}

impl log::Log for TgLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        TG_LOG_ENABLED && metadata.level() <= Level::Error
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut msg = format!("[ERROR] {}\n        at {}", record.args(), record.target());
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
