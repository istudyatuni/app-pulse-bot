use std::{future::Future, time::Duration};

use anyhow::Result;
use dotenvy_macro::dotenv;
use reqwest::Client;
use simplelog::*;
use teloxide::{prelude::*, utils::command::BotCommands};

use tokio::{signal, sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;

use bot_handlers::{callback_handler, message_handler, start_updates_notify_job, Command};
use db::DB;
use sources::{start_update_loop, UpdateSource};

use crate::handlers::tg_logs::start_tg_logs_job;
use crate::logger::TgLogger;

mod handlers;
mod logger;

const DB_FILE: &str = dotenv!("DB_URL");
const TG_BOT_TOKEN: &str = if IS_PROD {
    dotenv!("PROD_BOT_TOKEN")
} else {
    dotenv!("BOT_TOKEN")
};
const LOG_CHAT_ID: &str = dotenv!("LOG_CHAT_ID");
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

const IS_PROD: bool = cfg!(feature = "prod");
const LOG_LEVEL: LevelFilter = if IS_PROD {
    LevelFilter::Error
} else {
    LevelFilter::Debug
};
const TG_LOG_ENABLED: bool = IS_PROD;

#[tokio::main]
async fn main() -> Result<()> {
    let tg_logs_chan = mpsc::channel(100);
    let log_chat_id = LOG_CHAT_ID.parse().ok().map(ChatId);

    CombinedLogger::init(vec![
        TermLogger::new(
            LOG_LEVEL,
            term_logger_config(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        TgLogger::new(tg_logs_chan.0),
    ])
    .expect("failed to init logger");

    let db = DB::init(&db_path()).await?;

    let bot = Bot::with_client(
        TG_BOT_TOKEN,
        Client::builder().timeout(REQUEST_TIMEOUT).build()?,
    );
    bot.set_my_commands(Command::bot_commands()).await?;

    let updates_chan = mpsc::channel(100);
    let cancel_token = CancellationToken::new();

    let mut jobs = JoinSet::new();
    if let Some(log_chat_id) = log_chat_id {
        jobs.spawn(spawn_with_token(
            cancel_token.clone(),
            start_tg_logs_job(bot.clone(), log_chat_id, tg_logs_chan.1),
        ));
    } else {
        log::warn!("LOG_CHAT_ID env not set, skip starting tg logs job")
    }
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

fn db_path() -> String {
    if DB_FILE.is_empty() {
        panic!("DB_URL env variable is empty")
    }
    let db_file = if IS_PROD {
        let home = match std::env::var("HOME") {
            Ok(s) => s,
            Err(_) => "/".to_string(),
        };
        format!("{home}/{DB_FILE}")
    } else {
        DB_FILE.to_string()
    };
    log::debug!("opening db at {db_file}");
    db_file
}

fn term_logger_config() -> Config {
    if IS_PROD {
        Config::default()
    } else {
        ConfigBuilder::new()
            .add_filter_ignore_str("h2")
            .add_filter_ignore_str("hyper")
            .add_filter_ignore_str("reqwest")
            .add_filter_ignore_str("rustls")
            .add_filter_ignore_str("sqlx")
            .build()
    }
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
            Update::filter_message().branch(
                dptree::entry()
                    .filter_command::<Command>()
                    .endpoint(message_handler),
            ),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));
    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![db])
        .default_handler(|_update| async move { log::error!("unhandled update") })
        .error_handler(LoggingErrorHandler::with_custom_text("error in dispatcher"))
        // .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
