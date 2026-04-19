// src/tui/render/mod.rs
pub mod cell;
pub mod confirm;
pub mod grid;
pub mod start_screen;
pub mod status_bar;

use crate::puzzle::GameState;
use crate::tui::colors::ColorScheme;
use crate::tui::digit_style::DigitStyle;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, ResetColor, SetBackgroundColor},
};
use std::io::{self, Write};

/// All possible UI screens.
pub enum Screen<'a> {
    Start { selected: usize },
    DifficultySelect { selected: usize },
    Game {
        state: &'a GameState,
        cursor: (usize, usize),
        note_mode: bool,
        elapsed_ms: u64,
        paused: bool,
    },
    Confirm {
        /// Screen rendered underneath the overlay.
        underneath: Box<Screen<'a>>,
        message: String,
    },
}

/// Render the full terminal frame for the given screen.
pub fn render_frame(
    out: &mut impl Write,
    screen: &Screen<'_>,
    colors: &ColorScheme,
    style: &dyn DigitStyle,
) -> io::Result<()> {
    // No full Clear — we overwrite every position explicitly via MoveTo.
    // Clearing causes a blank frame between renders, which creates visible flicker.
    queue!(out, MoveTo(0, 0))?;

    match screen {
        Screen::Start { selected } => {
            start_screen::render_start(out, (2, 4), *selected, colors)?;
        }
        Screen::DifficultySelect { selected } => {
            start_screen::render_difficulty(out, (2, 4), *selected, colors)?;
        }
        Screen::Game { state, cursor, note_mode, elapsed_ms, paused } => {
            grid::render_grid(out, (1, 2), state, *cursor, *note_mode, colors, style)?;
            // Panel to the right of the grid: col 2 + 73 (grid) + 2 (gap) = 77
            status_bar::render_panel(out, (1, 77), *elapsed_ms, *note_mode, colors)?;
            if *paused {
                queue!(out,
                    MoveTo(20, 18),
                    SetBackgroundColor(colors.cell_active_bg),
                    Print("  PAUSED — press Space to continue  "),
                    ResetColor
                )?;
            }
        }
        Screen::Confirm { underneath, message } => {
            render_frame(out, underneath, colors, style)?;
            confirm::render_confirm(out, (17, 20), message, colors)?;
        }
    }

    queue!(out, ResetColor)
}
