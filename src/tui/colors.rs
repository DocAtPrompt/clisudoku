// src/tui/colors.rs
use crossterm::style::Color;

/// Complete color scheme for the game UI.
/// All fields are foreground colors unless named `_bg`.
/// Matches the key names from the spec's Farbsystem section.
#[derive(Debug, Clone, PartialEq)]
pub struct ColorScheme {
    // Frame 1 — background & grid lines
    pub ui_background:    Color,
    pub grid_border:      Color,
    pub grid_box:         Color,
    pub grid_cell:        Color,

    // Frame 2 — cell backgrounds
    pub cell_normal_bg:   Color,
    pub cell_active_bg:   Color,
    pub cell_active_box_bg: Color,
    pub cell_active_cross_bg: Color,

    // Frame 3 — digits (foreground)
    pub digit_given:      Color,
    pub digit_user:       Color,
    pub digit_error:      Color,
    pub digit_highlight:  Color,

    // Frame 4 — notes
    pub note_normal:      Color,
    pub note_highlight:   Color,

    // Frame 5 — UI text
    pub ui_text:          Color,
    pub ui_text_dim:      Color,
    pub ui_cursor_bg:     Color,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            ui_background:        Color::Black,
            grid_border:          Color::Grey,
            grid_box:             Color::White,
            grid_cell:            Color::DarkGrey,

            cell_normal_bg:       Color::Black,
            cell_active_bg:       Color::DarkBlue,
            cell_active_box_bg:   Color::Rgb { r: 20, g: 20, b: 60 },
            cell_active_cross_bg: Color::Rgb { r: 10, g: 10, b: 35 },

            digit_given:          Color::White,
            digit_user:           Color::Cyan,
            digit_error:          Color::Red,
            digit_highlight:      Color::Yellow,

            note_normal:          Color::Grey,
            note_highlight:       Color::Yellow,

            ui_text:              Color::White,
            ui_text_dim:          Color::Grey,
            ui_cursor_bg:         Color::DarkBlue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scheme_has_dark_background() {
        let s = ColorScheme::default();
        assert_ne!(s.ui_background, Color::White);
    }

    #[test]
    fn active_cell_differs_from_normal() {
        let s = ColorScheme::default();
        assert_ne!(s.cell_active_bg, s.cell_normal_bg);
    }

    #[test]
    fn given_digit_differs_from_user_digit() {
        let s = ColorScheme::default();
        assert_ne!(s.digit_given, s.digit_user);
    }
}
