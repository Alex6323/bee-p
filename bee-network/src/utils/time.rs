use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

pub fn timestamp_millis() -> u64 {
    let unix_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock error");

    unix_time.as_secs() * 1000 + u64::from(unix_time.subsec_millis())
}
