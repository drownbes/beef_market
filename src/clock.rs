use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mockall::automock;

#[automock]
pub trait Clock: Send {
    fn utc(&self) -> Duration;
}

pub struct DefaultClock;

impl Clock for DefaultClock {
    fn utc(&self) -> Duration {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
    }
}
