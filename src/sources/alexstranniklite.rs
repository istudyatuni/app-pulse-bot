use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

use async_trait::async_trait;

use super::*;

pub(crate) struct Source {
    timeout: Duration,
    // probably RwLock is wrong
    timer: RwLock<Instant>,
}

impl Source {
    fn elapsed(&self) -> Duration {
        self.timer
            .read()
            .expect("failed to read from timer: RwLock<Instant>")
            .elapsed()
    }
    fn reset_timer(&self) {
        let mut t = self
            .timer
            .write()
            .expect("failed to write timer: RwLock<Instant>: already blocked");
        *t = Instant::now()
    }
}

#[async_trait]
impl UpdateSource for Source {
    fn new() -> Self {
        Self::with_timeout(TG_SOURCE_TIMEOUT)
    }

    fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            timer: RwLock::new(Instant::now() + timeout),
        }
    }

    fn wait_remains(&self) -> Option<Duration> {
        if self.elapsed() < self.timeout {
            Some(self.timeout - self.elapsed())
        } else {
            None
        }
    }

    async fn get_updates(&self) -> Vec<super::Update> {
        if self.wait_remains().is_some() {
            return vec![];
        }

        vec![Update::link(
            "https://t.me/alexstranniklite/18210",
            "daylio",
        )]
    }

    fn reset_timer(&self) {
        self.reset_timer()
    }
}
