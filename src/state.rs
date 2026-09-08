use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Phase::Work => "Work",
            Phase::ShortBreak => "Short break",
            Phase::LongBreak => "Long break",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Phase::Work => "WORK",
            Phase::ShortBreak => "BREAK",
            Phase::LongBreak => "LONG BREAK",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub status: Status,
    pub phase: Phase,
    pub active_preset: String,
    pub phase_duration_secs: u64,
    /// Authoritative remaining time while idle/paused.
    pub remaining_secs: u64,
    /// Authoritative end timestamp (ms since epoch) while running.
    pub end_epoch_ms: Option<u128>,
    pub completed_work_sessions: u32,
    pub updated_at_ms: u128,
}

impl Default for State {
    fn default() -> Self {
        State {
            status: Status::Idle,
            phase: Phase::Work,
            active_preset: "classic".into(),
            phase_duration_secs: 25 * 60,
            remaining_secs: 25 * 60,
            end_epoch_ms: None,
            completed_work_sessions: 0,
            updated_at_ms: 0,
        }
    }
}

impl State {
    pub fn path(state_dir: &Path) -> PathBuf {
        state_dir.join("state.json")
    }

    pub fn load_or_default(state_dir: &Path, default: State) -> State {
        let p = Self::path(state_dir);
        match fs::read_to_string(&p) {
            Ok(s) => serde_json::from_str(&s).unwrap_or(default),
            Err(_) => default,
        }
    }

    /// Save atomically (write to a temp file, then rename) so concurrent
    /// readers (the TUI pane and one-shot action invocations) never see a
    /// half-written file.
    pub fn save(&self, state_dir: &Path) -> io::Result<()> {
        fs::create_dir_all(state_dir)?;
        let final_path = Self::path(state_dir);
        let tmp_path = state_dir.join(format!("state.json.tmp.{}", std::process::id()));
        let data = serde_json::to_string_pretty(self).unwrap_or_default();
        fs::write(&tmp_path, data)?;
        fs::rename(tmp_path, final_path)?;
        Ok(())
    }

    pub fn now_ms() -> u128 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
    }

    /// Remaining seconds right now, derived from the wall clock when
    /// running. This is what makes the timer accurate without a background
    /// daemon: any process (a one-shot action or the TUI pane) can compute
    /// the true remaining time purely from `end_epoch_ms`.
    pub fn live_remaining_secs(&self) -> u64 {
        match self.status {
            Status::Running => {
                let now = Self::now_ms();
                match self.end_epoch_ms {
                    Some(end) if end > now => ((end - now) / 1000) as u64,
                    _ => 0,
                }
            }
            _ => self.remaining_secs,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.status == Status::Running && self.live_remaining_secs() == 0
    }
}
