use std::time::Duration;

pub(crate) trait Instant: Copy {
    fn now() -> Self;
    fn add(&mut self, duration: std::time::Duration);
    fn elapsed(&self) -> Duration;
}

impl Instant for std::time::Instant {
    fn now() -> Self {
        Self::now()
    }

    fn add(&mut self, duration: std::time::Duration) {
        *self += duration
    }

    fn elapsed(&self) -> Duration {
        self.elapsed()
    }
}

impl Instant for tokio::time::Instant {
    fn now() -> Self {
        Self::now()
    }

    fn add(&mut self, duration: std::time::Duration) {
        *self += duration
    }

    fn elapsed(&self) -> Duration {
        self.elapsed()
    }
}
