// src/tui/render/status_bar.rs
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Format elapsed milliseconds as "MM:SS", capped at 99:59.
pub fn format_elapsed_ms(ms: u64) -> String {
    let total_secs = (ms / 1000).min(99 * 60 + 59);
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

/// Render a 22-wide × 37-tall info panel to the right of the grid.
///
/// Layout:
///   ╔════════════════════╗
///   ║  Time:  MM:SS      ║
///   ║  Mode:  Solution   ║
///   ╠════════════════════╣
///   ║  Controls          ║
///   ║  ↑↓←→   move       ║
///   ║  …                 ║
///   ╚════════════════════╝
///
/// Total width: 22 chars (║ + 20 inner + ║).
/// Total height: 37 rows (matches grid height).
pub fn render_panel(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    elapsed_ms: u64,
    note_mode: bool,
    colors: &ColorScheme,
) -> io::Result<()> {
    let time_str = format_elapsed_ms(elapsed_ms);
    let mode_label = if note_mode { "Notes" } else { "Solution" };

    let b  = colors.grid_border;
    let t  = colors.ui_text;
    let d  = colors.ui_text_dim;
    let bg = colors.ui_background;

    // ── Top border ────────────────────────────────────────────────────────────
    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(b), SetBackgroundColor(bg),
        Print(format!("╔{}╗", "═".repeat(20)))
    )?;

    // ── Content rows (35 rows, indices 0..35) ─────────────────────────────────
    // Each entry: (text, fg, is_divider)
    let mut rows: Vec<(String, Color, bool)> = vec![
        (format!("  Time:  {}", time_str),   t, false),
        (String::new(),                       t, false),
        (format!("  Mode:  {}", mode_label),  t, false),
        (String::new(),                       t, false),
        // divider
        (String::new(),                       b, true),
        ("  Controls".into(),                 t, false),
        (String::new(),                       d, false),
        ("  \u{2191}\u{2193}\u{2190}\u{2192}   move".into(),  d, false),
        ("  Enter  goto".into(),              d, false),
        ("  1-9    digit".into(),             d, false),
        ("  0      toggle".into(),            d, false),
        ("  u/^Z   undo".into(),              d, false),
        ("  r/^Y   redo".into(),              d, false),
        ("  -      clear".into(),             d, false),
        ("  Spc    pause".into(),             d, false),
        ("  Esc    quit".into(),              d, false),
    ];

    // Fill remaining rows with blanks (reserved for M4 mouse buttons)
    while rows.len() < 35 {
        rows.push((String::new(), d, false));
    }

    for (i, (text, fg, is_divider)) in rows.iter().enumerate() {
        let term_row = row_off + 1 + i as u16;
        if *is_divider {
            queue!(out,
                MoveTo(col_off, term_row),
                SetForegroundColor(b), SetBackgroundColor(bg),
                Print(format!("╠{}╣", "═".repeat(20)))
            )?;
        } else {
            queue!(out,
                MoveTo(col_off, term_row),
                SetForegroundColor(b),  SetBackgroundColor(bg), Print('║'),
                SetForegroundColor(*fg), SetBackgroundColor(bg),
                Print(format!(" {:<18} ", text)),
                SetForegroundColor(b),  SetBackgroundColor(bg), Print('║')
            )?;
        }
    }

    // ── Bottom border ─────────────────────────────────────────────────────────
    queue!(out,
        MoveTo(col_off, row_off + 36),
        SetForegroundColor(b), SetBackgroundColor(bg),
        Print(format!("╚{}╝", "═".repeat(20)))
    )?;

    Ok(())
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
    fn panel_contains_time_and_mode() {
        let mut buf = Vec::new();
        render_panel(&mut buf, (0, 0), 65_000, false, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("01:05"));
        assert!(s.contains("Solution"));
    }

    #[test]
    fn panel_shows_note_mode() {
        let mut buf = Vec::new();
        render_panel(&mut buf, (0, 0), 0, true, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Notes"));
    }

    #[test]
    fn panel_has_border_chars() {
        let mut buf = Vec::new();
        render_panel(&mut buf, (0, 0), 0, false, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains('╔'));
        assert!(s.contains('╚'));
        assert!(s.contains('╠'));
    }
}
