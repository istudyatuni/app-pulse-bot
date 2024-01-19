use std::fmt::Display;

use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, ParseMode},
    Bot,
};
use tokio::sync::mpsc::Receiver;

pub(crate) async fn start_tg_logs_job(bot: Bot, chat_id: ChatId, mut rx: Receiver<LogMessage>) {
    while let Some(msg) = rx.recv().await {
        if let Err(e) = bot
            .send_message(chat_id, msg.to_string())
            .parse_mode(ParseMode::MarkdownV2)
            .await
        {
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
    pub(crate) fn log_error(s: impl Into<String>) -> Self {
        Self::LogError(s.into())
    }
    /// Message should respect telegram's markdown requirements
    pub(crate) fn simple(s: impl Into<String>) -> Self {
        Self::Simple(s.into())
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
