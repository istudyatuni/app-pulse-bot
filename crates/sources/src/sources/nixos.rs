#![expect(unused)]

use std::time::Duration;

use common::types::Id;
use db::DB;

use crate::{
    extractor::nixpkgs::{ClientInitError, Nixpkgs},
    timer::Timer,
    UpdateSource,
};

pub(crate) struct Source {
    id: Id,
    timer: Timer,
    client: Nixpkgs,
}

impl UpdateSource for Source {
    type InitError = ClientInitError;

    fn name() -> &'static str {
        "nixpkgs"
    }

    fn new(db: DB, timeout: Duration, source_id: Id) -> Result<Self, Self::InitError> {
        Ok(Self {
            id: source_id,
            timer: Timer::new(timeout),
            client: Nixpkgs::new()?,
        })
    }

    fn wait_remains(&self) -> Option<Duration> {
        self.timer.elapsed_remains()
    }

    fn reset_timer(&self) {
        self.timer.reset()
    }
}
