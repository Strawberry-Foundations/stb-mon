use chrono::Utc;
use std::time::UNIX_EPOCH;

pub fn current_unix_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn time_rel(time: i64) -> String {
    let current_time = Utc::now().timestamp();
    let mut diff = current_time - time;
    if diff < 0 {
        return "in the future (how??)".to_string();
    }

    if diff < 60 {
        if diff == 1 {
            return format!("{diff} second ago");
        }
        return format!("{diff} seconds ago");
    }

    diff /= 60;
    if diff < 60 {
        if diff == 1 {
            return format!("{diff} minute ago");
        }
        return format!("{diff} minutes ago");
    }

    diff /= 60;
    if diff < 24 {
        if diff == 1 {
            return format!("{diff} hour ago");
        }
        return format!("{diff} hours ago");
    }

    diff /= 24;
    if diff < 30 {
        if diff == 1 {
            return format!("{diff} day ago");
        }
        return format!("{diff} days ago");
    }

    diff /= 30;
    if diff < 12 {
        if diff == 1 {
            return format!("{diff} month ago");
        }
        return format!("{diff} months ago");
    }

    diff /= 12;
    if diff == 1 {
        return format!("{diff} year ago");
    }
    format!("{diff} years ago")
}
