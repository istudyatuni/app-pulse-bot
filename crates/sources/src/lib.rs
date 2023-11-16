//! Updates sources
use std::time::Duration;

use async_trait::async_trait;
use reqwest::Url;
use tokio::sync::mpsc::Sender;

pub mod alexstranniklite;

mod extractor;

pub(crate) const TG_SOURCE_TIMEOUT: Duration = Duration::from_secs(60 * 60);

#[async_trait]
pub trait UpdateSource {
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
pub struct Update {
    description: Option<String>,
    description_link: Option<Url>,
    update_link: Option<Url>,
    app_id: String,
}

impl Update {
    pub(crate) fn builder() -> UpdateBuilder {
        UpdateBuilder {
            update: Update::default(),
        }
    }
    pub fn app_id(&self) -> &str {
        &self.app_id
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn description_link(&self) -> &Option<Url> {
        &self.description_link
    }
    pub fn update_link(&self) -> &Option<Url> {
        &self.update_link
    }
}

#[derive(Debug)]
pub(crate) struct UpdateBuilder {
    update: Update,
}

impl UpdateBuilder {
    #[allow(unused)]
    pub(crate) fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.update.description = Some(description.into());
        self
    }
    /// `description_link` should be valid URL, otherwise it will be ignored
    pub(crate) fn description_link(mut self, description_link: &str) -> Self {
        let Ok(url) = Url::parse(description_link) else {
            return self;
        };
        self.update.description_link = Some(url);
        self
    }
    /// `update_link` should be valid URL, otherwise it will be ignored
    pub(crate) fn update_link(mut self, update_link: &str) -> Self {
        let Ok(url) = Url::parse(update_link) else {
            return self;
        };
        self.update.update_link = Some(url);
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

pub async fn start_update_loop<S>(source: S, tx: Sender<Vec<Update>>)
where
    S: UpdateSource + Send + Sync,
{
    loop {
        let updates = source.get_updates_or_sleep().await;
        match tx.send(updates).await {
            Ok(_) => log::debug!("sending updates"),
            Err(_) => log::error!("failed to send update to mpsc, dropping"),
        }
    }
}
