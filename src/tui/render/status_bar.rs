// src/tui/render/status_bar.rs
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Format elapsed milliseconds as "MM:SS", capped at 99:59.
pub fn format_elapsed_ms(ms: u64) -> String {
    let total_secs = (ms / 1000).min(99 * 60 + 59);
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

/// Render a one-line status bar showing the timer and current input mode.
pub fn render_status(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    elapsed_ms: u64,
    note_mode: bool,
    colors: &ColorScheme,
) -> io::Result<()> {
    let time_str = format_elapsed_ms(elapsed_ms);
    let mode_str = if note_mode { "Note" } else { "Solution" };
    let text = format!(" {} │ Mode: {} │ [u]ndo  [r]edo  [-]clear  [0]toggle  [Esc]quit ",
        time_str, mode_str);

    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(colors.ui_text),
        SetBackgroundColor(colors.ui_background),
        Print(&text),
        ResetColor
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::colors::ColorScheme;

    #[test]
    fn format_zero_elapsed() {
        assert_eq!(format_elapsed_ms(0), "00:00");
    }

    #[test]
    fn format_90_seconds() {
        assert_eq!(format_elapsed_ms(90_000), "01:30");
    }

    #[test]
    fn format_over_one_hour_caps_at_99_minutes() {
        assert_eq!(format_elapsed_ms(6_000_000), "99:59");
    }

    #[test]
    fn status_bar_contains_time_and_mode() {
        let mut buf = Vec::new();
        render_status(&mut buf, (0, 0), 65_000, false, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("01:05"));
        assert!(s.contains("Solution"));
    }

    #[test]
    fn status_bar_shows_note_mode() {
        let mut buf = Vec::new();
        render_status(&mut buf, (0, 0), 0, true, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Note"));
    }
}
