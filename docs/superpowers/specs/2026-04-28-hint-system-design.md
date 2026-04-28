# Hint System & Panel Expansion — Design Spec

## Overview

Two tightly coupled features:

1. **Panel expansion** from 20 to 36 inner characters — prerequisite for readable hint text and unabbreviated control labels.
2. **Hint system** — tiered, strategy-based hints with a learning focus. Players are guided like a good solver would be: logical strategies first, notes guidance when needed, direct reveal only as a last resort.

---

## 1. Panel Expansion

### Width

- Inner width: **36 characters** (was 20)
- Total panel width: 38 characters (including 2 border chars `╔`/`╗`)
- Panel starts at column 77 (unchanged)
- Panel ends at column 114 (77 + 38 − 1)
- Terminal minimum: **117 columns** (was 100)

### Terminal Layout (full column map)

```
Columns 0–1    left margin (unused)
Columns 2–74   grid (73 wide)
Columns 75–76  gap (2 chars)
Columns 77–114 panel (38 wide = 2 borders + 36 inner)
Columns 115–116 right margin
Total: 117 columns minimum
```

### Minimum Size Check

The existing `MIN_COLS` constant in `src/tui/mod.rs` is updated from 100 to 117. The existing `wait_for_adequate_size` loop already handles the resize-wait UX — no new exit-on-start logic needed. The loop simply uses the updated constant.

### Panel Layout

```
╔══════════════════════════════════════╗
║  Time / Mode / Errors / Progress     ║
║  Digit grid (9 cells)                ║
║  Controls (full labels, no abbrevs)  ║
║  [Hint area — only when hint active] ║
╚══════════════════════════════════════╝
```

Actual game key bindings (controls section uses these, no change to bindings):
```
0 / -   Note mode
s       Scan mode
e       Error display
p       Pause
h       Hint          ← new
Esc     Menu
```

---

## 2. Hint System Architecture

### New AppAction variant and key binding

`src/tui/input.rs` gets a new `AppAction::RequestHint` variant and a `KeyCode::Char('h')` mapping in `map_key_to_action`. `h` is not currently mapped to anything so there is no conflict.

### Strategy Trait

Each strategy is an independent type implementing:

```rust
pub trait Strategy: Send + Sync {
    fn name_en(&self) -> &'static str;
    fn name_de(&self) -> &'static str;
    fn find(&self, state: &GameState, solution: &Grid) -> Option<Hint>;
}
```

`GameState` is passed (not a separate `Notes` type) because it already carries both the current grid and the notes mask via `notes_mask(row, col)`. `solution` is the pre-computed unique solution already stored in `App`.

New strategies are added to the registry without touching any other code.

### Hint Struct

```rust
pub struct Hint {
    /// Cells explaining WHY the hint works (green/cyan border).
    pub cause_cells:    Vec<(usize, usize)>,
    /// Cells where a candidate can be eliminated (red/magenta border).
    pub elim_cells:     Vec<(usize, usize)>,
    /// The cell where the player should act (blinking yellow background).
    /// For Notes Hints, this is the cell in the suggested region with the
    /// fewest candidates (most constrained), giving the player a starting point.
    pub target_cell:    (usize, usize),
    /// Digit being eliminated (used in explanation text placeholders).
    pub elim_digit:     Option<u8>,
    /// Digit that goes into the target cell (used in explanation text).
    pub target_digit:   Option<u8>,
    /// Pre-formatted explanation in English.
    pub explanation_en: String,
    /// Pre-formatted explanation in German.
    pub explanation_de: String,
}
```

### Strategy Registry

Ordered list — first match wins when `h` is pressed:

**Tier 1 (initial implementation):**

| # | Strategy | Cause cells | Elim cells | Target |
|---|---|---|---|---|
| 1 | Full House | — | — | Last empty cell in unit |
| 2 | Naked Single | — | — | Cell with one candidate |
| 3 | Hidden Single | Cells eliminating digit from other positions | — | Only possible cell for digit in unit |
| — | *Notes Hint* | All cells of suggested unit | — | Most constrained cell in unit |
| 4 | Naked Pairs | The two pair cells | Other cells in unit containing those candidates | — |
| 5 | Hidden Pairs | Two cells sharing a hidden pair | Other candidates in those cells | — |
| 6 | Pointing Pairs | 2–3 box cells in same row/col | Same digit outside box in that row/col | — |
| 7 | Box-Line Reduction | 2–3 row/col cells in same box | Same digit in rest of box | — |

**Tier 2 (subsequent iterations):**

| # | Strategy | Notes |
|---|---|---|
| 8 | Naked Triples | Generalisation of Naked Pairs to 3 cells |
| 9 | Hidden Triples | Generalisation of Hidden Pairs to 3 cells |
| 10 | Naked Quads | 4-cell variant |
| 11 | Hidden Quads | 4-cell variant |
| 12 | X-Wing | 4-cell 2×2 grid; eliminates digit from 2 rows or columns |
| 13 | XY-Wing | Pivot + 2 pincers; eliminates shared candidate at intersection |
| 14 | XYZ-Wing | Extension of XY-Wing with 3-candidate pivot |
| 15 | Remote Pairs | Chain of cells with identical two candidates |
| 16 | Simple Coloring | Conjugate pairs coloured two ways; contradiction eliminates |
| 17 | WXYZ-Wing | 4-cell wing |
| 18 | Swordfish | 3×3 generalisation of X-Wing |

**Special entries (not strategies, inserted at fixed positions in registry):**

**Notes Hint** — fires when no logical strategy finds anything AND at least one empty cell has a zero notes mask (no notes at all for that cell). Selection of unit to suggest:
- Consider all units (rows, cols, boxes) that contain at least one empty cell without notes
- Prefer boxes over rows/cols when tied on fewest empty cells (boxes are the natural annotation unit)
- Exclude units with zero empty cells (complete)
- `target_cell` = the most constrained empty cell within the suggested unit (fewest non-zero notes mask bits, or if all zero: the first empty cell)

**Reveal** — fires only when no strategy finds anything AND every empty cell has a non-zero notes mask (at least one candidate noted). "Notes present" means the notes mask for every empty cell is non-zero; it does not require that all valid candidates are present. Fills the most constrained empty cell (fewest bits set in notes mask, tiebroken by reading order) with the correct value from `solution`. `hint_count` is incremented. After applying the digit, `check_completion()` must be called explicitly (the key is consumed before `handle_action`, so the normal completion check does not fire). Error-tracking logic (`revealed_errors`, `errors_shown`) is skipped — Reveal always inserts the correct digit.

---

## 3. Resolution Flow

When `h` is pressed (`AppAction::RequestHint`):

```
RequestHint received
    │
    ├─ Hint already active? → close it, search for new hint
    │
    ├─ Try strategies 1–N in order
    │       Found? → display hint, hint_count++
    │
    ├─ Nothing found + any empty cell has zero notes mask?
    │       → Notes Hint: highlight suggested unit,
    │         target_cell = most constrained cell in unit
    │         hint_count++
    │
    ├─ Nothing found + all empty cells have non-zero notes mask?
    │       → Reveal: fill most constrained empty cell
    │         hint_count++
    │
    └─ Puzzle already solved? → no action
```

**While hint is active:**
- Cells light up (coloured borders + blinking target)
- Panel shows strategy name + explanation text
- **Any keypress closes the hint only** — the key is consumed and not forwarded to `handle_action`. This matches the existing `info_overlay` dismissal pattern in `src/tui/mod.rs`.
- Cursor stays in place

---

## 4. Visual System

### New ColorScheme fields

| Field | Dark | Light | High Contrast |
|---|---|---|---|
| `hint_cause_border` | `Green` | `Green` | `Cyan` |
| `hint_elim_border` | `Red` | `Red` | `Magenta` |
| `hint_target_bg` | `Yellow` | `Yellow` | `Yellow` |

### Cell rendering during hint

- **Cause cells**: normal background + coloured border segments (`│` `─` characters in `hint_cause_border`; intersection characters `┼` `╋` etc. remain normal colour)
- **Elimination cells**: normal background + coloured border segments in `hint_elim_border`
- **Target cell**: blinking yellow background (`hint_target_bg`)

### Cursor on target cell

If the cursor sits on the target cell, the background alternates between `hint_target_bg` (yellow) and `cell_active_bg` (blue/gold depending on theme), communicating both roles simultaneously.

### AnimState changes

Two new fields:
- `hint_blink: bool` — set to true when a hint with a target cell is active
- `hint_blink_tick: u32` — separate tick counter for hint phase; incremented independently of `error_blink_tick` so that simultaneous error mode + active hint do not interfere

When `hint_blink` is true:
- `hint_blink_tick` drives the yellow↔cursor-colour phase alternation on the target cell
- `is_active()` must return `true` when `hint_blink` is true (otherwise the 80 ms poll rate is not engaged and the cell never blinks)

---

## 5. i18n for Hints

Hint strings are added as new fields directly to the existing `Strings` struct in `src/i18n/mod.rs`, maintaining the established single-struct pattern and compile-time width guarantees.

All 13 language constants get the new fields. German (`DE`) gets real German translations. All other 11 languages use the English text as a placeholder:

```rust
pub struct Strings {
    // ... existing fields ...
    pub hint_full_house_name:           &'static str,
    pub hint_full_house_explanation:    &'static str,
    pub hint_naked_single_name:         &'static str,
    pub hint_naked_single_explanation:  &'static str,
    pub hint_hidden_single_name:        &'static str,
    pub hint_hidden_single_explanation: &'static str,
    pub hint_notes_name:                &'static str,
    pub hint_notes_explanation:         &'static str,
    pub hint_naked_pairs_name:          &'static str,
    pub hint_naked_pairs_explanation:   &'static str,
    pub hint_hidden_pairs_name:         &'static str,
    pub hint_hidden_pairs_explanation:  &'static str,
    pub hint_pointing_pairs_name:       &'static str,
    pub hint_pointing_pairs_explanation: &'static str,
    pub hint_box_line_name:             &'static str,
    pub hint_box_line_explanation:      &'static str,
    pub hint_reveal_name:               &'static str,
    pub hint_reveal_explanation:        &'static str,
}
```

Explanations use `{row}`, `{col}`, `{box}`, `{digit}` placeholders resolved at runtime via `str::replacen`.

`hint_count` is stored in `GameStats` for future database integration (see Out of Scope). It is not displayed in the panel in this iteration.

---

## 6. File Map

| File | Change |
|---|---|
| `src/hint/mod.rs` | **new** — `Strategy` trait, `Hint` struct, registry, resolution logic |
| `src/hint/strategies/mod.rs` | **new** — `pub mod tier1; pub mod tier2;` |
| `src/hint/strategies/tier1.rs` | **new** — Full House through Box-Line Reduction |
| `src/hint/strategies/tier2.rs` | **new** — Naked Triples through Swordfish (later iterations, file created empty) |
| `src/i18n/mod.rs` | add hint `name` + `explanation` fields to `Strings`; fill DE + EN; all other 11 languages copy EN |
| `src/tui/colors.rs` | add `hint_cause_border`, `hint_elim_border`, `hint_target_bg` to all three themes |
| `src/tui/anim.rs` | add `hint_blink: bool`; include in `is_active()`; yellow↔cursor phase logic |
| `src/tui/input.rs` | add `AppAction::RequestHint`; map `KeyCode::Char('h')` |
| `src/tui/mod.rs` | `hint_count` in `GameStats`; `RequestHint` handler; active hint state; update `MIN_COLS` to 117 |
| `src/tui/render/grid.rs` | coloured border segments for cause/elim cells; blinking target cell |
| `src/tui/render/status_bar.rs` | expand panel to 36 inner chars; hint text area replaces controls when hint active; full control labels |
| `src/lib.rs` | add `pub mod hint` |

**Unchanged:** `solver::backtracking`, `puzzle::GameState`, `puzzle::Grid`, `render_info_overlay`

---

## 7. Out of Scope (later)

- Extreme difficulty level
- Designer / pattern Sudokus
- Hexadoku (16×16)
- Hint statistics written to database (`hint_count` is tracked but not yet consumed)
- Tier 2 strategy implementations (file scaffolded but empty)
