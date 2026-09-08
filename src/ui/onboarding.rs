use super::{App, Mood};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Mood,
    Preset,
    Sound,
    Notify,
    AutoStart,
}

const FIELDS: [Field; 5] =
    [Field::Mood, Field::Preset, Field::Sound, Field::Notify, Field::AutoStart];

/// First-run setup wizard state: which field is selected. Values live
/// directly on `App.cfg` (mutated in place as the user cycles them) so the
/// live preview below the field list is just a normal render call -- no
/// separate "draft config" to keep in sync.
pub struct State {
    selected: usize,
}

impl State {
    pub fn new() -> State {
        State { selected: 0 }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }
}

impl Default for State {
    fn default() -> Self {
        State::new()
    }
}

pub fn select_prev(ob: &mut State) {
    ob.selected = (ob.selected + FIELDS.len() - 1) % FIELDS.len();
}

pub fn select_next(ob: &mut State) {
    ob.selected = (ob.selected + 1) % FIELDS.len();
}

/// Change the currently selected field's value. `delta` is +1 or -1.
pub fn cycle_value(app: &mut App, selected: usize, delta: i32) {
    match FIELDS[selected] {
        Field::Mood => {
            let m = Mood::parse(&app.cfg.mood);
            app.cfg.mood = if delta > 0 { m.next() } else { m.prev() }.as_str().to_string();
        }
        Field::Preset => {
            if app.cfg.presets.is_empty() {
                return;
            }
            let names: Vec<String> = app.cfg.presets.iter().map(|p| p.name.clone()).collect();
            let idx = names.iter().position(|n| n == &app.cfg.active_preset).unwrap_or(0) as i32;
            let new_idx = (idx + delta).rem_euclid(names.len() as i32) as usize;
            let new_name = names[new_idx].clone();
            crate::timer::apply_preset_change(&mut app.state, &app.cfg, &new_name);
            app.cfg.active_preset = new_name;
        }
        Field::Sound => app.cfg.sound = !app.cfg.sound,
        Field::Notify => app.cfg.notify = !app.cfg.notify,
        Field::AutoStart => app.cfg.auto_start_next = !app.cfg.auto_start_next,
    }
}

fn on_off(b: bool) -> &'static str {
    if b {
        "on"
    } else {
        "off"
    }
}

fn field_text(app: &App, field: Field) -> String {
    match field {
        Field::Mood => format!("View: {}", app.cfg.mood),
        Field::Preset => {
            let p = app.cfg.preset(&app.cfg.active_preset);
            format!(
                "Preset: {} ({}/{}/{} min)",
                p.name, p.work_minutes, p.short_break_minutes, p.long_break_minutes
            )
        }
        Field::Sound => format!("Bell on phase complete: {}", on_off(app.cfg.sound)),
        Field::Notify => format!("Desktop notifications: {}", on_off(app.cfg.notify)),
        Field::AutoStart => format!("Auto-start next phase: {}", on_off(app.cfg.auto_start_next)),
    }
}

pub fn render(f: &mut Frame<'_>, area: Rect, app: &App, ob: &State) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(FIELDS.len() as u16),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    f.render_widget(
        Paragraph::new(vec![
            Line::styled("herdr-pomodoro setup", Style::default().add_modifier(Modifier::BOLD)),
            Line::from("pick your defaults -- you can always change these later"),
        ]),
        rows[0],
    );

    let field_lines: Vec<Line> = FIELDS
        .iter()
        .enumerate()
        .map(|(i, &field)| {
            let selected = i == ob.selected;
            let marker = if selected { "> " } else { "  " };
            let text = format!("{marker}{}", field_text(app, field));
            if selected {
                Line::styled(text, Style::default().add_modifier(Modifier::BOLD))
            } else {
                Line::from(text)
            }
        })
        .collect();
    f.render_widget(Paragraph::new(field_lines), rows[1]);

    f.render_widget(
        Paragraph::new("up/down select field, left/right change value").wrap(Wrap { trim: true }),
        rows[3],
    );
    f.render_widget(Paragraph::new("enter: start   esc: skip (use these defaults)"), rows[4]);

    if rows[5].height >= 6 && rows[5].width >= 20 {
        let mood = app.effective_mood(rows[5]);
        super::render_mood(f, rows[5], app, mood);
    }
}
