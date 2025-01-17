use reqwest::Url;

use common::{
    types::{AppId, SourceId},
    UnixDateTime,
};

#[derive(Debug, Default)]
pub struct UpdatesList {
    pub updates: Vec<Update>,
    pub source_id: SourceId,
    pub last_update: UnixDateTime,
}

impl UpdatesList {
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }
    pub fn count(&self) -> usize {
        self.updates.len()
    }
}

#[derive(Debug, Default, bon::Builder)]
pub struct Update {
    description: Option<String>,
    description_link: Option<Url>,
    update_link: Option<Url>,
    /// When `None`, this update is about new app
    app_id: Option<AppId>,
    name: String,
    update_time: Option<UnixDateTime>,
    update_version: Option<String>,
}

impl Update {
    pub fn app_id(&self) -> Option<AppId> {
        self.app_id
    }
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
    pub fn description_link(&self) -> &Option<Url> {
        &self.description_link
    }
    pub fn update_link(&self) -> &Option<Url> {
        &self.update_link
    }
    pub fn update_time(&self) -> Option<UnixDateTime> {
        self.update_time
    }
    pub fn update_version(&self) -> Option<&str> {
        self.update_version.as_deref()
    }
}
