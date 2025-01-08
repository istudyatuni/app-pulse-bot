use teloxide::{
    payloads::SendMessageSetters,
    prelude::{Requester, ResponseResult},
    types::Message,
    Bot,
};

use db::{models::Stats, DB};
use i18n::{tr, tr_literal};

use crate::{bot_messages::get_user_lang, commands::AdminCommand, utils::escape};

pub async fn admin_command_handler(
    bot: Bot,
    msg: Message,
    cmd: AdminCommand,
    db: DB,
) -> ResponseResult<()> {
    let user = db.select_user(msg.chat.id).await.ok().flatten();
    let lang = get_user_lang(user.as_ref(), msg.from());

    match cmd {
        AdminCommand::Stats => match db.load_stats().await {
            Ok(stats) => {
                bot.send_message(msg.chat.id, escape(translate_stats(&stats, &lang)))
                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                    .await?;
            }
            Err(e) => log::error!("failed to get stats: {e}"),
        },
    }

    Ok(())
}

fn translate_stats(stats: &Stats, lang: &str) -> String {
    [
        tr!(stats_header, lang),
        "".to_string(),
        [
            (stats.apps, "stats-apps"),
            (stats.sources, "stats-sources"),
            (stats.users, "stats-users"),
            (stats.blocked_users, "stats-users-blocked"),
        ]
        .into_iter()
        .map(|(value, tr_key)| tr_literal!(tr_key, lang) + ": " + &value.to_string())
        .collect::<Vec<_>>()
        .join("\n"),
    ]
    .join("\n")
}
