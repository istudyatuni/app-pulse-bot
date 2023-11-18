# Interly

Internalization in Rust

## Usage

```fluent
# locales/en.ftl
hello = Hello, { name }!
```

```rs
// main.rs
use fluent_i18n::localize;

#[localize]
pub(crate) struct Localize;

fn main() {
    println!("{}", tr!(hello, "en", "world")); // Hello, world!
}

// other/module.rs
use crate::tr;

fn your_function() {
    println!("{:?}", );
}
```

## Notes and current limitations

- Default folder is `locales`
- `-` in filenames are converted to `_`, so `hello-world` and `hello_world` would be considered equivalent, so it would be an error.
- [Attributes](https://projectfluent.org/fluent/guide/attributes.html) not supported
- Variables types not detected (I don't know how), only strings
- Only supported files structure is
```sh
locales
├── en.ftl
└── *.ftl
```
- Languages accepted in form `"en"`, `"en_US"`, etc. Case insensitive, `_` required.
- Languages will always fallback to global fallback. If you have languages `["en", "ru"]` and call `tr!(name, "ru_RU")`, it will fallback to `"en"`.

## Whats generated

TBD
