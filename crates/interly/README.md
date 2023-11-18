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
    println!("{}", tr!(hello, "en", "world"));
}
```

## Notes and current limitations

- Default folder is `locales`.
- `-` in filenames are converted to `_`, so `hello-world` and `hello_world` would be considered equivalent, and it would be an error.
- Variables types not detected (I don't know how), only strings.
- Only supported files structure is
```sh
locales
├── en.ftl
└── *.ftl
```
- Languages accepted in form `"en"`, `"en_US"`, etc. Case insensitive, `_` required.
- Languages will always fallback to global fallback. For example, if you have languages `["en", "ru"]` and call `tr!(name, "ru_RU")`, it will fallback to `"en"`.
- Macros should be called at crate top.

## Roadmap

- [x] Default generation with simple .ftl files.
- [ ] Support [selectors] ([docs.rs][expression_select])
- [ ] Support [attributes][attributes].
- [ ] Support [terms] as static methods.
- Macros parameters:
    - [ ] `path` - path to folder with localizations.
    - [ ] `resolver` - how files are stored:
        - `files` - `{path}/*.ftl` (_current behaviour_).
        - `folder` - `{path}/{locale}/*.ftl`.
    - [ ] `set_locale` - how to specify current locale:
        - `init` - set locale on startup.
        - `state` - store locale as state.
        - `call` - specify locale on each function call (_current behaviour_).
    - [ ] `fallback` - global fallback locale.
    - [ ] `sources` - how to load locales sources. Probably this could be solved by providing macro for embedding, and regular struct for manual initialization.
        - `embed` - embed sources to binary (_current behaviour_).
        - `load` - load sources at startup from file system.
    - [ ] `errors` - how to handle errors (probably not required):
        - `ignore`
        - `log`
        - `panic` (_current behaviour_)
- [ ] Fallback with respect to [region][unic_langid_LanguageIdentifier] (`"ru_RU"` -> `"ru"`).
- [ ] Support defining not at crate top (now just not tested, probably this already works).
- [ ] More [translation formats][tr-formats-list] support (long-term)

[selectors]: https://projectfluent.org/fluent/guide/selectors.html
[expression_select]: https://docs.rs/fluent-syntax/latest/fluent_syntax/ast/enum.Expression.html#variant.Select
[attributes]: https://projectfluent.org/fluent/guide/attributes.html
[terms]: https://projectfluent.org/fluent/guide/terms.html
[unic_langid_LanguageIdentifier]: https://docs.rs/unic-langid/latest/unic_langid/struct.LanguageIdentifier.html
[tr-formats-list]: https://docs.weblate.org/en/latest/formats.html

## Q&A

#### Interly doesn't fit your use-case, but do you want to use this library?

Open an issue, probably interly wants this too!

## Whats generated

<details>

For source files

```fluent
# locales/en.ftl
hello-world = Hello, { $name }!
```

```fluent
# locales/ru.ftl
hello-world = Привет, { $name }!
```

```rs
use interly::localize;

#[localize]
pub(crate) struct Localize;

fn main() {
    println!("{}", tr!(hello_world, "en", "world"));
    println!("{}", tr!(hello_world, "ru", "мир"));
    println!("{}", tr!(hello_world, "ru-RU", "мир"));
}
```

Generated (unrelated parts removed):

```rs
// main.rs
pub(crate) struct Localize {
    bundles: __interly::Bundles,
}

mod __interly {
    use ::std::collections::HashMap;
    use ::std::sync::Arc;
    use ::interly::{
        FluentArgs,
        FluentBundle,
        FluentResource,
        IntlLangMemoizer,
        LanguageIdentifier,
        Lazy,
    };

    use super::Localize;

    pub(super) type Bundles = HashMap<
        LANG,
        FluentBundle<Arc<FluentResource>, IntlLangMemoizer>,
    >;

    impl Localize {
        const FALLBACK_LANG: LANG = LANG::EN;

        pub(crate) fn init() -> Self {
            use ::interly::unic_langid::langid;

            let mut resources: HashMap<LanguageIdentifier, Arc<FluentResource>> = HashMap::new();
            let mut locales = vec![];

            let lang = langid!("en");
            locales.push((lang.clone(), "en"));
            resources
                .insert(
                    lang,
                    Arc::new(
                        FluentResource::try_new("hello-world = Hello, { $name }!\n".to_string())
                            .expect("invalid ftl"),
                    ),
                );

            let lang = langid!("ru");
            locales.push((lang.clone(), "ru"));
            resources
                .insert(
                    lang,
                    Arc::new(
                        FluentResource::try_new("hello-world = Привет, { $name }!\n".to_string())
                            .expect("invalid ftl"),
                    ),
                );

            let mut bundles = HashMap::new();
            for lang in locales {
                let mut bundle = FluentBundle::new_concurrent(vec![lang.0.clone()]);
                let _ = bundle.add_resource(resources.get(&lang.0).unwrap().clone());
                bundles.insert(lang.1.into(), bundle);
            }

            Self { bundles }
        }

        pub(crate) fn hello_world(&self, lang: impl Into<LANG>, name: &str) -> String {
            let lang = lang.into();
            let mut bundle = self.bundles.get(&lang).expect("no bundle");
            if !bundle.has_message("hello-world") {
                bundle = self
                    .bundles
                    .get(&Self::FALLBACK_LANG)
                    .expect("no fallback bundle");
            }
            let msg = bundle
                .get_message("hello-world")
                .expect("no message")
                .value()
                .expect("no value in message");
            let mut errs = vec![];
            bundle
                .format_pattern(
                    msg,
                    Some(&FluentArgs::from_iter(vec![("name", name)])),
                    &mut errs,
                )
                .to_string()
        }
    }

    pub(crate) static LOCALIZE: Lazy<Localize> = Lazy::new(|| { Localize::init() });

    #[derive(PartialEq, Eq, Hash)]
    pub(crate) enum LANG {
        EN,
        RU,
    }

    impl From<&str> for LANG {
        fn from(lang: &str) -> Self {
            match lang.to_lowercase().as_str() {
                "en" => Self::EN,
                "ru" => Self::RU,
                _ => Self::EN,
            }
        }
    }
}

#[allow(unused)]
macro_rules! tr {
    ($e:ident, $lang:expr) => {
        tr!($e, $lang,)
    };
    ($e:ident, $lang:expr, $($v:expr),*) => {
        $crate::__interly::LOCALIZE.$e($lang, $($v),*)
    };
}

pub(crate) use tr;

fn main() {
    println!("{}", crate::__interly::LOCALIZE.hello_world("en", "world"));
    println!("{}", crate::__interly::LOCALIZE.hello_world("ru", "мир"));
    println!("{}", crate::__interly::LOCALIZE.hello_world("ru-RU", "мир");
}
```

Output:

```
Hello, world!
Привет, мир!
Hello, мир!
```

</details>
