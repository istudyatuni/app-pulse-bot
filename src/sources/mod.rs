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

#[derive(Debug)]
pub(crate) struct Update {
    description_link: Option<String>,
    update_link: Option<String>,
    app_id: String,
}

impl Update {
    pub(crate) fn link(link: &str, app_id: &str) -> Self {
        Self {
            description_link: Some(link.to_string()),
            update_link: None,
            app_id: app_id.to_string(),
        }
    }
    pub(crate) fn format_message(&self) -> String {
        let mut msg = vec![];
        if let Some(ref link) = self.update_link {
            msg.push(link.as_str());
        }
        msg.join(" ")
    }
    pub(crate) fn app_id(&self) -> &str {
        &self.app_id
    }
    pub(crate) fn description_link(&self) -> &Option<String> {
        &self.description_link
    }
    pub(crate) fn update_link(&self) -> &Option<String> {
        &self.update_link
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
                tx.send(updates)
                    .await
                    .expect("failed to send updates to mspc");
            }
        } => {}
    }
}
