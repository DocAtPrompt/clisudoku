// src/tui/render/matrix_rain.rs
//
// Renders the Konami Code Matrix rain animation over the grid area.
//
// The game screen is rendered first (full grid, correct content).
// This overlay then:
//   - Settled cells  → skipped entirely; real grid content shows through
//   - Blank cells    → overdrawn with background colour (hides unsettled content)
//   - Rain cells     → coloured falling character
//
// As the animation progresses the settled zone grows upward from the bottom row,
// giving the "sand trickling down and piling up" effect.

use crate::tui::anim::{MatrixRainAnim, RainCell, RAIN_COLS, RAIN_ROWS};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

pub fn render_matrix_rain(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    rain: &MatrixRainAnim,
    bg: Color,
) -> io::Result<()> {
    for col in 0..RAIN_COLS {
        for row in 0..RAIN_ROWS {
            match rain.cell_at(col, row) {
                RainCell::Settled => {
                    // Real game content already rendered beneath — nothing to do.
                }
                RainCell::Blank => {
                    queue!(
                        out,
                        MoveTo(col_off + col as u16, row_off + row as u16),
                        SetBackgroundColor(bg),
                        SetForegroundColor(bg),
                        Print(' ')
                    )?;
                }
                RainCell::Rain(ch, level) => {
                    let fg = match level {
                        0 => Color::White,     // head — brightest flash
                        1 => Color::Green,     // near trail
                        _ => Color::DarkGreen, // far trail — fading
                    };
                    queue!(
                        out,
                        MoveTo(col_off + col as u16, row_off + row as u16),
                        SetBackgroundColor(bg),
                        SetForegroundColor(fg),
                        Print(ch)
                    )?;
                }
            }
        }
    }
    Ok(())
}
