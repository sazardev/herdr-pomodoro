# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
(`MAJOR.MINOR.PATCH`): MAJOR for breaking config/manifest/CLI changes,
MINOR for new backwards-compatible features, PATCH for fixes and polish.

## [Unreleased]

### Added

- `analog` renders a progress ring (a colored arc tracing remaining time,
  like a kitchen visual timer) with 12 rim ticks and a current-position
  marker, aspect-corrected so it stays round at any pane size.
- A one-line toast (started/paused/resumed/phase-complete/view or preset
  changed) shows for ~3s in every mood, not just `minimal`.
- First-run setup wizard (view/preset/sound/notifications/auto-start, with
  a live preview per view) shown once via a `[[startup]]` hook the first
  time Herdr starts after install, or the first time the pane is opened on
  an already-running session. Redo it any time with the `reset-onboarding`
  action.
- Work-session history: every completed/skipped work session appends a
  timestamp to `history.log`. Press `i` for a rolling last-24h / 7-day /
  all-time overlay, or run `herdr-pomodoro history` headlessly.

### Fixed

- `digital`/`flashy` no longer clip when a pane is too short for the
  block-digit layout (needs 10 rows) -- they now fall all the way back to
  `minimal` instead of rendering a `digital` view that doesn't fit.
- `minimal` no longer truncates the status word in very narrow panes
  (tightened field spacing).
- The main `timer` pane now declares `placement = "split"` explicitly
  instead of relying on Herdr's implicit default. An unspecified placement
  registers as `overlay`, which opens fullscreen over the active pane and
  captures all input; if the process doesn't exit cleanly it can get stuck
  there, blocking whatever pane it covered.

## [0.1.0] - 2026-09-08

### Added

- Dockable Pomodoro pane for Herdr with four static, text-only moods:
  `minimal` (one-line drawer strip), `digital` (plain big block digits,
  the calm default), `analog` (countdown dial), and `flashy` (same layout
  as `digital`, colored by phase). No icons/emoji, no blinking or pulsing
  animation, and no border drawn around the pane content (Herdr already
  frames the pane, so an inner box just looked like a pane nested inside a
  pane). Auto-downgrades to a smaller mood when the pane doesn't have room.
- Classic (25/5/15), deep-work (50/10/20), and quick (15/3/10) presets,
  plus support for user-defined `[[preset]]` blocks in `config.toml`.
- Theme-adaptive rendering via the standard (non-bright) ANSI palette, with
  optional hex color overrides in `config.toml`.
- Global shortcuts and one-shot subcommands: `open`, `start`/`resume`,
  `pause`, `toggle`, `skip`, `reset [--all]`, `cycle-mood`, `cycle-preset`,
  `status`.
- In-pane keybindings (`space`/`s`, `n`, `r`/`R`, `m`, `p`, `?`, `q`/`esc`).
- Desktop notifications and terminal bell on phase completion.
- No-daemon design: timer truth is a wall-clock end-timestamp in a state
  file, so nothing runs in the background while the pane is closed.

[Unreleased]: https://github.com/sazardev/herdr-pomodoro/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/sazardev/herdr-pomodoro/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/sazardev/herdr-pomodoro/releases/tag/v0.1.0
