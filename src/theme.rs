use crate::config::Colors;
use crate::state::Phase;
use ratatui::style::Color;

// Herdr's plugin panes are plain terminal content: the plugin draws with
// standard ANSI SGR codes and Herdr's active theme (gruvbox, dracula,
// catppuccin, nord, ...) remaps those 16 base colors, the same way any
// terminal color scheme does. So the default (and recommended) way to
// "follow the active theme" is to stick to the named ANSI colors below
// instead of hardcoded hex values -- it's automatic and needs no plugin
// theme API. Users who want a fixed look regardless of theme can still
// override any of these with a hex color in config.toml's [colors].
fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub work: Color,
    pub short_break: Color,
    pub long_break: Color,
    pub muted: Color,
}

impl Palette {
    pub fn from_config(colors: &Colors) -> Palette {
        Palette {
            work: colors.work.as_deref().and_then(parse_hex).unwrap_or(Color::Red),
            short_break: colors.short_break.as_deref().and_then(parse_hex).unwrap_or(Color::Green),
            long_break: colors.long_break.as_deref().and_then(parse_hex).unwrap_or(Color::Blue),
            muted: Color::DarkGray,
        }
    }

    pub fn phase_color(&self, phase: Phase) -> Color {
        match phase {
            Phase::Work => self.work,
            Phase::ShortBreak => self.short_break,
            Phase::LongBreak => self.long_break,
        }
    }
}
