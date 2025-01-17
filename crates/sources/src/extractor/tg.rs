#![allow(non_camel_case_types)]

use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use common::{LogError, UnixDateTime};

const API_URL: &str = "https://tg.i-c-a.su/json/";
const API_LIMIT_MSGS: u32 = 10;
const MAX_RETRIES: u32 = 5;

/// Returns messages in order from new to old.
pub(crate) async fn fetch_public_channel(name: &str) -> Result<Vec<Message>> {
    log::debug!("fetching updates for {name}");

    // retry on FLOOD_WAIT
    for _ in 0..MAX_RETRIES {
        match fetch_public_channel_impl(name).await {
            Err(FetchError::FloodWait(wait)) => {
                tokio::time::sleep(wait).await;
            },
            res => {
                res.log_error_msg("failed to fetch");
                return res.map_err(Into::into);
            },
        }
    }
    log::error!("failed to fetch telegram/{name} in {MAX_RETRIES} retries");
    Err(FetchError::FloodWaitFailed.into())
}

async fn fetch_public_channel_impl(name: &str) -> Result<Vec<Message>, FetchError> {
    const FLOOD_WAIT: &str = "FLOOD_WAIT_";

    log::debug!("fetching public channel {name}");
    let raw: String = reqwest::get(format!("{API_URL}{name}?limit={API_LIMIT_MSGS}"))
        .await?
        .text()
        .await?;
    let res: Response = serde_json::from_str(&raw)?;
    if let Some(errors) = res.errors {
        for e in &errors {
            match e {
                ResponseError::String(s) if s.starts_with(FLOOD_WAIT) => {
                    let parsed = s.trim_start_matches(FLOOD_WAIT).parse::<u64>();
                    let Ok(sec) = parsed else {
                        log::error!("failed to parse seconds from FLOOD_WAIT_X ({s}) error");
                        break;
                    };
                    let dur = Duration::from_secs(sec);
                    log::warn!("got FLOOD_WAIT_X, waiting for {dur:?}");
                    return Err(FetchError::FloodWait(dur));
                },
                _ => (),
            }
        }
        return Err(FetchError::Arbitrary(errors));
    }
    if res.messages.is_empty() {
        return Err(FetchError::Empty { full: raw });
    }
    Ok(res.messages)
}

#[derive(Debug, Serialize, Deserialize)]
struct Response {
    #[serde(default = "Vec::new")]
    messages: Vec<Message>,
    errors: Option<Vec<ResponseError>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ResponseError {
    String(String),
    Any(serde_json::Value),
}

#[derive(Debug, thiserror::Error)]
enum FetchError {
    #[error("flood wait: {0:?}")]
    FloodWait(Duration),
    #[error("retry on flood wait failed in {MAX_RETRIES} retries")]
    FloodWaitFailed,
    #[error("got errors: {0:?}")]
    Arbitrary(Vec<ResponseError>),
    #[error("got no messages, raw content: {full:?}")]
    Empty { full: String },
    #[error("network error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("invalid json: {0}")]
    JsonParse(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Default))]
pub(crate) struct Message {
    pub(crate) id: i32,
    #[serde(deserialize_with = "deserialize_null_default")]
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

// https://github.com/serde-rs/serde/issues/1098#issuecomment-760711617
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() -> Result<()> {
        let r: Response = serde_json::from_str(r#"{"errors": ["FLOOD_WAIT_23", ["asdf"]]}"#)?;
        assert!(r.messages.is_empty());
        assert!(r.errors.is_some_and(|e| !e.is_empty()
            && match &e[0] {
                ResponseError::String(s) => s == "FLOOD_WAIT_23",
                _ => false,
            }));

        let r: Response = serde_json::from_str(r#"{}"#)?;
        assert!(r.messages.is_empty());
        assert!(r.errors.is_none());

        Ok(())
    }
}
