use heck::{ToShoutySnakeCase, ToSnakeCase};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, Data, DeriveInput, Ident, Visibility};
use unic_langid::{langid, LanguageIdentifier};

use prepare::make_init;

mod locales;
mod prepare;
mod read;

use locales::{extract_messages, LangInfo};

use crate::prepare::make_messages_methods;

const DEFAULT_PATH: &str = "locales";
const DEFAULT_FALLBACK_LOCALE: LanguageIdentifier = langid!("en");

/// Usage
///
/// ```rust
/// use interly::localize;
///
/// #[localize]
/// pub(crate) struct Localize;
///
/// # fn main() {
/// assert_eq!(tr!("hello-world", "en", "your name"), "Hello, your name!".to_string());
/// # }
/// ```
#[proc_macro_attribute]
pub fn localize(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match input.data {
        Data::Struct(_) => (),
        Data::Enum(d) => {
            return quote_spanned! { d.enum_token.span => compile_error!("use struct"); }.into()
        }
        Data::Union(d) => {
            return quote_spanned! { d.union_token.span => compile_error!("use struct"); }.into()
        }
    }

    let dir = DEFAULT_PATH;
    let files = match read::read_files(dir) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to read .ftl files from \"{dir}\": {e}");
            return quote! { compile_error!(#msg); }.into();
        }
    };

    let messages = match extract_messages(files) {
        Ok(m) => m,
        Err(e) => {
            let msg = format!("invalid .ftl files:\n{e}");
            return quote! { compile_error!(#msg); }.into();
        }
    };

    let languages_names: Vec<_> = messages.iter().map(|(lang, _)| lang.to_string()).collect();

    let ident = input.ident;
    let vis = input.vis;
    let res = localize_base(
        vis.clone(),
        ident,
        messages,
        languages_names,
        DEFAULT_FALLBACK_LOCALE,
    );

    res.into()
}

fn localize_base(
    vis: Visibility,
    ident: Ident,
    messages: Vec<(LanguageIdentifier, LangInfo)>,
    languages_names: Vec<String>,
    fallback_locale: LanguageIdentifier,
) -> TokenStream2 {
    let init_fun = make_init(&messages);
    let message_methods = make_messages_methods(vis.clone(), &messages);

    let mut languages_enum_variants = vec![];
    let mut languages_enum_from = vec![];
    for (lang_enum, lang_str) in languages_names
        .iter()
        .map(|l| (l.to_shouty_snake_case(), l.to_snake_case()))
    {
        let lang_enum = syn::Ident::new(&lang_enum, Span::call_site());
        let languages_enum_variant = quote! {
            #lang_enum,
        };
        languages_enum_variants.push(languages_enum_variant);
        let language_enum_from = quote! {
            #lang_str => Self::#lang_enum,
        };
        languages_enum_from.push(language_enum_from);
    }

    let fallback_lang_enum = syn::Ident::new(
        fallback_locale.to_string().to_shouty_snake_case().as_str(),
        Span::call_site(),
    );
    languages_enum_from.push(quote! {
        _ => Self::#fallback_lang_enum,
    });

    quote! {
        #[derive(Default)]
        #vis struct #ident {
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

            use super::#ident;

            pub(super) type Bundles = HashMap<
                LANG,
                FluentBundle<Arc<FluentResource>, IntlLangMemoizer>,
            >;

            impl #ident {
                const FALLBACK_LANG: LANG = LANG::#fallback_lang_enum;

                #vis fn init() -> Self {
                    #init_fun
                }

                #message_methods
            }

            #vis static LOCALIZE: Lazy<#ident> = Lazy::new(|| { #ident::init() });

            #[derive(Debug, PartialEq, Eq, Hash)]
            #vis enum LANG {
                #(#languages_enum_variants)*
            }

            impl From<&str> for LANG {
                fn from(lang: &str) -> Self {
                    match lang.to_lowercase().as_str() {
                        #(#languages_enum_from)*
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

        #vis use tr;
    }
}
