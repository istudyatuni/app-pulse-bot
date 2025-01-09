use tokio::{sync::mpsc::Sender, task::JoinSet};
use tokio_util::sync::CancellationToken;

use common::spawn_with_token;

use crate::{start_list_update_loop, UpdateSource, UpdatesList};

mod alexstranniklite;

macro_rules! spawn_list_sources {
    () => {};
    ($jobs:ident, $token:ident, $tx:ident; $($module:ident),* $(,)?) => {
        $(
            match $module::Source::new() {
                Ok(source) => {
                    $jobs.spawn(spawn_with_token(
                        $token.clone(),
                        start_list_update_loop(source, $tx.clone()),
                    ));
                },
                Err(e) => log::error!("failed to start source {}: {e}", stringify!($module)),
            }
        )*
    };
}

pub fn spawn_sources_update_jobs(
    jobs: &mut JoinSet<()>,
    token: CancellationToken,
    tx: Sender<UpdatesList>,
) {
    spawn_list_sources![jobs, token, tx; alexstranniklite];
}
