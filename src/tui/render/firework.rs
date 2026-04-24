// src/tui/render/firework.rs
//
// Renders the ASCII firework overlay on puzzle completion.
// Drawn over the existing game screen without clearing it.

use crate::tui::anim::{burst_chars, FireworkAnim, BURST_SITES};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Overlay the firework for the current animation frame.
/// `(row_off, col_off)` is the top-left corner of the grid area.
pub fn render_firework(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    anim: &FireworkAnim,
) -> io::Result<()> {
    for site in BURST_SITES {
        let frame = anim.frame;
        if frame < site.start_frame { continue; }
        let local = frame - site.start_frame;

        let chars = burst_chars(local);
        if chars.is_empty() { continue; }

        for &(dr, dc, ch) in chars {
            let r = (row_off + site.row) as i16 + dr;
            let c = (col_off + site.col) as i16 + dc;
            if r < 0 || c < 0 { continue; }
            queue!(out,
                MoveTo(c as u16, r as u16),
                SetForegroundColor(site.color),
                SetBackgroundColor(Color::Black),
                Print(ch)
            )?;
        }
    }
    queue!(out, ResetColor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_firework_does_not_panic() {
        let mut buf = Vec::new();
        let anim = FireworkAnim::new();
        render_firework(&mut buf, (1, 2), &anim).unwrap();
        // Frame 0 has burst sites with start_frame > 0, so might be empty — just no panic.
    }

    #[test]
    fn render_firework_frame_3_produces_output() {
        let mut buf = Vec::new();
        let mut anim = FireworkAnim::new();
        for _ in 0..3 { anim.advance(); } // reach frame 3 (burst site 0 active)
        render_firework(&mut buf, (1, 2), &anim).unwrap();
        assert!(!buf.is_empty());
    }
}
