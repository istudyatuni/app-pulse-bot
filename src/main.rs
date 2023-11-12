use std::time::Duration;

use anyhow::Result;
use dotenvy_macro::dotenv;
use reqwest::Client;
use teloxide::{prelude::*, utils::command::BotCommands};

use db::DB;
use sources::{start_update_loop, UpdateSource, Update};
use tokio::sync::mpsc::{self, Receiver};

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

    let (tx, rx) = mpsc::channel(1);

    let source_check_job = tokio::spawn(start_update_loop(
        sources::alexstranniklite::Source::new(),
        tx,
    ));
    let user_reply_job = tokio::spawn(start_user_reply_job(bot.clone(), db.clone()));
    let updates_notify_job = tokio::spawn(start_updates_notify_job(bot.clone(), db.clone(), rx));

    tokio::try_join!(source_check_job, user_reply_job, updates_notify_job)?;

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

async fn start_updates_notify_job(bot: Bot, db: DB, mut rx: Receiver<Vec<Update>>) {
    while let Some(updates) = rx.recv().await {
        for update in updates {
            log::debug!("got update: {:?}", update);
            send_update(bot.clone(), &db).await;
        }
    }
}

async fn send_update(_bot: Bot, _db: &DB) {
    todo!()
}
