#![allow(unused)]

use std::thread;

use log::{Level, Metadata, Record};
use simplelog::{Config, SharedLogger};
use teloxide::{requests::Requester, types::ChatId, Bot};
use tokio::sync::mpsc::Sender;

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
        metadata.level() <= Level::Error
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{} :: {}", record.target(), record.args());
            thread::scope(|s| {
                s.spawn(|| {
                    self.sender.blocking_send(msg);
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
