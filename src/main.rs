use std::time::Duration;

use anyhow::Result;
use dotenvy_macro::dotenv;
use reqwest::Client;
use teloxide::{prelude::*, utils::command::BotCommands};
use tokio::{
    signal,
    sync::mpsc::{self, Receiver},
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use db::DB;
use sources::{start_update_loop, Update, UpdateSource};

mod db;
mod sources;

const TG_BOT_TOKEN: &str = dotenv!("BOT_TOKEN");
const REQUEST_TIMEOUT_SEC: u64 = 30;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    log::info!("starting bot");

    let bot = Bot::with_client(
        TG_BOT_TOKEN,
        Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SEC))
            .build()?,
    );
    let db = DB::new();

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
        Command::Start => {
            db.save_user(msg.chat.id.0);
        }
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
                    send_update(bot.clone(), &db).await;
                }
            }
        } => {}
    }
}

async fn send_update(_bot: Bot, _db: &DB) {
    todo!()
}
