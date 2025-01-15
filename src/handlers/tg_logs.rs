use std::fmt::Display;

use log::Level;
use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, ParseMode},
    utils::markdown::code_block_with_lang,
    Bot,
};
use tokio::sync::mpsc::Receiver;

use common::LogError;

pub(crate) async fn start_tg_logs_job(bot: Bot, chat_id: ChatId, mut rx: Receiver<LogMessage>) {
    while let Some(text) = rx.recv().await {
        bot.send_message(chat_id, text.to_string())
            .parse_mode(ParseMode::MarkdownV2)
            .await
            .log_error_msg("failed to send log");
    }
}

#[derive(Debug)]
pub(crate) enum LogMessage {
    Code(String),
    Markdown(String),
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
        Self::Code(msg)
    }
    /// Message should respect telegram's markdown requirements
    pub(crate) fn simple(s: impl Into<String>) -> Self {
        Self::Markdown(s.into())
    }
    /// Message should respect telegram's markdown requirements
    pub(crate) fn simple_with_level(s: impl Into<String>, level: Level) -> Self {
        Self::Markdown(format!("{}: {}", level_to_string(level), s.into()))
    }
}

impl Display for LogMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogMessage::Code(s) => code_block_with_lang(s, "log").fmt(f),
            LogMessage::Markdown(s) => s.fmt(f),
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
