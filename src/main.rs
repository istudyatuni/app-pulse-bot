use std::{future::Future, time::Duration};

use anyhow::Result;
use dotenvy_macro::dotenv;
use reqwest::Client;
use simplelog::*;
use teloxide::{prelude::*, types::Update as TgUpdate, utils::command::BotCommands};

use tokio::{
    signal,
    sync::mpsc::{self},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use db::DB;
use handlers::{
    bot_callback::callback_handler,
    bot_messages::{message_handler, Command},
    tg_logs::start_tg_logs_job,
    updates_notify::start_updates_notify_job,
};
use logger::TgLogger;
use sources::{start_update_loop, UpdateSource};

mod db;
mod handlers;
mod logger;
mod sources;
mod tg;

const DB_FILE: &str = "data.db";
const TG_BOT_TOKEN: &str = dotenv!("BOT_TOKEN");
const LOG_CHAT_ID: &str = dotenv!("LOG_CHAT_ID");
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;
#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::Error;

#[cfg(debug_assertions)]
const SET_BOT_COMMANDS: bool = false;
#[cfg(not(debug_assertions))]
const SET_BOT_COMMANDS: bool = true;

const NOTIFY_TOKEN: &str = "notify";
const IGNORE_TOKEN: &str = "ignore";

#[tokio::main]
async fn main() -> Result<()> {
    let tg_logs_chan = mpsc::channel(100);
    let log_chat_id = ChatId(LOG_CHAT_ID.parse().expect("invalid logs chat_id"));

    CombinedLogger::init(vec![
        TermLogger::new(
            LOG_LEVEL,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        TgLogger::new(tg_logs_chan.0),
    ])
    .expect("failed to init logger");

    let db = DB::init(DB_FILE).await?;

    let bot = Bot::with_client(
        TG_BOT_TOKEN,
        Client::builder().timeout(REQUEST_TIMEOUT).build()?,
    );
    if SET_BOT_COMMANDS {
        bot.set_my_commands(Command::bot_commands()).await?;
    }

    let updates_chan = mpsc::channel(100);
    let cancel_token = CancellationToken::new();

    let mut jobs = JoinSet::new();
    jobs.spawn(spawn_with_token(
        cancel_token.clone(),
        start_tg_logs_job(bot.clone(), log_chat_id, tg_logs_chan.1),
    ));
    jobs.spawn(spawn_with_token(
        cancel_token.clone(),
        start_bot(bot.clone(), db.clone()),
    ));
    jobs.spawn(spawn_with_token(
        cancel_token.clone(),
        start_update_loop(sources::alexstranniklite::Source::new(), updates_chan.0),
    ));
    jobs.spawn(spawn_with_token(
        cancel_token.clone(),
        start_updates_notify_job(bot.clone(), db, updates_chan.1),
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

    while (jobs.join_next().await).is_some() {}

    Ok(())
}

async fn spawn_with_token<R>(token: CancellationToken, f: impl Future<Output = R>) {
    tokio::select! {
        _ = token.cancelled() => {},
        _ = f => {},
    }
}

async fn start_bot(bot: Bot, db: DB) {
    log::debug!("starting bot");
    let handler = dptree::entry()
        .branch(
            TgUpdate::filter_message().branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(message_handler),
            ),
        )
        .branch(TgUpdate::filter_callback_query().endpoint(callback_handler));
    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![db])
        .default_handler(|_update| async move { log::error!("unhandled update") })
        .error_handler(LoggingErrorHandler::with_custom_text("error in dispatcher"))
        // .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
