// src/tui/render/start_screen.rs
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

const TITLE: &str = "   ___ _ _  ___ _   _    _         _  \n  / __| (_)/ __| | _| |__| |___  _| |___\n | (__| | |\\__ \\ || |/ _  / _ \\/ _  / _ \\\n  \\___|_|_||___/\\_,_|\\__,_\\___/\\__,_\\___/";

pub const START_ITEMS: &[&str] = &["New Game", "Quit"];
pub const DIFFICULTY_ITEMS: &[&str] = &["Easy", "Medium", "Hard"];

/// Render the main start menu.
pub fn render_start(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    selected: usize,
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

    let menu_row = row_off + 6;
    for (i, item) in START_ITEMS.iter().enumerate() {
        let (fg, bg) = if i == selected {
            (colors.ui_background, colors.ui_cursor_bg)
        } else {
            (colors.ui_text, colors.ui_background)
        };
        queue!(out,
            MoveTo(col_off + 2, menu_row + i as u16 * 2),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(format!("  {}  ", item))
        )?;
    }
    queue!(out, ResetColor)
}

/// Render the difficulty selection sub-menu.
pub fn render_difficulty(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    selected: usize,
    colors: &ColorScheme,
) -> io::Result<()> {
    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print("Select difficulty:")
    )?;
    for (i, item) in DIFFICULTY_ITEMS.iter().enumerate() {
        let (fg, bg) = if i == selected {
            (colors.ui_background, colors.ui_cursor_bg)
        } else {
            (colors.ui_text, colors.ui_background)
        };
        queue!(out,
            MoveTo(col_off + 2, row_off + 2 + i as u16),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(format!("  {}  ", item))
        )?;
    }
    queue!(out, ResetColor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::colors::ColorScheme;

    #[test]
    fn start_screen_render_does_not_panic() {
        let mut buf = Vec::new();
        render_start(&mut buf, (0, 0), 0, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("New Game"));
        assert!(s.contains("Quit"));
    }

    #[test]
    fn difficulty_screen_render_does_not_panic() {
        let mut buf = Vec::new();
        render_difficulty(&mut buf, (0, 0), 0, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Easy"));
        assert!(s.contains("Medium"));
        assert!(s.contains("Hard"));
    }
}
