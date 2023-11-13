use teloxide::{prelude::*, types::CallbackQuery};

use crate::{
    db::{models::ShouldNotify, DB},
    IGNORE_TOKEN, NOTIFY_TOKEN,
};

pub(crate) async fn callback_handler(bot: Bot, q: CallbackQuery, db: DB) -> ResponseResult<()> {
    bot.answer_callback_query(&q.id).await?;
    let chat_id = q.from.id;
    let Some(data) = q.data else {
        log::error!("got empty callback {} from user {}", q.id, chat_id);
        // todo: answer with alert https://stackoverflow.com/a/57390206
        bot.send_message(chat_id, "Something went wrong").await?;
        return Ok(());
    };
    log::debug!("got callback: {:?}", data);
    let data: Vec<_> = data.split(':').collect();
    if data.len() != 2 {
        log::error!("wrong callback: {data:?}");
        return Ok(());
    }
    let (app_id, should_notify) = (data[0], data[1]);
    let should_notify = match should_notify {
        NOTIFY_TOKEN => ShouldNotify::Notify,
        IGNORE_TOKEN => ShouldNotify::Ignore,
        _ => {
            log::error!("wrong callback: {data:?}");
            return Ok(());
        }
    };
    match db
        .save_should_notify_user(chat_id.into(), app_id, should_notify)
        .await
    {
        Ok(_) => (),
        Err(e) => log::error!("failed to save user should_notify: {e}"),
    }
    Ok(())
}
