use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub work_minutes: u64,
    pub short_break_minutes: u64,
    pub long_break_minutes: u64,
    pub sessions_before_long_break: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Colors {
    /// Optional hex overrides, e.g. "#ff6b6b". Leave unset to inherit
    /// whatever colors Herdr's active theme maps the base ANSI palette to.
    pub work: Option<String>,
    pub short_break: Option<String>,
    pub long_break: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub active_preset: String,
    /// "minimal" | "digital" | "analog" | "flashy"
    pub mood: String,
    pub auto_start_next: bool,
    pub sound: bool,
    pub notify: bool,
    #[serde(rename = "preset")]
    pub presets: Vec<Preset>,
    #[serde(default)]
    pub colors: Colors,
}

const DEFAULT_CONFIG_TOML: &str = r##"# herdr-pomodoro configuration
# Edit this file, then just reopen the pane (or press 'p'/'m' inside it) to
# pick up preset/mood changes. Full field reference: see README.md.

# Which [[preset]] below is active by default.
active_preset = "classic"

# Default view: "minimal" | "digital" | "analog" | "flashy"
# minimal: one-line drawer strip. digital: plain big digits, no color.
# analog: countdown clock face. flashy: same as digital but colored by
# phase. All are static (no blinking/animation) and text-only, no icons.
# The pane also auto-downgrades to a smaller view when the pane itself is
# small, regardless of this setting, so it stays readable in a narrow drawer.
mood = "digital"

# Automatically start the next phase (break -> work -> break...) instead of
# waiting at idle for you to press space/s.
auto_start_next = false

# Terminal bell (\a) when a phase completes.
sound = true

# Best-effort desktop notification (notify-send on Linux, osascript on
# macOS) when a phase completes. Silently does nothing if unavailable.
notify = true

[[preset]]
name = "classic"
work_minutes = 25
short_break_minutes = 5
long_break_minutes = 15
sessions_before_long_break = 4

[[preset]]
name = "deep-work"
work_minutes = 50
short_break_minutes = 10
long_break_minutes = 20
sessions_before_long_break = 2

[[preset]]
name = "quick"
work_minutes = 15
short_break_minutes = 3
long_break_minutes = 10
sessions_before_long_break = 4

# Optional color overrides (hex). Omit a line to keep inheriting Herdr's
# active theme via the standard ANSI palette (the default and recommended
# setting -- it's what makes the widget adapt automatically to gruvbox,
# dracula, catppuccin, etc). Colors are only ever used sparingly (the
# countdown and progress bar in "flashy", the clock hand in "analog").
[colors]
# work = "#ff6b6b"
# short_break = "#51cf66"
# long_break = "#339af0"
"##;

impl Config {
    pub fn path(config_dir: &Path) -> PathBuf {
        config_dir.join("config.toml")
    }

    pub fn load_or_init(config_dir: &Path) -> Config {
        let p = Self::path(config_dir);
        if let Ok(s) = fs::read_to_string(&p) {
            toml::from_str(&s).unwrap_or_else(|_| Self::parse_default())
        } else {
            let _ = fs::create_dir_all(config_dir);
            let _ = fs::write(&p, DEFAULT_CONFIG_TOML);
            Self::parse_default()
        }
    }

    fn parse_default() -> Config {
        toml::from_str(DEFAULT_CONFIG_TOML).expect("bundled default config.toml must parse")
    }

    pub fn save(&self, config_dir: &Path) {
        let _ = fs::create_dir_all(config_dir);
        if let Ok(data) = toml::to_string_pretty(self) {
            let _ = fs::write(Self::path(config_dir), data);
        }
    }

    pub fn preset(&self, name: &str) -> Preset {
        self.presets.iter().find(|p| p.name == name).cloned().unwrap_or_else(|| {
            self.presets.first().cloned().unwrap_or(Preset {
                name: "classic".into(),
                work_minutes: 25,
                short_break_minutes: 5,
                long_break_minutes: 15,
                sessions_before_long_break: 4,
            })
        })
    }

    pub fn next_preset_name(&self, current: &str) -> String {
        if self.presets.is_empty() {
            return current.to_string();
        }
        let idx = self.presets.iter().position(|p| p.name == current).unwrap_or(0);
        let next = (idx + 1) % self.presets.len();
        self.presets[next].name.clone()
    }
}
