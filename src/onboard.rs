use std::path::{Path, PathBuf};

/// Marker file recording that first-run setup has already been shown (or
/// explicitly skipped). Lives next to state.json rather than in the
/// user-editable config.toml, since it's an implementation detail, not a
/// setting.
fn marker_path(state_dir: &Path) -> PathBuf {
    state_dir.join(".onboarded")
}

pub fn is_done(state_dir: &Path) -> bool {
    marker_path(state_dir).exists()
}

pub fn mark_done(state_dir: &Path) {
    let _ = std::fs::create_dir_all(state_dir);
    let _ = std::fs::write(marker_path(state_dir), b"");
}

/// Undo `mark_done`, so `run` shows the wizard again next time it opens.
pub fn reset(state_dir: &Path) {
    let _ = std::fs::remove_file(marker_path(state_dir));
}
