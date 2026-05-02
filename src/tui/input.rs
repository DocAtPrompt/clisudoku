use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Which panel button was clicked via mouse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MousePanelButton {
    NotesSolToggle,
    Undo,
    Redo,
    Clear,
    Digit(u8),  // 1..=9
}

/// Current navigation sub-state (numpad 2-step selection).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NavState {
    pub mode: NavMode,
    /// Box index (0-8) selected in first numpad step, None if not yet selected.
    pub box_idx: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NavMode {
    Navigation,  // navigating, numpad 2-step selection; grid lights up
    #[default]
    Input,       // normal editing; digit keys write to the active cell
}

/// All actions the app can respond to, independent of key bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppAction {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    /// Numpad digit 1-9 pressed with no box selected yet → box index (0-indexed).
    NumpadBox(usize),
    /// Numpad digit 1-9 pressed with box already selected → within-box cell index (0-indexed).
    NumpadCell(usize),
    /// Digit 1-9 in input mode.
    Digit(u8),
    /// `0` key: toggle solution/note mode.
    ToggleMode,
    /// `-` key: request cell clear (app will show confirm dialog).
    ClearCell,
    Undo,
    Redo,
    /// Enter key — semantics depend on nav state.
    Enter,
    Pause,
    /// Boss Key — toggle disguise mode (hide game as terminal).
    BossKey,
    /// `s` key: toggle passive digit scan highlight.
    ToggleScan,
    /// `e` key: toggle error display (wrong digits shown in red).
    ToggleErrors,
    /// Player requests a hint.
    RequestHint,
    Back,
    ConfirmYes,
    ConfirmNo,
    /// `m`/`M` key: toggle mouse mode on/off.
    ToggleMouseMode,
    /// Mouse moved over grid cell (row, col).
    MouseHover(usize, usize),
    /// Mouse left-clicked on grid cell (row, col) → move cursor there.
    MouseSelectCell(usize, usize),
    /// Mouse left-clicked on a panel button.
    MouseButton(MousePanelButton),
    None,
}

/// Map a raw key event to an `AppAction`.
///
/// The mapping depends on `NavState` because numpad digits have different
/// meanings depending on whether a box has already been selected.
pub fn map_key_to_action(key: KeyEvent, nav: &NavState) -> AppAction {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

    match key.code {
        KeyCode::Up    => AppAction::MoveUp,
        KeyCode::Down  => AppAction::MoveDown,
        KeyCode::Left  => AppAction::MoveLeft,
        KeyCode::Right => AppAction::MoveRight,

        KeyCode::Enter => AppAction::Enter,
        KeyCode::Esc => AppAction::Back,

        KeyCode::Char(' ') => AppAction::Pause,
        KeyCode::Char('b') | KeyCode::Char('B') if !ctrl => AppAction::BossKey,
        KeyCode::Char('m') | KeyCode::Char('M') if !ctrl => AppAction::ToggleMouseMode,
        KeyCode::Char('s') | KeyCode::Char('S') if !ctrl => AppAction::ToggleScan,
        KeyCode::Char('e') | KeyCode::Char('E') if !ctrl => AppAction::ToggleErrors,
        KeyCode::Char('h') | KeyCode::Char('H') if !ctrl => AppAction::RequestHint,
        KeyCode::Char('-') => AppAction::ClearCell,
        KeyCode::Char('0') => AppAction::ToggleMode,

        KeyCode::Char('u') if !ctrl => AppAction::Undo,
        KeyCode::Char('z') if ctrl  => AppAction::Undo,
        KeyCode::Char('r') if !ctrl => AppAction::Redo,
        KeyCode::Char('y') if ctrl  => AppAction::Redo,

        KeyCode::Char('y') | KeyCode::Char('Y') if !ctrl => AppAction::ConfirmYes,
        KeyCode::Char('n') | KeyCode::Char('N') if !ctrl => AppAction::ConfirmNo,

        KeyCode::Char(c @ '1'..='9') => {
            let idx = (c as u8 - b'1') as usize;  // 0-indexed (1→0, …, 9→8)
            match nav.mode {
                NavMode::Input => AppAction::Digit(c as u8 - b'0'),
                NavMode::Navigation => {
                    if nav.box_idx.is_none() {
                        AppAction::NumpadBox(idx)
                    } else {
                        AppAction::NumpadCell(idx)
                    }
                }
            }
        }

        _ => AppAction::None,
    }
}

/// Map terminal cursor position to a grid cell (row, col), or None if the
/// position falls on a border or outside the 9×9 grid.
///
/// Grid cells start at terminal col 3 (col_off+1) and terminal row 2 (row_off+1).
/// Each cell occupies 8 terminal columns (7 content + 1 separator) and
/// 4 terminal rows (3 content + 1 separator).
///
/// Remainder 7 on the column axis = vertical separator → None.
/// Remainder 3 on the row axis    = horizontal separator → None.
pub fn hit_test_grid(term_col: u16, term_row: u16) -> Option<(usize, usize)> {
    if term_col < 3 || term_row < 2 {
        return None;
    }
    let dc = (term_col - 3) as usize;
    let dr = (term_row - 2) as usize;
    if dc % 8 == 7 || dr % 4 == 3 {
        return None;  // separator column or row
    }
    let grid_col = dc / 8;
    let grid_row = dr / 4;
    if grid_col >= 9 || grid_row >= 9 {
        return None;
    }
    Some((grid_row, grid_col))
}

/// Map terminal cursor position to a panel button, or None.
///
/// Panel origin: col_off=77, row_off=1. Drawable content area: cols 79–112.
/// Divider at terminal row 19. Mouse controls below:
///   Row 23: action buttons  (N/Sol | Undo | Redo | Clr)
///   Row 27: digits 1/2/3
///   Row 29: digits 4/5/6
///   Row 31: digits 7/8/9
///   All other rows → None.
///
/// Action button column ranges (border separator attributed to button on its left):
///   N/Sol: 79–88, Undo: 89–96, Redo: 97–104, Clr: 105–112
///
/// Digit column ranges:
///   Col 0 (1/4/7): 79–90, Col 1 (2/5/8): 91–101, Col 2 (3/6/9): 102–112
pub fn hit_test_panel_button(term_col: u16, term_row: u16) -> Option<MousePanelButton> {
    if !(79..=112).contains(&term_col) {
        return None;
    }
    let col = term_col as usize;

    match term_row {
        23 => match col {
            79..=88  => Some(MousePanelButton::NotesSolToggle),
            89..=96  => Some(MousePanelButton::Undo),
            97..=104 => Some(MousePanelButton::Redo),
            105..=112 => Some(MousePanelButton::Clear),
            _ => None,
        },
        27 | 29 | 31 => {
            let digit_col: u8 = match col {
                79..=90  => 0,
                91..=101 => 1,
                _        => 2,  // 102..=112, guarded by outer range check
            };
            let digit_row: u8 = match term_row {
                27 => 0,
                29 => 1,
                _  => 2,        // 31, guarded by outer pattern
            };
            Some(MousePanelButton::Digit(digit_row * 3 + digit_col + 1))
        }
        _ => None,
    }
}

/// Translate a raw crossterm `MouseEvent` to a semantic `AppAction`.
/// Returns `AppAction::None` when mouse mode is off or the event is irrelevant.
pub fn map_mouse_to_action(
    event: crossterm::event::MouseEvent,
    mouse_mode: bool,
) -> AppAction {
    if !mouse_mode {
        return AppAction::None;
    }
    use crossterm::event::{MouseButton, MouseEventKind};
    match event.kind {
        MouseEventKind::Moved | MouseEventKind::Drag(_) => {
            if let Some((r, c)) = hit_test_grid(event.column, event.row) {
                AppAction::MouseHover(r, c)
            } else {
                AppAction::None
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some((r, c)) = hit_test_grid(event.column, event.row) {
                AppAction::MouseSelectCell(r, c)
            } else if let Some(btn) = hit_test_panel_button(event.column, event.row) {
                AppAction::MouseButton(btn)
            } else {
                AppAction::None
            }
        }
        _ => AppAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
    fn ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[test]
    fn arrow_keys_produce_move_actions() {
        assert_eq!(map_key_to_action(key(KeyCode::Up),    &NavState::default()), AppAction::MoveUp);
        assert_eq!(map_key_to_action(key(KeyCode::Down),  &NavState::default()), AppAction::MoveDown);
        assert_eq!(map_key_to_action(key(KeyCode::Left),  &NavState::default()), AppAction::MoveLeft);
        assert_eq!(map_key_to_action(key(KeyCode::Right), &NavState::default()), AppAction::MoveRight);
    }

    #[test]
    fn digit_keys_produce_digit_action() {
        // Digits produce AppAction::Digit only when in Input mode
        let nav = NavState { mode: NavMode::Input, box_idx: None };
        for d in 1u8..=9 {
            let code = KeyCode::Char(char::from(b'0' + d));
            assert_eq!(map_key_to_action(key(code), &nav), AppAction::Digit(d));
        }
    }

    #[test]
    fn zero_toggles_mode() {
        assert_eq!(
            map_key_to_action(key(KeyCode::Char('0')), &NavState::default()),
            AppAction::ToggleMode
        );
    }

    #[test]
    fn minus_clears_cell() {
        assert_eq!(
            map_key_to_action(key(KeyCode::Char('-')), &NavState::default()),
            AppAction::ClearCell
        );
    }

    #[test]
    fn undo_redo_keys() {
        assert_eq!(map_key_to_action(key(KeyCode::Char('u')), &NavState::default()), AppAction::Undo);
        assert_eq!(map_key_to_action(ctrl(KeyCode::Char('z')), &NavState::default()), AppAction::Undo);
        assert_eq!(map_key_to_action(key(KeyCode::Char('r')), &NavState::default()), AppAction::Redo);
        assert_eq!(map_key_to_action(ctrl(KeyCode::Char('y')), &NavState::default()), AppAction::Redo);
    }

    #[test]
    fn escape_goes_back() {
        assert_eq!(
            map_key_to_action(key(KeyCode::Esc), &NavState::default()),
            AppAction::Back
        );
    }

    #[test]
    fn space_pauses() {
        assert_eq!(
            map_key_to_action(key(KeyCode::Char(' ')), &NavState::default()),
            AppAction::Pause
        );
    }

    #[test]
    fn numpad_in_nav_mode_selects_box() {
        let nav = NavState { mode: NavMode::Navigation, box_idx: None };
        assert_eq!(
            map_key_to_action(KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE), &nav),
            AppAction::NumpadBox(4)
        );
    }

    #[test]
    fn numpad_with_box_selected_picks_cell() {
        let nav = NavState { mode: NavMode::Navigation, box_idx: Some(4) };
        assert_eq!(
            map_key_to_action(KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE), &nav),
            AppAction::NumpadCell(4)
        );
    }

    #[test]
    fn h_key_maps_to_request_hint() {
        let nav = NavState::default();
        assert_eq!(map_key_to_action(key(KeyCode::Char('h')), &nav), AppAction::RequestHint);
        assert_eq!(map_key_to_action(key(KeyCode::Char('H')), &nav), AppAction::RequestHint);
    }

    // ── hit_test_grid ─────────────────────────────────────────────────────────

    #[test]
    fn hit_test_grid_first_cell() {
        // Top-left cell (0,0): starts at col 3, row 2
        assert_eq!(hit_test_grid(3, 2), Some((0, 0)));
        assert_eq!(hit_test_grid(9, 2), Some((0, 0)));   // last col of cell (0,0)
        assert_eq!(hit_test_grid(3, 4), Some((0, 0)));   // last row of cell (0,0)
    }

    #[test]
    fn hit_test_grid_second_cell_col() {
        // col 11 → dc=8 → cell col 1
        assert_eq!(hit_test_grid(11, 2), Some((0, 1)));
    }

    #[test]
    fn hit_test_grid_second_cell_row() {
        // row 6 → dr=4 → cell row 1
        assert_eq!(hit_test_grid(3, 6), Some((1, 0)));
    }

    #[test]
    fn hit_test_grid_vertical_border_returns_none() {
        // dc = 10-3 = 7 → remainder 7 = vertical border
        assert_eq!(hit_test_grid(10, 2), None);
    }

    #[test]
    fn hit_test_grid_horizontal_border_returns_none() {
        // dr = 5-2 = 3 → remainder 3 = horizontal border
        assert_eq!(hit_test_grid(3, 5), None);
    }

    #[test]
    fn hit_test_grid_out_of_range_returns_none() {
        assert_eq!(hit_test_grid(2, 2), None);   // col < 3
        assert_eq!(hit_test_grid(3, 1), None);   // row < 2
        assert_eq!(hit_test_grid(75, 2), None);  // col result >= 9
    }

    #[test]
    fn hit_test_grid_last_cell() {
        // Cell (8,8): col = 3 + 8*8 = 67; row = 2 + 8*4 = 34
        assert_eq!(hit_test_grid(67, 34), Some((8, 8)));
        assert_eq!(hit_test_grid(73, 34), Some((8, 8)));  // last col of (8,8)
    }

    // ── hit_test_panel_button ─────────────────────────────────────────────────

    #[test]
    fn hit_test_panel_button_action_buttons() {
        assert_eq!(hit_test_panel_button(79, 23), Some(MousePanelButton::NotesSolToggle));
        assert_eq!(hit_test_panel_button(88, 23), Some(MousePanelButton::NotesSolToggle));
        assert_eq!(hit_test_panel_button(89, 23), Some(MousePanelButton::Undo));
        assert_eq!(hit_test_panel_button(96, 23), Some(MousePanelButton::Undo));
        assert_eq!(hit_test_panel_button(97, 23), Some(MousePanelButton::Redo));
        assert_eq!(hit_test_panel_button(104, 23), Some(MousePanelButton::Redo));
        assert_eq!(hit_test_panel_button(105, 23), Some(MousePanelButton::Clear));
        assert_eq!(hit_test_panel_button(112, 23), Some(MousePanelButton::Clear));
    }

    #[test]
    fn hit_test_panel_button_digit_grid_row1() {
        assert_eq!(hit_test_panel_button(79, 27),  Some(MousePanelButton::Digit(1)));
        assert_eq!(hit_test_panel_button(90, 27),  Some(MousePanelButton::Digit(1)));
        assert_eq!(hit_test_panel_button(91, 27),  Some(MousePanelButton::Digit(2)));
        assert_eq!(hit_test_panel_button(101, 27), Some(MousePanelButton::Digit(2)));
        assert_eq!(hit_test_panel_button(102, 27), Some(MousePanelButton::Digit(3)));
        assert_eq!(hit_test_panel_button(112, 27), Some(MousePanelButton::Digit(3)));
    }

    #[test]
    fn hit_test_panel_button_digit_grid_rows2_and_3() {
        assert_eq!(hit_test_panel_button(79, 29),  Some(MousePanelButton::Digit(4)));
        assert_eq!(hit_test_panel_button(91, 29),  Some(MousePanelButton::Digit(5)));
        assert_eq!(hit_test_panel_button(102, 29), Some(MousePanelButton::Digit(6)));
        assert_eq!(hit_test_panel_button(79, 31),  Some(MousePanelButton::Digit(7)));
        assert_eq!(hit_test_panel_button(91, 31),  Some(MousePanelButton::Digit(8)));
        assert_eq!(hit_test_panel_button(102, 31), Some(MousePanelButton::Digit(9)));
    }

    #[test]
    fn hit_test_panel_button_border_rows_return_none() {
        assert_eq!(hit_test_panel_button(79, 22), None);  // action button top border
        assert_eq!(hit_test_panel_button(79, 24), None);  // action button bottom border
        assert_eq!(hit_test_panel_button(79, 26), None);  // digit grid top border
        assert_eq!(hit_test_panel_button(79, 28), None);  // digit grid mid border
        assert_eq!(hit_test_panel_button(79, 30), None);  // digit grid mid border
        assert_eq!(hit_test_panel_button(79, 32), None);  // digit grid bottom border
    }

    #[test]
    fn hit_test_panel_button_out_of_range_returns_none() {
        assert_eq!(hit_test_panel_button(78, 23), None);   // col < 79
        assert_eq!(hit_test_panel_button(113, 23), None);  // col > 112
        assert_eq!(hit_test_panel_button(79, 20), None);   // row label, not clickable
        assert_eq!(hit_test_panel_button(79, 25), None);   // blank row
    }

    #[test]
    fn m_key_maps_to_toggle_mouse_mode() {
        let nav = NavState::default();
        assert_eq!(
            map_key_to_action(key(KeyCode::Char('m')), &nav),
            AppAction::ToggleMouseMode
        );
        assert_eq!(
            map_key_to_action(key(KeyCode::Char('M')), &nav),
            AppAction::ToggleMouseMode
        );
    }

    // ── map_mouse_to_action ───────────────────────────────────────────────────

    fn mouse_event(kind: crossterm::event::MouseEventKind, col: u16, row: u16) -> crossterm::event::MouseEvent {
        crossterm::event::MouseEvent { kind, column: col, row, modifiers: crossterm::event::KeyModifiers::NONE }
    }

    #[test]
    fn map_mouse_disabled_returns_none() {
        use crossterm::event::{MouseButton, MouseEventKind};
        let e = mouse_event(MouseEventKind::Down(MouseButton::Left), 10, 10);
        assert_eq!(map_mouse_to_action(e, false), AppAction::None);
    }

    #[test]
    fn map_mouse_hover_on_grid() {
        use crossterm::event::MouseEventKind;
        let e = mouse_event(MouseEventKind::Moved, 3, 2);
        assert_eq!(map_mouse_to_action(e, true), AppAction::MouseHover(0, 0));
    }

    #[test]
    fn map_mouse_hover_off_grid_returns_none() {
        use crossterm::event::MouseEventKind;
        let e = mouse_event(MouseEventKind::Moved, 1, 1);
        assert_eq!(map_mouse_to_action(e, true), AppAction::None);
    }

    #[test]
    fn map_mouse_click_selects_cell() {
        use crossterm::event::{MouseButton, MouseEventKind};
        let e = mouse_event(MouseEventKind::Down(MouseButton::Left), 3, 2);
        assert_eq!(map_mouse_to_action(e, true), AppAction::MouseSelectCell(0, 0));
    }

    #[test]
    fn map_mouse_click_panel_button() {
        use crossterm::event::{MouseButton, MouseEventKind};
        let e = mouse_event(MouseEventKind::Down(MouseButton::Left), 79, 23);
        assert_eq!(map_mouse_to_action(e, true), AppAction::MouseButton(MousePanelButton::NotesSolToggle));
    }
}
