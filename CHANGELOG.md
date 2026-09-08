# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
(`MAJOR.MINOR.PATCH`): MAJOR for breaking config/manifest/CLI changes,
MINOR for new backwards-compatible features, PATCH for fixes and polish.

## [Unreleased]

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

[Unreleased]: https://github.com/sazardev/herdr-pomodoro/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sazardev/herdr-pomodoro/releases/tag/v0.1.0
