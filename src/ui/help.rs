use super::App;
use crate::state::Phase;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Plain-text progress through the current work/break cycle, e.g.
/// "session 2/4".
pub fn session_line(done: u32, total: u32, phase: Phase) -> Line<'static> {
    let n = if phase == Phase::Work { done + 1 } else { done.max(1) };
    Line::from(format!("session {n}/{total}"))
}

pub fn render(f: &mut Frame<'_>, area: Rect, _app: &App) {
    let width = area.width.min(38);
    let height = area.height.min(12);
    if width < 20 || height < 8 {
        return;
    }
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let popup = Rect { x, y, width, height };

    let lines = vec![
        Line::from("herdr-pomodoro").style(Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from("space / s  start, pause, resume"),
        Line::from("n          skip to next phase"),
        Line::from("r          reset current phase"),
        Line::from("R          full reset"),
        Line::from("m          cycle view"),
        Line::from("p          cycle preset"),
        Line::from("?          toggle this help"),
        Line::from("q / esc    close"),
    ];

    f.render_widget(Clear, popup);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" help -- v{} ", env!("CARGO_PKG_VERSION")));
    f.render_widget(Paragraph::new(lines).block(block).alignment(Alignment::Left), popup);
}
