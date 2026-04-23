use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    Back,
    ConfirmYes,
    ConfirmNo,
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
        KeyCode::Esc   => AppAction::Back,

        KeyCode::Char(' ') => AppAction::Pause,
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
}
