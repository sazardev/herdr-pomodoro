use super::App;
use crate::state::Status;
use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

fn status_word(status: Status) -> &'static str {
    match status {
        Status::Running => "running",
        Status::Paused => "paused",
        Status::Idle => "idle",
    }
}

/// The "drawer strip" view: as little as one line, no borders, so it stays
/// useful docked in a narrow sidebar-style pane. Plain text, no icons.
pub fn render(f: &mut Frame<'_>, area: Rect, app: &App) {
    let color = app.palette.phase_color(app.state.phase);

    let mut lines = vec![Line::from(format!(
        "{} {} {}",
        app.state.phase.short_label(),
        app.mmss(),
        status_word(app.state.status)
    ))];

    if area.height >= 2 && area.width >= 8 {
        let width = (area.width as usize).min(48);
        let filled = ((width as f64) * app.progress_fraction()).round() as usize;
        let bar: String =
            "\u{2588}".repeat(filled.min(width)) + &"\u{2591}".repeat(width.saturating_sub(filled));
        lines.push(Line::from(Span::styled(bar, Style::default().fg(color))));
    }

    if area.height >= 3 {
        if let Some((msg, at)) = &app.toast {
            if at.elapsed().as_secs() < 3 {
                lines.push(Line::from(msg.clone()));
            }
        }
    }

    f.render_widget(Paragraph::new(lines), area);
}
