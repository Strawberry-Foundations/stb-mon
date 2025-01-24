use std::time::UNIX_EPOCH;

pub fn current_unix_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
