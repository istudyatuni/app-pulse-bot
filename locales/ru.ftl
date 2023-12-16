## Messages

welcome = Привет!
welcome-choose-language = { welcome } Выбери язык:
welcome-suggest-subscribe = { welcome }

    Теперь вы можете подписаться, чтобы отслеживать обновления: /subscribe

new-app-msg = Новое приложение для отслеживания:
new-update-msg = Обновление для { $app }
subscribed = Вы подписаны
unsubscribed = Вы отписаны

about-description =
    Этот бот помогает отслеживать обновления приложений.
    Пока что поддерживается только один источник: @alexstranniklite

    Исходный код: https://github.com/istudyatuni/app-pulse-bot

bot-updated = Бот обновился! Посмотреть, что нового: /changelog

## Buttons messages

notify-button = Уведомлять
ignore-button = Игнорировать
see-update-button = Посмотреть обновление

## Other

lang-name = 🇷🇺 Русский

## Errors notifications

-something-went-wrong = Что-то пошло не так

something-wrong-empty-callback = { -something-went-wrong } (empty callback)
something-wrong-invalid-callback = { -something-went-wrong } (invalid callback)
something-wrong-unknown-callback-type = { -something-went-wrong } (unknown callback type)
something-wrong-try-again = { -something-went-wrong }, попробуйте ещё раз

## Notifications

-notifications = Уведомления

notifications-enabled = { -notifications } включены
notifications-disabled = { -notifications } выключены

lang-saved = Язык сохранён

not-implemented-already-subscribed = Вы уже подписаны

## Commands descriptions

commands-list-header = Поддерживаемые команды:
subscribe-command = Подписаться
unsubscribe-command = Отписаться
changelog-command = Список изменений
about-command = Об этом боте
help-command = Показать эту справку

## Changelog

changelog-header = Что нового:

# Only from latest update.
changelog-description =
    { changelog-header }

    - Больше не будут приходить несколько повторных сообщений об одном и том же обновлении одного и того же приложения
    - Бот теперь присылает сообщение о своём обновлении
