use chrono::Utc;
use std::time::UNIX_EPOCH;

pub fn current_unix_time() -> u64 {
    std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn time_diff_now(time: i64) -> String {
    let current_time = Utc::now().timestamp();
    let mut diff = current_time - time;
    if diff < 0 {
        return "in the future (how??)".to_string();
    }

    if diff < 60 {
        if diff == 1 {
            return format!("{diff} second");
        }
        return format!("{diff} seconds");
    }

    diff /= 60;
    if diff < 60 {
        if diff == 1 {
            return format!("{diff} minute");
        }
        return format!("{diff} minutes");
    }

    diff /= 60;
    if diff < 24 {
        if diff == 1 {
            return format!("{diff} hour");
        }
        return format!("{diff} hours");
    }

    diff /= 24;
    if diff < 30 {
        if diff == 1 {
            return format!("{diff} day");
        }
        return format!("{diff} days");
    }

    diff /= 30;
    if diff < 12 {
        if diff == 1 {
            return format!("{diff} month");
        }
        return format!("{diff} months");
    }

    diff /= 12;
    if diff == 1 {
        return format!("{diff} year");
    }
    format!("{diff} years")
}
