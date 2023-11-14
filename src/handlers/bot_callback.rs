use reqwest::Url;
use teloxide::{
    prelude::*,
    types::{
        CallbackQuery, MediaKind, MediaText, MessageCommon, MessageEntity, MessageEntityKind,
        MessageKind,
    },
};

use crate::{
    db::{models::ShouldNotify, DB},
    tg::{Keyboards, NewAppKeyboardKind},
    IGNORE_TOKEN, NOTIFY_TOKEN,
};

pub(crate) async fn callback_handler(bot: Bot, q: CallbackQuery, db: DB) -> ResponseResult<()> {
    let answer_err = bot.answer_callback_query(&q.id).show_alert(true);
    let chat_id = q.from.id;
    let Some(data) = q.data else {
        log::error!("got empty callback {} from user {}", q.id, chat_id);
        answer_err
            .text("Something went wrong (empty callback)")
            .await?;
        return Ok(());
    };
    log::debug!("got callback: {:?}", data);

    let data: Vec<_> = data.split(':').collect();
    if data.len() != 2 {
        log::error!("wrong callback: {data:?}");
        answer_err
            .text("Something went wrong (wrong callback)")
            .await?;
        return Ok(());
    }

    let (app_id, should_notify) = (data[0], data[1]);
    let should_notify = match should_notify {
        NOTIFY_TOKEN => ShouldNotify::Notify,
        IGNORE_TOKEN => ShouldNotify::Ignore,
        _ => {
            log::error!("wrong callback: {data:?}");
            answer_err
                .text("Something went wrong (unknown callback type)")
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
                .text("Something went wrong, please try again")
                .await?;
            return Ok(());
        }
    }

    let (popup_msg, keyboard_kind) = match should_notify {
        ShouldNotify::Notify => ("Notifications enabled", NewAppKeyboardKind::NotifyEnabled),
        ShouldNotify::Ignore => ("Notifications disabled", NewAppKeyboardKind::NotifyDisabled),
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
