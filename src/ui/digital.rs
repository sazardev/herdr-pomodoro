use super::{font, App};
use crate::state::Status;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::Line,
    widgets::Paragraph,
    Frame,
};

fn status_line(status: Status) -> Line<'static> {
    let txt = match status {
        Status::Running => "running",
        Status::Paused => "paused -- space to resume",
        Status::Idle => "idle -- space/s to start",
    };
    Line::from(txt)
}

/// Plain, static, monochrome big-digit countdown. No border (Herdr already
/// frames the pane, so drawing our own box just nests one pane-looking box
/// inside another), no icons, no color -- the calm default.
pub fn render(f: &mut Frame<'_>, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
            Constraint::Length(font::HEIGHT),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    f.render_widget(
        Paragraph::new(Line::from(app.state.phase.label())).alignment(Alignment::Center),
        chunks[1],
    );

    let big = font::big_text(&app.mmss());
    let digit_lines: Vec<Line> = big.iter().map(|row| Line::from(row.clone())).collect();
    f.render_widget(Paragraph::new(digit_lines).alignment(Alignment::Center), chunks[2]);

    f.render_widget(
        Paragraph::new(status_line(app.state.status)).alignment(Alignment::Center),
        chunks[3],
    );

    let (done, total) = app.cycle_position();
    let session = super::help::session_line(done, total, app.state.phase);
    f.render_widget(Paragraph::new(session).alignment(Alignment::Center), chunks[4]);

    if area.width > 4 {
        let width = (area.width as usize).saturating_sub(4).max(4);
        let filled = ((width as f64) * app.progress_fraction()).round() as usize;
        let bar =
            "\u{2588}".repeat(filled.min(width)) + &"\u{2591}".repeat(width.saturating_sub(filled));
        f.render_widget(Paragraph::new(Line::from(bar)).alignment(Alignment::Center), chunks[5]);
    }
}
