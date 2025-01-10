## Messages

welcome = Привет! Этот бот помогает отслеживать обновления приложений.
choose-language = Вы можете выбрать язык, или подписаться, чтобы отслеживать обновления: /subscribe
welcome-choose-language = { welcome } { choose-language }
welcome-suggest-subscribe = { welcome }

    Язык сохранен. Подписаться, чтобы отслеживать обновления: /subscribe

new-app-msg = Новое приложение для отслеживания:
new-update-msg = Обновление для { $app }
subscribed = Вы подписаны
unsubscribed = Вы отписаны

about-description =
    Этот бот помогает отслеживать обновления приложений.
    Пока что поддерживается только один источник: @alexstranniklite

    Исходный код: https://github.com/istudyatuni/app-pulse-bot

how-to-use-header = *Как пользоваться ботом:*

how-to-use = { how-to-use-header }

    Бот присылает уведомления о новых приложениях. Под каждым сообщением есть кнопки "{ notify-button }" и "{ ignore-button }".

    При нажатии кнопки "{ notify-button }" бот это запоминает, и будет присылать уведомления о последующих обновлениях этого приложения.

    При нажатии кнопки "{ ignore-button }" бот больше не будет присылать уведомления о последующих обновлениях.

    Если не нажать ни одну из этих кнопок, то бот будет присылать уведомления для этого приложения, но как будто это новое приложение, а не обновление.

bot-updated = Бот обновился!

    { changelog }

command-not-available-in-public = Эта команда доступна только в личном чате с ботом

unknown-message = Попробуйте /help

## Buttons messages

notify-button = Уведомлять
ignore-button = Игнорировать
see-update-button = Посмотреть обновление

## Misc

lang-name = 🇷🇺 Русский

## Errors notifications

-something-went-wrong = Что-то пошло не так

something-wrong-empty-callback = { -something-went-wrong } (empty callback)
something-wrong-invalid-callback = { -something-went-wrong } (invalid callback)
something-wrong-unknown-callback-type = { -something-went-wrong } (unknown callback type)
outdated-callback = Эта кнопка больше не может быть использована
something-wrong-try-again = { -something-went-wrong }, попробуйте ещё раз

## Notifications

-notifications = Уведомления

notifications-enabled = { -notifications } включены
notifications-disabled = { -notifications } выключены

lang-saved = Язык сохранён

not-implemented-already-subscribed = Вы уже подписаны

## Commands descriptions

commands-list-header = *Поддерживаемые команды:*
subscribe-command = Подписаться
unsubscribe-command = Отписаться
changelog-command = Список изменений
settings-command = Настройки
about-command = Об этом боте
help-command = Показать справку

# Admin commands

admin-commands-header = *Команды для админа:*
stats-command = Статистика

## Stats

stats-header = *Статистика*
stats-apps = Приложений
stats-sources = Источников
stats-users = Всего пользователей
stats-users-blocked = Пользователей, заблокировавших бота

## Changelog

changelog-header = *Что нового:*

# Only from latest update.
changelog-description =
    - В /help добавлено описание, как пользоваться ботом.
    - Добавлена команда /settings с возможностью изменить язык бота.

changelog =
    { changelog-header }

    { changelog-description }
