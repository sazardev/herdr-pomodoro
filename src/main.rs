mod config;
mod history;
mod notify;
mod onboard;
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

const USAGE: &str = "usage: herdr-pomodoro [run|open|start|pause|toggle|skip|reset [--all]|cycle-mood|cycle-preset|reset-onboarding|status|history|version]";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("run");

    match cmd {
        "run" => run_tui(),
        "open" => open_pane(),
        "auto-open" => auto_open_if_needed(),
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
        "reset-onboarding" => onboard::reset(&state_dir()),
        "status" => print_status(),
        "history" => print_history(),
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
    let before = st.completed_work_sessions;
    f(&mut st, &cfg);
    if st.completed_work_sessions > before {
        history::record(&sdir);
    }
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

fn print_history() {
    let s = history::summarize(&state_dir());
    println!("last_24h={} last_7d={} total={}", s.last_24h, s.last_7d, s.total);
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

/// Invoked from the manifest's `[[startup]]` hook, which runs once per
/// enabled plugin each time Herdr's server (re)starts -- not at install/link
/// time itself (Herdr doesn't run plugin commands then). This is the
/// closest thing to "show setup as soon as it's installed" the plugin
/// startup-hook API allows: the first time Herdr starts after install, the
/// onboarding marker is still absent, so this opens the pane and the user
/// lands straight in the setup wizard. Every later restart it's a silent
/// no-op.
fn auto_open_if_needed() {
    if onboard::is_done(&state_dir()) {
        return;
    }
    open_pane();
}

fn run_tui() {
    let sdir = state_dir();
    let cdir = config_dir();
    let cfg = Config::load_or_init(&cdir);
    let state = load_state(&cfg, &sdir);

    let entrypoint = env::var("HERDR_PLUGIN_ENTRYPOINT_ID").unwrap_or_default();
    let (initial_mood, persist_mood) =
        if entrypoint == "quick" { (Some(ui::Mood::Digital), false) } else { (None, true) };
    let show_onboarding = entrypoint != "quick" && !onboard::is_done(&sdir);

    let mut app = ui::App::new(
        state,
        cfg,
        initial_mood,
        persist_mood,
        cdir.clone(),
        sdir.clone(),
        show_onboarding,
    );

    if let Err(e) = tui::run(&mut app, &sdir, &cdir) {
        eprintln!("herdr-pomodoro: {e}");
    }
}

mod tui {
    use super::{config, history, notify, onboard, state, timer, ui};
    use crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::{backend::CrosstermBackend, Terminal};
    use state::{Phase, State, Status};
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

            let onboarding = matches!(app.screen, ui::Screen::Onboarding(_));

            if !onboarding {
                let prev_phase = app.state.phase;
                let prev_completed = app.state.completed_work_sessions;
                if timer::tick_check_completion(&mut app.state, &app.cfg) {
                    let _ = app.state.save(sdir);
                    last_state_mtime = mtime(&state_path);
                    if app.state.completed_work_sessions > prev_completed {
                        history::record(sdir);
                    }
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
            }

            guard.terminal.draw(|f| ui::render(f, app))?;

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    let quit = if onboarding {
                        handle_onboarding_key(app, sdir, cdir, key.code);
                        false
                    } else {
                        handle_key(app, sdir, key.code)
                    };
                    if quit {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_onboarding_key(app: &mut ui::App, sdir: &Path, cdir: &Path, code: KeyCode) {
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                if let ui::Screen::Onboarding(ob) = &mut app.screen {
                    ui::onboarding::select_prev(ob);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let ui::Screen::Onboarding(ob) = &mut app.screen {
                    ui::onboarding::select_next(ob);
                }
            }
            KeyCode::Left | KeyCode::Char('h') => onboarding_cycle(app, -1),
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => onboarding_cycle(app, 1),
            KeyCode::Enter => {
                app.cfg.save(cdir);
                let _ = app.state.save(sdir);
                onboard::mark_done(sdir);
                app.screen = ui::Screen::Timer;
                app.set_toast("setup saved -- press ? any time for help");
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                // Skip: discard any in-memory tweaks and reload pristine
                // defaults from disk rather than persisting a half-chosen
                // config.
                app.cfg = config::Config::load_or_init(cdir);
                app.state = super::load_state(&app.cfg, sdir);
                app.palette = crate::theme::Palette::from_config(&app.cfg.colors);
                onboard::mark_done(sdir);
                app.screen = ui::Screen::Timer;
            }
            _ => {}
        }
    }

    fn onboarding_cycle(app: &mut ui::App, delta: i32) {
        let selected = match &app.screen {
            ui::Screen::Onboarding(ob) => ob.selected(),
            ui::Screen::Timer => return,
        };
        ui::onboarding::cycle_value(app, selected, delta);
    }

    fn handle_key(app: &mut ui::App, sdir: &Path, code: KeyCode) -> bool {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Char(' ') | KeyCode::Char('s') => {
                let was = app.state.status;
                timer::toggle(&mut app.state, &app.cfg);
                let _ = app.state.save(sdir);
                let msg = match was {
                    Status::Idle => format!("{} started", app.state.phase.label().to_lowercase()),
                    Status::Paused => "resumed".to_string(),
                    Status::Running => "paused".to_string(),
                };
                app.set_toast(msg);
            }
            KeyCode::Char('n') => {
                let before = app.state.completed_work_sessions;
                timer::skip(&mut app.state, &app.cfg);
                let _ = app.state.save(sdir);
                if app.state.completed_work_sessions > before {
                    history::record(sdir);
                }
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
            KeyCode::Char('?') => {
                app.show_help = !app.show_help;
                app.show_stats = false;
            }
            KeyCode::Char('i') => {
                app.show_stats = !app.show_stats;
                app.show_help = false;
            }
            _ => {}
        }
        false
    }
}
