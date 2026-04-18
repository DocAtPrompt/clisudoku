// src/tui/digit_style.rs

/// A digit rendering style. Each digit 1–9 is represented as 3 rows of exactly
/// 3 characters each (half-block or full-block Unicode art).
pub trait DigitStyle: Send + Sync {
    /// Returns the 3 display rows for `digit` (1-indexed, 1–9).
    fn digit_rows(&self, digit: u8) -> [&'static str; 3];
}

/// Organic half-block Unicode style (`▞▀▚` family) as defined in the spec.
pub struct RetroStyle;

impl DigitStyle for RetroStyle {
    fn digit_rows(&self, digit: u8) -> [&'static str; 3] {
        match digit {
            1 => ["▗▐ ", " ▐ ", " ▐ "],
            2 => ["▞▀▚", " ▞ ", "▟▄▄"],
            3 => ["▞▀▚", "  ▚", "▚▄▞"],
            4 => ["▌ ▐", "▀▀▜", "  ▐"],
            5 => ["▛▀▀", "▀▀▚", "▚▄▞"],
            6 => ["▞▀ ", "▛▀▚", "▚▄▞"],
            7 => ["▀▀▞", " ▞ ", "▞  "],
            8 => ["▞▀▚", "▚▄▞", "▚▄▞"],
            9 => ["▞▀▚", "▚▄▞", " ▞ "],
            _ => ["   ", "   ", "   "],
        }
    }
}

/// Center a 3-char digit row inside a 7-wide cell: "  ROW  ".
pub fn cell_digit_lines(digit: u8, style: &dyn DigitStyle) -> [String; 3] {
    let rows = style.digit_rows(digit);
    rows.map(|r| format!("  {}  ", r))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retro_digit_rows_are_exactly_3_chars_each() {
        let style = RetroStyle;
        for d in 1u8..=9 {
            let rows = style.digit_rows(d);
            for row in &rows {
                assert_eq!(row.chars().count(), 3,
                    "digit {} row {:?} is not 3 chars", d, row);
            }
        }
    }

    #[test]
    fn retro_digit_1_correct() {
        assert_eq!(RetroStyle.digit_rows(1), ["▗▐ ", " ▐ ", " ▐ "]);
    }

    #[test]
    fn retro_digit_5_correct() {
        assert_eq!(RetroStyle.digit_rows(5), ["▛▀▀", "▀▀▚", "▚▄▞"]);
    }

    #[test]
    fn retro_digit_9_correct() {
        assert_eq!(RetroStyle.digit_rows(9), ["▞▀▚", "▚▄▞", " ▞ "]);
    }

    #[test]
    fn cell_lines_digit_centers_in_7_chars() {
        let lines = cell_digit_lines(5, &RetroStyle);
        for line in &lines {
            assert_eq!(line.chars().count(), 7);
        }
        assert_eq!(lines[0], "  ▛▀▀  ");
    }
}
