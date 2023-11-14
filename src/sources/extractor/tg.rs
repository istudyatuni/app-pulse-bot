#![allow(non_camel_case_types)]

use anyhow::Result;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://tg.i-c-a.su/json/";
const API_LIMIT_MSGS: u32 = 10;

pub(crate) async fn fetch_public_channel(name: &str) -> Result<Vec<Message>> {
    log::debug!("fetching public channel {name}");
    let res: Response = reqwest::get(format!("{API_URL}{name}?limit={API_LIMIT_MSGS}"))
        .await?
        .json()
        .await?;
    Ok(res.messages)
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    messages: Vec<Message>,
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub(crate) struct Message {
    pub(crate) id: i32,
    pub(crate) message: String,
    pub(crate) reply_markup: Option<ReplyMarkup>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "_")]
pub(crate) enum ReplyMarkup {
    replyInlineMarkup {
        rows: Vec<ReplyInlineMarkupRow>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "_")]
pub(crate) enum ReplyInlineMarkupRow {
    keyboardButtonRow {
        buttons: Vec<KeyboardButton>,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "_")]
pub(crate) enum KeyboardButton {
    keyboardButtonUrl {
        text: String,
        // url: Url,
    },
    #[serde(other)]
    Unknown,
}
