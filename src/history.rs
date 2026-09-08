use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const DAY_MS: u128 = 24 * 3600 * 1000;
const WEEK_MS: u128 = 7 * DAY_MS;

fn log_path(state_dir: &Path) -> PathBuf {
    state_dir.join("history.log")
}

fn now_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
}

/// Append one completed-work-session timestamp. One plain integer per
/// line -- no format to get wrong, and small enough (well under a
/// kilobyte per year of heavy use) that it never needs rotation. This is
/// best-effort: a failure here (disk full, permissions) never blocks the
/// timer itself.
pub fn record(state_dir: &Path) {
    let _ = std::fs::create_dir_all(state_dir);
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(log_path(state_dir)) {
        let _ = writeln!(f, "{}", now_ms());
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Summary {
    pub last_24h: u64,
    pub last_7d: u64,
    pub total: u64,
}

/// Rolling windows (last 24h / last 7 days) rather than calendar
/// day/week -- avoids pulling in a timezone-aware date crate just to
/// bucket a handful of integers.
pub fn summarize(state_dir: &Path) -> Summary {
    let now = now_ms();
    let mut s = Summary::default();
    let Ok(content) = std::fs::read_to_string(log_path(state_dir)) else {
        return s;
    };
    for line in content.lines() {
        let Ok(ts) = line.trim().parse::<u128>() else { continue };
        s.total += 1;
        let age = now.saturating_sub(ts);
        if age <= DAY_MS {
            s.last_24h += 1;
        }
        if age <= WEEK_MS {
            s.last_7d += 1;
        }
    }
    s
}
