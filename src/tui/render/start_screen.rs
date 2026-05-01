// src/tui/render/start_screen.rs
use crate::i18n::{Strings, LANGUAGE_NAMES};
use crate::tui::colors::{ColorScheme, THEME_NAMES};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

pub const TITLE: &str = r#"   ____ _     ___ ____            _       _
  / ___| |   |_ _/ ___| _   _  __| | ___ | | ___   _
 | |   | |    | |\___ \| | | |/ _` |/ _ \| |/ / | | |
 | |___| |___ | | ___) | |_| | (_| | (_) |   <| |_| |
  \____|_____|___|____/ \__,_|\__,_|\___/|_|\_\\__,_|"#;

/// Number of items in the start menu (New Game / Language / Theme / Quit).
pub const START_ITEM_COUNT: usize = 4;

fn render_title(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    colors: &ColorScheme,
) -> io::Result<()> {
    for (i, line) in TITLE.lines().enumerate() {
        queue!(out,
            MoveTo(col_off, row_off + i as u16),
            SetForegroundColor(colors.digit_given),
            SetBackgroundColor(colors.ui_background),
            Print(line)
        )?;
    }
    Ok(())
}

fn render_menu_items(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    items: &[&str],
    selected: usize,
    colors: &ColorScheme,
) -> io::Result<()> {
    for (i, item) in items.iter().enumerate() {
        let (fg, bg) = if i == selected {
            (colors.ui_cursor_fg, colors.ui_cursor_bg)
        } else {
            (colors.ui_text, colors.ui_background)
        };
        queue!(out,
            MoveTo(col_off + 2, row_off + i as u16 * 2),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(format!("  {}  ", item))
        )?;
    }
    queue!(out, ResetColor)
}

/// Render the main start menu.
pub fn render_start(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    selected: usize,
    strings: &'static Strings,
    colors: &ColorScheme,
) -> io::Result<()> {
    render_title(out, (row_off, col_off), colors)?;
    let items = [strings.menu_new_game, strings.menu_language, strings.menu_theme, strings.menu_quit];
    let menu_row = row_off + 7; // 5 title lines + 2 blank rows
    render_menu_items(out, (menu_row, col_off), &items, selected, colors)
}

/// Render the difficulty selection sub-menu.
///
/// Layout: difficulty items on the left, symmetry toggle on the right.
/// `→` / `←` switch focus between columns; `Enter` / `Space` toggle symmetry
/// when the right column is focused.
pub fn render_difficulty(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    selected: usize,
    sym_focused: bool,
    symmetry: bool,
    strings: &'static Strings,
    colors: &ColorScheme,
) -> io::Result<()> {
    // ── Title ────────────────────────────────────────────────────────────────
    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(strings.difficulty_title)
    )?;

    // ── Difficulty items (left column) ───────────────────────────────────────
    // When symmetry column has focus, the selected difficulty item is dimmed
    // so both the selected difficulty AND the toggle are simultaneously visible.
    let items = [
        strings.difficulty_easy,
        strings.difficulty_medium,
        strings.difficulty_hard,
        strings.difficulty_designer,
    ];
    for (i, item) in items.iter().enumerate() {
        let (fg, bg) = if i == selected && !sym_focused {
            (colors.ui_cursor_fg, colors.ui_cursor_bg)
        } else if i == selected {
            (colors.ui_text_dim, colors.ui_background)
        } else {
            (colors.ui_text, colors.ui_background)
        };
        queue!(out,
            MoveTo(col_off + 2, row_off + 2 + i as u16 * 2),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(format!("  {}  ", item))
        )?;
    }

    // ── Symmetry toggle (right column, aligned with middle difficulty item) ──
    let sym_col = col_off + 18;
    let sym_row = row_off + 2; // aligned with first item; label + toggle fit in 2 rows

    queue!(out,
        MoveTo(sym_col, sym_row),
        SetForegroundColor(colors.ui_text_dim),
        SetBackgroundColor(colors.ui_background),
        Print(strings.symmetry_label)
    )?;

    let (toggle_fg, toggle_bg) = if sym_focused {
        (colors.ui_cursor_fg, colors.ui_cursor_bg)
    } else {
        (colors.ui_text, colors.ui_background)
    };
    // Pad to the longer of the two toggle values so width stays constant when
    // the user toggles (e.g. "on"/"off" → width 3, "ndiyo"/"hapana" → width 6).
    let max_len = strings.toggle_on.chars().count()
        .max(strings.toggle_off.chars().count());
    let val = if symmetry { strings.toggle_on } else { strings.toggle_off };
    let toggle_label = format!("[ {:width$} ]", val, width = max_len);
    queue!(out,
        MoveTo(sym_col, sym_row + 1),
        SetForegroundColor(toggle_fg),
        SetBackgroundColor(toggle_bg),
        Print(toggle_label),
        ResetColor
    )
}

/// Render the language selection sub-menu.
pub fn render_language(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    selected: usize,
    strings: &'static Strings,
    colors: &ColorScheme,
) -> io::Result<()> {
    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(strings.language_title)
    )?;
    render_menu_items(out, (row_off + 2, col_off), LANGUAGE_NAMES, selected, colors)
}

/// Render the theme selection sub-menu.
pub fn render_theme(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    selected: usize,
    strings: &'static Strings,
    colors: &ColorScheme,
) -> io::Result<()> {
    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(strings.theme_title)
    )?;
    render_menu_items(out, (row_off + 2, col_off), THEME_NAMES, selected, colors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::EN;
    use crate::tui::colors::ColorScheme;

    #[test]
    fn start_screen_render_does_not_panic() {
        let mut buf = Vec::new();
        render_start(&mut buf, (0, 0), 0, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("New Game"));
        assert!(s.contains("Quit"));
    }

    #[test]
    fn difficulty_screen_render_does_not_panic() {
        let mut buf = Vec::new();
        render_difficulty(&mut buf, (0, 0), 0, false, true, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Easy"));
        assert!(s.contains("Medium"));
        assert!(s.contains("Hard"));
    }

    #[test]
    fn difficulty_screen_shows_designer_option() {
        let mut buf = Vec::new();
        render_difficulty(&mut buf, (0, 0), 3, false, true, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Designer"), "Expected Designer option");
    }

    #[test]
    fn difficulty_screen_shows_symmetry_toggle() {
        let mut buf = Vec::new();
        render_difficulty(&mut buf, (0, 0), 1, false, true, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Symmetry"));
        assert!(s.contains("[ on")); // fixed-width: "[ on     ]"

        buf.clear();
        render_difficulty(&mut buf, (0, 0), 1, true, false, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("[ off"));
    }
}
