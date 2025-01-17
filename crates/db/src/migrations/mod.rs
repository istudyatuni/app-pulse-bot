use macros::build_migrations;

mod op9_app_int_id;

// Provided example [1] for migrating from sqlx doesn't work, see [2]
//
// [1]: https://github.com/iamsauravsharma/sqlx_migrator#migrate-from-sqlx-default-sql-based-migration
// [2]: https://github.com/iamsauravsharma/sqlx_migrator/issues/39
build_migrations!(
    app: "main",
    folder: "./migrations",
    fake: 7,
    register_fake_fn: register_fake_migrations,
    register_fn: register_migrations,
    migrations: [
        1: "0001_schema",
        2: "0002_source-seed",
        3: "0003_app-table",
        4: "0004_bot-update-notify",
        5: "0005_bot-update-notify-int-version",
        6: "0006_bot-blocked",
        7: "0007_user-info",
        8: "0008_source-name-unique",
        9: "0009_app-int-id" => op9_app_int_id::Operation9AppIntId,
        10: "0010_app-last-updated-version",
        11: "0011_app-name-unique",
    ],
);
