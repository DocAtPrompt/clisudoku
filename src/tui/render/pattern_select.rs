// src/tui/render/pattern_select.rs

use crate::i18n::Strings;
use crate::pattern::PATTERNS;
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Terminal width assumed for this screen (matches crate::tui::MIN_COLS).
const TERMINAL_WIDTH: u16 = 117;

/// Center column for 117-wide terminal.
/// Miniature: 9 cells × 2 chars (char + space) − 1 trailing space = 17 chars wide.
/// Center on 117 cols: left margin = (117 − 17) / 2 = 50.
const MINIATURE_LEFT: u16 = 50;
const MINIATURE_TOP_ROW: u16 = 6;

pub fn render_pattern_select(
    out:      &mut impl Write,
    selected: usize,
    strings:  &'static Strings,
    colors:   &ColorScheme,
) -> io::Result<()> {
    let pattern = &PATTERNS[selected];
    let bg  = colors.ui_background;
    let fg  = colors.ui_text;
    let dim = colors.ui_text_dim;

    // Clear screen area
    for row in 0u16..24 {
        queue!(out,
            MoveTo(0, row),
            SetForegroundColor(bg),
            SetBackgroundColor(bg),
            Print(" ".repeat(TERMINAL_WIDTH as usize))
        )?;
    }

    // ── Title ────────────────────────────────────────────────────────────────
    let title = strings.designer_title;
    let title_col = (TERMINAL_WIDTH.saturating_sub(title.chars().count() as u16)) / 2;
    queue!(out,
        MoveTo(title_col, 2),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(title)
    )?;

    // ── Pattern name ─────────────────────────────────────────────────────────
    let name = if std::ptr::eq(strings, &crate::i18n::DE) {
        pattern.name_de
    } else {
        pattern.name_en
    };
    let name_col = (TERMINAL_WIDTH.saturating_sub(name.chars().count() as u16)) / 2;
    queue!(out,
        MoveTo(name_col, 4),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(name)
    )?;

    // ── Miniature ─────────────────────────────────────────────────────────────
    let block_fg = colors.digit_given;
    for row in 0..9usize {
        queue!(out, MoveTo(MINIATURE_LEFT, MINIATURE_TOP_ROW + row as u16))?;
        for col in 0..9usize {
            let is_pattern = pattern.mask[row * 9 + col];
            let (ch, cell_fg) = if is_pattern {
                ('\u{2588}', block_fg)   // █
            } else {
                ('\u{00b7}', dim)        // ·
            };
            queue!(out,
                SetForegroundColor(cell_fg),
                SetBackgroundColor(bg),
                Print(ch)
            )?;
            if col < 8 {
                queue!(out,
                    SetForegroundColor(dim),
                    Print(' ')
                )?;
            }
        }
    }

    // ── Cell count ───────────────────────────────────────────────────────────
    let count_str = format!("{} / 81", pattern.cell_count);
    let count_col = (TERMINAL_WIDTH.saturating_sub(count_str.chars().count() as u16)) / 2;
    queue!(out,
        MoveTo(count_col, 16),
        SetForegroundColor(dim),
        SetBackgroundColor(bg),
        Print(&count_str)
    )?;

    // ── Position indicator ◄ N / 28 ► ───────────────────────────────────────
    let pos_str = format!("\u{25c4}  {} / {}  \u{25ba}", selected + 1, PATTERNS.len());
    let pos_col = (TERMINAL_WIDTH.saturating_sub(pos_str.chars().count() as u16)) / 2;
    queue!(out,
        MoveTo(pos_col, 18),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(&pos_str)
    )?;

    // ── Navigation hint ───────────────────────────────────────────────────────
    let hint = "Enter: select   Esc: back";
    let hint_col = (TERMINAL_WIDTH.saturating_sub(hint.chars().count() as u16)) / 2;
    queue!(out,
        MoveTo(hint_col, 20),
        SetForegroundColor(dim),
        SetBackgroundColor(bg),
        Print(hint)
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::EN;
    use crate::tui::colors::ColorScheme;

    #[test]
    fn render_pattern_select_does_not_panic() {
        let mut buf = Vec::new();
        render_pattern_select(&mut buf, 0, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        // Should contain the first pattern name (Holy Crap)
        assert!(s.contains("Holy Crap"), "Expected first pattern name in output");
    }

    #[test]
    fn render_pattern_select_shows_count() {
        let mut buf = Vec::new();
        render_pattern_select(&mut buf, 0, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        // Should contain cell count for Holy Crap (46) and /81
        assert!(s.contains("46"), "Expected cell count 46 (Holy Crap)");
        assert!(s.contains("81"), "Expected /81 in output");
    }
}
