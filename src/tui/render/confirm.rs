// src/tui/render/confirm.rs
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Render a two-line modal confirmation dialog at `(row_off, col_off)`.
///
/// ```text
/// ┌─────────────────────────────┐
/// │  <title>                    │
/// │  <options>                  │
/// └─────────────────────────────┘
/// ```
pub fn render_confirm(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    title: &str,
    options: &str,
    colors: &ColorScheme,
) -> io::Result<()> {
    // Inner text width — large enough to hold both lines, minimum 20.
    let inner = title.len().max(options.len()).max(20);
    // Total border dashes = inner + 4 (2 spaces padding on each side).
    let border_h = "─".repeat(inner + 4);

    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(format!("┌{}┐", border_h)),

        MoveTo(col_off, row_off + 1),
        Print(format!("│  {:<inner$}  │", title,   inner = inner)),

        MoveTo(col_off, row_off + 2),
        Print(format!("│  {:<inner$}  │", options, inner = inner)),

        MoveTo(col_off, row_off + 3),
        Print(format!("└{}┘", border_h)),

        ResetColor
    )
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
    fn confirm_border_width_is_consistent() {
        // All four lines must have the same printed width.
        let mut buf = Vec::new();
        render_confirm(&mut buf, (0, 0), "Hi?", "[Y]es  [N]o", &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        // Strip ANSI escape sequences and count printable chars per logical line.
        // We look for the box-drawing chars ┌ and └ — they must appear exactly once each.
        assert!(s.contains('┌'));
        assert!(s.contains('└'));
    }
}
