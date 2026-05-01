// src/tui/render/matrix_rain.rs
//
// Renders the Konami Code Matrix rain animation over the grid area.
// Each column falls at its own speed and start delay; brightness fades from
// White (head) → Green (near trail) → DarkGreen (far trail).

use crate::tui::anim::{MatrixRainAnim, RAIN_COLS, RAIN_ROWS};
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
            queue!(out, MoveTo(col_off + col as u16, row_off + row as u16))?;
            match rain.cell_at(col, row) {
                None => {
                    // Blank out — hides the grid content underneath.
                    queue!(out,
                        SetBackgroundColor(bg),
                        SetForegroundColor(bg),
                        Print(' ')
                    )?;
                }
                Some((ch, level)) => {
                    let fg = match level {
                        0 => Color::White,      // head — brightest flash
                        1 => Color::Green,      // near trail
                        _ => Color::DarkGreen,  // far trail — fading out
                    };
                    queue!(out,
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
