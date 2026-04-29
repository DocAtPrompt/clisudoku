// src/tui/colors.rs
use crossterm::style::Color;

// ── Theme ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    HighContrast,
}

pub const THEME_NAMES: &[&str] = &["Dark", "Light", "High Contrast"];
pub const THEME_COUNT: usize = 3;

impl Theme {
    pub fn from_index(i: usize) -> Self {
        match i {
            1 => Theme::Light,
            2 => Theme::HighContrast,
            _ => Theme::Dark,
        }
    }

    pub fn as_index(self) -> usize { self as usize }

    /// Parse a CLI code: "dark", "light", "high-contrast" / "hc".
    pub fn from_code(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dark"                    => Some(Theme::Dark),
            "light"                   => Some(Theme::Light),
            "high-contrast" | "hc"   => Some(Theme::HighContrast),
            _                         => None,
        }
    }
}

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

    // Frame 5a — passive scan (same digit as cursor)
    pub digit_scan:       Color,

    // Frame 5 — UI text
    pub ui_text:          Color,
    pub ui_text_dim:      Color,
    pub ui_cursor_bg:     Color,
    /// Foreground color for selected menu items (contrast against ui_cursor_bg).
    pub ui_cursor_fg:     Color,

    // Hint system — border and target colours
    /// Border colour for cause cells (explains WHY the hint works).
    pub hint_cause_border: Color,
    /// Border colour for elimination cells (where a candidate is removed).
    pub hint_elim_border:  Color,
    /// Background colour for the target cell (blinking).
    pub hint_target_bg:    Color,
}

impl Default for ColorScheme {
    fn default() -> Self { Self::dark() }
}

impl ColorScheme {
    pub fn for_theme(theme: Theme) -> Self {
        match theme {
            Theme::Dark        => Self::dark(),
            Theme::Light       => Self::light(),
            Theme::HighContrast => Self::high_contrast(),
        }
    }

    // ── Dark (default) ────────────────────────────────────────────────────────
    pub fn dark() -> Self {
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

            digit_scan:           Color::Magenta,

            ui_text:              Color::White,
            ui_text_dim:          Color::Grey,
            ui_cursor_bg:         Color::DarkBlue,
            ui_cursor_fg:         Color::White,

            hint_cause_border:    Color::Green,
            hint_elim_border:     Color::Red,
            hint_target_bg:       Color::Yellow,
        }
    }

    // ── Light ─────────────────────────────────────────────────────────────────
    // Uses only named ANSI colors for maximum terminal compatibility.
    // Cursor is yellow (clearly visible on white), digits same palette as dark.
    pub fn light() -> Self {
        Self {
            ui_background:        Color::White,
            grid_border:          Color::DarkGrey,
            grid_box:             Color::Black,
            grid_cell:            Color::DarkGrey,

            cell_normal_bg:       Color::White,
            cell_active_bg:       Color::Yellow,
            cell_active_box_bg:   Color::Yellow,
            cell_active_cross_bg: Color::Yellow,

            digit_given:          Color::Black,
            digit_user:           Color::DarkBlue,
            digit_error:          Color::Red,
            digit_highlight:      Color::Yellow,

            note_normal:          Color::DarkGrey,
            note_highlight:       Color::Yellow,

            digit_scan:           Color::Magenta,

            ui_text:              Color::Black,
            ui_text_dim:          Color::DarkGrey,
            ui_cursor_bg:         Color::DarkBlue,
            ui_cursor_fg:         Color::White,

            hint_cause_border:    Color::Green,
            hint_elim_border:     Color::Red,
            hint_target_bg:       Color::Yellow,
        }
    }

    // ── High Contrast (colorblind-safe: no red/green distinction) ─────────────
    // Errors shown in Magenta (not Red) so red-green colorblind users can
    // clearly distinguish given (White), user (Yellow), error (Magenta).
    // Cursor uses a gold-yellow background that remains visible on black.
    pub fn high_contrast() -> Self {
        Self {
            ui_background:        Color::Black,
            grid_border:          Color::White,
            grid_box:             Color::White,
            grid_cell:            Color::Grey,

            cell_normal_bg:       Color::Black,
            cell_active_bg:       Color::Rgb { r: 60, g: 55, b: 0 },
            cell_active_box_bg:   Color::Rgb { r: 30, g: 28, b: 0 },
            cell_active_cross_bg: Color::Rgb { r: 18, g: 16, b: 0 },

            digit_given:          Color::White,
            digit_user:           Color::Yellow,
            digit_error:          Color::Magenta,
            digit_highlight:      Color::Cyan,

            note_normal:          Color::Grey,
            note_highlight:       Color::Cyan,

            digit_scan:           Color::Cyan,

            ui_text:              Color::White,
            ui_text_dim:          Color::Grey,
            ui_cursor_bg:         Color::Rgb { r: 60, g: 55, b: 0 },
            ui_cursor_fg:         Color::Black,

            hint_cause_border:    Color::Cyan,
            hint_elim_border:     Color::Magenta,
            hint_target_bg:       Color::Yellow,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scheme_has_dark_background() {
        let s = ColorScheme::default();
        assert_eq!(s.ui_background, Color::Black);
    }

    #[test]
    fn light_scheme_has_white_background() {
        let s = ColorScheme::light();
        assert_eq!(s.ui_background, Color::White);
    }

    #[test]
    fn high_contrast_error_is_not_red() {
        let s = ColorScheme::high_contrast();
        assert_ne!(s.digit_error, Color::Red);
        assert_ne!(s.digit_error, Color::DarkRed);
    }

    #[test]
    fn theme_round_trips_through_index() {
        for i in 0..THEME_COUNT {
            assert_eq!(Theme::from_index(i).as_index(), i);
        }
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

    #[test]
    fn hint_colors_defined_for_all_themes() {
        let dark = ColorScheme::dark();
        assert_eq!(dark.hint_cause_border, Color::Green);
        assert_eq!(dark.hint_elim_border,  Color::Red);
        assert_eq!(dark.hint_target_bg,    Color::Yellow);

        let light = ColorScheme::light();
        assert_eq!(light.hint_cause_border, Color::Green);
        assert_eq!(light.hint_elim_border,  Color::Red);
        assert_eq!(light.hint_target_bg,    Color::Yellow);

        let hc = ColorScheme::high_contrast();
        assert_eq!(hc.hint_cause_border, Color::Cyan);
        assert_eq!(hc.hint_elim_border,  Color::Magenta);
        assert_eq!(hc.hint_target_bg,    Color::Yellow);
    }
}
