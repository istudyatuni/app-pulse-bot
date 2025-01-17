use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
};

use teloxide::{
    prelude::*,
    types::{BotCommand, ChatKind, MessageKind},
};

use common::types;
use db::{models::User, DB};

use crate::{
    commands::AdminCommand,
    keyboards::{Keyboards, LanguagesKeyboardKind},
    tr,
    user::get_chat_name,
    utils::escape,
    Command, DEFAULT_USER_LANG,
};

#[derive(Debug, PartialEq, Eq, Hash)]
struct HelpCacheKey {
    lang: String,
    admin: bool,
}

impl HelpCacheKey {
    fn new(lang: &str, admin: bool) -> Self {
        Self {
            lang: lang.to_owned(),
            admin,
        }
    }
}

static HELP_CACHE: LazyLock<Mutex<HashMap<HelpCacheKey, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn command_handler(bot: Bot, msg: Message, cmd: Command, db: DB) -> ResponseResult<()> {
    let user = db.select_user(msg.chat.id).await.ok().flatten();
    let lang = get_user_lang(user.as_ref(), msg.from.as_ref());

    if !msg.chat.is_private() && !cmd.allowed_in_public() {
        bot.send_message(msg.chat.id, tr!(command_not_available_in_public, &lang))
            .await?;
        return Ok(());
    }

    // todo: rework how subsribe work with multiple sources
    match cmd {
        Command::Start => handle_start_command(bot.clone(), &db, user, &lang, msg).await?,
        Command::Subscribe => match db.save_user_subscribed(msg.chat.id, true).await {
            Ok(()) => {
                bot.send_message(msg.chat.id, tr!(subscribed, &lang)).await?;
                log::debug!("user {} subscribed", msg.chat.id);
            },
            Err(e) => log::error!("failed to subscribe user {}: {e}", msg.chat.id.0),
        },
        Command::Unsubscribe => match db.save_user_subscribed(msg.chat.id, false).await {
            Ok(()) => {
                bot.send_message(msg.chat.id, tr!(unsubscribed, &lang)).await?;
                log::debug!("user {} unsubscribed", msg.chat.id);
            },
            Err(e) => log::error!("failed to unsubscribe user {}: {e}", msg.chat.id.0),
        },
        Command::Changelog => {
            bot.send_message(msg.chat.id, escape(tr!(changelog, &lang)))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        },
        Command::Settings => {
            bot.send_message(msg.chat.id, tr!(choose_language, &lang))
                .reply_markup(Keyboards::languages(LanguagesKeyboardKind::Settings))
                .await?;
        },
        Command::About => {
            bot.send_message(msg.chat.id, tr!(about_description, &lang)).await?;
        },
        Command::Help => {
            bot.send_message(msg.chat.id, escape(get_help(&lang, false)))
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        },
    };

    Ok(())
}

async fn handle_start_command(
    bot: Bot,
    db: &DB,
    user: Option<User>,
    lang: &str,
    msg: Message,
) -> Result<(), teloxide::RequestError> {
    match user {
        Some(u) => {
            if u.bot_blocked() {
                log::info!(tg = true; "User {} returned", u.display());
                if let Err(e) = db.save_user_bot_blocked(u.user_id(), false).await {
                    log::error!("failed to save that user is returned: {e}")
                }
            }
            send_welcome_msg(bot.clone(), msg.chat.id, lang).await?;
        },
        None => {
            let id: types::ChatId = msg.chat.id.into();
            let user = User::builder().user_id(id.into()).lang(lang.to_owned());
            let user = if let ChatKind::Private(chat) = msg.chat.kind {
                user.maybe_username(chat.username.clone())
                    .maybe_name(get_chat_name(&chat))
                    .build()
            } else {
                log::error!("handler for command /start called not in private chat");
                user.build()
            };

            match db.add_user(user).await {
                Ok(()) => {
                    send_welcome_msg(bot.clone(), msg.chat.id, lang).await?;
                    log::debug!("user {} saved", msg.chat.id);
                },
                Err(e) => log::error!("failed to save user {}: {e}", msg.chat.id.0),
            }
        },
    };
    Ok(())
}

pub async fn message_handler(bot: Bot, msg: Message, db: DB) -> ResponseResult<()> {
    if msg.text().is_some_and(|m| m.starts_with("/")) {
        log::debug!("ignoring command sent to other bot");
        return Ok(());
    }
    if !matches!(msg.kind, MessageKind::Common(_)) {
        log::debug!("ignoring service message");
        return Ok(());
    }

    let user = db.select_user(msg.chat.id).await.ok().flatten();
    let lang = get_user_lang(user.as_ref(), msg.from.as_ref());

    bot.send_message(msg.chat.id, tr!(unknown_message, &lang)).await?;
    Ok(())
}

pub(crate) fn get_help(lang: &str, admin: bool) -> String {
    let key = HelpCacheKey::new(lang, admin);

    log::debug!("sending help, admin = {admin}");

    HELP_CACHE
        .lock()
        .unwrap()
        .entry(key)
        .or_insert_with(|| make_help(lang, admin))
        .to_owned()
}

fn make_help(lang: &str, admin: bool) -> String {
    fn build_commands(commands: impl IntoIterator<Item = BotCommand>) -> String {
        commands
            .into_iter()
            .map(|c| format!("{} - {}", c.command, c.description))
            .collect::<Vec<_>>()
            .join("\n")
    }

    [
        tr!(commands_list_header, lang),
        "".to_string(),
        build_commands(Command::bot_commands_translated(lang)),
        if admin {
            [
                "".to_string(),
                tr!(admin_commands_header, lang),
                "".to_string(),
                build_commands(AdminCommand::bot_commands_translated(lang)),
                "".to_string(),
            ]
            .join("\n")
        } else {
            ["".to_string(), tr!(how_to_use, lang)].join("\n")
        },
    ]
    .join("\n")
}

async fn send_welcome_msg(bot: Bot, chat_id: ChatId, lang: &str) -> ResponseResult<()> {
    bot.send_message(chat_id, tr!(welcome_choose_language, lang))
        .reply_markup(Keyboards::languages(LanguagesKeyboardKind::Start))
        .await?;
    Ok(())
}

pub(crate) fn get_user_lang(user: Option<&User>, from: Option<&teloxide::types::User>) -> String {
    get_user_lang_impl(user, from.and_then(|c| c.language_code.to_owned()).as_deref())
}

fn get_user_lang_impl(user: Option<&User>, tg_lang: Option<&str>) -> String {
    // 1. get lang from db
    // 2. get lang from msg.from.language_code and check, if it's available
    // 3. otherwise return DEFAULT_USER_LANG
    user.map(|u| u.lang().to_owned()).unwrap_or_else(|| {
        tg_lang
            .and_then(|lang| {
                i18n::Localize::languages()
                    .iter()
                    .find(|&&l| lang == l)
                    .map(|l| l.to_string())
            })
            .unwrap_or(DEFAULT_USER_LANG.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_lang() {
        assert!(["en", "ru"]
            .iter()
            .all(|lang| i18n::Localize::languages().iter().any(|l| lang == l)));

        let table = [
            (Some("en"), None, "en"),
            (Some("en"), Some("ru"), "en"),
            (None, Some("ru"), "ru"),
            (None, None, DEFAULT_USER_LANG),
        ];
        for (i, &(db_lang, tg_lang, expected)) in table.iter().enumerate() {
            let user = db_lang.map(|lang| User::builder().user_id(0.into()).lang(lang.to_string()).build());
            assert_eq!(get_user_lang_impl(user.as_ref(), tg_lang), expected, "test table[{i}]");
        }
    }
}
