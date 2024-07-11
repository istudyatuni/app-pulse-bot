use std::{
    sync::RwLock,
    time::{Duration, Instant},
};

#[derive(Debug)]
pub(crate) struct Timer {
    timeout: Duration,
    // probably RwLock is wrong
    timer: RwLock<Instant>,
}

impl Timer {
    pub(crate) fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            timer: RwLock::new(Instant::now() - timeout),
        }
    }
    pub(crate) fn elapsed(&self) -> Duration {
        self.timer
            .read()
            .expect("failed to read from timer: RwLock<Instant>")
            .elapsed()
    }
    pub(crate) fn reset(&self) {
        let mut t = self
            .timer
            .write()
            .expect("failed to write timer: RwLock<Instant>: already blocked");
        *t = Instant::now()
    }
    pub(crate) fn elapsed_remains(&self) -> Option<Duration> {
        if self.elapsed() < self.timeout {
            Some(self.timeout - self.elapsed())
        } else {
            None
        }
    }
}
