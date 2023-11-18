use reqwest::Url;
use teloxide::{
    prelude::*,
    types::{
        CallbackQuery, MediaKind, MediaText, MessageCommon, MessageEntity, MessageEntityKind,
        MessageKind,
    },
};

use db::{models::ShouldNotify, DB};

use crate::{
    keyboards::{Keyboards, NewAppKeyboardKind},
    tr, IGNORE_TOKEN, NOTIFY_TOKEN, USER_LANG,
};

pub async fn callback_handler(bot: Bot, q: CallbackQuery, db: DB) -> ResponseResult<()> {
    let answer_err = bot.answer_callback_query(&q.id).show_alert(true);
    let chat_id = q.from.id;
    let Some(data) = q.data else {
        log::error!("got empty callback {} from user {}", q.id, chat_id);
        answer_err
            .text(tr!(something_wrong_empty_callback, USER_LANG))
            .await?;
        return Ok(());
    };
    log::debug!("got callback: {:?}", data);

    let data: Vec<_> = data.split(':').collect();
    if data.len() != 2 {
        log::error!("invalid callback: {data:?}");
        answer_err
            .text(tr!(something_wrong_invalid_callback, USER_LANG))
            .await?;
        return Ok(());
    }

    let (app_id, should_notify) = (data[0], data[1]);
    let should_notify = match should_notify {
        NOTIFY_TOKEN => ShouldNotify::Notify,
        IGNORE_TOKEN => ShouldNotify::Ignore,
        _ => {
            log::error!("invalid callback: {data:?}");
            answer_err
                .text(tr!(something_wrong_unknown_callback_type, USER_LANG))
                .await?;
            return Ok(());
        }
    };

    match db
        .save_should_notify_user(chat_id.into(), app_id, should_notify)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            log::error!("failed to save user should_notify: {e}");
            answer_err
                .text(tr!(something_wrong_try_again, USER_LANG))
                .await?;
            return Ok(());
        }
    }

    let (popup_msg, keyboard_kind) = match should_notify {
        ShouldNotify::Notify => (
            tr!(notifications_enabled, USER_LANG),
            NewAppKeyboardKind::NotifyEnabled,
        ),
        ShouldNotify::Ignore => (
            tr!(notifications_disabled, USER_LANG),
            NewAppKeyboardKind::NotifyDisabled,
        ),
        _ => {
            log::error!("unreachable should_notify, data: {data:?}");
            return Ok(());
        }
    };
    bot.answer_callback_query(&q.id).text(popup_msg).await?;
    edit_callback_msg(q.message, bot, chat_id, app_id, keyboard_kind).await?;
    Ok(())
}

async fn edit_callback_msg(
    msg: Option<Message>,
    bot: Bot,
    chat_id: UserId,
    app_id: &str,
    keyboard_kind: NewAppKeyboardKind,
) -> ResponseResult<()> {
    if let Some(Message { id, kind, .. }) = msg {
        bot.edit_message_reply_markup(chat_id, id)
            .reply_markup(Keyboards::update_as_inline_keyboard(
                app_id,
                get_url_from_callback_msg(kind),
                keyboard_kind,
                USER_LANG,
            ))
            .await?;
    }
    Ok(())
}

fn get_url_from_callback_msg(kind: MessageKind) -> Option<Url> {
    if let Some((text, entities)) = get_msg_text_from_callback(kind) {
        for e in entities {
            if e.kind == MessageEntityKind::Url {
                return Url::parse(&text[e.offset..e.offset + e.length]).ok();
            }
        }
    }
    None
}

fn get_msg_text_from_callback(kind: MessageKind) -> Option<(String, Vec<MessageEntity>)> {
    if let MessageKind::Common(MessageCommon {
        media_kind: MediaKind::Text(MediaText { text, entities }),
        ..
    }) = kind
    {
        Some((text, entities))
    } else {
        None
    }
}
