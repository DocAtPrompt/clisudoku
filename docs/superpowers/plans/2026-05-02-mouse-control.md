# Mouse Control Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add optional mouse input to the Sudoku TUI, toggled with `M`/`m`, providing cell hover highlighting, click-to-select, and a clickable button panel for digits and actions.

**Architecture:** `crossterm` mouse events are translated in `input.rs` into semantic `AppAction` variants (hover/select/button) using two pure hit-test functions. The `App` struct gains `mouse_mode: bool` and `hover_cell: Option<(usize, usize)>` fields; the grid renderer shows a `DarkYellow` hover highlight; the panel renderer swaps in a mouse controls section when mouse mode is active.

**Tech Stack:** Rust, crossterm (mouse capture: `EnableMouseCapture`/`DisableMouseCapture`, `Event::Mouse`), existing TUI render pipeline.

**Spec:** `docs/superpowers/specs/2026-05-02-mouse-control-design.md`

---

## File Map

| File | Change |
|---|---|
| `src/i18n/mod.rs` | Add `ctrl_mouse` field to `Strings`; set value in all 13 language constants |
| `src/tui/terminal.rs` | Add `enable_mouse_capture()` / `disable_mouse_capture()` helper fns |
| `src/tui/input.rs` | Add `MousePanelButton` enum; add `AppAction::ToggleMouseMode/MouseHover/MouseSelectCell/MouseButton`; add `M`/`m` key mapping; add `hit_test_grid`, `hit_test_panel_button`, `map_mouse_to_action`; unit tests |
| `src/tui/render/mod.rs` | Add `mouse_mode: bool`, `hover_cell: Option<(usize, usize)>` to `Screen::Game`; pass through to `render_grid` and `render_panel` |
| `src/tui/render/grid.rs` | Add `hover_cell` param to `render_grid` and `cell_bg`; apply `Color::DarkYellow` hover highlight |
| `src/tui/render/status_bar.rs` | Add `mouse_mode` param to `render_panel`; add `render_mouse_controls()`; replace controls section when mouse mode active |
| `src/tui/mod.rs` | Add `mouse_mode`/`hover_cell` to `App`; handle `ToggleMouseMode`, mouse actions; `enter_game` reset; `Event::Mouse` in run loop; poll-timeout reduction; pass fields to renderer |

---

## Task 1: i18n — add `ctrl_mouse` string

**Files:**
- Modify: `src/i18n/mod.rs`

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)]` block at the bottom of `src/i18n/mod.rs`:

```rust
#[test]
fn ctrl_mouse_is_present_in_all_languages() {
    let langs: &[&Strings] = &[&EN, &DE, &ES, &IT, &FR, &SL, &EO, &TP, &LEET, &SW, &AF, &PY, &ID];
    for s in langs {
        assert!(!s.ctrl_mouse.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/alexandererben/Tresors/OrdiSync/6_Entwicklung/Claude/SudokuCLI
cargo test ctrl_mouse_is_present 2>&1 | tail -10
```

Expected: compile error — `ctrl_mouse` does not exist on `Strings`.

- [ ] **Step 3: Add `ctrl_mouse` to the `Strings` struct**

In `src/i18n/mod.rs`, after the existing `ctrl_hint` field (around line 65), add:

```rust
    /// Mouse mode toggle control label (≤ 34 chars).
    pub ctrl_mouse: &'static str,
```

- [ ] **Step 4: Add the value to all 13 language constants**

Each language constant is a `static Strings = Strings { ... }` block. After the `ctrl_hint` line in each block, add `ctrl_mouse`. Use the values below.

**English** (`EN`): `ctrl_mouse: "  m      mouse on/off",`  
**German** (`DE`): `ctrl_mouse: "  m      Maus an/aus",`  
**Spanish** (`ES`): `ctrl_mouse: "  m      ratón on/off",`  
**Italian** (`IT`): `ctrl_mouse: "  m      mouse on/off",`  
**French** (`FR`): `ctrl_mouse: "  m      souris on/off",`  
**Slovenian** (`SL`): `ctrl_mouse: "  m      miška on/off",`  
**Esperanto** (`EO`): `ctrl_mouse: "  m      muso on/off",`  
**Toki Pona** (`TP`): `ctrl_mouse: "  m      luka on/off",`  
**Leet** (`LEET`): `ctrl_mouse: "  m      m0u53 on/0ff",`  
**Swahili** (`SW`): `ctrl_mouse: "  m      panya on/off",`  
**Afrikaans** (`AF`): `ctrl_mouse: "  m      muis aan/af",`  
**Pinyin** (`PY`): `ctrl_mouse: "  m      shubiao on/off",`  
**Indonesian** (`ID`): `ctrl_mouse: "  m      mouse on/off",`  

- [ ] **Step 5: Run test to verify it passes**

```bash
cargo test ctrl_mouse_is_present 2>&1 | tail -5
```

Expected: `test i18n::tests::ctrl_mouse_is_present_in_all_languages ... ok`

- [ ] **Step 6: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```

Expected: all tests pass (≥219 passing, 0 failed).

- [ ] **Step 7: Commit**

```bash
git add src/i18n/mod.rs
git commit -m "feat(i18n): add ctrl_mouse string to all 13 languages"
```

---

## Task 2: terminal.rs — mouse capture helpers

**Files:**
- Modify: `src/tui/terminal.rs`

- [ ] **Step 1: Add the two helper functions**

In `src/tui/terminal.rs`, after the `impl Drop for Terminal` block, add:

```rust
/// Send the ANSI escape to enable mouse capture in the active terminal.
/// Best-effort — ignores errors so callers can use `let _ = enable_mouse_capture()`.
pub fn enable_mouse_capture() -> io::Result<()> {
    execute!(io::stdout(), crossterm::event::EnableMouseCapture)
}

/// Send the ANSI escape to disable mouse capture in the active terminal.
pub fn disable_mouse_capture() -> io::Result<()> {
    execute!(io::stdout(), crossterm::event::DisableMouseCapture)
}
```

Also add `use std::io;` at the top of the file (it's already present via the existing `use std::io::{self, Stdout};`).

- [ ] **Step 2: Verify it compiles**

```bash
cargo build 2>&1 | grep -E "^error" | head -5
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src/tui/terminal.rs
git commit -m "feat(terminal): add enable/disable_mouse_capture helpers"
```

---

## Task 3: input.rs — new action types, key mapping, hit-test functions, tests

**Files:**
- Modify: `src/tui/input.rs`

- [ ] **Step 1: Write the failing tests**

Add at the end of the `#[cfg(test)]` block in `src/tui/input.rs`:

```rust
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
        // col 11 → (10-3)=8 → cell col 1
        assert_eq!(hit_test_grid(11, 2), Some((0, 1)));
    }

    #[test]
    fn hit_test_grid_second_cell_row() {
        // row 6 → (6-2)=4 → cell row 1
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
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test hit_test 2>&1 | head -20
cargo test m_key_maps_to_toggle 2>&1 | head -10
```

Expected: compile errors — types and functions don't exist yet.

- [ ] **Step 3: Add `MousePanelButton` and new `AppAction` variants**

In `src/tui/input.rs`, after the `use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};` line, add:

```rust
/// Which panel button was clicked via mouse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MousePanelButton {
    NotesSolToggle,
    Undo,
    Redo,
    Clear,
    Digit(u8),  // 1..=9
}
```

In the `AppAction` enum, add these variants after `ConfirmNo`:

```rust
    /// `m`/`M` key: toggle mouse mode on/off.
    ToggleMouseMode,
    /// Mouse moved over grid cell (row, col).
    MouseHover(usize, usize),
    /// Mouse left-clicked on grid cell (row, col) → move cursor there.
    MouseSelectCell(usize, usize),
    /// Mouse left-clicked on a panel button.
    MouseButton(MousePanelButton),
```

- [ ] **Step 4: Add `M`/`m` key mapping**

In `map_key_to_action`, after the `'b'/'B'` BossKey line, add:

```rust
        KeyCode::Char('m') | KeyCode::Char('M') if !ctrl => AppAction::ToggleMouseMode,
```

- [ ] **Step 5: Add `hit_test_grid`**

After the `map_key_to_action` function, add:

```rust
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
```

- [ ] **Step 6: Add `hit_test_panel_button`**

After `hit_test_grid`, add:

```rust
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
    if term_col < 79 || term_col > 112 {
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
                79..=90   => 0,
                91..=101  => 1,
                102..=112 => 2,
                _ => return None,
            };
            let digit_row: u8 = match term_row {
                27 => 0,
                29 => 1,
                31 => 2,
                _  => return None,
            };
            Some(MousePanelButton::Digit(digit_row * 3 + digit_col + 1))
        }
        _ => None,
    }
}
```

- [ ] **Step 7: Add `map_mouse_to_action`**

After `hit_test_panel_button`, add:

```rust
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
```

- [ ] **Step 8: Run all new tests**

```bash
cargo test hit_test 2>&1 | tail -20
cargo test m_key_maps_to_toggle 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 9: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```

Expected: all tests pass (≥ previous count + ~20 new tests).

- [ ] **Step 10: Commit**

```bash
git add src/tui/input.rs
git commit -m "feat(input): add mouse action types, hit-test functions, M/m key mapping"
```

---

## Task 4: render scaffold — extend Screen::Game and render signatures

**Files:**
- Modify: `src/tui/render/mod.rs`
- Modify: `src/tui/render/grid.rs`
- Modify: `src/tui/render/status_bar.rs`

This task wires up the new parameters through the render pipeline with no-op placeholder implementations. Subsequent tasks fill in the actual rendering logic.

- [ ] **Step 1: Add fields to `Screen::Game` in `render/mod.rs` (enum variant + match pattern)**

**1a. Enum variant:** In `src/tui/render/mod.rs`, in the `Screen::Game` variant (around line 32), add two new fields after `matrix_mode`:

```rust
        /// Whether mouse input mode is active.
        mouse_mode: bool,
        /// Grid cell currently under the mouse cursor; `None` when mouse mode
        /// is off, game is paused, or no hover event received yet.
        hover_cell: Option<(usize, usize)>,
```

**1b. Match pattern:** In `render_frame`, the `Screen::Game { ... }` destructuring pattern (around line 109) names every field explicitly. Add `mouse_mode` and `hover_cell` to it:

```rust
Screen::Game { state, cursor, note_mode, scan_mode, error_mode, solution, errors_shown, elapsed_ms, paused, nav, anim, scan_digit, hint, hint_warning, hint_count, matrix_mode, mouse_mode, hover_cell } =>
```

- [ ] **Step 2: Update `render_grid` call in `render_frame`**

In `render_frame`, in the `Screen::Game` match arm (around line 110), the current call is:

```rust
grid::render_grid(out, (1, 2), state, *cursor, *note_mode, *paused, nav, anim, *scan_digit, *error_mode, *solution, *hint, colors, style, *matrix_mode)?;
```

Replace with:

```rust
// Suppress hover highlight while paused.
let effective_hover = if *paused { None } else { *hover_cell };
grid::render_grid(out, (1, 2), state, *cursor, *note_mode, *paused, nav, anim, *scan_digit, *error_mode, *solution, *hint, colors, style, *matrix_mode, effective_hover)?;
```

- [ ] **Step 3: Update `render_panel` call in `render_frame`**

In `render_frame`, the `render_panel` call (around line 135) currently ends with `hint_text)?;`. Add `*mouse_mode` at the end:

```rust
status_bar::render_panel(out, (1, 77), *elapsed_ms, *note_mode, *scan_mode, *error_mode, *errors_shown, filled_count, digit_counts, *scan_digit, colors, strings, *hint_count, hint_text, *mouse_mode)?;
```

- [ ] **Step 4: Update `render_grid` signature in `grid.rs`**

In `src/tui/render/grid.rs`, find the `pub fn render_grid` signature and add `hover_cell: Option<(usize, usize)>` as the last parameter:

```rust
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
    style: &dyn DigitStyle,
    matrix_mode: bool,
    hover_cell: Option<(usize, usize)>,  // new — used by cell_bg for DarkYellow highlight
) -> io::Result<()> {
```

Inside the function body, add `let _ = hover_cell;` temporarily (will be removed in Task 6):
Place it just after the `let _ = note_mode;` line that already exists.

Also update the **four `render_grid` call-sites in `grid.rs` tests** (in the `#[cfg(test)]` block). Each call currently passes 16 positional arguments; append `None` as the 17th (`hover_cell`):

```rust
// Before:
render_grid(&mut buf, (0, 0), &state, (0, 0), false, false, &nav_input(), &AnimState::default(), None, false, None, None, &ColorScheme::default(), &RetroStyle, false)
// After:
render_grid(&mut buf, (0, 0), &state, (0, 0), false, false, &nav_input(), &AnimState::default(), None, false, None, None, &ColorScheme::default(), &RetroStyle, false, None)
```

Apply this change to all call-sites in the test block that match this pattern (search for `render_grid(&mut buf` in grid.rs tests).

- [ ] **Step 5: Update `render_panel` signature in `status_bar.rs`**

In `src/tui/render/status_bar.rs`, find `pub fn render_panel` and add `mouse_mode: bool` as the last parameter:

```rust
pub fn render_panel(
    out:          &mut impl Write,
    (row_off, col_off): (u16, u16),
    elapsed_ms:   u64,
    note_mode:    bool,
    scan_mode:    bool,
    error_mode:   bool,
    errors_shown: u32,
    filled_count: u8,
    digit_counts: [u8; 10],
    scan_digit:   Option<u8>,
    colors:       &ColorScheme,
    strings:      &'static Strings,
    hint_count:   u32,
    hint_text:    Option<(&str, &str)>,
    mouse_mode:   bool,  // new — when true, show mouse controls instead of key list
) -> io::Result<()> {
```

Inside the function body, add `let _ = mouse_mode;` temporarily at the top of the function (will be removed in Task 7).

Also update **all `render_panel` call-sites in `status_bar.rs` tests** to append `false` as the new `mouse_mode` argument. Search the `#[cfg(test)]` block of `status_bar.rs` for every call to `render_panel(` (including inside any `call_render_panel` helper) and add `false` as the last argument:

```rust
// Before (last two args):
render_panel(..., 0, None).unwrap();
// After:
render_panel(..., 0, None, false).unwrap();
```

- [ ] **Step 6: Temporarily patch `tui/mod.rs` so the whole project compiles**

`tui/mod.rs` constructs `Screen::Game` in `render_current()` and will fail to compile with "missing fields `mouse_mode`, `hover_cell`" until Task 5 wires them up. Add placeholder values now so this task can be committed cleanly. In `render_current()`, inside the `Screen::Game { ... }` struct literal, add after `matrix_mode`:

```rust
                        matrix_mode: self.matrix_mode,
                        mouse_mode: false,    // placeholder — replaced in Task 5
                        hover_cell: None,     // placeholder — replaced in Task 5
```

- [ ] **Step 7: Verify it compiles**

```bash
cargo build 2>&1 | grep -E "^error" | head -10
```

Expected: no errors.

- [ ] **Step 8: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 9: Commit**

```bash
git add src/tui/render/mod.rs src/tui/render/grid.rs src/tui/render/status_bar.rs src/tui/mod.rs
git commit -m "feat(render): extend render pipeline for mouse_mode and hover_cell"
```

---

## Task 5: App state — fields, toggle, action handlers, run loop

**Files:**
- Modify: `src/tui/mod.rs`

- [ ] **Step 1: Write tests**

In the `#[cfg(test)]` block of `src/tui/mod.rs`, add:

```rust
    #[test]
    fn m_key_toggles_mouse_mode() {
        let mut app = make_app();
        // Navigate to game
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        assert!(matches!(app.screen, AppScreen::Game));
        assert!(!app.mouse_mode);
        // Toggle on
        app.handle_action(AppAction::ToggleMouseMode);
        assert!(app.mouse_mode);
        assert!(app.hover_cell.is_none());
        // Toggle off
        app.handle_action(AppAction::ToggleMouseMode);
        assert!(!app.mouse_mode);
        assert!(app.hover_cell.is_none());
    }

    #[test]
    fn mouse_hover_updates_hover_cell() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::ToggleMouseMode);
        app.handle_action(AppAction::MouseHover(3, 5));
        assert_eq!(app.hover_cell, Some((3, 5)));
        // Hover elsewhere
        app.handle_action(AppAction::MouseHover(0, 0));
        assert_eq!(app.hover_cell, Some((0, 0)));
    }

    #[test]
    fn mouse_select_moves_cursor() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::ToggleMouseMode);
        app.handle_action(AppAction::MouseSelectCell(4, 7));
        assert_eq!(app.cursor, (4, 7));
    }

    #[test]
    fn enter_game_resets_mouse_mode() {
        use crate::timer::FakeClock;
        let mut app = App::new(Box::new(FakeClock { ms: 1000 }));
        // Manually force mouse_mode true without IO (simulates prior activation)
        app.mouse_mode = true;
        app.hover_cell = Some((2, 3));
        // Start a game — should reset mouse state
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        assert!(!app.mouse_mode);
        assert!(app.hover_cell.is_none());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test m_key_toggles_mouse 2>&1 | head -15
```

Expected: compile errors — `mouse_mode` and `hover_cell` fields don't exist on `App` yet.

- [ ] **Step 3: Add fields to `App` struct**

In `src/tui/mod.rs`, in the `App` struct definition, after `drain_input: bool`, add:

```rust
    /// Whether mouse capture mode is active.
    pub mouse_mode: bool,
    /// Grid cell currently under the mouse cursor; `None` when not hovering a cell.
    pub hover_cell: Option<(usize, usize)>,
```

- [ ] **Step 4: Initialize fields in `App::new()`**

In `App::new()`, after `drain_input: false`, add:

```rust
            mouse_mode: false,
            hover_cell: None,
```

- [ ] **Step 5: Reset fields in `enter_game()`**

In `enter_game()`, after `self.boss_mode = false;`, add:

```rust
        // Always disable mouse capture when starting a new game.
        if self.mouse_mode {
            let _ = crate::tui::terminal::disable_mouse_capture();
        }
        self.mouse_mode = false;
        self.hover_cell = None;
```

- [ ] **Step 6: Handle new actions in `handle_game_action()`**

In `handle_game_action()`, in the main `match action {` block, add these arms before the final `_ => {}`:

```rust
            AppAction::ToggleMouseMode => {
                self.mouse_mode = !self.mouse_mode;
                self.hover_cell = None;
                let _ = if self.mouse_mode {
                    crate::tui::terminal::enable_mouse_capture()
                } else {
                    crate::tui::terminal::disable_mouse_capture()
                };
            }
            AppAction::MouseHover(r, c) => {
                self.hover_cell = Some((r, c));
            }
            AppAction::MouseSelectCell(r, c) => {
                self.cursor = (r, c);
                self.nav_state.mode = crate::tui::input::NavMode::Input;
                self.nav_state.box_idx = None;
            }
            AppAction::MouseButton(btn) => {
                use crate::tui::input::MousePanelButton;
                let action = match btn {
                    MousePanelButton::NotesSolToggle => AppAction::ToggleMode,
                    MousePanelButton::Undo           => AppAction::Undo,
                    MousePanelButton::Redo           => AppAction::Redo,
                    MousePanelButton::Clear          => AppAction::ClearCell,
                    MousePanelButton::Digit(d)       => AppAction::Digit(d),
                };
                self.handle_game_action(action);
            }
```

- [ ] **Step 7: Add `Event::Mouse` handling in the `run()` loop**

In `run()`, inside the `if event::poll(...)?` block, in the `match event::read()?` statement, add a new arm before the final `_ => {}`:

```rust
                    Event::Mouse(mouse_event)
                        if matches!(self.screen, AppScreen::Game) && self.mouse_mode =>
                    {
                        use crate::tui::input::map_mouse_to_action;
                        let action = map_mouse_to_action(mouse_event, true);
                        match action {
                            AppAction::MouseHover(r, c) => {
                                // Pure hover: update position, no hint/warning dismissal.
                                self.hover_cell = Some((r, c));
                            }
                            AppAction::MouseSelectCell(_) | AppAction::MouseButton(_) => {
                                // Clicks behave like key presses for hint/overlay dismissal.
                                if self.active_hint.is_some() {
                                    self.active_hint = None;
                                    self.anim.hint_blink = false;
                                    self.needs_clear = true;
                                } else if self.hint_warning.is_some() {
                                    self.hint_warning = None;
                                    self.needs_clear = true;
                                } else if self.info_overlay.is_some() {
                                    self.info_overlay = None;
                                    self.needs_clear = true;
                                } else {
                                    self.handle_action(action);
                                }
                            }
                            _ => {}
                        }
                    }
```

- [ ] **Step 8: Reduce poll timeout in mouse mode**

In `run()`, in the `poll_ms` calculation block, add `|| self.mouse_mode` to the `80` case:

```rust
            } else if self.anim.is_active() || self.mouse_mode {
                80
```

- [ ] **Step 9: Replace placeholder values in `render_current()` with real fields**

In `render_current()` in `tui/mod.rs`, find the two placeholder lines added in Task 4 Step 6 and replace them with the real field values:

```rust
// Replace these two placeholder lines:
                        mouse_mode: false,    // placeholder — replaced in Task 5
                        hover_cell: None,     // placeholder — replaced in Task 5
// With:
                        mouse_mode: self.mouse_mode,
                        hover_cell: self.hover_cell,
```

- [ ] **Step 10: Run tests**

```bash
cargo test m_key_toggles_mouse 2>&1 | tail -5
cargo test mouse_hover 2>&1 | tail -5
cargo test mouse_select 2>&1 | tail -5
cargo test enter_game_resets_mouse 2>&1 | tail -5
```

Expected: all pass.

- [ ] **Step 11: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 12: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat(app): add mouse_mode/hover_cell state, M/m toggle, mouse action handlers"
```

---

## Task 6: grid.rs — hover highlight

**Files:**
- Modify: `src/tui/render/grid.rs`

- [ ] **Step 1: Update `cell_bg` to accept and apply `hover_cell`**

Find the `fn cell_bg(` signature. Add `hover_cell: Option<(usize, usize)>` as a parameter (after `cursor`):

```rust
fn cell_bg(
    row: usize,
    col: usize,
    cursor: (usize, usize),
    hover_cell: Option<(usize, usize)>,   // new
    nav: &NavState,
    hint: Option<&crate::hint::Hint>,
    anim: &AnimState,
    colors: &ColorScheme,
) -> Color {
```

Inside `cell_bg`, after the hint block (the `if let Some(h) = hint { ... }` block) and before the `match (&nav.mode, nav.box_idx) {` line, insert:

```rust
    // Hover highlight: DarkYellow, distinct from cursor (Blue).
    // cursor takes priority — checked after hints, before nav mode highlights.
    if let Some(hc) = hover_cell {
        if hc == (row, col) && cursor != (row, col) {
            return Color::DarkYellow;
        }
    }
```

- [ ] **Step 2: Update `cell_bg` call sites in `render_grid`**

Inside `render_grid`, find all calls to `cell_bg(row, col, cursor, nav, hint, anim, colors)` and add `hover_cell` as the third argument (after `cursor`). There should be exactly one call site. Change it to:

```rust
cell_bg(row, col, cursor, hover_cell, nav, hint, anim, colors)
```

- [ ] **Step 3: Remove the placeholder `let _ = hover_cell;` line added in Task 4**

Find and delete: `let _ = hover_cell;` in `render_grid`.

- [ ] **Step 4: Verify compilation**

```bash
cargo build 2>&1 | grep -E "^error" | head -10
```

Expected: no errors.

- [ ] **Step 5: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/tui/render/grid.rs
git commit -m "feat(grid): add DarkYellow hover highlight for mouse mode"
```

---

## Task 7: status_bar.rs — mouse controls panel

**Files:**
- Modify: `src/tui/render/status_bar.rs`

- [ ] **Step 1: Remove the placeholder `let _ = mouse_mode;` added in Task 4**

Find and delete the line `let _ = mouse_mode;` added temporarily to `render_panel`.

- [ ] **Step 2: Add `ctrl_mouse` to the keyboard controls list**

In `render_panel`, in the `else` branch of `if let Some((name, explanation)) = hint_text {`, find the controls list. Add `ctrl_mouse` after `ctrl_clear`:

```rust
            (strings.ctrl_clear.into(),            d, false),
            (strings.ctrl_mouse.into(),            d, false),  // new
            (strings.ctrl_pause.into(),            d, false),
```

- [ ] **Step 3: Add the `render_mouse_controls` function**

After the `render_digit_grid` function, add the new function:

```rust
// ── Mouse controls section ────────────────────────────────────────────────────

/// Render the mouse controls into content rows 18–34 (terminal rows 20–35).
///
/// Layout (all strings are exactly 34 display chars — no outer ║ rendered here,
/// those come from the main render_panel loop):
///
///   Mouse Controls                    (row 18 / term 20)
///   (blank)                           (row 19 / term 21)
///   ┌────────┬───────┬───────┬───────┐ (row 20 / term 22)
///   │ N/Sol  │  Undo │  Redo │  Clr  │ (row 21 / term 23, clickable)
///   └────────┴───────┴───────┴───────┘ (row 22 / term 24)
///   (blank)                           (row 23 / term 25)
///   ┌──────────┬──────────┬──────────┐ (row 24 / term 26)
///   │    1     │    2     │    3     │ (row 25 / term 27, clickable)
///   ├──────────┼──────────┼──────────┤ (row 26 / term 28)
///   │    4     │    5     │    6     │ (row 27 / term 29, clickable)
///   ├──────────┼──────────┼──────────┤ (row 28 / term 30)
///   │    7     │    8     │    9     │ (row 29 / term 31, clickable)
///   └──────────┴──────────┴──────────┘ (row 30 / term 32)
///
/// Rows 31–34 remain blank (padding to bottom border).
fn render_mouse_controls(
    out:     &mut impl Write,
    row_off: u16,
    col_off: u16,
    colors:  &ColorScheme,
) -> io::Result<()> {
    let b  = colors.grid_border;
    let t  = colors.ui_text;
    let d  = colors.ui_text_dim;
    let bg = colors.ui_background;

    // Content rows 18–30 (terminal rows row_off+19 through row_off+31).
    let lines: &[(&str, Color)] = &[
        ("  Mouse Controls",                      t),  // 18
        ("",                                      d),  // 19
        ("┌────────┬───────┬───────┬───────┐",   b),  // 20
        ("│ N/Sol  │  Undo │  Redo │  Clr  │",   t),  // 21
        ("└────────┴───────┴───────┴───────┘",   b),  // 22
        ("",                                      d),  // 23
        ("┌──────────┬──────────┬──────────┐",   b),  // 24
        ("│    1     │    2     │    3     │",    t),  // 25
        ("├──────────┼──────────┼──────────┤",   b),  // 26
        ("│    4     │    5     │    6     │",    t),  // 27
        ("├──────────┼──────────┼──────────┤",   b),  // 28
        ("│    7     │    8     │    9     │",    t),  // 29
        ("└──────────┴──────────┴──────────┘",   b),  // 30
    ];

    for (i, (text, fg)) in lines.iter().enumerate() {
        let term_row = row_off + 1 + 18 + i as u16;
        // Truncate to 34 display chars (same contract as the main panel loop).
        let cell: String = text.chars().take(34).collect();
        queue!(out,
            MoveTo(col_off, term_row),
            SetForegroundColor(b),   SetBackgroundColor(bg), Print('║'),
            SetForegroundColor(*fg), SetBackgroundColor(bg),
            Print(format!(" {:<34} ", cell)),
            SetForegroundColor(b),   SetBackgroundColor(bg), Print('║'),
        )?;
    }

    // Rows 31–34: blank padding (terminal rows row_off+32 through row_off+35).
    for i in 31usize..=34 {
        let term_row = row_off + 1 + i as u16;
        queue!(out,
            MoveTo(col_off, term_row),
            SetForegroundColor(b), SetBackgroundColor(bg), Print('║'),
            SetForegroundColor(d), SetBackgroundColor(bg),
            Print(format!(" {:<34} ", "")),
            SetForegroundColor(b), SetBackgroundColor(bg), Print('║'),
        )?;
    }

    Ok(())
}
```

- [ ] **Step 4: Call `render_mouse_controls` from `render_panel`**

At the end of `render_panel`, just before the `// ── Bottom border` comment, add:

```rust
    // ── Mouse controls (replaces rows 18–34 rendered above when active) ──────
    if mouse_mode && hint_text.is_none() {
        render_mouse_controls(out, row_off, col_off, colors)?;
    }
```

The hint check is important: when a hint is active, it was already rendered into those rows by the `hint_text` branch. Mouse controls only appear when mouse mode is on AND no hint is showing.

- [ ] **Step 5: Verify compilation**

```bash
cargo build 2>&1 | grep -E "^error" | head -10
```

Expected: no errors.

- [ ] **Step 6: Run full test suite**

```bash
cargo test 2>&1 | tail -5
```

Expected: all tests pass.

- [ ] **Step 7: Smoke-test the build**

```bash
cargo build --release 2>&1 | grep -E "^error|warning.*unused" | head -10
```

Expected: clean build, no unused variable warnings.

- [ ] **Step 8: Commit**

```bash
git add src/tui/render/status_bar.rs
git commit -m "feat(panel): render mouse controls section when mouse mode active"
```

---

## Final verification

- [ ] **Run complete test suite one last time**

```bash
cargo test 2>&1
```

Expected: all tests pass (≥ ~239 tests — original 219 + ~20 new).

- [ ] **Build release binary**

```bash
cargo build --release 2>&1 | tail -3
```

Expected: `Finished release [optimized]`.
