use std::fmt::Display;

use log::Level;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, ParseMode},
    Bot,
};
use tokio::sync::mpsc::Receiver;

pub(crate) async fn start_tg_logs_job(bot: Bot, chat_id: ChatId, mut rx: Receiver<LogMessage>) {
    while let Some(text) = rx.recv().await {
        let msg = bot
            .send_message(chat_id, text.to_string())
            .parse_mode(ParseMode::MarkdownV2);
        if let Err(e) = msg.await {
            eprintln!("failed to send log: {e}");
        }
    }
}

#[derive(Debug)]
pub(crate) enum LogMessage {
    LogError(String),
    Simple(String),
}

impl LogMessage {
    pub(crate) fn log_error(
        s: impl Into<String>,
        target: &str,
        file: Option<&str>,
        line: Option<u32>,
    ) -> Self {
        let mut msg = format!("[ERROR] {}\n        at {target}", s.into());
        if let Some(file) = file {
            msg += &format!(": {file}");
            if let Some(line) = line {
                msg += &format!(":{line}");
            }
        }
        Self::LogError(msg)
    }
    /// Message should respect telegram's markdown requirements
    pub(crate) fn simple(s: impl Into<String>) -> Self {
        Self::Simple(s.into())
    }
    /// Message should respect telegram's markdown requirements
    pub(crate) fn simple_with_level(s: impl Into<String>, level: Level) -> Self {
        Self::Simple(format!("{}: {}", level_to_string(level), s.into()))
    }
}

impl Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogMessage::LogError(s) => write!(f, "```log\n{s}```"),
            LogMessage::Simple(s) => write!(f, "{s}"),
        }
    }
}

fn level_to_string(level: Level) -> String {
    let s = match level {
        Level::Error => "Error",
        Level::Warn => "Warning",
        Level::Info => "Info",
        Level::Debug => "Debug",
        Level::Trace => "Trace",
    };
    s.to_string()
}
