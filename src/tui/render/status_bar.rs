// src/tui/render/status_bar.rs
use crate::i18n::Strings;
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

/// Render a 38-wide × 37-tall info panel to the right of the grid.
///
/// Layout (rows 0-34 inside the borders):
///   Time / Mode / Errors / Filled / digit grid / Controls
pub fn render_panel(
    out:          &mut impl Write,
    (row_off, col_off): (u16, u16),
    elapsed_ms:   u64,
    note_mode:    bool,
    scan_mode:    bool,
    error_mode:   bool,
    errors_shown: u32,
    filled_count: u8,
    digit_counts: [u8; 10],  // digit_counts[d] = how many of digit d are placed (d=1..=9)
    scan_digit:   Option<u8>,
    colors:       &ColorScheme,
    strings:      &'static Strings,
    hint_count:   u32,
    // When `Some((name, explanation))`, replaces the controls section with hint text.
    hint_text:    Option<(&str, &str)>,
    mouse_mode:   bool,
) -> io::Result<()> {
    let _ = mouse_mode;
    let time_str   = format_elapsed_ms(elapsed_ms);
    let mode_label = if note_mode { strings.mode_notes } else { strings.mode_solution };
    let _scan_mode = scan_mode; // visual feedback via grid highlight; no separate panel label

    let b  = colors.grid_border;
    let t  = colors.ui_text;
    let d  = colors.ui_text_dim;
    let bg = colors.ui_background;

    // ── Top border ────────────────────────────────────────────────────────────
    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(b), SetBackgroundColor(bg),
        Print(format!("╔{}╗", "═".repeat(36)))
    )?;

    // ── Content rows (35 rows, indices 0..35) ─────────────────────────────────
    // Indices 10-16 are rendered as blank here; render_digit_grid overwrites them.
    let error_state_label = if error_mode { strings.toggle_on } else { strings.toggle_off };
    let mut rows: Vec<(String, Color, bool)> = vec![
        (strings.panel_time.replacen("{}", &time_str, 1),                    t, false),
        (String::new(),                                                       t, false),
        (strings.panel_mode.replacen("{}", mode_label, 1),                   t, false),
        (String::new(),                                                       t, false),
        (strings.panel_errors.replacen("{}", error_state_label, 1),          t, false),
        (strings.panel_count.replacen("{}", &errors_shown.to_string(), 1),   t, false),
        (String::new(),                                                       t, false),
        (strings.panel_filled.replacen("{}", &filled_count.to_string(), 1),  t, false),
        (strings.panel_remaining.into(),                                      d, false),
        // rows 10-16: placeholders for digit grid (overwritten below)
        (String::new(), t, false),
        (String::new(), t, false),
        (String::new(), t, false),
        (String::new(), t, false),
        (String::new(), t, false),
        (String::new(), t, false),
        (String::new(), t, false),
        // hint count
        (format!("  h: {}", hint_count),        t, false),
        // divider
        (String::new(),                        b, true),
    ];

    if let Some((name, explanation)) = hint_text {
        // Hint mode: replace controls with strategy name + explanation.
        rows.push((name.to_string(), t, false));
        rows.push((String::new(), d, false));
        for line in word_wrap(explanation, 34) {
            rows.push((line, d, false));
        }
        rows.push((String::new(), d, false));
        rows.push((strings.dismiss.into(), d, false));
    } else {
        rows.extend(vec![
            (strings.panel_controls.into(),        t, false),
            (String::new(),                        d, false),
            (strings.ctrl_move.into(),             d, false),
            (strings.ctrl_goto.into(),             d, false),
            (strings.ctrl_digit.into(),            d, false),
            (strings.ctrl_mode.into(),             d, false),
            (strings.ctrl_scan.into(),             d, false),
            (strings.ctrl_errors.into(),           d, false),
            (strings.ctrl_hint.into(),             d, false),
            (strings.ctrl_undo.into(),             d, false),
            (strings.ctrl_redo.into(),             d, false),
            (strings.ctrl_clear.into(),            d, false),
            (strings.ctrl_pause.into(),            d, false),
            (strings.ctrl_boss.into(),             d, false),
            (strings.ctrl_quit.into(),             d, false),
        ]);
    }

    while rows.len() < 35 {
        rows.push((String::new(), d, false));
    }

    for (i, (text, fg, is_divider)) in rows.iter().enumerate() {
        let term_row = row_off + 1 + i as u16;
        if *is_divider {
            queue!(out,
                MoveTo(col_off, term_row),
                SetForegroundColor(b), SetBackgroundColor(bg),
                Print(format!("╠{}╣", "═".repeat(36)))
            )?;
        } else {
            // Truncate to 34 display chars so panel width is always fixed,
            // regardless of translation length.
            let cell: String = text.chars().take(34).collect();
            queue!(out,
                MoveTo(col_off, term_row),
                SetForegroundColor(b),   SetBackgroundColor(bg), Print('║'),
                SetForegroundColor(*fg), SetBackgroundColor(bg),
                Print(format!(" {:<34} ", cell)),
                SetForegroundColor(b),   SetBackgroundColor(bg), Print('║')
            )?;
        }
    }

    // ── Digit availability grid (overwrites rows 10-16 above) ─────────────────
    // Position: row_off+10 (1 base + 9 rows + "Remaining:" label), col_off+3.
    render_digit_grid(out, row_off + 10, col_off + 3, &digit_counts, scan_digit, colors)?;

    // ── Bottom border ─────────────────────────────────────────────────────────
    queue!(out,
        MoveTo(col_off, row_off + 36),
        SetForegroundColor(b), SetBackgroundColor(bg),
        Print(format!("╚{}╝", "═".repeat(36)))
    )?;

    Ok(())
}

// ── Digit availability grid ───────────────────────────────────────────────────

/// Render a 3×3 grid of digit counters:
///
///   ┌1──┬2──┬3──┐
///   │ 8 │ · │ 7 │
///   ├4──┼5──┼6──┤
///   │ 2 │ 1 │ 9 │
///   ├7──┼8──┼9──┤
///   │ 6 │ · │ 3 │
///   └───┴───┴───┘
///
/// The digit label lives in the top-left corner of its cell border.
/// Count shows how many of that digit are *still available* (9 − placed).
/// Colors:
///   - complete (0 remaining): dim, shows · instead of 0
///   - 1 remaining: Yellow (warning)
///   - scan digit:  digit_scan color (Magenta)
///   - normal:      ui_text
fn render_digit_grid(
    out:          &mut impl Write,
    row_off:      u16,
    col_off:      u16,
    digit_counts: &[u8; 10],
    scan_digit:   Option<u8>,
    colors:       &ColorScheme,
) -> io::Result<()> {
    let bg = colors.ui_background;
    let bc = colors.grid_border;

    // Per-digit foreground and background based on remaining count and scan state.
    let dig_fg = |d: usize| -> Color {
        let remaining = 9u8.saturating_sub(digit_counts[d]);
        if remaining == 0 {
            colors.ui_text_dim
        } else if scan_digit == Some(d as u8) {
            colors.digit_scan          // Magenta
        } else if remaining == 1 {
            colors.digit_highlight     // Yellow
        } else {
            colors.ui_text
        }
    };

    // Highlighted scan cell gets a bright background so Magenta is clearly readable.
    let dig_bg = |d: usize| -> Color {
        if scan_digit == Some(d as u8) { Color::White } else { bg }
    };

    // Character shown for the remaining count.
    let dig_char = |d: usize| -> char {
        let remaining = 9u8.saturating_sub(digit_counts[d]);
        if remaining == 0 { '\u{00b7}' } else { char::from(b'0' + remaining) }
    };

    let box_rows: [[usize; 3]; 3] = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];

    for (bi, digits) in box_rows.iter().enumerate() {
        let header_row  = row_off + (bi * 2) as u16;
        let content_row = row_off + (bi * 2 + 1) as u16;

        let (lc, mc, rc) = if bi == 0 { ('┌', '┬', '┐') } else { ('├', '┼', '┤') };

        // Header row: e.g. ┌1──┬2──┬3──┐  (no background highlight here)
        queue!(out, MoveTo(col_off, header_row),
            SetForegroundColor(bc), SetBackgroundColor(bg), Print(lc))?;
        for (ci, &d) in digits.iter().enumerate() {
            queue!(out,
                SetForegroundColor(dig_fg(d)), SetBackgroundColor(bg),
                Print(char::from(b'0' + d as u8)),
                SetForegroundColor(bc), SetBackgroundColor(bg), Print("──"),
                Print(if ci < 2 { mc } else { rc })
            )?;
        }

        // Content row: e.g. │ 8 │ · │ 7 │
        // Only the single counter digit gets the grey background; spaces and │ unchanged.
        queue!(out, MoveTo(col_off, content_row))?;
        for &d in digits.iter() {
            let fg  = dig_fg(d);
            let cbg = dig_bg(d);
            queue!(out,
                SetForegroundColor(bc), SetBackgroundColor(bg), Print('│'),
                SetForegroundColor(bc), SetBackgroundColor(bg), Print(' '),
                SetForegroundColor(fg), SetBackgroundColor(cbg), Print(dig_char(d)),
                SetForegroundColor(bc), SetBackgroundColor(bg), Print(' ')
            )?;
        }
        queue!(out, SetForegroundColor(bc), SetBackgroundColor(bg), Print('│'))?;
    }

    // Bottom border
    queue!(out,
        MoveTo(col_off, row_off + 6),
        SetForegroundColor(bc), SetBackgroundColor(bg),
        Print("└───┴───┴───┘")
    )
}

// ── Word wrap helper ──────────────────────────────────────────────────────────

/// Wrap `text` to lines of at most `width` characters, splitting on whitespace.
/// Words longer than `width` get their own (overflowing) line.
fn word_wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
            if current.chars().count() >= width {
                lines.push(current.clone());
                current.clear();
            }
        } else if current.chars().count() + 1 + word.chars().count() <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current.clone());
            current = word.to_string();
        }
    }
    if !current.is_empty() { lines.push(current); }
    lines
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::EN;
    use crate::tui::colors::ColorScheme;

    fn call_render_panel(buf: &mut Vec<u8>, elapsed_ms: u64, note_mode: bool) {
        render_panel(
            buf, (0, 0), elapsed_ms, note_mode, false, false, 0, 0,
            [0u8; 10], None, &ColorScheme::default(), &EN, 0, None, false,
        ).unwrap();
    }

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
        call_render_panel(&mut buf, 65_000, false);
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("01:05"));
        assert!(s.contains("Solution"));
    }

    #[test]
    fn panel_shows_note_mode() {
        let mut buf = Vec::new();
        call_render_panel(&mut buf, 0, true);
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Notes"));
    }

    #[test]
    fn panel_has_border_chars() {
        let mut buf = Vec::new();
        call_render_panel(&mut buf, 0, false);
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains('╔'));
        assert!(s.contains('╚'));
        assert!(s.contains('╠'));
    }

    #[test]
    fn digit_grid_shows_remaining_counts() {
        let mut buf = Vec::new();
        // Place 7 of digit 5 → 2 remaining
        let mut counts = [0u8; 10];
        counts[5] = 7;
        render_panel(
            &mut buf, (0, 0), 0, false, false, false, 0, 0,
            counts, None, &ColorScheme::default(), &EN, 0, None, false,
        ).unwrap();
        let s = String::from_utf8_lossy(&buf);
        // Digit 5 header char '5' and count '2' should appear
        assert!(s.contains('5'));
        assert!(s.contains('2'));
    }

    #[test]
    fn panel_shows_hint_name_when_hint_active() {
        let mut buf = Vec::new();
        render_panel(
            &mut buf, (0, 0), 0, false, false, false, 0, 0,
            [0u8; 10], None, &ColorScheme::default(), &EN, 0,
            Some(("Naked Single", "Only 5 fits in this cell.")), false,
        ).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Naked Single"));
        assert!(s.contains("Only 5 fits"));
        // dismiss line
        assert!(s.contains("press any key"));
        // Controls heading should NOT appear in hint mode
        assert!(!s.contains("Controls"));
    }

    #[test]
    fn word_wrap_splits_on_width() {
        let lines = word_wrap("Only 5 fits in this cell because all others are eliminated.", 20);
        assert!(lines.iter().all(|l| l.chars().count() <= 20));
        assert!(lines.len() >= 2);
    }

    #[test]
    fn digit_grid_shows_dot_for_complete_digit() {
        let mut buf = Vec::new();
        let mut counts = [0u8; 10];
        counts[3] = 9; // digit 3 fully placed
        render_panel(
            &mut buf, (0, 0), 0, false, false, false, 0, 0,
            counts, None, &ColorScheme::default(), &EN, 0, None, false,
        ).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains('\u{00b7}')); // · for completed digit
    }
}
