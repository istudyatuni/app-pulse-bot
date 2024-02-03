# Telegram bot for checking updates

Supported sources:

- https://t.me/alexstranniklite

## Developing

First of all, you should define some variables. Copy `.env`

```sh
cp .env.sample .env
```

and fill it.

## Building for production

```sh
cargo b --release --features=prod

# or build with cross
just build-release
```

## Interly

Also take a look at [Interly](crates/interly/README.md), internalization library with [Fluent](https://projectfluent.org) support. This project translated with Interly.

## Development

Enable git hooks:

```sh
git config core.hooksPath .githooks
```
