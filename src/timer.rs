use crate::config::Config;
use crate::state::{Phase, State, Status};

pub fn full_phase_secs(cfg: &Config, phase: Phase, preset_name: &str) -> u64 {
    let p = cfg.preset(preset_name);
    let mins = match phase {
        Phase::Work => p.work_minutes,
        Phase::ShortBreak => p.short_break_minutes,
        Phase::LongBreak => p.long_break_minutes,
    };
    (mins * 60).max(1)
}

pub fn start_or_resume(state: &mut State, cfg: &Config) {
    match state.status {
        Status::Running => {}
        Status::Paused => {
            let now = State::now_ms();
            state.end_epoch_ms = Some(now + (state.remaining_secs as u128) * 1000);
            state.status = Status::Running;
        }
        Status::Idle => {
            let secs = full_phase_secs(cfg, state.phase, &state.active_preset);
            state.phase_duration_secs = secs;
            state.remaining_secs = secs;
            let now = State::now_ms();
            state.end_epoch_ms = Some(now + (secs as u128) * 1000);
            state.status = Status::Running;
        }
    }
    state.updated_at_ms = State::now_ms();
}

pub fn pause(state: &mut State) {
    if state.status == Status::Running {
        state.remaining_secs = state.live_remaining_secs();
        state.end_epoch_ms = None;
        state.status = Status::Paused;
        state.updated_at_ms = State::now_ms();
    }
}

pub fn toggle(state: &mut State, cfg: &Config) {
    if state.status == Status::Running {
        pause(state);
    } else {
        start_or_resume(state, cfg);
    }
}

/// Reset the current phase back to its full duration, keeping session
/// counters and which phase we're in.
pub fn reset_current(state: &mut State, cfg: &Config) {
    let secs = full_phase_secs(cfg, state.phase, &state.active_preset);
    state.phase_duration_secs = secs;
    state.remaining_secs = secs;
    state.end_epoch_ms = None;
    state.status = Status::Idle;
    state.updated_at_ms = State::now_ms();
}

/// Full reset: back to phase 1 of the work/break cycle.
pub fn reset_all(state: &mut State, cfg: &Config) {
    state.phase = Phase::Work;
    state.completed_work_sessions = 0;
    reset_current(state, cfg);
}

/// Move on to the next phase per standard pomodoro rules: every Nth
/// completed work session earns a long break instead of a short one.
pub fn advance_phase(state: &mut State, cfg: &Config) {
    let preset = cfg.preset(&state.active_preset);
    match state.phase {
        Phase::Work => {
            state.completed_work_sessions += 1;
            let n = preset.sessions_before_long_break.max(1);
            state.phase = if state.completed_work_sessions.is_multiple_of(n) {
                Phase::LongBreak
            } else {
                Phase::ShortBreak
            };
        }
        Phase::ShortBreak | Phase::LongBreak => {
            state.phase = Phase::Work;
        }
    }

    let secs = full_phase_secs(cfg, state.phase, &state.active_preset);
    state.phase_duration_secs = secs;
    state.remaining_secs = secs;

    if cfg.auto_start_next {
        let now = State::now_ms();
        state.end_epoch_ms = Some(now + (secs as u128) * 1000);
        state.status = Status::Running;
    } else {
        state.end_epoch_ms = None;
        state.status = Status::Idle;
    }
    state.updated_at_ms = State::now_ms();
}

pub fn skip(state: &mut State, cfg: &Config) {
    advance_phase(state, cfg);
}

/// Call once per tick. Returns true (and performs the phase transition)
/// when the running phase has just hit zero.
pub fn tick_check_completion(state: &mut State, cfg: &Config) -> bool {
    if state.is_finished() {
        advance_phase(state, cfg);
        true
    } else {
        false
    }
}

pub fn apply_preset_change(state: &mut State, cfg: &Config, new_preset: &str) {
    state.active_preset = new_preset.to_string();
    if state.status == Status::Idle {
        let secs = full_phase_secs(cfg, state.phase, &state.active_preset);
        state.phase_duration_secs = secs;
        state.remaining_secs = secs;
    }
    state.updated_at_ms = State::now_ms();
}
