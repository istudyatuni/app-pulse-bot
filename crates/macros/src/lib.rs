use proc_macro::TokenStream;
use syn::parse_macro_input;

mod migrations;

/// Build migrations and functions for filling `sqlx_migrate::Migrator`
///
/// ```ignore
/// struct RustOperation;
///
/// impl Operation<sqlx::Sqlite> for RustOperation {
///     // ...
/// }
///
/// build_migrations!(
///     app: "main",
///     folder: "./migrations",
///     fake: 1,
///     register_fake_fn: register_fake_migrations,
///     register_fn: register_migrations,
///     migrations: [
///         1: "0001_schema",
///         2: "custom" => RustOperation,
///     ]
/// );
/// ```
///
/// #### Arguments
///
/// - `folder`: folder with migrations written in SQL, using sqlx file names: {name}.{up/down}.sql
/// - `fake`: number of fake migrations at the start
/// - `register_fake_fn`: name of function that will register fake migrations
/// - register_fn``: name of function that will register normal migrations
///
/// `fake` and `register_fake_fn` is optional
///
/// #### Defining operations
///
/// - `{number}: "name"` will add files "./migrations/name.up.sql" and "./migrations/name.down.sql"
/// - `{number}: "name" => CustomOperation` will use CustomOperation
///
/// #### Required top-level dependencies:
///
/// - sqlx
/// - sqlx_migrator
#[proc_macro]
pub fn build_migrations(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as migrations::Migrations);
    migrations::build_migrations(input).into()
}
