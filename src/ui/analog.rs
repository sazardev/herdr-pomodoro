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

const TEXT_ROWS: u16 = 5; // phase, time, status, session, toast

/// A countdown dial: a wedge shrinks clockwise from a full circle as the
/// current phase elapses, like a classic kitchen visual timer. No border
/// (Herdr already frames the pane) and no icons -- the dial itself is the
/// visual.
///
/// Terminal character cells are roughly twice as tall as they are wide, so
/// a canvas with equal x/y bounds only looks round when its *character*
/// width is about twice its height. We size and center a fixed-aspect box
/// for the circle instead of letting it stretch across whatever the pane
/// happens to be, which is what made it look distorted/awkward before.
pub fn render(f: &mut Frame<'_>, area: Rect, app: &App) {
    let color = app.palette.phase_color(app.state.phase);

    let avail_h = area.height.saturating_sub(TEXT_ROWS);
    let circle_h = avail_h.clamp(4, 28);
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

    // A progress ring: the colored arc traces *remaining* time, like a Time
    // Timer's colored disc shrinking clockwise from 12 o'clock. Drawn as a
    // band of points along the rim rather than filled radial lines -- lines
    // converging on the center produced visual noise once the arc covered
    // more than about half the circle.
    let remaining_fraction = 1.0 - app.progress_fraction();
    let sweep_end = remaining_fraction * 2.0 * PI;
    let arc_steps = ((sweep_end / (PI / 90.0)).ceil() as usize).max(1);
    let mut arc_points: Vec<(f64, f64)> = Vec::with_capacity(arc_steps * 3);
    if remaining_fraction > 0.0 {
        for i in 0..=arc_steps {
            let a = (i as f64 / arc_steps as f64) * sweep_end;
            let (s, c) = (a.sin(), a.cos());
            for r in [0.88, 1.0, 1.12] {
                arc_points.push((s * r, c * r));
            }
        }
    }

    // A short marker right at the current position, and a fine radial line
    // back to center so the exact "now" angle still reads clearly even
    // when the remaining arc is very short.
    let edge_angle = sweep_end;
    let hand = (edge_angle.sin(), edge_angle.cos());

    // 12 short rim ticks, with the four cardinal ones poking out a bit
    // further for a proper clock-face feel.
    let minor_ticks: Vec<(f64, f64)> = (0..12)
        .map(|i| {
            let a = (i as f64) / 12.0 * 2.0 * PI;
            (a.sin() * 1.0, a.cos() * 1.0)
        })
        .collect();
    let major_ticks: [(f64, f64); 4] = [(0.0, 1.2), (1.2, 0.0), (0.0, -1.2), (-1.2, 0.0)];
    let muted = app.palette.muted;

    let canvas = Canvas::default()
        .marker(symbols::Marker::Braille)
        .x_bounds([-1.35, 1.35])
        .y_bounds([-1.35, 1.35])
        .paint(move |ctx| {
            ctx.draw(&Circle { x: 0.0, y: 0.0, radius: 1.0, color: muted });
            ctx.draw(&Points { coords: &minor_ticks, color: muted });
            if !arc_points.is_empty() {
                ctx.draw(&Points { coords: &arc_points, color });
                ctx.draw(&CanvasLine {
                    x1: hand.0 * 0.78,
                    y1: hand.1 * 0.78,
                    x2: hand.0 * 1.12,
                    y2: hand.1 * 1.12,
                    color,
                });
            }
            ctx.draw(&Points { coords: &major_ticks, color: muted });
            ctx.draw(&Points { coords: &[(0.0, 0.0)], color: muted });
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

    f.render_widget(Paragraph::new(super::toast_line(app)).alignment(Alignment::Center), rows[6]);
}
