//! Updates sources
#![allow(unused)]

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

pub(crate) mod alexstranniklite;

pub(crate) const TG_SOURCE_TIMEOUT: Duration = Duration::from_secs(60 * 60);

#[async_trait]
pub(crate) trait UpdateSource {
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
    async fn get_updates(&self) -> Vec<Update>;

    fn reset_timer(&self);

    /// Sleep if timeout isn't end, then fetch updates
    async fn get_updates_or_sleep(&self) -> Vec<Update> {
        self.sleep().await;
        let res = self.get_updates().await;
        self.reset_timer();
        res
    }
}

#[derive(Debug, Default)]
pub(crate) struct Update {
    description: Option<String>,
    description_link: Option<String>,
    update_link: Option<String>,
    app_id: String,
}

impl Update {
    pub(crate) fn builder() -> UpdateBuilder {
        UpdateBuilder {
            update: Update::default(),
        }
    }
    pub(crate) fn app_id(&self) -> &str {
        &self.app_id
    }
    pub(crate) fn description(&self) -> &Option<String> {
        &self.description
    }
    pub(crate) fn description_link(&self) -> &Option<String> {
        &self.description_link
    }
    pub(crate) fn update_link(&self) -> &Option<String> {
        &self.update_link
    }
}

#[derive(Debug)]
pub(crate) struct UpdateBuilder {
    update: Update,
}

impl UpdateBuilder {
    pub(crate) fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.update.description = Some(description.into());
        self
    }
    pub(crate) fn description_link<S: Into<String>>(mut self, description_link: S) -> Self {
        self.update.description_link = Some(description_link.into());
        self
    }
    pub(crate) fn update_link<S: Into<String>>(mut self, update_link: S) -> Self {
        self.update.update_link = Some(update_link.into());
        self
    }
    pub(crate) fn app_id<S: Into<String>>(mut self, app_id: S) -> Self {
        self.update.app_id = app_id.into();
        self
    }
    pub(crate) fn build(self) -> Update {
        self.update
    }
}

pub(crate) async fn start_update_loop<S>(
    token: CancellationToken,
    source: S,
    tx: Sender<Vec<Update>>,
) where
    S: UpdateSource + Send + Sync,
{
    tokio::select! {
        _ = token.cancelled() => {}
        _ = async {
            let source = Arc::new(source);
            loop {
                let updates = source.get_updates_or_sleep().await;
                match tx.send(updates).await {
                    Ok(_) => (),
                    Err(_) => log::error!("failed to send update to mpsc, dropping"),
                }
            }
        } => {}
    }
}
