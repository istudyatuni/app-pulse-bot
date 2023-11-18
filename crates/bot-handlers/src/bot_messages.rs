use teloxide::{prelude::*, utils::command::BotCommands};

use db::DB;

use crate::{tr, USER_LANG};

// todo: translate command descriptions
#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Supported commands")]
pub enum Command {
    #[command(description = "off")]
    Start,
    #[command(description = "Subscribe")]
    Subscribe,
    #[command(description = "Display this text")]
    Help,
}

pub async fn message_handler(bot: Bot, msg: Message, cmd: Command, db: DB) -> ResponseResult<()> {
    match cmd {
        Command::Start => match db.save_user(msg.chat.id.into()).await {
            Ok(_) => {
                bot.send_message(msg.chat.id, tr!(welcome, USER_LANG))
                    .await?;
                log::debug!("saved user: {:?}", db.select_user(msg.chat.id.into()).await);
            }
            Err(e) => log::error!("failed to save user {}: {e}", msg.chat.id.0),
        },
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Subscribe => {
            bot.send_message(
                msg.chat.id,
                tr!(not_implemented_already_subscribed, USER_LANG),
            )
            .await?;
        }
    };

    Ok(())
}
