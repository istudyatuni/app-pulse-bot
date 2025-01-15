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

#[derive(Debug, Default, bon::Builder)]
pub struct Update {
    description: Option<String>,
    description_link: Option<Url>,
    update_link: Option<Url>,
    /// When `None`, this update is about new app
    app_id: Option<Id>,
    #[expect(unused)]
    #[builder(default = "".to_string())]
    name: String,
    update_time: UnixDateTime,
}

impl Update {
    pub fn app_id(&self) -> Option<Id> {
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
