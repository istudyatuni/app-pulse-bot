use i18n::tr_literal;
use teloxide::{macros::BotCommands, types::BotCommand};

#[derive(BotCommands, Clone, Copy)]
#[command(rename_rule = "lowercase")]
pub enum Command {
    #[command(description = "off")]
    Start,
    #[command(description = "$subscribe-command")]
    Subscribe,
    #[command(description = "$unsubscribe-command")]
    Unsubscribe,
    #[command(description = "$changelog-command")]
    Changelog,
    #[command(description = "$settings-command")]
    Settings,
    #[command(description = "$about-command")]
    About,
    #[command(description = "$help-command")]
    Help,
}

impl Command {
    pub fn bot_commands_translated(lang: &str) -> impl IntoIterator<Item = BotCommand> {
        use teloxide::utils::command::BotCommands;

        let lang = lang.to_owned();
        Self::bot_commands().into_iter().map(move |c| {
            if c.description.starts_with("$") {
                let description = c.description.trim_start_matches('$');
                let description = tr_literal!(description, lang.as_str());
                c.description(description)
            } else {
                c
            }
        })
    }
    /// Check if command allowed in public chats
    pub(crate) fn allowed_in_public(self) -> bool {
        match self {
            Self::Start | Self::Subscribe | Self::Unsubscribe => false,
            Self::Changelog | Self::Settings | Self::About | Self::Help => true,
        }
    }
}
