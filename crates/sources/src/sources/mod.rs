use tokio::{sync::mpsc::Sender, task::JoinSet};
use tokio_util::sync::CancellationToken;

use common::spawn_with_token;
use db::DB;

use crate::{start_list_update_loop, UpdateSource, UpdatesList};

mod alexstranniklite;
mod nixos;

macro_rules! spawn_list_sources {
    () => {};
    ($db:ident, $jobs:ident, $token:ident, $tx:ident; $($module:ident),* $(,)?) => {
        $(
            'source: {
                match $db.add_source_or_ignore($module::Source::name()).await {
                    Ok(()) => (),
                    Err(e) => {
                        log::error!("failed to register source {} in db: {e}", stringify!($module));
                        break 'source;
                    },
                }

                match $module::Source::new(
                    $db.clone(),
                    crate::SOURCE_TIMEOUT,
                    $db.get_source_id_by_source_name($module::Source::name())
                        .await
                        // todo: no panic on Err
                        .expect("should get source_id without errors")
                        .expect("source should be in db")
                ) {
                    Ok(source) => {
                        $jobs.spawn(spawn_with_token(
                            $token.clone(),
                            start_list_update_loop(source, $tx.clone()),
                        ));
                    },
                    Err(e) => log::error!("failed to init source {}: {e}", stringify!($module)),
                }
            }
        )*
    };
}

pub async fn spawn_sources_update_jobs(
    db: DB,
    jobs: &mut JoinSet<()>,
    token: CancellationToken,
    tx: Sender<UpdatesList>,
) {
    spawn_list_sources![
        db, jobs, token, tx;
        alexstranniklite,
    ];
}
