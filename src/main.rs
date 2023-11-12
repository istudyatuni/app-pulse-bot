use std::time::Duration;

use anyhow::Result;
use dotenvy_macro::dotenv;
use reqwest::Client;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup, ReplyMarkup},
    utils::command::BotCommands,
};
use tokio::{
    signal,
    sync::mpsc::{self, Receiver},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use db::{models::ShouldNotify, DB};
use sources::{start_update_loop, Update, UpdateSource};

mod db;
mod sources;

const DB_FILE: &str = "data.db";
const TG_BOT_TOKEN: &str = dotenv!("BOT_TOKEN");
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    log::info!("starting bot");

    let db = DB::init(DB_FILE).await?;

    let bot = Bot::with_client(
        TG_BOT_TOKEN,
        Client::builder().timeout(REQUEST_TIMEOUT).build()?,
    );
    bot.set_my_commands(Command::bot_commands()).await?;

    let (tx, rx) = mpsc::channel(100);
    let cancel_token = CancellationToken::new();

    let mut jobs = JoinSet::new();
    jobs.spawn(start_update_loop(
        cancel_token.clone(),
        sources::alexstranniklite::Source::new(),
        tx,
    ));
    jobs.spawn(start_user_reply_job(bot.clone(), db.clone()));
    jobs.spawn(start_updates_notify_job(
        cancel_token.clone(),
        bot.clone(),
        db.clone(),
        rx,
    ));
    jobs.spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {}
            Err(e) => {
                log::error!("failed to listen for SIGINT: {e}");
            }
        }
        cancel_token.cancel();
    });

    while let Some(_) = jobs.join_next().await {}

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "Supported commands")]
enum Command {
    #[command(description = "off")]
    Start,
    #[command(description = "Subscribe")]
    Subscribe,
    #[command(description = "Display this text")]
    Help,
}

async fn answer(bot: Bot, msg: Message, cmd: Command, db: DB) -> ResponseResult<()> {
    match cmd {
        Command::Start => match db.save_user(msg.chat.id.0).await {
            Ok(_) => {
                bot.send_message(msg.chat.id, "Welcome").await?;
                log::debug!("saved user: {:?}", db.select_user(msg.chat.id.0).await);
            }
            Err(e) => log::error!("failed to save user {}: {e}", msg.chat.id.0),
        },
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Subscribe => {
            bot.send_message(msg.chat.id, "Subscribed to @alexstranniklite")
                .await?;
        }
    };

    Ok(())
}

async fn start_user_reply_job(bot: Bot, db: DB) {
    Command::repl(bot, move |b, msg, cmd| answer(b, msg, cmd, db.clone())).await
}

async fn start_updates_notify_job(
    token: CancellationToken,
    bot: Bot,
    db: DB,
    mut rx: Receiver<Vec<Update>>,
) {
    // todo: handle remaining updates after cancel
    tokio::select! {
        _ = token.cancelled() => {}
        _ = async {
            while let Some(updates) = rx.recv().await {
                for update in updates {
                    log::debug!("got update: {:?}", update);
                    let users = match db.select_users().await {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("failed to select users: {e}");
                            continue;
                        },
                    };
                    for user in users {
                        let user_id = user.user_id();
                        let chat_id = ChatId(user_id);
                        match db.should_notify_user(user_id, update.app_id()).await {
                            ShouldNotify::Unspecified => {
                                match send_suggest_update(bot.clone(), chat_id, &update).await {
                                    Ok(_) => (),
                                    Err(e) => {
                                        log::error!("failed to send suggest update: {e}");
                                    },
                                }
                            },
                            ShouldNotify::Notify => {
                                send_update(bot.clone(), chat_id).await;
                            },
                            ShouldNotify::Ignore => (),
                        };
                    }
                }
            }
        } => {}
    }
}

async fn send_suggest_update(bot: Bot, chat_id: ChatId, update: &Update) -> Result<()> {
    let mut text = vec!["New app to track updates\n".to_string()];
    if let Some(url) = update.description_link() {
        text.push(url.to_string());
    } else if let Some(url) = update.update_link() {
        text.push(url.to_string());
    }
    let keys = &[[
        InlineKeyboardButton::new(
            "Notify",
            InlineKeyboardButtonKind::CallbackData(format!(
                "{chat_id}:{app}:notify",
                app = update.app_id()
            )),
        ),
        InlineKeyboardButton::new(
            "Ignore",
            InlineKeyboardButtonKind::CallbackData(format!(
                "{chat_id}:{app}:ignore",
                app = update.app_id()
            )),
        ),
    ]];
    bot.send_message(chat_id, text.join(" "))
        .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(
            keys.to_owned(),
        )))
        .await?;
    Ok(())
}

async fn send_update(_bot: Bot, _chat_id: ChatId) {
    log::error!("send_update not implemented")
}
