## Messages

welcome = Welcome! This bot help you track applications updates.
choose-language = You can choose a language, or /subscribe to keep track of updates
welcome-choose-language = { welcome } { choose-language }
welcome-suggest-subscribe = { welcome }

    Language saved. Subscribe to keep track of updates: /subscribe

new-app-msg = New app to track updates:
new-update-msg = Update for { $app }
subscribed = Subscribed
unsubscribed = Unsubscribed

about-description =
    This bot help you track applications updates.
    Currently only one source supported: @alexstranniklite

    Source code: https://github.com/istudyatuni/app-pulse-bot

how-to-use-header = *How to use this bot:*

how-to-use = { how-to-use-header }

    Bot send you notifications about new apps. Under each message you will see buttons "{ notify-button }" and "{ ignore-button }".

    When you click "{ notify-button }" bot will remember this, and will send you notifications about future updates to this app.

    When you click "{ ignore-button }" bot will stop sending you notifications about future updates.

    If you do not click any of these buttons, bot will send you notifications about this app, but as if it's a new app for you, not an update.

bot-updated = Bot has been updated!

    { changelog }

command-not-available-in-public = This command is available only in private chat with bot

unknown-message = Try /help

## Buttons messages

notify-button = Notify
ignore-button = Ignore
see-update-button = See update

## Other

# https://emojipedia.org/flags
lang-name = 🇺🇸 English

## Errors notifications

-something-went-wrong = Something went wrong

something-wrong-empty-callback = { -something-went-wrong } (empty callback)
something-wrong-invalid-callback = { -something-went-wrong } (invalid callback)
something-wrong-unknown-callback-type = { -something-went-wrong } (unknown callback type)
something-wrong-try-again = { -something-went-wrong }, please try again

## Notifications

-notifications = Notifications

notifications-enabled = { -notifications } enabled
notifications-disabled = { -notifications } disabled

lang-saved = Language saved

not-implemented-already-subscribed = You are already subscribed

## Commands descriptions

commands-list-header = *Supported commands:*
subscribe-command = Subscribe
unsubscribe-command = Unsubscribe
changelog-command = Changelog
settings-command = Configuration
about-command = About this bot
help-command = Display help

## Changelog

changelog-header = *What's new:*

# Only from latest update.
changelog-description =
    - A description of how to use the bot has been added to /help.
    - Added /settings command with the ability to change the language of the bot.

changelog =
    { changelog-header }

    { changelog-description }
