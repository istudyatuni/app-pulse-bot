use teloxide::{prelude::*, utils::command::BotCommands};

use db::DB;

use crate::{keyboards::Keyboards, tr, DEFAULT_USER_LANG};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    Start,
    Subscribe,
    Help,
}

pub async fn message_handler(bot: Bot, msg: Message, cmd: Command, db: DB) -> ResponseResult<()> {
    let user = db.select_user(msg.chat.id.into()).await.ok().flatten();
    let lang = user
        .as_ref()
        .map(|u| u.lang().to_string())
        .unwrap_or(DEFAULT_USER_LANG.to_string());

    match cmd {
        Command::Start => match user {
            Some(_) => {
                bot.send_message(msg.chat.id, tr!(welcome, &lang))
                    .reply_markup(Keyboards::languages())
                    .await?;
            }
            None => match db.save_user(msg.chat.id.into()).await {
                Ok(_) => {
                    bot.send_message(msg.chat.id, tr!(welcome, &lang))
                        .reply_markup(Keyboards::languages())
                        .await?;
                    log::debug!("saved user: {:?}", db.select_user(msg.chat.id.into()).await);
                }
                Err(e) => log::error!("failed to save user {}: {e}", msg.chat.id.0),
            },
        },
        Command::Help => {
            bot.send_message(msg.chat.id, make_command_descriptions(&lang))
                .await?;
        }
        Command::Subscribe => {
            bot.send_message(msg.chat.id, tr!(not_implemented_already_subscribed, &lang))
                .await?;
        }
    };

    Ok(())
}

fn make_command_descriptions(lang: &str) -> String {
    [
        tr!(commands_list_header, lang),
        "".to_string(),
        // "/subscribe - ".to_string() + tr!(subscribe_command, lang).as_str(),
        "/help - ".to_string() + tr!(help_command, lang).as_str(),
    ]
    .join("\n")
}
