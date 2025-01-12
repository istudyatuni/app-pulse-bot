use camino::Utf8PathBuf;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Lit, LitStr, Result, Token, Type,
};

#[derive(Debug)]
pub enum Migration {
    /// Name of migration from folder with SQL migrations
    Raw(String),
    /// Name and struct of migration written in rust
    Rust(String, Type),
}

#[derive(Debug)]
pub struct Migrations {
    pub app: String,
    pub fake: Option<usize>,
    pub folder: String,
    pub register_fake_fn: Option<Ident>,
    pub register_fn: Ident,
    pub migrations: Vec<(usize, Migration)>,
}

impl Parse for Migration {
    fn parse(input: ParseStream) -> Result<Self> {
        let arrow = Token![=>];
        if input.peek(LitStr) && input.peek2(arrow) {
            // Parse "name" => SomeStruct
            let name: LitStr = input.parse()?;
            input.parse::<Token![=>]>()?;
            let ty: Type = input.parse()?;
            Ok(Migration::Rust(name.value(), ty))
        } else if input.peek(LitStr) {
            let name: LitStr = input.parse()?;
            Ok(Migration::Raw(name.value()))
        } else {
            Err(input.error("Expected a string literal or an identifier"))
        }
    }
}

impl Parse for Migrations {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut app = None;
        let mut fake = None;
        let mut folder = None;
        let mut register_fake_fn = None;
        let mut register_fn = None;
        let mut list = Vec::new();

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "app" => {
                    if app.is_some() {
                        return Err(input.error("Duplicate key 'app'"));
                    }
                    let value: Lit = input.parse()?;
                    if let Lit::Str(lit_str) = value {
                        app = Some(lit_str.value());
                    } else {
                        return Err(input.error("Expected a string for 'app'"));
                    }
                }
                "fake" => {
                    if fake.is_some() {
                        return Err(input.error("Duplicate key 'fake'"));
                    }
                    let value: Lit = input.parse()?;
                    if let Lit::Int(lit_int) = value {
                        fake = Some(lit_int.base10_parse::<usize>()?);
                    } else {
                        return Err(input.error("Expected an integer for 'fake'"));
                    }
                }
                "folder" => {
                    if folder.is_some() {
                        return Err(input.error("Duplicate key 'folder'"));
                    }
                    let value: Lit = input.parse()?;
                    if let Lit::Str(lit_str) = value {
                        folder = Some(lit_str.value());
                    } else {
                        return Err(input.error("Expected a string for 'folder'"));
                    }
                }
                "register_fake_fn" => {
                    if register_fake_fn.is_some() {
                        return Err(input.error("Duplicate key 'register_fake_fn'"));
                    }
                    register_fake_fn = Some(input.parse()?);
                }
                "register_fn" => {
                    if register_fn.is_some() {
                        return Err(input.error("Duplicate key 'register_fn'"));
                    }
                    register_fn = Some(input.parse()?);
                }
                "migrations" => {
                    if !list.is_empty() {
                        return Err(input.error("Duplicate key 'migrations'"));
                    }
                    let content;
                    syn::bracketed!(content in input);
                    let pairs: Punctuated<_, _> = content.parse_terminated(
                        |content| {
                            let key: Lit = content.parse()?;
                            content.parse::<Token![:]>()?;
                            let value: Migration = content.parse()?;
                            Ok((key, value))
                        },
                        Token![,],
                    )?;
                    for (key, value) in pairs {
                        if let Lit::Int(lit_int) = key {
                            list.push((lit_int.base10_parse::<usize>()?, value));
                        } else {
                            return Err(input.error("Expected an integer key for migrations"));
                        }
                    }
                }
                _ => return Err(input.error("Unexpected key in input")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        match (fake, &register_fake_fn) {
            (Some(_), None) => {
                return Err(input.error("'fake' requires 'register_fake_fn' to be defined"))
            }
            (None, Some(_)) => {
                return Err(input.error("'register_fake_fn' requires 'fake' to be defined"))
            }
            _ => (),
        }

        Ok(Migrations {
            app: app.ok_or_else(|| input.error("Missing key 'app'"))?,
            fake,
            folder: folder.ok_or_else(|| input.error("Missing key 'folder'"))?,
            register_fake_fn,
            register_fn: register_fn.ok_or_else(|| input.error("Missing key 'register_fn'"))?,
            migrations: list,
        })
    }
}

pub fn build_migrations(input: Migrations) -> TokenStream {
    let operation_struct = |i| Ident::new(&format!("Operation{i}"), Span::call_site());
    let migration_struct = |i| Ident::new(&format!("Migration{i}"), Span::mixed_site());

    let app = &input.app;

    let structs_defs = input.migrations.iter().map(|(i, m)| {
        let migration_ident = migration_struct(i);
        let op = match m {
            Migration::Raw(_) => {
                let op_ident = operation_struct(i);
                quote! { pub struct #op_ident; }
            }
            Migration::Rust(_, _) => quote! {},
        };
        quote! {
            pub struct #migration_ident;
            #op
        }
    });

    let path = Utf8PathBuf::from(input.folder);
    let migrations = input.migrations.iter().map(|(i, m)| match m {
        Migration::Raw(name) => {
            let ident = migration_struct(i);
            let path_prefix = path.join(name).to_string();
            quote! {
                ::sqlx_migrator::sqlite_migration! (
                    #ident,
                    #app,
                    #name,
                    ::sqlx_migrator::vec_box![],
                    ::sqlx_migrator::vec_box![(
                        include_str!(concat!(#path_prefix, ".up.sql")),
                        include_str!(concat!(#path_prefix, ".down.sql")),
                    )]
                );
            }
        }
        Migration::Rust(name, op_type) => {
            let migration_ident = migration_struct(i);
            quote! {
                ::sqlx_migrator::sqlite_migration! (
                    #migration_ident,
                    #app,
                    #name,
                    ::sqlx_migrator::vec_box![],
                    ::sqlx_migrator::vec_box![#op_type]
                );
            }
        }
    });

    let migrations_idents: Vec<_> = input
        .migrations
        .iter()
        .map(|(i, _)| migration_struct(i))
        .collect();
    let fake_fn = match (input.register_fake_fn, input.fake) {
        (Some(ident), Some(fake)) => {
            let migrations_idents = migrations_idents.iter().take(fake);
            quote! {
                pub fn #ident(migrator: &mut ::sqlx_migrator::Migrator<::sqlx::Sqlite>) {
                    use ::sqlx_migrator::Info;

                    #(migrator.add_migration(Box::new(#migrations_idents));)*
                }
            }
        }
        (None, None) => quote! {},
        _ => unreachable!("fake and fake_fn should be both defined"),
    };

    let migrations_idents = migrations_idents.iter().skip(input.fake.unwrap_or(0));
    let register_ident = input.register_fn;
    let register_fn = quote! {
        pub fn #register_ident(migrator: &mut ::sqlx_migrator::Migrator<::sqlx::Sqlite>) {
            use ::sqlx_migrator::Info;

            #(migrator.add_migration(Box::new(#migrations_idents));)*
        }
    };

    quote! {
        #(#structs_defs)*
        #(#migrations)*
        #fake_fn
        #register_fn
    }
}
