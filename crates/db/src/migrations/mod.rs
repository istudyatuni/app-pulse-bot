use paste::paste;
use sqlx::Sqlite;
use sqlx_migrator::{sqlite_migration, vec_box, Info, Migrator};

/// Provided example [1] for migrating from sqlx doesn't work, see [2]
///
/// [1]: https://github.com/iamsauravsharma/sqlx_migrator#migrate-from-sqlx-default-sql-based-migration
/// [2]: https://github.com/iamsauravsharma/sqlx_migrator/issues/39
macro_rules! make_migration {
    () => {};
    ($num:literal : $name:literal) => {
        paste! {
            #[allow(dead_code)]
            pub(crate) struct [<Operation $num>];

            sqlite_migration! (
                [<Operation $num>],
                "main", // name of app
                $name,
                vec_box![],
                vec_box![(
                    include_str!(concat!("../../../../migrations/", $name, ".up.sql")),
                    include_str!(concat!("../../../../migrations/", $name, ".down.sql")),
                )]
            );
        }
    };
}

macro_rules! make_migrations {
    () => {};
    ($register:ident; $($num:literal : $name:literal),* $(,)?) => {
        $(make_migration!($num : $name);)*

        paste! {
            #[allow(unused)]
            pub(crate) fn $register(migrator: &mut Migrator<Sqlite>) {
                $({
                    migrator.add_migration(Box::new([<Operation $num>]))
                })*
            }
        }
    };
}

// original migrations
make_migrations!(
    register_fake_migrations;
    1: "0001_schema",
    2: "0002_source-seed",
    3: "0003_app-table",
    4: "0004_bot-update-notify",
    5: "0005_bot-update-notify-int-version",
    6: "0006_bot-blocked",
    7: "0007_user-info",
    8: "0008_source-name-unique",
);

// new migrations
make_migrations!(
    register_migrations;
);
