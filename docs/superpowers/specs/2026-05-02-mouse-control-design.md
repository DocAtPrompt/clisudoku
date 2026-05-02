# Mouse Control Design

## Goal

Add optional mouse input to the Sudoku TUI, toggled with `M`/`m`. Mouse mode coexists with keyboard input and provides cell navigation and digit entry via clickable panel buttons.

## Architecture

### Toggle

Pressing `M` or `m` (both cases, consistent with all other keys in `input.rs`) toggles mouse mode on/off. On activation, `crossterm::event::EnableMouseCapture` is sent to the terminal; on deactivation, `DisableMouseCapture`. Toggling off also resets `hover_cell` to `None`. Starting a new game while mouse mode is active must call `DisableMouseCapture` and reset both `mouse_mode` and `hover_cell` — so the new game always starts without mouse capture active.

The existing `AppState` gains two new fields:

```rust
mouse_mode: bool,            // false by default
hover_cell: Option<(usize, usize)>,  // (row, col) grid cell under mouse
```

### Input Pipeline

`crossterm` delivers `Event::Mouse(MouseEvent)` events alongside key events. The input layer translates mouse events into semantic `AppAction` variants **before** they reach `handle_action` — layout knowledge stays out of `mod.rs`:

```rust
AppAction::MouseHover(usize, usize)     // grid cell (row, col) under cursor
AppAction::MouseSelectCell(usize, usize) // grid cell clicked → move cursor
AppAction::MouseButton(MousePanelButton) // panel button clicked
```

```rust
pub enum MousePanelButton {
    NotesSolToggle,
    Undo,
    Redo,
    Clear,
    Digit(u8),  // 1..=9
}
```

Two pure hit-test functions in `src/tui/input.rs` (no I/O, fully unit-testable) perform the translation:

#### `hit_test_grid(term_col: u16, term_row: u16) -> Option<(usize, usize)>`

Grid cells start at `col_off + 1 = 3` (column) and `row_off + 1 = 2` (row), confirmed in `grid.rs`:
`cell_term_col = col_off + 1 + col * 8`, `term_row = row_off + 1 + row * 4 + line_idx`.

- Each grid column occupies **8 terminal columns** (7 content + 1 separator).
- Each grid row occupies **4 terminal rows** (3 content + 1 separator).

Mapping (correct bases):
```
grid_col = (term_col - 3) / 8    valid when term_col >= 3, remainder != 7, result < 9
grid_row = (term_row - 2) / 4    valid when term_row >= 2, remainder != 3, result < 9
```

`remainder == 7` means a vertical border column (rejected). `remainder == 3` means a horizontal border row (rejected). Returns `None` for borders, out-of-range coords, or cells outside `0..9`.

#### `hit_test_panel_button(term_col: u16, term_row: u16) -> Option<MousePanelButton>`

Panel renders at `col_off = 77, row_off = 1`. The `║` borders are at cols 77 and 114. Inner content occupies cols 78–113 (1 leading space + 34 chars + 1 trailing space). The actual drawable area (inside the space padding) is cols 79–112 (34 chars).

The divider `╠═══╣` is at content row index 17 → terminal row 19.

Mouse controls occupy terminal rows 20–35:
- **Row 20**: label — not a button.
- **Row 21**: blank.
- **Row 22**: action button top border — not clickable.
- **Row 23**: action button content → `NotesSolToggle`, `Undo`, `Redo`, or `Clear`.
- **Row 24**: action button bottom border — not clickable.
- **Row 25**: blank.
- **Rows 26–32**: 3×3 digit grid.

**Action button column ranges** (layout: `┌────────┬───────┬───────┬───────┐` = 1+8+1+7+1+7+1+7+1 = 34):
- N/Sol: cols 79–88 (left border + 8 content chars + separator — 10 cols)
- Undo:  cols 89–96 (8 cols)
- Redo:  cols 97–104 (8 cols)
- Clr:   cols 105–112 (8 cols)

Border separator columns are attributed to the button to their left.

**Digit grid rows** (terminal rows 26–32):
- Row 27: digits 1/2/3 → `Digit(1)`, `Digit(2)`, `Digit(3)`
- Row 29: digits 4/5/6 → `Digit(4)`, `Digit(5)`, `Digit(6)`
- Row 31: digits 7/8/9 → `Digit(7)`, `Digit(8)`, `Digit(9)`
- Rows 26, 28, 30, 32 are border rows → `None`.

**Digit grid column ranges** (layout: `┌──────────┬──────────┬──────────┐` = 1+10+1+10+1+10+1 = 34):
- Digit col 0 (1/4/7): cols 79–90  (left border + 10 content + separator — 12 cols)
- Digit col 1 (2/5/8): cols 91–101 (11 cols)
- Digit col 2 (3/6/9): cols 102–112 (11 cols)

Border separator columns are attributed to the cell to their left.

Returns `None` if outside the mouse-controls row/col range.

### Action Dispatch

- `MouseHover`: updates `hover_cell`; triggers re-render.
- `MouseSelectCell(r, c)`: moves cursor to `(r, c)`, same as keyboard navigation.
- `MouseButton(btn)`: fires the corresponding existing action:
  - `NotesSolToggle` → `AppAction::ToggleMode`
  - `Undo` → `AppAction::Undo`
  - `Redo` → `AppAction::Redo`
  - `Clear` → `AppAction::ClearCell`
  - `Digit(d)` → `AppAction::Digit(d)`

Keyboard input remains fully functional in mouse mode. All existing key handlers are unchanged.

**No visual click feedback** — buttons fire immediately with no momentary highlight. This keeps the render path simple and is consistent with the keyboard-driven design of the app.

### Hover Highlight

The grid renderer receives `hover_cell: Option<(usize, usize)>` alongside the existing `cursor`. A hovered cell gets `Color::DarkYellow` as background — distinct from the cursor (`Color::Blue`) and all other cell states. The hover highlight is suppressed when `mouse_mode` is inactive **or** when the game is paused (consistent with all other overlay states). No `Color::Rgb` or `Color::AnsiValue` used anywhere (CLAUDE.md rule).

## Panel Layout

The controls section (below the `╠═══╣` divider, content rows 18–34) is replaced when `mouse_mode` is active. The digit buttons use the full 34-char inner width: 3 columns × 10 chars + 4 border chars = 34. When a hint is active, hint text covers this area exactly as it does today (the existing `if let Some(hint_text)` branch in `render_panel` replaces the controls section regardless of mouse mode).

```
╠════════════════════════════════════╣   ← row 19 (divider)
║  Mouse Controls                    ║   ← row 20
║                                    ║   ← row 21
║ ┌────────┬───────┬───────┬───────┐ ║   ← row 22
║ │ N/Sol  │  Undo │  Redo │  Clr  │ ║   ← row 23 (clickable)
║ └────────┴───────┴───────┴───────┘ ║   ← row 24
║                                    ║   ← row 25
║ ┌──────────┬──────────┬──────────┐ ║   ← row 26
║ │    1     │    2     │    3     │ ║   ← row 27 (clickable)
║ ├──────────┼──────────┼──────────┤ ║   ← row 28
║ │    4     │    5     │    6     │ ║   ← row 29 (clickable)
║ ├──────────┼──────────┼──────────┤ ║   ← row 30
║ │    7     │    8     │    9     │ ║   ← row 31 (clickable)
║ └──────────┴──────────┴──────────┘ ║   ← row 32
╚════════════════════════════════════╝   ← row 37
```

- **N/Sol**: toggles note/solution mode.
- **Undo / Redo**: undo/redo last move.
- **Clr**: clears the cursor cell.
- **1–9**: enters the digit into the cursor cell (note or solution depending on active mode).

## i18n

Add one string to `Strings`:

```rust
ctrl_mouse: &'static str,  // e.g. "m  mouse on/off"
```

This string appears in the **keyboard-shortcut list** (controls section) when mouse mode is **off**, advertising how to enable it. When mouse mode is on, the controls section is replaced by the mouse panel entirely — no separate "turn off" hint is needed since the user already knows `M` toggles it.

All 13 language constants in `src/i18n/mod.rs` receive this string.

## Files Changed

| File | Change |
|---|---|
| `src/tui/terminal.rs` | Send `EnableMouseCapture` / `DisableMouseCapture` on toggle; expose helper to disable on game start |
| `src/tui/input.rs` | Parse `Event::Mouse` → semantic `AppAction`; `hit_test_grid`, `hit_test_panel_button` pure functions + unit tests |
| `src/tui/mod.rs` | `mouse_mode`, `hover_cell` fields; `M`/`m` key toggle; mouse action handler; reset on new game; pass `hover_cell` to renderer |
| `src/tui/render/grid.rs` | `Color::DarkYellow` hover highlight when `hover_cell` is `Some` and not paused |
| `src/tui/render/status_bar.rs` | `render_mouse_controls()` renders action buttons + digit grid; replaces keyboard-shortcut list when `mouse_mode` is active |
| `src/tui/render/mod.rs` | Pass `mouse_mode` and `hover_cell` through `Screen::Game` |
| `src/i18n/mod.rs` | Add `ctrl_mouse` string to all 13 language constants |

## Constraints

- ANSI colours only. No `Color::Rgb` or `Color::AnsiValue`.
- Mouse events are processed only on the `Game` screen; ignored elsewhere.
- Mouse mode does not persist across sessions.
- Hit-test functions are pure and covered by unit tests.
- No click visual feedback (buttons fire immediately).
- Hover suppressed when paused.
