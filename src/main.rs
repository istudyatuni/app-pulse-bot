use std::time::Duration;

use anyhow::Result;
use dotenvy_macro::dotenv;
use reqwest::Client;
use simplelog::LevelFilter;
use teloxide::{prelude::*, types::BotCommandScope};

use tokio::{
    signal,
    sync::mpsc::{self, Sender},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use bot_handlers::{
    admin_command_handler, callback_handler, command_handler, message_handler,
    run_collect_user_names_job, start_updates_notify_job, AdminCommand, Command,
};
use common::{is_admin_chat_id, spawn_with_token, LogError};
use db::DB;
use sources::spawn_sources_update_jobs;

use crate::{
    handlers::tg_logs::{start_tg_logs_job, LogMessage},
    logger::TgLogger,
};

mod handlers;
mod logger;

const DB_FILE: &str = dotenv!("DB_URL");
const LOG_CHAT_ID: &str = dotenv!("LOG_CHAT_ID");
const BOT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

const IS_REAL_PROD: bool = cfg!(feature = "prod");
const IS_TEST_PROD: bool = cfg!(feature = "test-prod");
const IS_PROD: bool = IS_REAL_PROD || IS_TEST_PROD;

const TG_BOT_TOKEN: &str = if IS_REAL_PROD {
    dotenv!("PROD_BOT_TOKEN")
} else {
    dotenv!("BOT_TOKEN")
};
const LOG_LEVEL: LevelFilter = if IS_REAL_PROD {
    LevelFilter::Error
} else {
    LevelFilter::Debug
};
const TG_LOG_ENABLED: bool = IS_PROD;

#[tokio::main]
async fn main() -> Result<()> {
    let tg_logs_chan = mpsc::channel(100);
    let log_chat_id = LOG_CHAT_ID.parse().ok().map(ChatId);

    init_logger(tg_logs_chan.0);

    if IS_PROD {
        log::info!(tg = true; "Bot started");
    }

    let db = DB::init(&db_path()).await?;

    let bot = Bot::with_client(
        TG_BOT_TOKEN,
        Client::builder().timeout(BOT_REQUEST_TIMEOUT).build()?,
    );
    set_bot_commands(bot.clone()).await?;

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
    spawn_sources_update_jobs(&mut jobs, cancel_token.clone(), updates_chan.0);
    jobs.spawn(spawn_with_token(
        cancel_token.clone(),
        run_collect_user_names_job(bot.clone(), db.clone()),
    ));
    jobs.spawn(spawn_with_token(
        cancel_token.clone(),
        start_updates_notify_job(bot.clone(), db, updates_chan.1),
    ));

    jobs.spawn(async move {
        signal::ctrl_c()
            .await
            .log_error_msg("failed to listen for SIGINT");
        cancel_token.cancel();
    });

    while (jobs.join_next().await).is_some() {}

    Ok(())
}

async fn set_bot_commands(bot: Bot) -> Result<()> {
    for lang in i18n::Localize::languages() {
        bot.set_my_commands(Command::bot_commands_translated(lang))
            .language_code(lang)
            .scope(BotCommandScope::AllPrivateChats)
            .await?;
    }

    Ok(())
}

fn db_path() -> String {
    #[allow(clippy::const_is_empty)]
    if DB_FILE.is_empty() {
        panic!("DB_URL env variable is empty")
    }
    let db_file = if IS_REAL_PROD {
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

fn init_logger(sender: Sender<LogMessage>) {
    use simplelog::*;

    use logger::{Config as TgConfig, ConfigBuilder as TgConfigBuilder};

    let term_config = if IS_REAL_PROD {
        Config::default()
    } else {
        ConfigBuilder::new()
            .add_filter_ignore_str("h2")
            .add_filter_ignore_str("hyper")
            .add_filter_ignore_str("reqwest")
            .add_filter_ignore_str("rustls")
            .add_filter_ignore_str("sqlx")
            .build()
    };

    let tg_config = if IS_REAL_PROD {
        TgConfigBuilder::new()
            .add_ignore("ConnectionReset")
            .add_ignore("TerminatedByOtherGetUpdates")
            .build()
    } else {
        TgConfig::default()
    };

    CombinedLogger::init(vec![
        TermLogger::new(
            LOG_LEVEL,
            term_config,
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        TgLogger::new(sender, tg_config),
    ])
    .expect("failed to init logger");
}

async fn start_bot(bot: Bot, db: DB) {
    log::debug!("starting bot");
    let handler = dptree::entry()
        .branch(
            Update::filter_message()
                .branch(
                    dptree::entry()
                        .filter_command::<AdminCommand>()
                        .filter(|msg: Message| is_admin_chat_id(msg.chat.id.0))
                        .endpoint(admin_command_handler),
                )
                .branch(
                    dptree::entry()
                        .filter_command::<Command>()
                        .endpoint(command_handler),
                )
                .endpoint(message_handler),
        )
        .branch(Update::filter_callback_query().endpoint(callback_handler));
    Dispatcher::builder(bot.clone(), handler)
        .dependencies(dptree::deps![db])
        .default_handler(|update| async move { log::error!("unhandled update: {update:?}") })
        .error_handler(LoggingErrorHandler::with_custom_text("error in dispatcher"))
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}
