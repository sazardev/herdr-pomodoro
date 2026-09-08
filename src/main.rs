mod config;
mod notify;
mod state;
mod theme;
mod timer;
mod ui;

use config::Config;
use state::State;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Single source of truth for the plugin's version: Cargo's own
/// `CARGO_PKG_VERSION`, which cargo derives from `[package] version` in
/// Cargo.toml at compile time. `scripts/bump-version` keeps that value in
/// sync with `herdr-plugin.toml` -- see CHANGELOG.md for release history.
const VERSION: &str = env!("CARGO_PKG_VERSION");

const USAGE: &str = "usage: herdr-pomodoro [run|open|start|pause|toggle|skip|reset [--all]|cycle-mood|cycle-preset|status|version]";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("run");

    match cmd {
        "run" => run_tui(),
        "open" => open_pane(),
        "start" | "resume" => apply(timer::start_or_resume),
        "pause" => apply(|s, _c| timer::pause(s)),
        "toggle" => apply(timer::toggle),
        "skip" => apply(timer::skip),
        "reset" => {
            let all = args.get(2).map(String::as_str) == Some("--all");
            if all {
                apply(timer::reset_all);
            } else {
                apply(timer::reset_current);
            }
        }
        "cycle-mood" => cycle_mood(),
        "cycle-preset" => cycle_preset(),
        "status" => print_status(),
        "-V" | "--version" | "version" => println!("herdr-pomodoro {VERSION}"),
        "-h" | "--help" | "help" => println!("{USAGE}"),
        other => {
            eprintln!("herdr-pomodoro: unknown command '{other}'");
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

fn state_dir() -> PathBuf {
    env::var_os("HERDR_PLUGIN_STATE_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

fn config_dir() -> PathBuf {
    env::var_os("HERDR_PLUGIN_CONFIG_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
}

fn load_state(cfg: &Config, sdir: &Path) -> State {
    let phase = state::Phase::Work;
    let active_preset = cfg.active_preset.clone();
    let secs = timer::full_phase_secs(cfg, phase, &active_preset);
    let seed = State {
        active_preset,
        phase_duration_secs: secs,
        remaining_secs: secs,
        ..State::default()
    };
    State::load_or_default(sdir, seed)
}

/// Load config + state, run a mutation, save state. This is what every
/// headless one-shot subcommand (as invoked by a Herdr keybinding/action)
/// boils down to -- no daemon, just a quick read-modify-write against the
/// shared state file also used by the interactive pane.
fn apply(f: impl FnOnce(&mut State, &Config)) {
    let sdir = state_dir();
    let cdir = config_dir();
    let cfg = Config::load_or_init(&cdir);
    let mut st = load_state(&cfg, &sdir);
    f(&mut st, &cfg);
    let _ = st.save(&sdir);
}

fn cycle_mood() {
    let cdir = config_dir();
    let mut cfg = Config::load_or_init(&cdir);
    let next = ui::Mood::parse(&cfg.mood).next();
    cfg.mood = next.as_str().to_string();
    cfg.save(&cdir);
}

fn cycle_preset() {
    let sdir = state_dir();
    let cdir = config_dir();
    let mut cfg = Config::load_or_init(&cdir);
    let mut st = load_state(&cfg, &sdir);
    let next = cfg.next_preset_name(&st.active_preset);
    timer::apply_preset_change(&mut st, &cfg, &next);
    cfg.active_preset = next;
    cfg.save(&cdir);
    let _ = st.save(&sdir);
}

fn print_status() {
    let sdir = state_dir();
    let cdir = config_dir();
    let cfg = Config::load_or_init(&cdir);
    let st = load_state(&cfg, &sdir);
    let rem = st.live_remaining_secs();
    println!(
        "{:?} {} {:02}:{:02} preset={}",
        st.status,
        st.phase.short_label(),
        rem / 60,
        rem % 60,
        st.active_preset
    );
}

/// Ask the running Herdr instance to open this plugin's pane, using the
/// binary Herdr itself points us at via HERDR_BIN_PATH so this works the
/// same whether Herdr is talking over a Unix socket or a Windows named
/// pipe.
fn open_pane() {
    let bin = env::var("HERDR_BIN_PATH").unwrap_or_else(|_| "herdr".to_string());
    let plugin_id = env::var("HERDR_PLUGIN_ID").unwrap_or_default();
    if plugin_id.is_empty() {
        eprintln!("herdr-pomodoro: HERDR_PLUGIN_ID is not set -- run this via `herdr plugin action invoke`, not directly.");
        return;
    }
    let status = Command::new(bin)
        .args([
            "plugin",
            "pane",
            "open",
            "--plugin",
            &plugin_id,
            "--entrypoint",
            "timer",
            "--placement",
            "split",
            "--direction",
            "right",
            "--focus",
        ])
        .status();
    if let Err(e) = status {
        eprintln!("herdr-pomodoro: failed to open pane: {e}");
    }
}

fn run_tui() {
    let sdir = state_dir();
    let cdir = config_dir();
    let cfg = Config::load_or_init(&cdir);
    let state = load_state(&cfg, &sdir);

    let entrypoint = env::var("HERDR_PLUGIN_ENTRYPOINT_ID").unwrap_or_default();
    let (initial_mood, persist_mood) =
        if entrypoint == "quick" { (Some(ui::Mood::Digital), false) } else { (None, true) };

    let mut app = ui::App::new(state, cfg, initial_mood, persist_mood, cdir.clone());

    if let Err(e) = tui::run(&mut app, &sdir, &cdir) {
        eprintln!("herdr-pomodoro: {e}");
    }
}

mod tui {
    use super::{config, notify, state, timer, ui};
    use crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use state::{Phase, State};
    use std::io::{self, Stdout};
    use std::path::Path;
    use std::time::{Duration, SystemTime};

    struct TerminalGuard {
        terminal: Terminal<CrosstermBackend<Stdout>>,
    }

    impl TerminalGuard {
        fn new() -> io::Result<Self> {
            enable_raw_mode()?;
            let mut stdout = io::stdout();
            execute!(stdout, EnterAlternateScreen)?;
            let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
            Ok(Self { terminal })
        }
    }

    impl Drop for TerminalGuard {
        fn drop(&mut self) {
            let _ = disable_raw_mode();
            let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
            let _ = self.terminal.show_cursor();
        }
    }

    fn mtime(p: &Path) -> Option<SystemTime> {
        std::fs::metadata(p).ok()?.modified().ok()
    }

    pub fn run(app: &mut ui::App, sdir: &Path, cdir: &Path) -> io::Result<()> {
        let mut guard = TerminalGuard::new()?;
        let state_path = State::path(sdir);
        let config_path = config::Config::path(cdir);
        let mut last_state_mtime = mtime(&state_path);
        let mut last_config_mtime = mtime(&config_path);

        loop {
            let current_state_mtime = mtime(&state_path);
            if current_state_mtime != last_state_mtime {
                app.state = State::load_or_default(sdir, app.state.clone());
                last_state_mtime = current_state_mtime;
            }

            let current_config_mtime = mtime(&config_path);
            if current_config_mtime != last_config_mtime {
                app.reload_config();
                last_config_mtime = current_config_mtime;
            }

            let prev_phase = app.state.phase;
            if timer::tick_check_completion(&mut app.state, &app.cfg) {
                let _ = app.state.save(sdir);
                last_state_mtime = mtime(&state_path);
                if app.cfg.sound {
                    notify::bell();
                }
                if app.cfg.notify {
                    let body = match prev_phase {
                        Phase::Work => "Work session complete -- take a break",
                        Phase::ShortBreak | Phase::LongBreak => "Break's over -- back to work",
                    };
                    notify::system_notify("herdr-pomodoro", body);
                }
                app.note_phase_complete(prev_phase);
            }

            guard.terminal.draw(|f| ui::render(f, app))?;

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && handle_key(app, sdir, key.code) {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key(app: &mut ui::App, sdir: &Path, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char(' ') | KeyCode::Char('s') => {
                timer::toggle(&mut app.state, &app.cfg);
                let _ = app.state.save(sdir);
            }
            KeyCode::Char('n') => {
                timer::skip(&mut app.state, &app.cfg);
                let _ = app.state.save(sdir);
                app.set_toast("skipped to next phase");
            }
            KeyCode::Char('R') => {
                timer::reset_all(&mut app.state, &app.cfg);
                let _ = app.state.save(sdir);
                app.set_toast("full reset");
            }
            KeyCode::Char('r') => {
                timer::reset_current(&mut app.state, &app.cfg);
                let _ = app.state.save(sdir);
                app.set_toast("phase reset");
            }
            KeyCode::Char('m') => app.cycle_mood(),
            KeyCode::Char('p') => app.cycle_preset(),
            KeyCode::Char('?') => app.show_help = !app.show_help,
            _ => {}
        }
        false
    }
}
