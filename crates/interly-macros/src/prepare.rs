use std::collections::HashMap;

use heck::ToSnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Visibility;
use unic_langid::LanguageIdentifier;

use crate::locales::{LangInfo, MessageInfo};

pub(crate) fn make_messages_methods(
    vis: Visibility,
    messages: &Vec<(LanguageIdentifier, LangInfo)>,
) -> TokenStream {
    let mut msgs = vec![];
    let mut msgs_map = HashMap::new();
    for (_, info) in messages {
        for msg @ MessageInfo { id, .. } in &info.messages {
            msgs_map.insert(id, msg);
        }
    }
    for msg in msgs_map.values() {
        msgs.push(make_msg_fn(vis.clone(), msg));
    }
    quote! { #(#msgs)* }
}

fn make_msg_fn(vis: Visibility, msg: &MessageInfo) -> TokenStream {
    let fn_name = syn::Ident::new(msg.id.to_snake_case().as_str(), Span::call_site());
    let mut args = vec![];
    let mut pat_args = vec![];
    for arg_name in &msg.attrs {
        let arg = syn::Ident::new(arg_name.as_str(), Span::call_site());
        args.push(quote! { #arg: &str });
        pat_args.push(quote! { (#arg_name, #arg) });
    }
    let msg_id = &msg.id;
    let pat_args = if args.len() > 0 {
        quote! { Some(&FluentArgs::from_iter(::std::vec![#(#pat_args),*])) }
    } else {
        quote! { None }
    };
    // todo: move most logic to one function like __format_msg(&self, lang, id, args) -> String
    quote! {
        #vis fn #fn_name(&self, lang: impl Into<LANG>, #(#args),*) -> String {
            let lang = lang.into();
            let mut bundle = self.bundles.get(&lang).expect("no bundle");
            if !bundle.has_message(#msg_id) {
                bundle = self.bundles.get(&Self::FALLBACK_LANG).expect("no fallback bundle");
            }
            let msg = bundle
                .get_message(#msg_id)
                .expect("no message")
                .value()
                .expect("no value in message");
            let mut errs = ::std::vec![];
            bundle
                .format_pattern(msg, #pat_args, &mut errs)
                .to_string()
        }
    }
}

pub(crate) fn make_init(messages: &Vec<(LanguageIdentifier, LangInfo)>) -> TokenStream {
    let mut resources_fill = vec![];
    for (lang, info) in messages {
        let lang = lang.to_string();
        let source = info.source.clone();
        let resource_fill = quote! {
            let lang = langid!(#lang);
            locales.push((lang.clone(), #lang));
            resources
                .insert(
                    lang,
                    Arc::new(
                        FluentResource::try_new(#source.to_string()).expect("invalid ftl"),
                    )
                );
        };
        resources_fill.push(resource_fill);
    }

    quote! {
        use ::interly::unic_langid::langid;

        // all imports are available from __interly
        let mut resources: HashMap<LanguageIdentifier, Arc<FluentResource>> = HashMap::new();
        let mut locales = ::std::vec![];

        #(#resources_fill)*

        let mut bundles = HashMap::new();
        for lang in locales {
            let mut bundle = FluentBundle::new_concurrent(::std::vec![lang.0.clone()]);
            let _ = bundle.add_resource(resources.get(&lang.0).unwrap().clone());
            bundles.insert(lang.1.into(), bundle);
        }

        Self { bundles }
    }
}
