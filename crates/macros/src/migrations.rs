use camino::{Utf8Path, Utf8PathBuf};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
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

    let up_path = |folder: &Utf8Path, name| folder.join(format!("{name}.up.sql"));
    let down_path = |folder: &Utf8Path, name| folder.join(format!("{name}.down.sql"));

    let sql = |ty, text: &str| format!("#### {ty} migration\n```sql\n{}\n```", text.trim());

    let app = &input.app;
    let path = Utf8PathBuf::from(input.folder);

    let defs = input.migrations.iter().map(|(i, m)| {
        let migration_ident = migration_struct(i);
        let operation = match m {
            Migration::Raw(name) => {
                let op_ident = operation_struct(i);

                let cur = std::env::current_dir().unwrap();
                let up = up_path(&path, name);
                let down = down_path(&path, name);
                let up = std::fs::read_to_string(&up)
                    .unwrap_or_else(|e| format!("-- failed to read {}/{up}: {e}", cur.display()));
                let down = std::fs::read_to_string(&down)
                    .unwrap_or_else(|e| format!("-- failed to read {}/{down}: {e}", cur.display()));
                let op_doc = format!(
                    "Operation for `{name}`\n{}\n{}",
                    sql("Up", &up),
                    sql("Down", &down)
                );

                quote! {
                    #[doc = #op_doc]
                    pub struct #op_ident;

                    ::sqlx_migrator::sqlite_migration! (
                        #migration_ident,
                        #app,
                        #name,
                        ::sqlx_migrator::vec_box![],
                        ::sqlx_migrator::vec_box![(#up, #down)]
                    );
                }
            }
            Migration::Rust(_, _) => quote! {},
        };
        let migration = match m {
            Migration::Raw(_) => {
                quote! {}
            }
            Migration::Rust(name, op_type) => {
                quote! {
                    ::sqlx_migrator::sqlite_migration! (
                        #migration_ident,
                        #app,
                        #name,
                        ::sqlx_migrator::vec_box![],
                        ::sqlx_migrator::vec_box![super::#op_type]
                    );
                }
            }
        };
        let doc = match m {
            Migration::Raw(name) => {
                let op_ident = operation_struct(i);
                format!(
                    "Migration for `{name}`\n\nSee [`super::__migrations::{}`]",
                    op_ident.to_token_stream().to_string().replace(' ', "")
                )
            }
            Migration::Rust(name, ty) => format!(
                "Migration `{name}`\n\nSee [`super::{}`]",
                ty.to_token_stream().to_string().replace(' ', "")
            ),
        };
        quote! {
            #[doc = #doc]
            pub struct #migration_ident;
            #operation
            #migration
        }
    });

    let migrations_idents: Vec<_> = input
        .migrations
        .iter()
        .map(|(i, _)| migration_struct(i))
        .collect();
    let fake_fn = match (input.register_fake_fn, input.fake) {
        (Some(fake_ident), Some(fake)) => {
            let migrations_idents = migrations_idents.iter().take(fake);
            quote! {
                /// Register fake migrations
                pub fn #fake_ident(migrator: &mut ::sqlx_migrator::Migrator<::sqlx::Sqlite>) {
                    use ::sqlx_migrator::Info;

                    #(migrator.add_migration(Box::new(__migrations::#migrations_idents));)*
                }
            }
        }
        (None, None) => quote! {},
        _ => unreachable!("fake and fake_fn should be both defined"),
    };

    let migrations_idents = migrations_idents.iter().skip(input.fake.unwrap_or(0));
    let register_ident = input.register_fn;
    let register_fn = quote! {
        /// Register migrations
        pub fn #register_ident(migrator: &mut ::sqlx_migrator::Migrator<::sqlx::Sqlite>) {
            use ::sqlx_migrator::Info;

            #(migrator.add_migration(Box::new(__migrations::#migrations_idents));)*
        }
    };

    quote! {
        /// Module with generated operations and migrations
        mod __migrations {
            #(#defs)*
        }
        #fake_fn
        #register_fn
    }
}
