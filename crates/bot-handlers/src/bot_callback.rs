use reqwest::Url;
use teloxide::{
    prelude::*,
    types::{CallbackQuery, InlineKeyboardButtonKind, MessageCommon, MessageKind},
};

use db::{models::ShouldNotify, DB};

use crate::{
    keyboards::{Keyboards, NewAppKeyboardKind},
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
    },
}

pub async fn callback_handler(bot: Bot, q: CallbackQuery, db: DB) -> ResponseResult<()> {
    let answer_err = bot.answer_callback_query(&q.id).show_alert(true);
    let chat_id = q.from.id;

    let lang = db
        .select_user(chat_id.into())
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
            if data.len() != 2 {
                log::error!("invalid callback: {data:?}");
                answer_err
                    .text(tr!(something_wrong_invalid_callback, &lang))
                    .await?;
                return Ok(());
            }

            let new_lang = data[1].to_string();
            Callback::SetLang { lang: new_lang }
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
            let res = handle_update_callback(should_notify, db, chat_id, &app_id, &lang).await?;
            match res {
                Ok((popup_msg, keyboard_kind)) => {
                    bot.answer_callback_query(&q.id).text(popup_msg).await?;
                    edit_callback_msg(q.message, bot, chat_id, &app_id, keyboard_kind, &lang)
                        .await?;
                }
                Err(Some(e)) => {
                    answer_err.text(e).await?;
                }
                _ => (),
            }
        }
        Callback::SetLang { lang } => {
            let res = handle_lang_callback(db, chat_id, &lang).await?;
            match res {
                Ok(popup_msg) => {
                    bot.answer_callback_query(&q.id).text(popup_msg).await?;
                    remove_callback_keyboard(&q.message, bot.clone(), chat_id).await?;
                    edit_welcome_msg(&q.message, bot, chat_id, &lang).await?;
                }
                Err(Some(e)) => {
                    answer_err.text(e).await?;
                }
                _ => (),
            }
        }
    }

    Ok(())
}

async fn handle_update_callback(
    should_notify: ShouldNotify,
    db: DB,
    chat_id: UserId,
    app_id: &str,
    lang: &str,
) -> ResponseResult<Result<(String, NewAppKeyboardKind), Option<String>>> {
    match db
        .save_should_notify_user(chat_id.into(), app_id, should_notify)
        .await
    {
        Ok(()) => (),
        Err(e) => {
            log::error!("failed to save user should_notify: {e}");
            return Ok(Err(Some(tr!(something_wrong_try_again, lang))));
        }
    }
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
            return Ok(Err(None));
        }
    };
    Ok(Ok((popup_msg, keyboard_kind)))
}

async fn handle_lang_callback(
    db: DB,
    chat_id: UserId,
    lang: &str,
) -> ResponseResult<Result<String, Option<String>>> {
    match db.save_user_lang(chat_id.into(), lang).await {
        Ok(()) => (),
        Err(e) => {
            log::error!("failed to update lang for user: {e}");
            return Ok(Err(Some(tr!(something_wrong_try_again, lang))));
        }
    }

    Ok(Ok(tr!(lang_saved, lang)))
}

async fn edit_welcome_msg(
    msg: &Option<Message>,
    bot: Bot,
    chat_id: UserId,
    lang: &str,
) -> ResponseResult<()> {
    if let Some(Message { id, .. }) = msg {
        bot.edit_message_text(chat_id, id.clone(), tr!(welcome_suggest_subscribe, lang))
            .await?;
    }
    Ok(())
}

async fn edit_callback_msg(
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

async fn remove_callback_keyboard(
    msg: &Option<Message>,
    bot: Bot,
    chat_id: UserId,
) -> ResponseResult<()> {
    if let Some(Message { id, .. }) = msg {
        bot.edit_message_reply_markup(chat_id, id.clone()).await?;
    }
    Ok(())
}

// Assuming in message's keyboard only one "Url" button
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
