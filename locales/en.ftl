## Messages

welcome = Welcome!
welcome-choose-language = { welcome } Choose language:
welcome-suggest-subscribe = { welcome }

    Now you can /subscribe to keep track of updates

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

bot-updated = Bot has been updated! Check what's new: /changelog

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
about-command = About this bot
help-command = Display this help

## Changelog

changelog-header = What's new:

# Only from latest update.
changelog-description =
    { changelog-header }

    - No more multiple repeated messages about the same update for the same app
    - Bot now sends a message about its update
