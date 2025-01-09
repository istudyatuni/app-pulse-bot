//! Updates sources
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

mod extractor;
mod sources;
mod timer;
mod update;

pub use sources::spawn_sources_update_jobs;
pub use update::*;

pub(crate) const SOURCE_TIMEOUT: Duration = Duration::from_secs(60 * 60);

/// Source that can be fetched for update
#[async_trait]
pub trait UpdateSource {
    type InitError;

    /// Create source with default timeout
    fn new() -> Result<Self, Self::InitError>
    where
        Self: Sized,
    {
        Self::with_timeout(SOURCE_TIMEOUT)
    }

    /// Create source with specific timeout
    fn with_timeout(timeout: Duration) -> Result<Self, Self::InitError>
    where
        Self: std::marker::Sized;

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

#[async_trait]
pub trait UpdateSourceList: UpdateSource {
    /// Fetch updates
    async fn get_updates(&self) -> UpdatesList;

    /// Sleep if timeout isn't end, then fetch updates
    async fn get_updates_after_sleep(&self) -> UpdatesList {
        self.sleep().await;
        let res = self.get_updates().await;
        self.reset_timer();
        res
    }
}

/// Start update loop for UpdateSourceList
pub async fn start_list_update_loop<S>(source: S, tx: Sender<UpdatesList>)
where
    S: UpdateSourceList + Send + Sync,
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
