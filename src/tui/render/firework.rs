// src/tui/render/firework.rs
//
// Two-phase completion overlay:
//
//   Phase 1 (ticks 0–24, ~2 s): "BRAVO!" in figlet standard font, blinking.
//   Phase 2 (ticks 25–74, ~4 s): particle fireworks — rockets rise, explode,
//                                 sparks fly with ASCII-art glyphs and fall under gravity.

use crate::tui::anim::{FireworkAnim, CONGRATS_TICKS};
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

// ── Congrats ASCII art (figlet "standard" font) ───────────────────────────────

/// "BRAVO!" rendered with figlet standard font, 6 lines tall.
/// "BRAVO!" is universally understood across languages as an exclamation of
/// praise, making it a language-neutral celebration message.
const CONGRATS_ART: &[&str] = &[
    " ____   ____      _   __     __  ___  _   ",
    "| __ ) |  _ \\    / \\  \\ \\   / / / _ \\ | | ",
    "|  _ \\ | |_) |  / _ \\  \\ \\ / / | | | || | ",
    "| |_) ||  _ <  / ___ \\  \\ V /  | |_| ||_| ",
    "|____/ |_| \\_\\/_/   \\_\\  \\_/    \\___/ (_) ",
    "                                           ",
];

// Grid geometry (matches render_frame: grid at row_off=1, col_off=2, 73×37).
const GRID_ROW: u16 = 1;
const GRID_COL: u16 = 2;
const GRID_W: u16 = 73;
const GRID_H: u16 = 37;
// Absolute terminal column/row bounds for clipping particles.
// CLIP_ROW_MIN is kept 2 rows below GRID_ROW to prevent particles from
// touching the top border rows, which causes flickering on Windows Terminal.
const CLIP_COL_MIN: i16 = GRID_COL as i16;
const CLIP_COL_MAX: i16 = (GRID_COL + GRID_W - 1) as i16;
const CLIP_ROW_MIN: i16 = GRID_ROW as i16 + 2;
const CLIP_ROW_MAX: i16 = (GRID_ROW + GRID_H - 1) as i16;

// ── Public entry point ────────────────────────────────────────────────────────

/// Overlay the completion animation for the current tick.
pub fn render_firework(
    out: &mut impl Write,
    _grid_origin: (u16, u16), // kept for API symmetry; geometry is hard-coded above
    anim: &FireworkAnim,
    colors: &ColorScheme,
) -> io::Result<()> {
    // Darken the entire grid area first — fireworks look best on a night sky.
    render_dim_overlay(out)?;

    // Phase 1 (ticks 0–24): "BRAVO!" blinking text.
    // Phase 2 (ticks 25+):  text fades out; only rockets/particles remain.
    // Both phases render particles so rockets burst right through the text.
    if anim.tick < CONGRATS_TICKS {
        render_congrats(out, anim.tick, colors)?;
    }
    render_particles(out, anim)?;
    queue!(out, ResetColor)
}

// ── Phase 1: blinking "BRAVO!" ───────────────────────────────────────────────

fn render_congrats(out: &mut impl Write, tick: u32, colors: &ColorScheme) -> io::Result<()> {
    // Blink rhythm: 4 ticks visible (320 ms), 2 ticks hidden (160 ms).
    if tick % 6 >= 4 {
        return Ok(());
    }

    let art_w = CONGRATS_ART
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(47) as u16;
    let art_h = CONGRATS_ART.len() as u16;

    let col = GRID_COL + (GRID_W.saturating_sub(art_w)) / 2;
    let row = GRID_ROW + (GRID_H.saturating_sub(art_h)) / 2;

    // Alternate between two colours for a subtle pulse.
    let fg = if (tick / 6) % 2 == 0 {
        colors.digit_given
    } else {
        colors.ui_text
    };

    for (i, line) in CONGRATS_ART.iter().enumerate() {
        queue!(
            out,
            MoveTo(col, row + i as u16),
            SetForegroundColor(fg),
            SetBackgroundColor(colors.ui_background),
            Print(line)
        )?;
    }
    Ok(())
}

// ── Phase 2: particle fireworks ───────────────────────────────────────────────

fn render_particles(out: &mut impl Write, anim: &FireworkAnim) -> io::Result<()> {
    let bg = Color::Black;

    // Draw rockets — a rising `|` with a two-cell trail.
    for r in &anim.rockets {
        let col = r.x.round() as i16;
        for (dy, ch) in [(0i16, '|'), (1, '|'), (2, '·')] {
            let row = r.y.round() as i16 + dy;
            if in_bounds(col, row) {
                queue!(
                    out,
                    MoveTo(col as u16, row as u16),
                    SetForegroundColor(r.color),
                    SetBackgroundColor(bg),
                    Print(ch)
                )?;
            }
        }
    }

    // Draw particles.
    for p in &anim.particles {
        let col = p.x.round() as i16;
        let row = p.y.round() as i16;
        if !in_bounds(col, row) {
            continue;
        }

        // Dim colour for fading particles.
        let fg = if p.life as f32 / (p.max_life as f32) < 0.4 {
            dim_color(p.color)
        } else {
            p.color
        };

        queue!(
            out,
            MoveTo(col as u16, row as u16),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            Print(p.glyph())
        )?;
    }
    Ok(())
}

// ── Dim overlay ───────────────────────────────────────────────────────────────

/// Fill the entire grid area with black-background spaces.
/// This darkens the puzzle content so the firework reads cleanly on a night-sky.
fn render_dim_overlay(out: &mut impl Write) -> io::Result<()> {
    let blank = " ".repeat(GRID_W as usize);
    for row in GRID_ROW..=(GRID_ROW + GRID_H - 1) {
        queue!(
            out,
            MoveTo(GRID_COL, row),
            SetBackgroundColor(Color::Black),
            Print(&blank)
        )?;
    }
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

#[inline]
fn in_bounds(col: i16, row: i16) -> bool {
    col >= CLIP_COL_MIN && col <= CLIP_COL_MAX && row >= CLIP_ROW_MIN && row <= CLIP_ROW_MAX
}

/// Returns a darker variant of a named colour for the fade-out effect.
fn dim_color(c: Color) -> Color {
    match c {
        Color::Yellow => Color::DarkYellow,
        Color::Cyan => Color::DarkCyan,
        Color::Magenta => Color::DarkMagenta,
        Color::Green => Color::DarkGreen,
        Color::Red => Color::DarkRed,
        Color::White => Color::Grey,
        other => other,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::colors::ColorScheme;

    #[test]
    fn render_firework_does_not_panic() {
        let mut buf = Vec::new();
        let anim = FireworkAnim::new();
        render_firework(&mut buf, (1, 2), &anim, &ColorScheme::default()).unwrap();
    }

    #[test]
    fn render_firework_congrats_phase_produces_output() {
        // Tick 0 is in the congrats phase and blink-visible (0 % 6 = 0 < 4).
        // The figlet art is rendered — check for a distinctive fragment from the art.
        let mut buf = Vec::new();
        let anim = FireworkAnim::new();
        render_firework(&mut buf, (1, 2), &anim, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        // "\\ V /  | |_| |" is the distinctive V+O row in the "BRAVO!" figlet art.
        assert!(
            s.contains("\\ V /  | |_| |"),
            "expected figlet art fragment in output"
        );
    }

    #[test]
    fn render_firework_particle_phase_after_rockets() {
        let mut anim = FireworkAnim::new();
        // Advance past congrats phase and first rocket launch + enough ticks for explosion.
        for _ in 0..50 {
            anim.advance();
        }
        let mut buf = Vec::new();
        render_firework(&mut buf, (1, 2), &anim, &ColorScheme::default()).unwrap();
        assert!(!buf.is_empty());
    }

    #[test]
    fn animation_completes_within_expected_ticks() {
        let mut anim = FireworkAnim::new();
        // Run until done or we exceed a generous budget.
        let mut count = 0;
        while !anim.done() && count < 200 {
            anim.advance();
            count += 1;
        }
        assert!(anim.done(), "animation should complete within 200 ticks");
    }
}
