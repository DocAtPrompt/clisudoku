// src/tui/render/confirm.rs
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Dialog visual constants.
const BORDER_FG:  Color = Color::Cyan;
const DIALOG_BG:  Color = Color::DarkGrey;
const SHADOW_FG:  Color = Color::DarkGrey;
const SHADOW_BG:  Color = Color::Black;
const SHADOW_CH:  char  = '░';

/// Render a two-line modal confirmation dialog at `(row_off, col_off)`.
///
/// ```text
/// ┌──────────────────────────────┐
/// │  <title>                     │░
/// │  <options>                   │░
/// └──────────────────────────────┘░
///  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
/// ```
///
/// Cyan border on DarkGrey background; one-character `░` shadow on right and bottom.
pub fn render_confirm(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    title: &str,
    options: &str,
    _colors: &ColorScheme,
) -> io::Result<()> {
    // Inner text width — wide enough for both lines, minimum 20.
    let inner = title.len().max(options.len()).max(20);
    // Dashes = inner + 4 (2 spaces padding left + right).
    let border_h = "─".repeat(inner + 4);
    // Total box width in columns: ┌ + dashes + ┐ = inner + 6.
    let box_w = (inner + 6) as u16;

    // ── Top border ────────────────────────────────────────────────────────────
    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(BORDER_FG), SetBackgroundColor(DIALOG_BG),
        Print(format!("┌{}┐", border_h))
    )?;

    // ── Content rows + right shadow ───────────────────────────────────────────
    for (i, text) in [title, options].iter().enumerate() {
        queue!(out,
            MoveTo(col_off, row_off + 1 + i as u16),
            SetForegroundColor(BORDER_FG), SetBackgroundColor(DIALOG_BG), Print('│'),
            SetForegroundColor(Color::White), SetBackgroundColor(DIALOG_BG),
            Print(format!("  {:<inner$}  ", text, inner = inner)),
            SetForegroundColor(BORDER_FG), SetBackgroundColor(DIALOG_BG), Print('│'),
            SetForegroundColor(SHADOW_FG), SetBackgroundColor(SHADOW_BG), Print(SHADOW_CH)
        )?;
    }

    // ── Bottom border + right shadow ──────────────────────────────────────────
    queue!(out,
        MoveTo(col_off, row_off + 3),
        SetForegroundColor(BORDER_FG), SetBackgroundColor(DIALOG_BG),
        Print(format!("└{}┘", border_h)),
        SetForegroundColor(SHADOW_FG), SetBackgroundColor(SHADOW_BG), Print(SHADOW_CH)
    )?;

    // ── Bottom shadow row ─────────────────────────────────────────────────────
    queue!(out,
        MoveTo(col_off + 1, row_off + 4),
        SetForegroundColor(SHADOW_FG), SetBackgroundColor(SHADOW_BG),
        Print(SHADOW_CH.to_string().repeat(box_w as usize))
    )?;

    queue!(out, ResetColor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::colors::ColorScheme;

    #[test]
    fn confirm_render_shows_title_and_options() {
        let mut buf = Vec::new();
        render_confirm(&mut buf, (5, 10), "Clear this cell?", "[Y]es  [N]o", &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Clear this cell?"));
        assert!(s.contains("[Y]es"));
        assert!(s.contains("[N]o"));
    }

    #[test]
    fn confirm_render_contains_shadow_char() {
        let mut buf = Vec::new();
        render_confirm(&mut buf, (0, 0), "Hi?", "[Y]es  [N]o", &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains(SHADOW_CH));
    }

    #[test]
    fn confirm_border_chars_present() {
        let mut buf = Vec::new();
        render_confirm(&mut buf, (0, 0), "Hi?", "[Y]es  [N]o", &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains('┌'));
        assert!(s.contains('└'));
    }
}
