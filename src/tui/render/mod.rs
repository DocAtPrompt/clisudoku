// src/tui/render/mod.rs
pub mod boss;
pub mod cell;
pub mod confirm;
pub mod firework;
pub mod grid;
pub mod start_screen;
pub mod status_bar;

use crate::puzzle::GameState;
use crate::tui::anim::{AnimState, FireworkAnim};
use crate::tui::colors::ColorScheme;
use crate::tui::digit_style::DigitStyle;
use crate::tui::input::NavState;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
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
        nav: &'a NavState,
        anim: &'a AnimState,
    },
    Confirm {
        /// Screen rendered underneath the overlay.
        underneath: Box<Screen<'a>>,
        /// First line: short description of what is being confirmed.
        title: String,
        /// Second line: the available key options.
        options: String,
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
    queue!(out, MoveTo(0, 0))?;

    match screen {
        Screen::Start { selected } => {
            start_screen::render_start(out, (2, 4), *selected, colors)?;
        }
        Screen::DifficultySelect { selected } => {
            start_screen::render_difficulty(out, (2, 4), *selected, colors)?;
        }
        Screen::Game { state, cursor, note_mode, elapsed_ms, paused, nav, anim } => {
            grid::render_grid(out, (1, 2), state, *cursor, *note_mode, *paused, nav, anim, colors, style)?;
            // Panel to the right of the grid: col 2 + 73 (grid) + 2 (gap) = 77
            status_bar::render_panel(out, (1, 77), *elapsed_ms, *note_mode, colors)?;
            if *paused {
                let msg = "  ■  PAUSED  —  [Space] fortsetzen  ";
                let col = 2 + (73u16 - msg.chars().count() as u16) / 2;
                queue!(out,
                    MoveTo(col, 19),
                    SetForegroundColor(colors.ui_text),
                    SetBackgroundColor(colors.cell_active_bg),
                    Print(msg),
                    ResetColor
                )?;
            }
            // Firework overlay on top of everything
            if let Some(fw) = &anim.firework {
                firework::render_firework(out, (1, 2), fw)?;
            }
        }
        Screen::Confirm { underneath, title, options } => {
            render_frame(out, underneath, colors, style)?;
            confirm::render_confirm(out, (17, 20), title, options, colors)?;
        }
    }

    queue!(out, ResetColor)
}

/// Render a simple read-only info overlay (easter egg messages, etc.).
/// Dismissed by any key. Centred at `(row, col)`.
pub fn render_info_overlay(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    message: &str,
    colors: &ColorScheme,
) -> io::Result<()> {
    // Reuse confirm colours: cyan border, dark-grey bg, white text.
    let inner = message.len().max(20);
    let border_h = "─".repeat(inner + 4);
    let _ = colors;

    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(Color::Cyan), SetBackgroundColor(Color::DarkGrey),
        Print(format!("┌{}┐", border_h)),

        MoveTo(col_off, row_off + 1),
        SetForegroundColor(Color::Cyan),  SetBackgroundColor(Color::DarkGrey), Print('│'),
        SetForegroundColor(Color::White), SetBackgroundColor(Color::DarkGrey),
        Print(format!("  {:<inner$}  ", message, inner = inner)),
        SetForegroundColor(Color::Cyan),  SetBackgroundColor(Color::DarkGrey), Print('│'),

        MoveTo(col_off, row_off + 2),
        SetForegroundColor(Color::Cyan),    SetBackgroundColor(Color::DarkGrey), Print('│'),
        SetForegroundColor(Color::DarkGrey), SetBackgroundColor(Color::DarkGrey),
        Print(format!("  {:<inner$}  ", "— any key to dismiss —", inner = inner)),
        SetForegroundColor(Color::Cyan),    SetBackgroundColor(Color::DarkGrey), Print('│'),

        MoveTo(col_off, row_off + 3),
        SetForegroundColor(Color::Cyan), SetBackgroundColor(Color::DarkGrey),
        Print(format!("└{}┘", border_h)),

        ResetColor
    )
}

// ── Completion helper ─────────────────────────────────────────────────────────

/// Cells of a grid row (reading-order box indices already handled by caller).
pub fn row_cells(r: usize)     -> Vec<(usize, usize)> { (0..9).map(|c| (r, c)).collect() }
pub fn col_cells(c: usize)     -> Vec<(usize, usize)> { (0..9).map(|r| (r, c)).collect() }
pub fn box_cells(b: usize)     -> Vec<(usize, usize)> {
    let br = (b / 3) * 3;
    let bc = (b % 3) * 3;
    (0..3).flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc))).collect()
}
