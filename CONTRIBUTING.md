# Contributing

## Setup

```sh
scripts/install-hooks
```

One-time per clone. It points git at the versioned hooks in `.githooks/`
(git's default `.git/hooks` isn't tracked, so this step is required after
cloning -- git does not do it for you).

## Hooks

- **pre-commit** -- `cargo fmt --check` + `cargo check`. Fast, compile-only
  checks so bad formatting or a broken build never gets committed.
- **pre-push** -- `cargo clippy -- -D warnings` + `cargo test`. Heavier
  analysis that only needs to run once per push, not per commit.

Both use debug builds (`cargo check`/`clippy`/`test`), never `--release`
-- this crate's release profile (`opt-level = "z"`, LTO, one codegen unit)
is tuned for a tiny binary, not fast rebuilds, and would make the hooks
noticeably slower for no benefit at commit/push time.

If a hook is ever wrong and you need to bypass it, use `--no-verify`
deliberately and explain why in the PR -- don't make it a habit.

## Commit messages

[Conventional Commits](https://www.conventionalcommits.org/), one line,
imperative mood:

```
<type>: <summary>
```

Types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `perf`. Example:
`fix: clamp progress bar width on narrow panes`. `scripts/bump-version`
generates `chore(release): vX.Y.Z` commits itself -- don't write those by
hand.

## Changes that affect behavior

Add a bullet under `## [Unreleased]` in [CHANGELOG.md](CHANGELOG.md) in
the same commit (`### Added` / `### Changed` / `### Fixed` / `### Removed`
per [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)). See the
[Versioning](README.md#versioning) section in the README for how that
turns into a release.

## Workflow

Single `main` branch, small focused commits, no long-lived feature
branches -- this is a small single-maintainer plugin, not a project that
benefits from a heavier branching model.
