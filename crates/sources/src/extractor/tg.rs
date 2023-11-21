#![allow(non_camel_case_types)]

use anyhow::Result;
use serde::{Deserialize, Serialize};

use common::UnixDateTime;

const API_URL: &str = "https://tg.i-c-a.su/json/";
const API_LIMIT_MSGS: u32 = 10;

pub(crate) async fn fetch_public_channel(name: &str) -> Result<Vec<Message>> {
    log::debug!("fetching public channel {name}");
    let res: Response = reqwest::get(format!("{API_URL}{name}?limit={API_LIMIT_MSGS}"))
        .await?
        .json()
        .await?;
    if let Some(errors) = res.errors {
        return Err(ResponseError::Arbitrary(errors).into());
    }
    Ok(res.messages)
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    #[serde(default = "Vec::new")]
    messages: Vec<Message>,
    errors: Option<serde_json::Value>,
}

#[derive(Debug, thiserror::Error)]
enum ResponseError {
    #[error("got error when fetching: {0:?}")]
    Arbitrary(serde_json::Value),
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub(crate) struct Message {
    pub(crate) id: i32,
    pub(crate) message: String,
    pub(crate) reply_markup: Option<ReplyMarkup>,
    pub(crate) media: Option<Media>,
    pub(crate) date: UnixDateTime,
}

// reply_markup

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

// media

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "_")]
pub(crate) enum Media {
    messageMediaDocument {
        document: Document,
    },
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "_")]
pub(crate) enum Document {
    document {
        mime_type: String,
    },
    #[serde(other)]
    Unknown,
}
