use reqwest::Url;
use teloxide::{
    prelude::*,
    types::{
        CallbackQuery, InlineKeyboardButtonKind, InlineKeyboardMarkup, MessageCommon, MessageKind,
    },
};

use db::{models::ShouldNotify, DB};

use crate::{
    keyboards::{Keyboards, LanguagesKeyboardToken, NewAppKeyboardKind},
    tr, DEFAULT_USER_LANG, IGNORE_TOKEN, NOTIFY_FLAG, NOTIFY_TOKEN, SET_LANG_FLAG,
};

#[derive(Debug)]
enum Callback {
    Notify {
        app_id: String,
        should_notify: ShouldNotify,
    },
    SetLang {
        lang: String,
        token: LanguagesKeyboardToken,
    },
}

pub async fn callback_handler(bot: Bot, q: CallbackQuery, db: DB) -> ResponseResult<()> {
    let answer_err = bot.answer_callback_query(&q.id).show_alert(true);
    let chat_id = q.from.id;

    let lang = db
        .select_user(chat_id)
        .await
        .ok()
        .flatten()
        .map(|u| u.lang().to_string())
        .unwrap_or(DEFAULT_USER_LANG.to_string());

    let Some(data) = q.data else {
        log::error!("got empty callback {} from user {}", q.id, chat_id);
        answer_err
            .text(tr!(something_wrong_empty_callback, &lang))
            .await?;
        return Ok(());
    };
    log::debug!("got callback: {:?}", data);

    let data: Vec<_> = data.split(':').collect();
    let callback_type = match data[0] {
        NOTIFY_FLAG => {
            if data.len() != 3 {
                log::error!("invalid callback: {data:?}");
                answer_err
                    .text(tr!(something_wrong_invalid_callback, &lang))
                    .await?;
                return Ok(());
            }

            let (app_id, should_notify) = (data[1].to_string(), data[2]);
            let should_notify = match should_notify {
                NOTIFY_TOKEN => ShouldNotify::Notify,
                IGNORE_TOKEN => ShouldNotify::Ignore,
                _ => {
                    return Ok(());
                }
            };
            Callback::Notify {
                app_id,
                should_notify,
            }
        }
        SET_LANG_FLAG => {
            if data.len() != 3 {
                log::error!("invalid callback: {data:?}");
                answer_err
                    .text(tr!(something_wrong_invalid_callback, &lang))
                    .await?;
                return Ok(());
            }

            let token: Option<LanguagesKeyboardToken> = data[1].try_into().ok();
            let Some(token) = token else {
                log::error!("invalid token in callback: {data:?}");
                answer_err
                    .text(tr!(something_wrong_invalid_callback, &lang))
                    .await?;
                return Ok(());
            };
            Callback::SetLang {
                lang: data[2].to_string(),
                token,
            }
        }
        _ => {
            log::error!("invalid callback: {data:?}");
            answer_err
                .text(tr!(something_wrong_unknown_callback_type, &lang))
                .await?;
            return Ok(());
        }
    };

    match callback_type {
        Callback::Notify {
            app_id,
            should_notify,
        } => {
            let res = handle_update_callback(should_notify, db, chat_id, &app_id, &lang).await;
            match res {
                Ok((popup_msg, keyboard_kind)) => {
                    bot.answer_callback_query(&q.id).text(popup_msg).await?;
                    edit_update_msg(q.message, bot, chat_id, &app_id, keyboard_kind, &lang).await?;
                }
                Err(Some(e)) => {
                    answer_err.text(e).await?;
                }
                _ => (),
            }
        }
        Callback::SetLang { lang, token } => match handle_lang_callback(db, chat_id, &lang).await {
            Ok(popup_msg) => {
                bot.answer_callback_query(&q.id).text(popup_msg).await?;
                let (text, markup) = match token {
                    LanguagesKeyboardToken::Start => (tr!(welcome_suggest_subscribe, &lang), None),
                    LanguagesKeyboardToken::Settings => (
                        tr!(choose_language, &lang),
                        Some(Keyboards::languages(token)),
                    ),
                };
                edit_msg_text(&q.message, bot, chat_id, text, markup).await?;
            }
            Err(e) => {
                answer_err.text(e).await?;
            }
        },
    }

    Ok(())
}

async fn handle_update_callback(
    should_notify: ShouldNotify,
    db: DB,
    chat_id: UserId,
    app_id: &str,
    lang: &str,
) -> Result<(String, NewAppKeyboardKind), Option<String>> {
    db.save_should_notify_user(chat_id, app_id, should_notify)
        .await
        .map_err(|e| {
            log::error!("failed to save user should_notify: {e}");
            Some(tr!(something_wrong_try_again, lang))
        })?;
    let (popup_msg, keyboard_kind) = match should_notify {
        ShouldNotify::Notify => (
            tr!(notifications_enabled, lang),
            NewAppKeyboardKind::NotifyEnabled,
        ),
        ShouldNotify::Ignore => (
            tr!(notifications_disabled, lang),
            NewAppKeyboardKind::NotifyDisabled,
        ),
        _ => {
            log::error!("unreachable: ShouldNotify::Unspecified");
            return Err(None);
        }
    };
    Ok((popup_msg, keyboard_kind))
}

async fn handle_lang_callback(db: DB, chat_id: UserId, lang: &str) -> Result<String, String> {
    db.save_user_lang(chat_id, lang).await.map_err(|e| {
        log::error!("failed to update lang for user: {e}");
        tr!(something_wrong_try_again, lang)
    })?;
    Ok(tr!(lang_saved, lang))
}

async fn edit_msg_text<S, M>(
    msg: &Option<Message>,
    bot: Bot,
    chat_id: UserId,
    text: S,
    markup: Option<M>,
) -> ResponseResult<()>
where
    S: Into<String>,
    M: Into<InlineKeyboardMarkup>,
{
    if let Some(Message { id, .. }) = msg {
        let mut e = bot.edit_message_text(chat_id, *id, text);
        if let Some(m) = markup {
            e = e.reply_markup(m.into());
        }
        e.await?;
    }
    Ok(())
}

async fn edit_update_msg(
    msg: Option<Message>,
    bot: Bot,
    chat_id: UserId,
    app_id: &str,
    keyboard_kind: NewAppKeyboardKind,
    lang: &str,
) -> ResponseResult<()> {
    if let Some(Message { id, kind, .. }) = msg {
        bot.edit_message_reply_markup(chat_id, id)
            .reply_markup(
                Keyboards::update(
                    app_id,
                    extract_url_from_callback_msg(kind),
                    keyboard_kind,
                    lang,
                )
                .into(),
            )
            .await?;
    }
    Ok(())
}

/// Assuming in message's keyboard only one
/// [`InlineKeyboardButtonKind::Url`] button
fn extract_url_from_callback_msg(kind: MessageKind) -> Option<Url> {
    if let MessageKind::Common(MessageCommon {
        reply_markup: Some(markup),
        ..
    }) = kind
    {
        for button in markup.inline_keyboard.iter().flatten() {
            match &button.kind {
                InlineKeyboardButtonKind::Url(url) => return Some(url.clone()),
                _ => continue,
            }
        }
    }
    None
}
