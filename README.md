# herdr-pomodoro

A minimal, elegant Pomodoro timer for [Herdr](https://herdr.dev), written in
Rust. Dock it as a one-line strip in a drawer, a small popup for a quick
glance, or a full analog/digital/flashy widget pane -- it adapts to however
much room you give it, and to whatever Herdr theme you're running.

- **Tiny & dependency-light**: single static binary, ~900 KB release build
  (`opt-level = "z"`, LTO, stripped). No daemon: the timer's truth is a
  wall-clock end-timestamp in a state file, so nothing needs to run in the
  background while the pane is closed.
- **Four static, text-only moods**: `minimal` (one-line drawer strip),
  `digital` (plain big block digits, the calm default), `analog` (a
  countdown dial), `flashy` (same layout as `digital`, colored by phase).
  No icons/emoji, no blinking or pulsing, and no border drawn around the
  content -- Herdr already frames the pane, so an inner box just looks like
  a pane nested inside a pane. The pane auto-downgrades to a smaller mood
  when it doesn't have room for the configured one, so it never overflows a
  narrow split.
- **Presets**: classic 25/5/15, deep-work 50/10/20, quick 15/3/10, or your
  own -- edit `config.toml`.
- **Theme-adaptive by construction**: colors are drawn with the standard
  (non-bright) ANSI palette instead of hardcoded hex values, the same way
  any theme-aware terminal app is, and used sparingly -- just the countdown
  and progress bar in `flashy`, and the clock hand in `analog`. Herdr's
  active theme (gruvbox, dracula, catppuccin, nord, ...) remaps that
  palette, so the widget just follows along automatically. Override any
  color with a hex value in `config.toml` if you want a fixed look
  regardless of theme.
- **Global shortcuts** to open/start/pause/skip/reset/cycle-view without
  focusing the pane, plus in-pane keys for everything else.

## Install

Local development (this checkout):

```sh
herdr plugin link /path/to/herdr-pomodoro
```

`plugin link` does not build for you -- run `cargo build --release` first
(and again after pulling changes). Once you publish this to GitHub, others
can install it with:

```sh
herdr plugin install <owner>/herdr-pomodoro
```

which clones, previews, and runs the `[[build]]` step (`cargo build
--release`) automatically.

### First-run setup

A `[[startup]]` hook opens the main pane straight into a setup wizard the
first time Herdr starts after install -- pick your default view, preset,
sound, notifications, and auto-start behavior, with a live preview of each
view as you cycle through them. `enter` saves and starts; `esc`/`q` skips
and keeps the defaults. It only shows once (tracked by a marker file in the
plugin's state dir); redo it any time with `prefix+alt+p` after running
`herdr plugin action invoke reset-onboarding --plugin sazardev.pomodoro`.

Herdr doesn't run plugin commands at install/link time, only when its
server (re)starts -- so on a session that's already running (like right
after `herdr plugin link`), the wizard shows the first time you open the
pane yourself rather than instantly. From the next `herdr` launch onward,
the startup hook opens it for you automatically until it's been completed
or skipped once.

## Using it

- `herdr plugin action invoke open --plugin sazardev.pomodoro` (or the
  `prefix+alt+p` shortcut) opens the dockable pane as a split next to your
  focused pane. From there use Herdr's normal pane commands
  (`herdr pane move`, `herdr pane swap`, or just drag/rebind) to park it
  wherever you want -- left drawer, right rail, its own tab, zoomed, etc.
- The `quick` entrypoint is a small popup (`herdr plugin pane open --plugin
  sazardev.pomodoro --entrypoint quick`) for a glance without touching your
  layout. It starts in a compact digital view regardless of your saved
  mood, and doesn't persist any mood change you make inside it.

### Default shortcuts

All under `prefix+alt+...` so they don't collide with Herdr's own
`prefix+<letter>` bindings. Rebind any of them in your own
`~/.config/herdr/config.toml` (`[[keys.command]]`, `type = "plugin_action"`,
`command = "sazardev.pomodoro.<action>"`).

| Shortcut | Action |
| --- | --- |
| `prefix+alt+p` | Open / focus the pane |
| `prefix+alt+s` | Start / pause |
| `prefix+alt+n` | Skip to next phase |
| `prefix+alt+r` | Reset current phase |
| `prefix+alt+m` | Cycle view (mood) |

### In-pane keys (when the pane is focused)

| Key | Action |
| --- | --- |
| `space` / `s` | Start / pause / resume |
| `n` | Skip to next phase |
| `r` | Reset current phase |
| `R` | Full reset (back to session 1) |
| `m` | Cycle view: minimal -> digital -> analog -> flashy |
| `p` | Cycle preset |
| `i` | Toggle work-session history (last 24h / 7 days / all-time) |
| `?` | Toggle help |
| `q` / `esc` | Close the pane |

Changing mood or preset from anywhere (a global shortcut, `herdr plugin
action invoke`, or the in-pane keys) shows up live in an already-open pane
within ~200ms -- there's no need to reopen it.

Closing the pane does **not** pause the timer: remaining time is derived
from a stored end-timestamp, so it keeps counting down accurately whether
or not anything is watching it, and a phase-complete bell/notification
still fires the next time any `herdr-pomodoro` process (a pane, or a
one-shot action) touches the state file. To always run visible in the
background, keep the pane open, or open it again after the notification.

### History

Every completed work session (finished naturally or skipped) appends one
timestamp to `$HERDR_PLUGIN_STATE_DIR/history.log` -- a plain-text file, one
epoch-millisecond integer per line, small enough (well under a kilobyte a
year) to never need rotation. Press `i` in the pane for a small overlay with
rolling last-24h / last-7-days / all-time counts, or run `herdr-pomodoro
history` for the same numbers from the command line. There's no calendar
day/week bucketing (would need a timezone-aware date crate this plugin
otherwise has no reason to depend on) -- the windows are relative to *now*.

## Configuration

`herdr plugin config-dir sazardev.pomodoro` prints the config directory. On
first run it's seeded with a commented `config.toml`:

```toml
active_preset = "classic"
mood = "digital"
auto_start_next = false
sound = true
notify = true

[[preset]]
name = "classic"
work_minutes = 25
short_break_minutes = 5
long_break_minutes = 15
sessions_before_long_break = 4

# ...deep-work, quick...

[colors]
# work = "#ff6b6b"
# short_break = "#51cf66"
# long_break = "#339af0"
```

Add your own `[[preset]]` blocks freely; `p` / `cycle-preset` walks them in
file order.

## Architecture notes

- State lives in `$HERDR_PLUGIN_STATE_DIR/state.json`; config in
  `$HERDR_PLUGIN_CONFIG_DIR/config.toml`. Both are plain files the plugin
  owns outright (there's no plugin storage API in Herdr's v1 plugin
  surface), written atomically (state) or directly (config).
- Every subcommand (`start`, `pause`, `toggle`, `skip`, `reset`,
  `cycle-mood`, `cycle-preset`, `status`) is a quick read-modify-write
  against that shared state and exits immediately -- that's what makes them
  safe to bind as one-shot Herdr actions/shortcuts. The interactive `run`
  TUI polls the same files (~200ms) so external changes and its own
  keypresses stay in sync without any IPC.
- `open` shells out to `$HERDR_BIN_PATH plugin pane open` using
  `$HERDR_PLUGIN_ID`, so it works regardless of what id you give the plugin
  and whether Herdr is reachable over a Unix socket or a Windows named
  pipe.

## Platforms

Built and tested on Linux (WSL2) against Herdr 0.8.2. `platforms` in the
manifest is currently `["linux", "macos"]`; macOS should work as-is (same
Rust/crossterm stack) but hasn't been tested here. Windows needs a couple
of manifest tweaks this repo doesn't ship yet: build with `cargo build
--release` targeting `x86_64-pc-windows-msvc`, and change the `command`
arrays to point at `.\target\release\herdr-pomodoro.exe` (Herdr resolves
`PATHEXT` shims for bare commands on `PATH`, but not for explicit relative
paths, so the `.exe` needs to be spelled out).

## Versioning

`herdr-pomodoro` follows [Semantic Versioning](https://semver.org/) --
`MAJOR.MINOR.PATCH`, where MAJOR is a breaking config/manifest/CLI change,
MINOR is a backwards-compatible feature, and PATCH is a fix or polish pass.

`Cargo.toml`'s `[package] version` is the single source of truth; the
binary reads it back at compile time (`herdr-pomodoro --version`), and
`herdr-plugin.toml`'s `version` is kept in lockstep with it. See
[CHANGELOG.md](CHANGELOG.md) for release history.

To cut a release:

1. As you make changes, add bullets under `## [Unreleased]` in
   `CHANGELOG.md` (grouped as `### Added` / `### Changed` / `### Fixed`
   etc., per [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)).
2. Run `scripts/bump-version <major|minor|patch|X.Y.Z>`. It bumps both
   TOML files, promotes `[Unreleased]` into a dated `[X.Y.Z]` section,
   refreshes `Cargo.lock`, and -- inside a git repo -- commits and tags
   `vX.Y.Z`. Pass `--dry-run` to preview, or `--no-git` to skip the
   commit/tag.
3. `git push && git push origin vX.Y.Z` (the script prints this reminder).

See [CONTRIBUTING.md](CONTRIBUTING.md) for the commit convention and local
git hook setup (formatting/lint/test gates on commit and push).

## License

MIT
