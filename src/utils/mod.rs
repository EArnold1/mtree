pub mod file;

pub mod date {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn unix_timestamp(time: SystemTime) -> u64 {
        time.duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}
