// src/tui/render/generating.rs

use crate::i18n::Strings;
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

const TERMINAL_WIDTH: u16 = 117;
const TERMINAL_HEIGHT: u16 = 39;

/// Render the "generating sudoku..." progress screen.
///
/// The message is centred in the grid area (cols 2–74, rows 1–37).
/// Grid centre: col ≈ 38, row ≈ 19.
pub fn render_generating(
    out:           &mut impl Write,
    verb:          &str,
    countdown:     u8,
    show_new_seed: bool,
    strings:       &'static Strings,
    colors:        &ColorScheme,
) -> io::Result<()> {
    let bg  = colors.ui_background;
    let fg  = colors.ui_text;
    let dim = colors.ui_text_dim;

    // Clear full terminal
    for row in 0u16..TERMINAL_HEIGHT {
        queue!(out,
            MoveTo(0, row),
            SetForegroundColor(bg),
            SetBackgroundColor(bg),
            Print(" ".repeat(TERMINAL_WIDTH as usize))
        )?;
    }

    // ── Main message ─────────────────────────────────────────────────────────
    let main_line = if show_new_seed {
        strings.using_new_seed.to_string()
    } else {
        format!("{} sudoku...   {}", verb, countdown)
    };

    let msg_col = (TERMINAL_WIDTH.saturating_sub(main_line.chars().count() as u16)) / 2;
    queue!(out,
        MoveTo(msg_col, 19),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(&main_line)
    )?;

    // ── Cancel hint ───────────────────────────────────────────────────────────
    let cancel = strings.generating_cancel;
    let cancel_col = (TERMINAL_WIDTH.saturating_sub(cancel.chars().count() as u16)) / 2;
    queue!(out,
        MoveTo(cancel_col, 23),
        SetForegroundColor(dim),
        SetBackgroundColor(bg),
        Print(cancel)
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::EN;
    use crate::tui::colors::ColorScheme;

    #[test]
    fn render_generating_normal_does_not_panic() {
        let mut buf = Vec::new();
        render_generating(&mut buf, "baking", 2, false, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("baking sudoku"));
        assert!(s.contains('2'));
    }

    #[test]
    fn render_generating_new_seed_shows_message() {
        let mut buf = Vec::new();
        render_generating(&mut buf, "frying", 0, true, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("using new seed") || s.contains("new seed"));
    }
}
