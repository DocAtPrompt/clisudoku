// src/tui/render/cell.rs
use crate::puzzle::CellKind;
use crate::tui::digit_style::DigitStyle;

/// Returns the 3 display rows (each exactly 7 chars) for a cell.
///
/// Decision order:
/// 1. If cell is Given: show digit with `given_style`.
/// 2. If cell is Filled: show digit with `filled_style`.
/// 3. If cell is Empty and has notes (notes != 0): show note candidates.
/// 4. Otherwise: 7 spaces per row.
pub fn cell_display_lines(
    cell: &CellKind,
    notes: u16,
    given_style: &dyn DigitStyle,
    filled_style: &dyn DigitStyle,
) -> [String; 3] {
    match cell {
        CellKind::Given(d) => digit_display_lines(*d, given_style),
        CellKind::Filled(d) => digit_display_lines(*d, filled_style),
        CellKind::Empty if notes != 0 => note_display_lines(notes),
        _ => empty_display_lines(),
    }
}

/// 3-row digit display, each row 7 chars: "  ROW  ".
pub fn digit_display_lines(digit: u8, style: &dyn DigitStyle) -> [String; 3] {
    style.digit_rows(digit).map(|r| format!("  {}  ", r))
}

/// 3-row note display, each row 7 chars: " N N N " (digit or space per candidate).
pub fn note_display_lines(notes: u16) -> [String; 3] {
    let n = |d: u8| -> char {
        if notes & (1u16 << d) != 0 {
            char::from(b'0' + d)
        } else {
            ' '
        }
    };
    [
        format!(" {} {} {} ", n(1), n(2), n(3)),
        format!(" {} {} {} ", n(4), n(5), n(6)),
        format!(" {} {} {} ", n(7), n(8), n(9)),
    ]
}

/// 3 rows of 7 spaces.
pub fn empty_display_lines() -> [String; 3] {
    ["       ".into(), "       ".into(), "       ".into()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::CellKind;
    use crate::tui::digit_style::{AwkwardRetroStyle, RetroStyle};

    #[test]
    fn empty_cell_is_7_spaces_per_row() {
        let lines = cell_display_lines(&CellKind::Empty, 0, &RetroStyle, &AwkwardRetroStyle);
        for line in &lines {
            assert_eq!(line, "       ");
        }
    }

    #[test]
    fn given_cell_uses_given_style() {
        let lines = cell_display_lines(&CellKind::Given(5), 0, &RetroStyle, &AwkwardRetroStyle);
        assert_eq!(lines[0], "  ▛▀▀  ");
        assert_eq!(lines[0].chars().count(), 7);
    }

    #[test]
    fn filled_cell_uses_filled_style() {
        // AwkwardRetroStyle digit 3 row 0 = "██ " → "  ██   "
        let lines = cell_display_lines(&CellKind::Filled(3), 0, &RetroStyle, &AwkwardRetroStyle);
        assert_eq!(lines[0], "  ██   ");
        assert_eq!(lines[0].chars().count(), 7);
    }

    #[test]
    fn given_and_filled_same_digit_render_differently() {
        let given = cell_display_lines(&CellKind::Given(5), 0, &RetroStyle, &AwkwardRetroStyle);
        let filled = cell_display_lines(&CellKind::Filled(5), 0, &RetroStyle, &AwkwardRetroStyle);
        assert_ne!(given, filled, "given and filled cells must use different styles");
    }

    #[test]
    fn notes_all_set_shows_all_digits() {
        let all_notes: u16 = 0b1111111110; // bits 1-9 set
        let lines = note_display_lines(all_notes);
        assert_eq!(lines[0], " 1 2 3 ");
        assert_eq!(lines[1], " 4 5 6 ");
        assert_eq!(lines[2], " 7 8 9 ");
    }

    #[test]
    fn notes_none_set_shows_spaces() {
        let lines = note_display_lines(0);
        assert_eq!(lines[0], "       ");
    }

    #[test]
    fn notes_partial_shows_correct_digits() {
        let notes: u16 = (1 << 1) | (1 << 5) | (1 << 9);
        let lines = note_display_lines(notes);
        assert_eq!(lines[0], " 1     ");
        assert_eq!(lines[1], "   5   ");
        assert_eq!(lines[2], "     9 ");
    }

    #[test]
    fn note_lines_are_7_chars_each() {
        let lines = note_display_lines(0b0101010101010);
        for line in &lines {
            assert_eq!(line.chars().count(), 7);
        }
    }

    #[test]
    fn empty_cell_with_notes_shows_notes() {
        let notes: u16 = (1 << 1) | (1 << 2);
        let lines = cell_display_lines(&CellKind::Empty, notes, &RetroStyle, &AwkwardRetroStyle);
        assert_eq!(lines[0], " 1 2   ");
    }
}
