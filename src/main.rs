use std::time::Duration;

use anyhow::Result;
use dotenvy_macro::dotenv;
use reqwest::Client;
use teloxide::{
    prelude::*,
    types::{InlineKeyboardButton, InlineKeyboardMarkup, ReplyMarkup, Update as TgUpdate},
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

const NOTIFY_TOKEN: &str = "notify";
const IGNORE_TOKEN: &str = "ignore";

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

async fn message_handler(bot: Bot, msg: Message, cmd: Command, db: DB) -> ResponseResult<()> {
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
            bot.send_message(msg.chat.id, "Subscribed to @alexstranniklite")
                .await?;
        }
    };

    Ok(())
}

async fn callback_handler(bot: Bot, q: CallbackQuery, db: DB) -> ResponseResult<()> {
    bot.answer_callback_query(&q.id).await?;
    let chat_id = q.from.id;
    let Some(data) = q.data else {
        log::error!("got empty callback {} from user {}", q.id, chat_id);
        // todo: answer with alert https://stackoverflow.com/a/57390206
        bot.send_message(chat_id, "Something went wrong").await?;
        return Ok(());
    };
    log::debug!("got callback: {:?}", data);
    let data: Vec<_> = data.split(":").collect();
    if data.len() != 2 {
        log::error!("wrong callback: {data:?}");
        return Ok(());
    }
    let (app_id, should_notify) = (data[0], data[1]);
    let should_notify = match should_notify {
        NOTIFY_TOKEN => ShouldNotify::Notify,
        IGNORE_TOKEN => ShouldNotify::Ignore,
        _ => {
            log::error!("wrong callback: {data:?}");
            return Ok(());
        }
    };
    match db
        .save_should_notify_user(chat_id.into(), app_id, should_notify)
        .await
    {
        Ok(_) => (),
        Err(e) => log::error!("failed to save user should_notify: {e}"),
    }
    Ok(())
}

async fn start_user_reply_job(bot: Bot, db: DB) {
    let handler = dptree::entry()
        .branch(TgUpdate::filter_message().endpoint(message_handler))
        .branch(TgUpdate::filter_callback_query().endpoint(callback_handler));
    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![db])
        .default_handler(|_update| async move { log::error!("unhandled update") })
        .error_handler(LoggingErrorHandler::with_custom_text("error in dispatcher"))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn start_updates_notify_job(
    token: CancellationToken,
    bot: Bot,
    db: DB,
    mut rx: Receiver<Vec<Update>>,
) {
    while let Some(updates) = rx.recv().await {
        for update in updates {
            // probably this is unnecessary
            if token.is_cancelled() {
                rx.close();
                log::info!("aborting updates, waiting for handling remaining updates")
            }

            log::debug!("got update: {:?}", update);
            let users = match db.select_users().await {
                Ok(v) => v,
                Err(e) => {
                    log::error!("failed to select users: {e}");
                    continue;
                }
            };
            for user in users {
                let user_id = user.user_id();
                let chat_id = user_id.into();
                let app_id = update.app_id();
                match db.should_notify_user(user_id, app_id).await {
                    Ok(s) => match s {
                        ShouldNotify::Unspecified => {
                            send_suggest_update(bot.clone(), chat_id, &update)
                                .await
                                .log_on_error()
                                .await
                        }
                        ShouldNotify::Notify => {
                            send_update(bot.clone(), chat_id, &update)
                                .await
                                .log_on_error()
                                .await;
                        }
                        ShouldNotify::Ignore => {
                            log::debug!("ignoring update {app_id} for user {user_id}")
                        }
                    },
                    Err(e) => log::error!("failed to check, if should notify user {user_id}: {e}"),
                };
            }
        }
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
        InlineKeyboardButton::callback(
            "Notify",
            format!("{app}:{NOTIFY_TOKEN}", app = update.app_id()),
        ),
        InlineKeyboardButton::callback(
            "Ignore",
            format!("{app}:{IGNORE_TOKEN}", app = update.app_id()),
        ),
    ]];
    bot.send_message(chat_id, text.join(""))
        .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(
            keys.to_owned(),
        )))
        .await?;
    Ok(())
}

async fn send_update(bot: Bot, chat_id: ChatId, update: &Update) -> Result<()> {
    let mut text = vec!["New update\n".to_string()];
    if let Some(url) = update.update_link() {
        text.push(url.to_string());
    } else if let Some(url) = update.description_link() {
        text.push(url.to_string());
    }
    let keys = &[[InlineKeyboardButton::callback(
        "Ignore",
        format!("{app}:{IGNORE_TOKEN}", app = update.app_id()),
    )]];
    bot.send_message(chat_id, text.join(""))
        .reply_markup(ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(
            keys.to_owned(),
        )))
        .await?;
    Ok(())
}
