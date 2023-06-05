use serde::Serialize;
use std::time::{Duration, SystemTime};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Instant(std::time::Instant);

impl Serialize for Instant {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let now = std::time::SystemTime::now();
        let elapsed = self.0.elapsed();
        let time = now.checked_sub(elapsed).unwrap_or(SystemTime::UNIX_EPOCH);

        let datetime: DateTime<Utc> = time.into();
        let timestamp = datetime.format("%Y-%m-%dT%H:%M:%S.%fZ").to_string();
        timestamp.serialize(serializer)
    }
}

impl Instant {
    pub fn now() -> Self {
        Self(std::time::Instant::now())
    }

    pub fn duration_since(&self, earlier: Self) -> Duration {
        self.0.duration_since(earlier.0)
    }

    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}
