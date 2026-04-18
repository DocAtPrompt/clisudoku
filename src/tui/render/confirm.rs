// src/tui/render/confirm.rs
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Render a modal confirmation dialog at `(row_off, col_off)`.
pub fn render_confirm(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    message: &str,
    colors: &ColorScheme,
) -> io::Result<()> {
    let width = message.len().max(20) + 4;
    let border_h = "─".repeat(width);

    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(format!("┌{}┐", border_h)),
        MoveTo(col_off, row_off + 1),
        Print(format!("│  {:<width$}  │", message, width = width - 2)),
        MoveTo(col_off, row_off + 2),
        Print(format!("│  {:<width$}  │", "[Y] Yes   [N] No", width = width - 2)),
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
    fn confirm_render_shows_message_and_options() {
        let mut buf = Vec::new();
        render_confirm(&mut buf, (5, 10), "Clear this cell?", &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Clear this cell?"));
        assert!(s.contains('['));
    }
}
