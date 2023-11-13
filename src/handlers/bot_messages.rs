use teloxide::{prelude::*, utils::command::BotCommands};

use crate::db::DB;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Supported commands")]
pub(crate) enum Command {
    #[command(description = "off")]
    Start,
    #[command(description = "Subscribe")]
    Subscribe,
    #[command(description = "Display this text")]
    Help,
}

pub(crate) async fn message_handler(bot: Bot, msg: Message, cmd: Command, db: DB) -> ResponseResult<()> {
    match cmd {
        Command::Start => match db.save_user(msg.chat.id.into()).await {
            Ok(_) => {
                bot.send_message(msg.chat.id, "Welcome").await?;
                log::debug!("saved user: {:?}", db.select_user(msg.chat.id.into()).await);
            }
            Err(e) => log::error!("failed to save user {}: {e}", msg.chat.id.0),
        },
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Subscribe => {
            bot.send_message(msg.chat.id, "Not implemented (already subscribed)")
                .await?;
        }
    };

    Ok(())
}
