mod analog;
mod digital;
mod flashy;
mod font;
mod help;
mod minimal;

use crate::config::Config;
use crate::state::{Phase, State};
use crate::theme::Palette;
use ratatui::layout::Rect;
use ratatui::Frame;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Minimal,
    Digital,
    Analog,
    Flashy,
}

impl Mood {
    pub fn parse(s: &str) -> Mood {
        match s {
            "minimal" => Mood::Minimal,
            "analog" => Mood::Analog,
            "flashy" => Mood::Flashy,
            _ => Mood::Digital,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Mood::Minimal => "minimal",
            Mood::Digital => "digital",
            Mood::Analog => "analog",
            Mood::Flashy => "flashy",
        }
    }

    pub fn next(self) -> Mood {
        match self {
            Mood::Minimal => Mood::Digital,
            Mood::Digital => Mood::Analog,
            Mood::Analog => Mood::Flashy,
            Mood::Flashy => Mood::Minimal,
        }
    }
}

pub struct App {
    pub state: State,
    pub cfg: Config,
    pub palette: Palette,
    /// In-memory-only mood override (used by the popup "quick glance"
    /// entrypoint and the 'm' key). Not persisted unless the user presses
    /// 'm' in the main dockable pane, which writes it back to config.toml.
    pub mood_override: Option<Mood>,
    pub show_help: bool,
    pub toast: Option<(String, Instant)>,
    pub persist_mood: bool,
    config_dir: PathBuf,
}

impl App {
    pub fn new(
        state: State,
        cfg: Config,
        initial_mood: Option<Mood>,
        persist_mood: bool,
        config_dir: PathBuf,
    ) -> App {
        let palette = Palette::from_config(&cfg.colors);
        App {
            state,
            cfg,
            palette,
            mood_override: initial_mood,
            show_help: false,
            toast: None,
            persist_mood,
            config_dir,
        }
    }

    pub fn configured_mood(&self) -> Mood {
        self.mood_override.unwrap_or_else(|| Mood::parse(&self.cfg.mood))
    }

    /// The mood actually used, downgrading to something that fits when the
    /// pane is small. This is what makes the same binary work equally well
    /// as a full analog/flashy pane or a one-line drawer strip.
    pub fn effective_mood(&self, area: Rect) -> Mood {
        let base = self.configured_mood();
        if area.width < 18 || area.height < 3 {
            Mood::Minimal
        } else if (area.width < 34 || area.height < 10) && base != Mood::Minimal {
            Mood::Digital
        } else {
            base
        }
    }

    pub fn cycle_mood(&mut self) {
        let next = self.configured_mood().next();
        self.mood_override = Some(next);
        if self.persist_mood {
            self.cfg.mood = next.as_str().to_string();
            self.cfg.save(self.config_dir());
        }
        self.set_toast(format!("view: {}", next.as_str()));
    }

    /// Re-read config.toml from disk. Called when a background action
    /// (e.g. the global "cycle mood"/"cycle preset" shortcuts) edited it
    /// while this pane is already open, so changes made elsewhere show up
    /// immediately instead of only on the next `run`.
    pub fn reload_config(&mut self) {
        let cfg = Config::load_or_init(&self.config_dir);
        self.palette = Palette::from_config(&cfg.colors);
        self.cfg = cfg;
        self.mood_override = None;
    }

    pub fn cycle_preset(&mut self) {
        let next = self.cfg.next_preset_name(&self.state.active_preset);
        crate::timer::apply_preset_change(&mut self.state, &self.cfg, &next);
        self.cfg.active_preset = next.clone();
        self.cfg.save(self.config_dir());
        self.set_toast(format!("preset: {}", next));
    }

    pub fn set_toast(&mut self, msg: impl Into<String>) {
        self.toast = Some((msg.into(), Instant::now()));
    }

    pub fn note_phase_complete(&mut self, just_finished: Phase) {
        let msg = match just_finished {
            Phase::Work => "work session done -- take a break",
            Phase::ShortBreak | Phase::LongBreak => "break's over -- back to work",
        };
        self.set_toast(msg);
    }

    fn config_dir(&self) -> &std::path::Path {
        &self.config_dir
    }

    pub fn remaining_secs(&self) -> u64 {
        self.state.live_remaining_secs()
    }

    pub fn progress_fraction(&self) -> f64 {
        let total = self.state.phase_duration_secs.max(1) as f64;
        let rem = self.remaining_secs() as f64;
        ((total - rem) / total).clamp(0.0, 1.0)
    }

    pub fn mmss(&self) -> String {
        let s = self.remaining_secs();
        format!("{:02}:{:02}", s / 60, s % 60)
    }

    pub fn cycle_position(&self) -> (u32, u32) {
        let n = self.cfg.preset(&self.state.active_preset).sessions_before_long_break.max(1);
        let done_in_cycle = self.state.completed_work_sessions % n;
        (done_in_cycle, n)
    }
}

pub fn render(f: &mut Frame<'_>, app: &App) {
    let area = f.area();
    let mood = app.effective_mood(area);
    match mood {
        Mood::Minimal => minimal::render(f, area, app),
        Mood::Digital => digital::render(f, area, app),
        Mood::Analog => analog::render(f, area, app),
        Mood::Flashy => flashy::render(f, area, app),
    }
    if app.show_help && mood != Mood::Minimal {
        help::render(f, area, app);
    }
}
