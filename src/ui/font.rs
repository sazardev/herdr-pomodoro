/// Tiny 5-row block-character font for big countdown digits. Self-contained
/// (no font assets) to keep the binary small.
const GLYPH_HEIGHT: usize = 5;

fn glyph(c: char) -> [&'static str; GLYPH_HEIGHT] {
    match c {
        '0' => [" ███ ", "█   █", "█   █", "█   █", " ███ "],
        '1' => ["  █  ", " ██  ", "  █  ", "  █  ", " ███ "],
        '2' => [" ███ ", "█   █", "  ██ ", " █   ", "█████"],
        '3' => [" ███ ", "█   █", "  ██ ", "█   █", " ███ "],
        '4' => ["█  █ ", "█  █ ", "█████", "   █ ", "   █ "],
        '5' => ["█████", "█    ", "████ ", "    █", "████ "],
        '6' => [" ███ ", "█    ", "████ ", "█   █", " ███ "],
        '7' => ["█████", "   █ ", "  █  ", " █   ", " █   "],
        '8' => [" ███ ", "█   █", " ███ ", "█   █", " ███ "],
        '9' => [" ███ ", "█   █", " ████", "    █", " ███ "],
        ':' => ["   ", " █ ", "   ", " █ ", "   "],
        _ => ["     ", "     ", "     ", "     ", "     "],
    }
}

/// Render a string of digits/colons as 5 lines of big block text.
pub fn big_text(s: &str) -> [String; GLYPH_HEIGHT] {
    let mut rows: [String; GLYPH_HEIGHT] = Default::default();
    for c in s.chars() {
        let g = glyph(c);
        for i in 0..GLYPH_HEIGHT {
            rows[i].push_str(g[i]);
            rows[i].push(' ');
        }
    }
    rows
}

pub const HEIGHT: u16 = GLYPH_HEIGHT as u16;
