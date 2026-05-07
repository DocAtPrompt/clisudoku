// src/tui/render/grid.rs
use crate::puzzle::{CellKind, GameState, Grid};
use crate::tui::anim::AnimState;
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
const TL: char = '╔';
const TR: char = '╗';
const BL: char = '╚';
const BR: char = '╝';
const OUTER_H: char = '═';
const OUTER_V: char = '║';
const TOP_SEP: char = '╤';
const BOT_SEP: char = '╧';
const L_SEP: char = '╟';
const R_SEP: char = '╢';
const BOX_V: char = '┃';
const THIN_V: char = '│';
const BOX_H: char = '━';
const BOX_X_BOX: char = '╋';
const BOX_X_THIN: char = '┿';
const THIN_H: char = '─';
const THIN_X_BOX: char = '╂';
const THIN_X_THIN: char = '┼';

// ── Hint role helpers ──────────────────────────────────────────────────────
#[derive(PartialEq, Eq, Clone, Copy)]
enum HintRole {
    Cause,
    Elim,
    Target,
}

fn hint_role(row: usize, col: usize, hint: &crate::hint::Hint) -> Option<HintRole> {
    if hint.target_cell == (row, col) {
        return Some(HintRole::Target);
    }
    if hint.cause_cells.contains(&(row, col)) {
        return Some(HintRole::Cause);
    }
    if hint.elim_cells.contains(&(row, col)) {
        return Some(HintRole::Elim);
    }
    None
}

fn hint_role_color(role: HintRole, colors: &ColorScheme) -> Color {
    match role {
        HintRole::Cause => colors.hint_cause_border,
        HintRole::Elim => colors.hint_elim_border,
        HintRole::Target => colors.hint_cause_border,
    }
}

/// Colour for the vertical separator to the RIGHT of cell (row, col), if a
/// hint border applies. Returns None when both adjacent cells share the same
/// hint role (no border between them) or when neither has a role.
fn hint_right_border_color(
    row: usize,
    col: usize,
    hint: Option<&crate::hint::Hint>,
    colors: &ColorScheme,
) -> Option<Color> {
    let h = hint?;
    let role_left = hint_role(row, col, h);
    let role_right = if col < 8 {
        hint_role(row, col + 1, h)
    } else {
        None
    };
    if role_left == role_right {
        return None;
    }
    let role = role_left.or(role_right)?;
    Some(hint_role_color(role, colors))
}

/// Colour for the horizontal segment between row `row_above` and `row_above+1`
/// at column `col`, if a hint border applies.
fn hint_h_seg_color(
    row_above: usize,
    col: usize,
    hint: Option<&crate::hint::Hint>,
    colors: &ColorScheme,
) -> Option<Color> {
    let h = hint?;
    let role_above = hint_role(row_above, col, h);
    let role_below = if row_above + 1 < 9 {
        hint_role(row_above + 1, col, h)
    } else {
        None
    };
    if role_above == role_below {
        return None;
    }
    let role = role_above.or(role_below)?;
    Some(hint_role_color(role, colors))
}

fn is_box_col(col: usize) -> bool {
    col == 2 || col == 5
}
fn is_box_row(row: usize) -> bool {
    row == 2 || row == 5
}

fn v_sep(col: usize) -> char {
    if col == 8 {
        OUTER_V
    } else if is_box_col(col) {
        BOX_V
    } else {
        THIN_V
    }
}

fn h_cross(heavy: bool, col: usize) -> char {
    match (heavy, is_box_col(col)) {
        (true, true) => BOX_X_BOX,
        (true, false) => BOX_X_THIN,
        (false, true) => THIN_X_BOX,
        (false, false) => THIN_X_THIN,
    }
}

/// Which 3×3 box (0-8, reading order: 0=top-left … 8=bottom-right) does this cell belong to?
fn box_of(row: usize, col: usize) -> usize {
    (row / 3) * 3 + col / 3
}

/// Convert a numpad box index (as stored in NavState::box_idx) to a reading-order box index.
///
/// Numpad layout: '1'→idx 0 = bottom-left, '9'→idx 8 = top-right.
/// Reading order: 0 = top-left, 8 = bottom-right.
///
/// Mapping: reading_box = (2 - numpad_idx/3) * 3 + numpad_idx%3
fn numpad_to_reading_box(numpad_idx: usize) -> usize {
    (2 - numpad_idx / 3) * 3 + numpad_idx % 3
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
    hover_cell: Option<(usize, usize)>,
    nav: &NavState,
    hint: Option<&crate::hint::Hint>,
    anim: &AnimState,
    colors: &ColorScheme,
) -> Color {
    // Hint background takes priority over nav/cursor highlights.
    if let Some(h) = hint {
        match hint_role(row, col, h) {
            Some(HintRole::Target) => {
                // Blink regardless of cursor position: yellow ↔ normal (or cursor-blue
                // if the cursor happens to be on this cell).
                if anim.hint_cell_yellow_phase() {
                    return colors.hint_target_bg;
                } else if cursor == (row, col) {
                    return colors.cell_active_bg;
                } else {
                    return colors.cell_normal_bg;
                }
            }
            Some(HintRole::Cause) | Some(HintRole::Elim) => {
                return colors.hint_target_bg;
            }
            None => {}
        }
    }

    // Hover highlight: distinct from cursor colour, routed through ColorScheme.
    // Cursor takes priority — checked after hints, before nav mode highlights.
    if let Some(hc) = hover_cell {
        if hc == (row, col) && cursor != (row, col) {
            return colors.hover_bg;
        }
    }

    match (&nav.mode, nav.box_idx) {
        (NavMode::Navigation, None) => {
            // Stage 1: whole grid is the "selection pending" hint
            colors.cell_active_bg
        }
        (NavMode::Navigation, Some(numpad_box)) => {
            // Stage 2: only the chosen box is highlighted.
            // nav.box_idx is a numpad index; convert to reading-order before comparing.
            if box_of(row, col) == numpad_to_reading_box(numpad_box) {
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

/// Matrix Mode green palette — used when `matrix_mode` is active.
const MATRIX_BRIGHT: Color = Color::Green; // standard ANSI bright green — given digits
const MATRIX_MID: Color = Color::DarkGreen; // user-filled digits
const MATRIX_DIM: Color = Color::DarkGreen; // notes

/// Render the full 73×37 Sudoku grid at terminal position `(row_off, col_off)`.
pub fn render_grid(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    state: &GameState,
    cursor: (usize, usize),
    note_mode: bool,
    paused: bool,
    nav: &NavState,
    anim: &AnimState,
    scan_digit: Option<u8>,
    error_mode: bool,
    solution: Option<&Grid>,
    hint: Option<&crate::hint::Hint>,
    colors: &ColorScheme,
    given_style: &dyn DigitStyle,
    filled_style: &dyn DigitStyle,
    matrix_mode: bool,
    hover_cell: Option<(usize, usize)>,
) -> io::Result<()> {
    let _ = note_mode; // reserved for future cursor highlight differentiation
    let overlay_bg = Color::DarkGrey;

    // ── Top border ──────────────────────────────────────────────────────────
    queue!(
        out,
        MoveTo(col_off, row_off),
        SetForegroundColor(colors.grid_border),
        SetBackgroundColor(colors.ui_background),
        Print(TL)
    )?;
    for col in 0..9usize {
        for _ in 0..7 {
            queue!(out, Print(OUTER_H))?;
        }
        queue!(out, Print(if col < 8 { TOP_SEP } else { TR }))?;
    }

    // ── Cell rows and separators ────────────────────────────────────────────
    for row in 0..9usize {
        // 3 content lines per row
        for line_idx in 0..3usize {
            let term_row = row_off + 1 + (row * 4 + line_idx) as u16;
            queue!(
                out,
                MoveTo(col_off, term_row),
                SetForegroundColor(colors.grid_border),
                SetBackgroundColor(colors.ui_background),
                Print(OUTER_V)
            )?;

            for col in 0..9usize {
                let (fg, bg, content, note_scan_col, blink) = if paused {
                    (overlay_bg, overlay_bg, "       ".to_string(), None, false)
                } else {
                    let cell = state.grid().get(row, col);
                    let notes_mask = state.notes_mask(row, col);
                    let content_lines =
                        cell_display_lines(&cell, notes_mask, given_style, filled_style);
                    // Sweep animation: invert this cell if it is the active step.
                    // Sweep takes priority over all other colour decisions.
                    if let Some((sweep_fg, sweep_bg)) = anim.sweep_highlight(row, col) {
                        (
                            sweep_fg,
                            sweep_bg,
                            content_lines[line_idx].clone(),
                            None,
                            false,
                        )
                    } else {
                        // Matrix Mode overrides: remap standard digit/note colours to green.
                        let (c_given, c_user, c_note) = if matrix_mode {
                            (MATRIX_BRIGHT, MATRIX_MID, MATRIX_DIM)
                        } else {
                            (colors.digit_given, colors.digit_user, colors.note_normal)
                        };

                        // Passive scan: highlight matching digit in scan colour.
                        let (fg, note_scan_col, blink) = match (&cell, scan_digit) {
                            (CellKind::Given(d), Some(sd)) if *d == sd => {
                                (colors.digit_scan, None, false)
                            }
                            // Error check: wrong filled digit → red + blink when error_mode is on.
                            (CellKind::Filled(d), _) => {
                                let wrong = solution
                                    .and_then(|sol| sol.get(row, col).value())
                                    .map(|correct| correct != *d)
                                    .unwrap_or(false);
                                let show_red = wrong && error_mode;
                                let col_fg = if show_red {
                                    colors.digit_error
                                } else if scan_digit == Some(*d) {
                                    colors.digit_scan
                                } else {
                                    c_user
                                };
                                (col_fg, None, show_red)
                            }
                            (CellKind::Given(_), _) => (c_given, None, false),
                            (CellKind::Empty, Some(sd)) if notes_mask & (1 << sd) != 0 => {
                                let note_line = (sd - 1) / 3;
                                let nsc = if line_idx == note_line as usize {
                                    Some(((sd - 1) % 3) * 2 + 1)
                                } else {
                                    None
                                };
                                (c_note, nsc, false)
                            }
                            (CellKind::Empty, _) if notes_mask != 0 => (c_note, None, false),
                            _ => (colors.grid_cell, None, false),
                        };
                        let bg = cell_bg(row, col, cursor, hover_cell, nav, hint, anim, colors);
                        // Ensure readable contrast: use dark text on the yellow hint
                        // background so digits don't disappear against the highlight.
                        let on_hint_yellow = hint.and_then(|h| hint_role(row, col, h)).is_some()
                            && bg == colors.hint_target_bg;
                        let fg = if on_hint_yellow { Color::Black } else { fg };
                        (
                            fg,
                            bg,
                            content_lines[line_idx].clone(),
                            note_scan_col,
                            blink,
                        )
                    }
                };

                // Recompute hint-yellow flag for use in the note-scan overlay below
                // (on_hint_yellow is computed inside the else branch above but not
                // accessible here after destructuring).
                let on_hint_yellow = !paused
                    && hint.and_then(|h| hint_role(row, col, h)).is_some()
                    && bg == colors.hint_target_bg;

                let sep_fg = if paused {
                    overlay_bg
                } else if let Some(c) = hint_right_border_color(row, col, hint, colors) {
                    c
                } else if col == 8 {
                    colors.grid_border
                } else if col == 2 || col == 5 {
                    colors.grid_box
                } else {
                    colors.grid_cell
                };
                let sep_bg = if paused {
                    overlay_bg
                } else if col == 8 {
                    colors.ui_background
                } else {
                    bg
                };
                // Each cell occupies 8 terminal columns (7 content + 1 separator).
                // Cell col `c` starts at col_off + 1 + c * 8 (after the outer OUTER_V).
                // Use an explicit MoveTo so cursor displacement from any overlay cannot
                // corrupt subsequent cells.
                let cell_term_col = col_off + 1 + col as u16 * 8;
                // Software blink: when blink is true and the error cell is in its
                // "off" phase, overwrite the cell with spaces (hide the digit).
                let (print_fg, print_content) = if blink && !anim.error_cell_visible() {
                    (bg, "       ".to_string())
                } else {
                    (fg, content.clone())
                };
                queue!(
                    out,
                    MoveTo(cell_term_col, term_row),
                    SetForegroundColor(print_fg),
                    SetBackgroundColor(bg),
                    Print(&print_content),
                    SetForegroundColor(sep_fg),
                    SetBackgroundColor(sep_bg),
                    Print(v_sep(col))
                )?;

                // Overlay scan-highlighted note digit in magenta (black on hint yellow).
                if let Some(char_off) = note_scan_col {
                    let note_term_col = cell_term_col + char_off as u16;
                    let sd = scan_digit.unwrap();
                    let scan_fg = if on_hint_yellow {
                        Color::Black
                    } else {
                        colors.digit_scan
                    };
                    queue!(
                        out,
                        MoveTo(note_term_col, term_row),
                        SetForegroundColor(scan_fg),
                        SetBackgroundColor(bg),
                        Print(char::from(b'0' + sd)),
                    )?;
                }
            }
        }

        // Separator row after this row (not after row 8)
        if row < 8 {
            let heavy = is_box_row(row);
            let fill = if paused {
                ' '
            } else if heavy {
                BOX_H
            } else {
                THIN_H
            };
            let border_fg = if paused {
                overlay_bg
            } else if heavy {
                colors.grid_box
            } else {
                colors.grid_cell
            };
            let row_bg = if paused {
                overlay_bg
            } else {
                colors.ui_background
            };
            let term_row = row_off + 1 + (row * 4 + 3) as u16;
            queue!(
                out,
                MoveTo(col_off, term_row),
                SetForegroundColor(if paused {
                    overlay_bg
                } else {
                    colors.grid_border
                }),
                SetBackgroundColor(row_bg),
                Print(if paused { ' ' } else { L_SEP })
            )?;
            for col in 0..9usize {
                let seg_fg = if paused {
                    border_fg
                } else {
                    hint_h_seg_color(row, col, hint, colors).unwrap_or(border_fg)
                };
                queue!(out, SetForegroundColor(seg_fg), SetBackgroundColor(row_bg))?;
                for _ in 0..7 {
                    queue!(out, Print(fill))?;
                }
                if col < 8 {
                    // Cross: use hint colour if either adjacent segment (this col or next col)
                    // is highlighted. Prefer the colour from this column.
                    let cross_fg = if paused {
                        border_fg
                    } else {
                        hint_h_seg_color(row, col, hint, colors)
                            .or_else(|| hint_h_seg_color(row, col + 1, hint, colors))
                            .unwrap_or(border_fg)
                    };
                    queue!(
                        out,
                        SetForegroundColor(cross_fg),
                        SetBackgroundColor(row_bg),
                        Print(if paused { ' ' } else { h_cross(heavy, col) })
                    )?;
                }
            }
            queue!(
                out,
                SetForegroundColor(if paused {
                    overlay_bg
                } else {
                    colors.grid_border
                }),
                SetBackgroundColor(row_bg),
                Print(if paused { ' ' } else { R_SEP })
            )?;
        }
    }

    // ── Bottom border ───────────────────────────────────────────────────────
    let bottom_row = row_off + 1 + (8 * 4 + 3) as u16; // = row_off + 36
    queue!(
        out,
        MoveTo(col_off, bottom_row),
        SetForegroundColor(colors.grid_border),
        SetBackgroundColor(colors.ui_background),
        Print(BL)
    )?;
    for col in 0..9usize {
        for _ in 0..7 {
            queue!(out, Print(OUTER_H))?;
        }
        queue!(out, Print(if col < 8 { BOT_SEP } else { BR }))?;
    }

    queue!(out, ResetColor)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{GameState, Grid};
    use crate::tui::anim::AnimState;
    use crate::tui::colors::ColorScheme;
    use crate::tui::digit_style::RetroStyle;
    use crate::tui::input::{NavMode, NavState};

    fn empty_state() -> GameState {
        GameState::new(
            Grid::from_str(
                "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
        )
    }

    fn nav_input() -> NavState {
        NavState {
            mode: NavMode::Input,
            box_idx: None,
        }
    }
    fn nav_grid() -> NavState {
        NavState {
            mode: NavMode::Navigation,
            box_idx: None,
        }
    }
    fn nav_box(b: usize) -> NavState {
        NavState {
            mode: NavMode::Navigation,
            box_idx: Some(b),
        }
    }

    #[test]
    fn grid_render_contains_outer_border_chars() {
        let state = empty_state();
        let mut buf = Vec::new();
        render_grid(
            &mut buf,
            (0, 0),
            &state,
            (0, 0),
            false,
            false,
            &nav_input(),
            &AnimState::default(),
            None,
            false,
            None,
            None,
            &ColorScheme::default(),
            &RetroStyle,
            &RetroStyle,
            false,
            None,
        )
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
        render_grid(
            &mut buf,
            (0, 0),
            &state,
            (0, 0),
            false,
            false,
            &nav_input(),
            &AnimState::default(),
            None,
            false,
            None,
            None,
            &ColorScheme::default(),
            &RetroStyle,
            &RetroStyle,
            false,
            None,
        )
        .unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains('┃'));
        assert!(s.contains('━'));
        assert!(s.contains('╋'));
    }

    #[test]
    fn grid_render_does_not_panic_with_filled_grid() {
        let grid = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        let state = GameState::new(grid);
        let mut buf = Vec::new();
        render_grid(
            &mut buf,
            (0, 0),
            &state,
            (4, 4),
            false,
            false,
            &nav_input(),
            &AnimState::default(),
            None,
            false,
            None,
            None,
            &ColorScheme::default(),
            &RetroStyle,
            &RetroStyle,
            false,
            None,
        )
        .unwrap();
        assert!(!buf.is_empty());
    }

    #[test]
    fn nav_grid_stage_all_cells_active_bg() {
        // Stage 1: NavMode::Navigation with no box — entire grid should use active_bg
        let _state = empty_state();
        let colors = ColorScheme::default();
        let anim = AnimState::default();
        assert_eq!(
            cell_bg(0, 0, (4, 4), None, &nav_grid(), None, &anim, &colors),
            colors.cell_active_bg
        );
        assert_eq!(
            cell_bg(8, 8, (4, 4), None, &nav_grid(), None, &anim, &colors),
            colors.cell_active_bg
        );
        assert_eq!(
            cell_bg(4, 4, (4, 4), None, &nav_grid(), None, &anim, &colors),
            colors.cell_active_bg
        );
    }

    #[test]
    fn nav_box_stage_only_selected_box_active() {
        let colors = ColorScheme::default();
        let anim = AnimState::default();

        // Numpad '5' (idx 4) = center box → reading-order box 4 → rows 3-5, cols 3-5
        let nav = nav_box(4);
        assert_eq!(
            cell_bg(3, 3, (0, 0), None, &nav, None, &anim, &colors),
            colors.cell_active_bg
        );
        assert_eq!(
            cell_bg(5, 5, (0, 0), None, &nav, None, &anim, &colors),
            colors.cell_active_bg
        );
        assert_eq!(
            cell_bg(0, 0, (0, 0), None, &nav, None, &anim, &colors),
            colors.cell_normal_bg
        );
        assert_eq!(
            cell_bg(8, 8, (0, 0), None, &nav, None, &anim, &colors),
            colors.cell_normal_bg
        );

        // Numpad '9' (idx 8) → reading-order box 2 → top-right → rows 0-2, cols 6-8
        let nav9 = nav_box(8);
        assert_eq!(
            cell_bg(0, 6, (0, 0), None, &nav9, None, &anim, &colors),
            colors.cell_active_bg
        );
        assert_eq!(
            cell_bg(2, 8, (0, 0), None, &nav9, None, &anim, &colors),
            colors.cell_active_bg
        );
        assert_eq!(
            cell_bg(6, 6, (0, 0), None, &nav9, None, &anim, &colors),
            colors.cell_normal_bg
        );

        // Numpad '1' (idx 0) → reading-order box 6 → bottom-left → rows 6-8, cols 0-2
        let nav1 = nav_box(0);
        assert_eq!(
            cell_bg(6, 0, (0, 0), None, &nav1, None, &anim, &colors),
            colors.cell_active_bg
        );
        assert_eq!(
            cell_bg(8, 2, (0, 0), None, &nav1, None, &anim, &colors),
            colors.cell_active_bg
        );
        assert_eq!(
            cell_bg(0, 0, (0, 0), None, &nav1, None, &anim, &colors),
            colors.cell_normal_bg
        );
    }

    #[test]
    fn nav_input_stage_only_cursor_active() {
        let colors = ColorScheme::default();
        let anim = AnimState::default();
        let nav = nav_input();
        assert_eq!(
            cell_bg(4, 4, (4, 4), None, &nav, None, &anim, &colors),
            colors.cell_active_bg
        );
        assert_eq!(
            cell_bg(0, 0, (4, 4), None, &nav, None, &anim, &colors),
            colors.cell_normal_bg
        );
    }

    #[test]
    fn cell_bg_hover_shows_hover_bg() {
        // Non-cursor hover cell should show the hover_bg color.
        use crate::tui::input::NavMode;
        let nav = NavState { mode: NavMode::Input, box_idx: None };
        let colors = ColorScheme::default();
        let anim = AnimState::default();
        // cursor at (0,0), hover at (1,1) → cell (1,1) gets hover_bg
        let bg = cell_bg(1, 1, (0, 0), Some((1, 1)), &nav, None, &anim, &colors);
        assert_eq!(bg, colors.hover_bg, "hovered non-cursor cell should use hover_bg");
    }

    #[test]
    fn cell_bg_cursor_beats_hover() {
        // When hover is on the same cell as cursor, cursor color wins.
        use crate::tui::input::NavMode;
        let nav = NavState { mode: NavMode::Input, box_idx: None };
        let colors = ColorScheme::default();
        let anim = AnimState::default();
        // cursor and hover both at (1,1)
        let bg = cell_bg(1, 1, (1, 1), Some((1, 1)), &nav, None, &anim, &colors);
        assert_eq!(bg, colors.cell_active_bg, "cursor should beat hover when on same cell");
    }

    #[test]
    fn grid_render_with_hint_does_not_panic() {
        use crate::hint::Hint;
        let state = empty_state();
        let hint = Hint {
            cause_cells: vec![(0, 0), (0, 1)],
            elim_cells: vec![(1, 0), (1, 1)],
            target_cell: (4, 4),
            elim_digit: Some(3),
            target_digit: Some(5),
            explanation_en: "test".into(),
            explanation_de: "test".into(),
            name_en: "Test",
            name_de: "Test",
        };
        let mut buf = Vec::new();
        render_grid(
            &mut buf,
            (0, 0),
            &state,
            (4, 4),
            false,
            false,
            &nav_input(),
            &AnimState::default(),
            None,
            false,
            None,
            Some(&hint),
            &ColorScheme::default(),
            &RetroStyle,
            &RetroStyle,
            false,
            None,
        )
        .unwrap();
        assert!(!buf.is_empty());
    }
}
