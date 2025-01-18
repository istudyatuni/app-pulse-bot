use std::time::Duration;

use common::types::SourceId;
use db::DB;

use crate::{
    extractor::nixpkgs::{ClientInitError, Nixpkgs},
    timer::Timer,
    Update, UpdateSource,
};

pub(crate) struct Source {
    id: SourceId,
    timer: Timer,
    client: Nixpkgs,
    db: DB,
}

impl Source {
    async fn get_updates_list(&self) -> super::UpdatesList {
        let apps = match self.db.get_apps_to_check_updates(self.id).await {
            Ok(a) => a,
            Err(e) => {
                log::error!(
                    "failed to get apps from db to check updates for source {}: {e}",
                    Self::name()
                );
                return super::UpdatesList::default();
            },
        };

        let mut updates = vec![];

        for app in apps {
            log::debug!("querying app {app:?} from nixpkgs");
            let data = match self.client.search_exact(app.name()).await {
                Ok(a) => a,
                Err(e) => {
                    log::error!("failed to search app {} in {}: {e}", app.name(), Self::name());
                    continue;
                },
            };
            let Some(data) = data.first() else {
                continue;
            };
            let app_name = app.name().to_owned();
            let app_id = match self.db.get_app_id(self.id, &app_name).await {
                Ok(id) => {
                    if id.is_none() {
                        log::warn!("app by name {app_name} not found in db");
                    }
                    id
                },
                Err(e) => {
                    log::error!("failed to get app_id by app_name ({app_name}): {e}");
                    continue;
                },
            };
            updates.push(
                Update::builder()
                    .name(app_name)
                    .maybe_app_id(app_id.map(Into::into))
                    .update_version(data.version.clone())
                    .build(),
            );
        }

        super::UpdatesList {
            updates,
            source_id: self.id,
            last_update: 0,
        }
    }
}

impl UpdateSource for Source {
    type InitError = ClientInitError;

    fn name() -> &'static str {
        "nixpkgs@packages"
    }

    fn description() -> &'static str {
        "NixOS packages"
    }

    fn new(db: DB, timeout: Duration, source_id: SourceId) -> Result<Self, Self::InitError> {
        Ok(Self {
            id: source_id,
            timer: Timer::new(timeout),
            client: Nixpkgs::new()?,
            db,
        })
    }

    async fn get_updates(&self) -> super::UpdatesList {
        self.get_updates_list().await
    }

    fn wait_remains(&self) -> Option<Duration> {
        self.timer.elapsed_remains()
    }

    fn reset_timer(&self) {
        self.timer.reset()
    }
}
