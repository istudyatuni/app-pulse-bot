use teloxide::{prelude::*, utils::command::BotCommands};

use db::DB;

use crate::{keyboards::Keyboards, tr, DEFAULT_USER_LANG};

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "off")]
    Start,
    #[command(description = "Subscribe")]
    Subscribe,
    #[command(description = "Unsubscribe")]
    Unsubscribe,
    #[command(description = "Show latest update")]
    Changelog,
    #[command(description = "About")]
    About,
    #[command(description = "Display this text")]
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
                bot.send_message(msg.chat.id, tr!(welcome_choose_language, &lang))
                    .reply_markup(Keyboards::languages())
                    .await?;
            }
            None => match db.add_user(msg.chat.id.into()).await {
                Ok(()) => {
                    bot.send_message(msg.chat.id, tr!(welcome, &lang))
                        .reply_markup(Keyboards::languages())
                        .await?;
                    log::debug!("saved user: {:?}", db.select_user(msg.chat.id.into()).await);
                }
                Err(e) => log::error!("failed to save user {}: {e}", msg.chat.id.0),
            },
        },
        Command::Subscribe => match db.save_user_subscribed(msg.chat.id.into(), true).await {
            Ok(()) => {
                bot.send_message(msg.chat.id, tr!(subscribed, &lang))
                    .await?;
                log::debug!("subscribed user: {:?}", msg.chat.id);
            }
            Err(e) => log::error!("failed to subscribe user {}: {e}", msg.chat.id.0),
        },
        Command::Unsubscribe => match db.save_user_subscribed(msg.chat.id.into(), false).await {
            Ok(()) => {
                bot.send_message(msg.chat.id, tr!(unsubscribed, &lang))
                    .await?;
                log::debug!("unsubscribed user: {:?}", msg.chat.id);
            }
            Err(e) => log::error!("failed to unsubscribe user {}: {e}", msg.chat.id.0),
        },
        Command::Changelog => {
            bot.send_message(msg.chat.id, tr!(changelog_description, &lang))
                .await?;
        }
        Command::About => {
            bot.send_message(msg.chat.id, tr!(about_description, &lang))
                .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, make_command_descriptions(&lang))
                .await?;
        }
    };

    Ok(())
}

fn make_command_descriptions(lang: &str) -> String {
    [
        tr!(commands_list_header, lang),
        "".to_string(),
        "/subscribe - ".to_string() + tr!(subscribe_command, lang).as_str(),
        "/unsubscribe - ".to_string() + tr!(unsubscribe_command, lang).as_str(),
        "/changelog - ".to_string() + tr!(changelog_command, lang).as_str(),
        "/about - ".to_string() + tr!(about_command, lang).as_str(),
        "/help - ".to_string() + tr!(help_command, lang).as_str(),
    ]
    .join("\n")
}
