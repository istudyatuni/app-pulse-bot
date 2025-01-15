use reqwest::Url;

use common::UnixDateTime;
use db::types::Id;

#[derive(Debug, Default)]
pub struct UpdatesList {
    pub updates: Vec<Update>,
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

#[derive(Debug, Default)]
pub struct Update {
    description: Option<String>,
    description_link: Option<Url>,
    update_link: Option<Url>,
    app_id: Id,
    name: String,
    update_time: UnixDateTime,
}

impl Update {
    // todo: use bon
    pub(crate) fn builder() -> UpdateBuilder {
        UpdateBuilder {
            update: Update::default(),
        }
    }
    pub fn app_id(&self) -> Id {
        self.app_id
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
    pub fn update_time(&self) -> UnixDateTime {
        self.update_time
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
    pub(crate) fn app_id(mut self, name: Id) -> Self {
        self.update.app_id = name;
        self
    }
    pub(crate) fn update_time(mut self, update_time: UnixDateTime) -> Self {
        self.update.update_time = update_time;
        self
    }
    pub(crate) fn build(self) -> Update {
        self.update
    }
}
