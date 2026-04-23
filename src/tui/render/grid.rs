// src/tui/render/grid.rs
use crate::puzzle::{CellKind, GameState};
use crate::tui::colors::ColorScheme;
use crate::tui::digit_style::DigitStyle;
use crate::tui::input::{NavMode, NavState};
use crate::tui::render::cell::cell_display_lines;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

// ── Border characters ──────────────────────────────────────────────────────
const TL: char = '╔'; const TR: char = '╗';
const BL: char = '╚'; const BR: char = '╝';
const OUTER_H: char = '═'; const OUTER_V: char = '║';
const TOP_SEP: char = '╤'; const BOT_SEP: char = '╧';
const L_SEP: char = '╟'; const R_SEP: char = '╢';
const BOX_V: char = '┃'; const THIN_V: char = '│';
const BOX_H: char = '━';
const BOX_X_BOX: char = '╋'; const BOX_X_THIN: char = '┿';
const THIN_H: char = '─';
const THIN_X_BOX: char = '╂'; const THIN_X_THIN: char = '┼';

fn is_box_col(col: usize) -> bool { col == 2 || col == 5 }
fn is_box_row(row: usize) -> bool { row == 2 || row == 5 }

fn v_sep(col: usize) -> char {
    if col == 8 { OUTER_V }
    else if is_box_col(col) { BOX_V }
    else { THIN_V }
}

fn h_cross(heavy: bool, col: usize) -> char {
    match (heavy, is_box_col(col)) {
        (true,  true)  => BOX_X_BOX,
        (true,  false) => BOX_X_THIN,
        (false, true)  => THIN_X_BOX,
        (false, false) => THIN_X_THIN,
    }
}

/// Which 3×3 box (0-8) does this cell belong to?
fn box_of(row: usize, col: usize) -> usize {
    (row / 3) * 3 + col / 3
}

/// Cell background for normal (non-paused) rendering.
///
/// Stage rules:
///   NavMode::Navigation, no box → entire grid highlighted (pending first digit)
///   NavMode::Navigation, box b  → only the selected box highlighted (pending second digit)
///   NavMode::Input              → only cursor cell highlighted (normal editing)
fn cell_bg(
    row: usize,
    col: usize,
    cursor: (usize, usize),
    nav: &NavState,
    colors: &ColorScheme,
) -> Color {
    match (&nav.mode, nav.box_idx) {
        (NavMode::Navigation, None) => {
            // Stage 1: whole grid is the "selection pending" hint
            colors.cell_active_bg
        }
        (NavMode::Navigation, Some(selected_box)) => {
            // Stage 2: only the chosen box is highlighted
            if box_of(row, col) == selected_box {
                colors.cell_active_bg
            } else {
                colors.cell_normal_bg
            }
        }
        (NavMode::Input, _) => {
            // Normal editing: only cursor cell
            let (cr, cc) = cursor;
            if row == cr && col == cc {
                colors.cell_active_bg
            } else {
                colors.cell_normal_bg
            }
        }
    }
}

/// Render the full 73×37 Sudoku grid at terminal position `(row_off, col_off)`.
pub fn render_grid(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    state: &GameState,
    cursor: (usize, usize),
    note_mode: bool,
    paused: bool,
    nav: &NavState,
    colors: &ColorScheme,
    style: &dyn DigitStyle,
) -> io::Result<()> {
    let _ = note_mode; // reserved for future cursor highlight differentiation
    let overlay_bg = Color::Rgb { r: 35, g: 35, b: 35 };

    // ── Top border ──────────────────────────────────────────────────────────
    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(colors.grid_border),
        SetBackgroundColor(colors.ui_background),
        Print(TL)
    )?;
    for col in 0..9usize {
        for _ in 0..7 { queue!(out, Print(OUTER_H))?; }
        queue!(out, Print(if col < 8 { TOP_SEP } else { TR }))?;
    }

    // ── Cell rows and separators ────────────────────────────────────────────
    for row in 0..9usize {
        // 3 content lines per row
        for line_idx in 0..3usize {
            let term_row = row_off + 1 + (row * 4 + line_idx) as u16;
            queue!(out,
                MoveTo(col_off, term_row),
                SetForegroundColor(colors.grid_border),
                SetBackgroundColor(colors.ui_background),
                Print(OUTER_V)
            )?;

            for col in 0..9usize {
                let (fg, bg, content) = if paused {
                    (overlay_bg, overlay_bg, "       ".to_string())
                } else {
                    let cell = state.grid().get(row, col);
                    let notes_mask = state.notes_mask(row, col);
                    let content_lines = cell_display_lines(&cell, notes_mask, style);
                    let fg = match &cell {
                        CellKind::Given(_) => colors.digit_given,
                        CellKind::Filled(_) => colors.digit_user,
                        CellKind::Empty if notes_mask != 0 => colors.note_normal,
                        _ => colors.grid_cell,
                    };
                    (fg, cell_bg(row, col, cursor, nav, colors), content_lines[line_idx].clone())
                };

                let sep_fg = if paused { overlay_bg }
                             else if col == 8 { colors.grid_border }
                             else if col == 2 || col == 5 { colors.grid_box }
                             else { colors.grid_cell };
                let sep_bg = if paused { overlay_bg }
                             else if col == 8 { colors.ui_background }
                             else { bg };
                queue!(out,
                    SetForegroundColor(fg),
                    SetBackgroundColor(bg),
                    Print(&content),
                    SetForegroundColor(sep_fg),
                    SetBackgroundColor(sep_bg),
                    Print(v_sep(col))
                )?;
            }
        }

        // Separator row after this row (not after row 8)
        if row < 8 {
            let heavy = is_box_row(row);
            let fill = if paused { ' ' } else if heavy { BOX_H } else { THIN_H };
            let border_fg = if paused { overlay_bg }
                            else if heavy { colors.grid_box }
                            else { colors.grid_cell };
            let row_bg = if paused { overlay_bg } else { colors.ui_background };
            let term_row = row_off + 1 + (row * 4 + 3) as u16;
            queue!(out,
                MoveTo(col_off, term_row),
                SetForegroundColor(if paused { overlay_bg } else { colors.grid_border }),
                SetBackgroundColor(row_bg),
                Print(if paused { ' ' } else { L_SEP })
            )?;
            for col in 0..9usize {
                queue!(out,
                    SetForegroundColor(border_fg),
                    SetBackgroundColor(row_bg)
                )?;
                for _ in 0..7 { queue!(out, Print(fill))?; }
                if col < 8 {
                    queue!(out,
                        SetForegroundColor(border_fg),
                        SetBackgroundColor(row_bg),
                        Print(if paused { ' ' } else { h_cross(heavy, col) })
                    )?;
                }
            }
            queue!(out,
                SetForegroundColor(if paused { overlay_bg } else { colors.grid_border }),
                SetBackgroundColor(row_bg),
                Print(if paused { ' ' } else { R_SEP })
            )?;
        }
    }

    // ── Bottom border ───────────────────────────────────────────────────────
    let bottom_row = row_off + 1 + (8 * 4 + 3) as u16; // = row_off + 36
    queue!(out,
        MoveTo(col_off, bottom_row),
        SetForegroundColor(colors.grid_border),
        SetBackgroundColor(colors.ui_background),
        Print(BL)
    )?;
    for col in 0..9usize {
        for _ in 0..7 { queue!(out, Print(OUTER_H))?; }
        queue!(out, Print(if col < 8 { BOT_SEP } else { BR }))?;
    }

    queue!(out, ResetColor)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{Grid, GameState};
    use crate::tui::colors::ColorScheme;
    use crate::tui::digit_style::RetroStyle;
    use crate::tui::input::{NavMode, NavState};

    fn empty_state() -> GameState {
        GameState::new(Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        ).unwrap())
    }

    fn nav_input() -> NavState { NavState { mode: NavMode::Input, box_idx: None } }
    fn nav_grid()  -> NavState { NavState { mode: NavMode::Navigation, box_idx: None } }
    fn nav_box(b: usize) -> NavState { NavState { mode: NavMode::Navigation, box_idx: Some(b) } }

    #[test]
    fn grid_render_contains_outer_border_chars() {
        let state = empty_state();
        let mut buf = Vec::new();
        render_grid(&mut buf, (0, 0), &state, (0, 0), false, false, &nav_input(), &ColorScheme::default(), &RetroStyle)
            .unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains('╔'));
        assert!(s.contains('╗'));
        assert!(s.contains('╚'));
        assert!(s.contains('╝'));
    }

    #[test]
    fn grid_render_contains_box_separators() {
        let state = empty_state();
        let mut buf = Vec::new();
        render_grid(&mut buf, (0, 0), &state, (0, 0), false, false, &nav_input(), &ColorScheme::default(), &RetroStyle)
            .unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains('┃'));
        assert!(s.contains('━'));
        assert!(s.contains('╋'));
    }

    #[test]
    fn grid_render_does_not_panic_with_filled_grid() {
        let grid = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
        ).unwrap();
        let state = GameState::new(grid);
        let mut buf = Vec::new();
        render_grid(&mut buf, (0, 0), &state, (4, 4), false, false, &nav_input(), &ColorScheme::default(), &RetroStyle)
            .unwrap();
        assert!(!buf.is_empty());
    }

    #[test]
    fn nav_grid_stage_all_cells_active_bg() {
        // Stage 1: NavMode::Navigation with no box — entire grid should use active_bg
        let state = empty_state();
        let colors = ColorScheme::default();
        assert_eq!(cell_bg(0, 0, (4, 4), &nav_grid(), &colors), colors.cell_active_bg);
        assert_eq!(cell_bg(8, 8, (4, 4), &nav_grid(), &colors), colors.cell_active_bg);
        assert_eq!(cell_bg(4, 4, (4, 4), &nav_grid(), &colors), colors.cell_active_bg);
    }

    #[test]
    fn nav_box_stage_only_selected_box_active() {
        // Stage 2: box 4 (center) = rows 3-5, cols 3-5
        let state = empty_state();
        let _ = state;
        let colors = ColorScheme::default();
        let nav = nav_box(4);
        // Inside box 4
        assert_eq!(cell_bg(3, 3, (0, 0), &nav, &colors), colors.cell_active_bg);
        assert_eq!(cell_bg(5, 5, (0, 0), &nav, &colors), colors.cell_active_bg);
        // Outside box 4
        assert_eq!(cell_bg(0, 0, (0, 0), &nav, &colors), colors.cell_normal_bg);
        assert_eq!(cell_bg(8, 8, (0, 0), &nav, &colors), colors.cell_normal_bg);
    }

    #[test]
    fn nav_input_stage_only_cursor_active() {
        let colors = ColorScheme::default();
        let nav = nav_input();
        assert_eq!(cell_bg(4, 4, (4, 4), &nav, &colors), colors.cell_active_bg);
        assert_eq!(cell_bg(0, 0, (4, 4), &nav, &colors), colors.cell_normal_bg);
    }
}
