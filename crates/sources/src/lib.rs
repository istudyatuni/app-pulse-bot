//! Updates sources
#![expect(async_fn_in_trait)]

use std::time::Duration;

use tokio::sync::mpsc::Sender;

use common::types::SourceId;
use db::DB;

pub use sources::spawn_sources_update_jobs;
pub use update::*;

mod extractor;
mod sources;
mod timer;
mod update;

pub(crate) const SOURCE_TIMEOUT: Duration = Duration::from_secs(60 * 60);

/// Source that can be fetched for update
pub trait UpdateSource {
    type InitError;

    fn name() -> &'static str;

    fn description() -> &'static str;

    fn new(db: DB, timeout: Duration, source_id: SourceId) -> Result<Self, Self::InitError>
    where
        Self: Sized;

    /// Fetch updates
    async fn get_updates(&self) -> UpdatesList;

    /// Sleep if timeout isn't end, then fetch updates
    async fn get_updates_after_sleep(&self) -> UpdatesList {
        self.sleep().await;
        let res = self.get_updates().await;
        self.reset_timer();
        res
    }

    /// How long should wait until next fetch, None if should not wait
    fn wait_remains(&self) -> Option<Duration>;

    /// Wait until timeout end
    async fn sleep(&self) {
        if let Some(d) = self.wait_remains() {
            tokio::time::sleep(d).await;
        }
    }

    fn reset_timer(&self);
}

/// Start update loop for UpdateSourceList
pub async fn start_list_update_loop<S>(source: S, tx: Sender<UpdatesList>)
where
    S: UpdateSource + Send + Sync,
{
    loop {
        let updates = source.get_updates_after_sleep().await;
        if updates.is_empty() {
            continue;
        }
        match tx.send(updates).await {
            Ok(()) => log::debug!("sending updates"),
            Err(_) => log::error!("failed to send update to mpsc, dropping"),
        }
    }
}
