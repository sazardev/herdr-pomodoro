use super::App;
use crate::state::Status;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Circle, Line as CanvasLine, Points},
        Paragraph,
    },
    Frame,
};
use std::f64::consts::PI;

const TEXT_ROWS: u16 = 4; // phase, time, status, session

/// A countdown dial: a single hand sweeps clockwise from 12 o'clock as the
/// current phase elapses, like a kitchen timer. No border (Herdr already
/// frames the pane) and no icons -- the circle itself is the visual.
///
/// Terminal character cells are roughly twice as tall as they are wide, so
/// a canvas with equal x/y bounds only looks round when its *character*
/// width is about twice its height. We size and center a fixed-aspect box
/// for the circle instead of letting it stretch across whatever the pane
/// happens to be, which is what made it look distorted/awkward before.
pub fn render(f: &mut Frame<'_>, area: Rect, app: &App) {
    let color = app.palette.phase_color(app.state.phase);

    let avail_h = area.height.saturating_sub(TEXT_ROWS);
    let circle_h = avail_h.clamp(4, 16);
    let circle_w = (circle_h * 2).min(area.width).max(4);
    let circle_h = (circle_w / 2).max(3);
    let content_h = circle_h + TEXT_ROWS;
    let top_pad = area.height.saturating_sub(content_h) / 2;

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(top_pad),
            Constraint::Length(1),
            Constraint::Length(circle_h),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    f.render_widget(
        Paragraph::new(Line::from(app.state.phase.label())).alignment(Alignment::Center),
        rows[1],
    );

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(circle_w), Constraint::Min(0)])
        .split(rows[2]);

    let fraction = app.progress_fraction();
    let angle = fraction * 2.0 * PI;
    let hx = angle.sin();
    let hy = angle.cos();

    // Cardinal ticks only (12/3/6/9) -- quieter than a full ring of marks.
    let ticks: [(f64, f64); 4] = [(0.0, 1.0), (1.0, 0.0), (0.0, -1.0), (-1.0, 0.0)];
    let muted = app.palette.muted;

    let canvas = Canvas::default()
        .marker(symbols::Marker::Braille)
        .x_bounds([-1.2, 1.2])
        .y_bounds([-1.2, 1.2])
        .paint(move |ctx| {
            ctx.draw(&Circle { x: 0.0, y: 0.0, radius: 1.0, color: muted });
            ctx.draw(&Points { coords: &ticks, color: muted });
            ctx.draw(&CanvasLine { x1: 0.0, y1: 0.0, x2: hx * 0.82, y2: hy * 0.82, color });
            ctx.draw(&Points { coords: &[(0.0, 0.0)], color });
        });
    f.render_widget(canvas, cols[1]);

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            app.mmss(),
            Style::default().add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center),
        rows[3],
    );

    let status_txt = match app.state.status {
        Status::Running => "running",
        Status::Paused => "paused -- space to resume",
        Status::Idle => "idle -- space/s to start",
    };
    f.render_widget(Paragraph::new(Line::from(status_txt)).alignment(Alignment::Center), rows[4]);

    let (done, total) = app.cycle_position();
    let session = super::help::session_line(done, total, app.state.phase);
    f.render_widget(Paragraph::new(session).alignment(Alignment::Center), rows[5]);
}
