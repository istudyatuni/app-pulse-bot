use reqwest::Url;
use teloxide::{
    prelude::*,
    types::{
        CallbackQuery, InlineKeyboardButtonKind, InlineKeyboardMarkup, MaybeInaccessibleMessage,
        MessageCommon, MessageKind,
    },
};

use common::types::Id;
use db::{models::ShouldNotify, DB};

use crate::{
    callback::{Callback, CallbackParseError},
    keyboards::{Keyboards, LanguagesKeyboardKind, NewAppKeyboardKind},
    tr, DEFAULT_USER_LANG,
};

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

    let callback = match Callback::try_from(&data) {
        Ok(c) => c,
        Err(e) => {
            let msg = match e {
                CallbackParseError::InvalidCallback => {
                    log::error!("invalid callback: {data:?}");
                    tr!(something_wrong_invalid_callback, &lang)
                }
                CallbackParseError::InvalidToken => {
                    log::error!("invalid token in callback: {data:?}");
                    tr!(something_wrong_invalid_callback, &lang)
                }
                CallbackParseError::OutdatedCallback => {
                    log::warn!("handling outdated callback");
                    tr!(outdated_callback, &lang)
                }
                CallbackParseError::UnknownCallbackType => {
                    log::error!("unknown callback: {data:?}");
                    tr!(something_wrong_unknown_callback_type, &lang)
                }
            };
            answer_err.text(msg).await?;
            return Ok(());
        }
    };

    match callback {
        Callback::Notify {
            source_id,
            app_id,
            should_notify,
        } => {
            let res =
                handle_update_callback(should_notify, db, chat_id, source_id, app_id, &lang).await;
            match res {
                Ok((popup_msg, keyboard_kind)) => {
                    bot.answer_callback_query(&q.id).text(popup_msg).await?;
                    edit_update_msg(
                        q.message,
                        bot,
                        chat_id,
                        source_id,
                        app_id,
                        keyboard_kind,
                        &lang,
                    )
                    .await?;
                }
                Err(Some(e)) => {
                    answer_err.text(e).await?;
                }
                _ => (),
            }
        }
        Callback::SetLang { lang, kind } => match handle_lang_callback(db, chat_id, &lang).await {
            Ok(popup_msg) => {
                bot.answer_callback_query(&q.id).text(popup_msg).await?;
                let (text, markup) = match kind {
                    LanguagesKeyboardKind::Start => (tr!(welcome_suggest_subscribe, &lang), None),
                    LanguagesKeyboardKind::Settings => (
                        tr!(choose_language, &lang),
                        Some(Keyboards::languages(kind)),
                    ),
                };
                edit_msg_text(q.message, bot, chat_id, text, markup).await?;
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
    source_id: Id,
    app_id: Id,
    lang: &str,
) -> Result<(String, NewAppKeyboardKind), Option<String>> {
    db.save_should_notify_user(chat_id, source_id, app_id, should_notify)
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
    msg: Option<MaybeInaccessibleMessage>,
    bot: Bot,
    chat_id: UserId,
    text: S,
    markup: Option<M>,
) -> ResponseResult<()>
where
    S: Into<String>,
    M: Into<InlineKeyboardMarkup>,
{
    if let Some(Message { id, .. }) = msg.and_then(|m| m.regular_message().cloned()) {
        let mut e = bot.edit_message_text(chat_id, id, text);
        if let Some(m) = markup {
            e = e.reply_markup(m.into());
        }
        e.await?;
    } else {
        log::error!("tried edit msg in chat {chat_id}, but it's not accessible")
    }
    Ok(())
}

async fn edit_update_msg(
    msg: Option<MaybeInaccessibleMessage>,
    bot: Bot,
    chat_id: UserId,
    source_id: Id,
    app_id: Id,
    keyboard_kind: NewAppKeyboardKind,
    lang: &str,
) -> ResponseResult<()> {
    if let Some(Message { id, kind, .. }) = msg.and_then(|m| m.regular_message().cloned()) {
        bot.edit_message_reply_markup(chat_id, id)
            .reply_markup(
                Keyboards::update(
                    source_id,
                    app_id,
                    extract_url_from_callback_msg(kind),
                    keyboard_kind,
                    lang,
                )
                .into(),
            )
            .await?;
    } else {
        log::error!("tried edit update msg in chat {chat_id}, but it's not accessible")
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
