use teloxide::{prelude::*, utils::command::BotCommands};

use db::DB;

use crate::{
    keyboards::{Keyboards, LanguagesKeyboardToken},
    tr,
    utils::escape,
    DEFAULT_USER_LANG,
};

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
    #[command(description = "Configuration")]
    Settings,
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
                send_welcome_msg(bot.clone(), msg.chat.id, &lang).await?;
            }
            None => match db.add_user(msg.chat.id.into()).await {
                Ok(()) => {
                    send_welcome_msg(bot.clone(), msg.chat.id, &lang).await?;
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
            bot.send_message(msg.chat.id, escape(tr!(changelog, &lang)))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
        Command::Settings => {
            bot.send_message(msg.chat.id, tr!(choose_language, &lang))
                .reply_markup(Keyboards::languages(LanguagesKeyboardToken::Settings))
                .await?;
        }
        Command::About => {
            bot.send_message(msg.chat.id, tr!(about_description, &lang))
                .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, escape(make_help(&lang)))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
    };

    Ok(())
}

fn make_help(lang: &str) -> String {
    [
        tr!(commands_list_header, lang),
        "".to_string(),
        "/subscribe - ".to_string() + tr!(subscribe_command, lang).as_str(),
        "/unsubscribe - ".to_string() + tr!(unsubscribe_command, lang).as_str(),
        "/changelog - ".to_string() + tr!(changelog_command, lang).as_str(),
        "/settings - ".to_string() + tr!(settings_command, lang).as_str(),
        "/about - ".to_string() + tr!(about_command, lang).as_str(),
        "/help - ".to_string() + tr!(help_command, lang).as_str(),
        "".to_string(),
        tr!(how_to_use, lang),
    ]
    .join("\n")
}

async fn send_welcome_msg(bot: Bot, chat_id: ChatId, lang: &str) -> ResponseResult<()> {
    bot.send_message(chat_id, tr!(welcome_choose_language, lang))
        .reply_markup(Keyboards::languages(LanguagesKeyboardToken::Start))
        .await?;
    Ok(())
}
