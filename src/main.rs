use std::time::Duration;

use anyhow::Result;
use dotenvy_macro::dotenv;
use reqwest::Client;
use teloxide::prelude::*;

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

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        bot.send_dice(msg.chat.id).await?;
        Ok(())
    })
    .await;

    Ok(())
}
