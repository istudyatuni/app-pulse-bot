//! Updates sources
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

pub mod alexstranniklite;

mod extractor;
mod timer;
mod update;

pub use update::*;

pub(crate) const SOURCE_TIMEOUT: Duration = Duration::from_secs(60 * 60);

#[async_trait]
pub trait UpdateSource {
    const SOURCE_TYPE: UpdateSourceType;

    /// Create source with default timeout
    fn new() -> Self;

    /// Create source with specific timeout
    fn with_timeout(timeout: Duration) -> Self;

    /// How long should wait until next fetch, None if could not wait
    fn wait_remains(&self) -> Option<Duration>;

    /// Wait until timeout end
    async fn sleep(&self) {
        if let Some(d) = self.wait_remains() {
            tokio::time::sleep(d).await;
        }
    }

    /// Fetch updates
    async fn get_updates(&self) -> UpdatesList;

    fn reset_timer(&self);

    /// Sleep if timeout isn't end, then fetch updates
    async fn get_updates_or_sleep(&self) -> UpdatesList {
        self.sleep().await;
        let res = self.get_updates().await;
        self.reset_timer();
        res
    }
}

#[derive(Debug)]
pub enum UpdateSourceType {
    /// Source send all updates
    List,
}

pub async fn start_update_loop<S>(source: S, tx: Sender<UpdatesList>)
where
    S: UpdateSource + Send + Sync,
{
    loop {
        let updates = source.get_updates_or_sleep().await;
        if updates.is_empty() {
            continue;
        }
        match tx.send(updates).await {
            Ok(()) => log::debug!("sending updates"),
            Err(_) => log::error!("failed to send update to mpsc, dropping"),
        }
    }
}
