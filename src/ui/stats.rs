use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::Path;

pub fn render(f: &mut Frame<'_>, area: Rect, state_dir: &Path) {
    let summary = crate::history::summarize(state_dir);
    let width = area.width.min(30);
    let height = area.height.min(8);
    if width < 20 || height < 7 {
        return;
    }
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let popup = Rect { x, y, width, height };

    let lines = vec![
        Line::styled("work sessions", Style::default().add_modifier(Modifier::BOLD)),
        Line::from(""),
        Line::from(format!("last 24h:    {}", summary.last_24h)),
        Line::from(format!("last 7 days: {}", summary.last_7d)),
        Line::from(format!("all-time:    {}", summary.total)),
    ];

    f.render_widget(Clear, popup);
    let block = Block::default().borders(Borders::ALL).title(" stats ");
    f.render_widget(Paragraph::new(lines).block(block).alignment(Alignment::Left), popup);
}
