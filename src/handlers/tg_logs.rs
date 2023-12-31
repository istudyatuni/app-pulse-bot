use teloxide::{
    payloads::SendMessageSetters,
    requests::Requester,
    types::{ChatId, ParseMode},
    Bot,
};
use tokio::sync::mpsc::Receiver;

pub(crate) async fn start_tg_logs_job(bot: Bot, chat_id: ChatId, mut rx: Receiver<String>) {
    while let Some(msg) = rx.recv().await {
        match bot
            .send_message(chat_id, format!("```log\n{msg}```"))
            .parse_mode(ParseMode::MarkdownV2)
            .await
        {
            Ok(_) => (),
            Err(e) => println!("failed to send log: {e}"),
        }
    }
}
