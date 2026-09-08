use super::{font, App};
use crate::state::Status;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::Paragraph,
    Frame,
};

/// Same layout as `digital`, but colored by phase (steady, no pulsing) --
/// the one mood with some color in it. No icons, no border, no animation.
pub fn render(f: &mut Frame<'_>, area: Rect, app: &App) {
    let color = app.palette.phase_color(app.state.phase);

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
        Paragraph::new(Line::styled(
            app.state.phase.label(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        chunks[1],
    );

    let big = font::big_text(&app.mmss());
    let digit_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
    let digit_lines: Vec<Line> =
        big.iter().map(|row| Line::styled(row.clone(), digit_style)).collect();
    f.render_widget(Paragraph::new(digit_lines).alignment(Alignment::Center), chunks[2]);

    let status_txt = match app.state.status {
        Status::Running => "running",
        Status::Paused => "paused -- space to resume",
        Status::Idle => "idle -- space/s to start",
    };
    f.render_widget(Paragraph::new(Line::from(status_txt)).alignment(Alignment::Center), chunks[3]);

    let (done, total) = app.cycle_position();
    let session = super::help::session_line(done, total, app.state.phase);
    f.render_widget(Paragraph::new(session).alignment(Alignment::Center), chunks[4]);

    if area.width > 4 {
        let width = (area.width as usize).saturating_sub(4).max(4);
        let filled = ((width as f64) * app.progress_fraction()).round() as usize;
        let bar =
            "\u{2588}".repeat(filled.min(width)) + &"\u{2591}".repeat(width.saturating_sub(filled));
        f.render_widget(
            Paragraph::new(Line::styled(bar, Style::default().fg(color)))
                .alignment(Alignment::Center),
            chunks[5],
        );
    }
}
