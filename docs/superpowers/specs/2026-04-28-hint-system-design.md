# Hint System & Panel Expansion — Design Spec

## Overview

Two tightly coupled features:

1. **Panel expansion** from 20 to 36 inner characters — prerequisite for readable hint text and unabbreviated control labels.
2. **Hint system** — tiered, strategy-based hints with a learning focus. Players are guided like a good solver would be: logical strategies first, notes guidance when needed, direct reveal only as a last resort.

---

## 1. Panel Expansion

### Width

- Inner width: **36 characters** (was 20)
- Total panel width: 38 characters (including 2 border chars)
- Panel starts at column 77 (unchanged)
- Terminal minimum: **117 columns** (was 99)

### Startup Check

On launch, before entering raw mode, check terminal width:

```rust
if terminal_cols < 117 {
    eprintln!("Terminal too narrow ({} cols). Minimum 117 required.", terminal_cols);
    std::process::exit(1);
}
```

### Panel Layout

```
╔══════════════════════════════════════╗
║  Time / Mode / Errors / Progress     ║
║  Digit grid (9 cells)                ║
║  Controls (full labels, no abbrevs)  ║
║  [Hint area — only when hint active] ║
╚══════════════════════════════════════╝
```

Controls section uses full labels now that space allows:
```
h    Hint
n    Note mode
e    Error display
s    Scan mode
p    Pause
Esc  Menu
```

---

## 2. Hint System Architecture

### Strategy Trait

Each strategy is an independent type implementing:

```rust
pub trait Strategy: Send + Sync {
    fn name_en(&self) -> &'static str;
    fn name_de(&self) -> &'static str;
    fn find(&self, grid: &Grid, notes: &Notes, solution: &Grid) -> Option<Hint>;
}
```

New strategies are added to the registry without touching any other code.

### Hint Struct

```rust
pub struct Hint {
    /// Cells explaining WHY the hint works (green/cyan border).
    pub cause_cells:     Vec<(usize, usize)>,
    /// Cells where a candidate can be eliminated (red/magenta border).
    pub elim_cells:      Vec<(usize, usize)>,
    /// The cell where the player should act (blinking yellow background).
    pub target_cell:     (usize, usize),
    /// Digit being eliminated (for explanation text placeholders).
    pub elim_digit:      Option<u8>,
    /// Digit that goes into the target cell (for explanation text).
    pub target_digit:    Option<u8>,
    /// Explanation in English (may contain {row}, {col}, {box}, {digit} placeholders).
    pub explanation_en:  String,
    /// Explanation in German.
    pub explanation_de:  String,
}
```

### Strategy Registry

Ordered list — first match wins when `h` is pressed:

**Tier 1 (initial implementation):**

| # | Strategy | Cause cells | Elim cells | Target |
|---|---|---|---|---|
| 1 | Full House | — | — | Last empty cell in unit |
| 2 | Naked Single | — | — | Cell with one candidate |
| 3 | Hidden Single | Cells eliminating the digit from other positions | — | Only possible cell for digit in unit |
| — | *Notes Hint* | Promising region | — | — |
| 4 | Naked Pairs | The two pair cells | Other cells in unit containing those candidates | — |
| 5 | Hidden Pairs | Two cells sharing a hidden pair | Other candidates in those cells | — |
| 6 | Pointing Pairs | 2–3 box cells in same row/col | Same digit outside box in that row/col | — |
| 7 | Box-Line Reduction | 2–3 row/col cells in same box | Same digit in rest of box | — |

**Tier 2 (subsequent iterations, no dependency on Extreme difficulty):**

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

**Special entries (not strategies, but part of the resolution flow):**

- **Notes Hint** — inserted after Hidden Single in the registry. Fires when no logical strategy finds anything AND some empty cells lack notes. Highlights the most promising region (unit with fewest empty cells) and suggests the player add notes there.
- **Reveal** — absolute last resort. Fires only when no strategy finds anything AND all empty cells have notes. Fills the most constrained empty cell (fewest remaining candidates) with its correct value. hint_count is incremented.

---

## 3. Resolution Flow

When `h` is pressed:

```
h pressed
    │
    ├─ Hint already active? → close it, then search for new hint
    │
    ├─ Try strategies 1–N in order
    │       Found? → display hint, hint_count++
    │
    ├─ Nothing found + notes incomplete?
    │       → Notes Hint: highlight most promising region
    │         "Complete notes in box X / row Y"
    │         hint_count++
    │
    ├─ Nothing found + all notes present?
    │       → Reveal: fill most constrained cell (fewest candidates)
    │         hint_count++
    │
    └─ Puzzle already solved? → no action
```

**While hint is active:**
- Cells light up (coloured borders + blinking target)
- Panel shows strategy name + explanation text
- Any keypress closes the hint; if the key has a normal action it is also executed
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

If the cursor happens to sit on the target cell, the background alternates between `hint_target_bg` (yellow) and `cell_active_bg` (blue), communicating both roles simultaneously.

### AnimState changes

New `hint_blink: bool` flag. When true, the blink timer drives the yellow↔blue phase alternation on the target cell. Uses the existing blink infrastructure already present for error cells.

---

## 5. i18n for Hints

Hints use a separate `HintStrings` struct. Only **English and German** are provided; all other languages fall back to English.

```rust
pub struct HintStrings {
    pub full_house_name:           &'static str,
    pub full_house_explanation:    &'static str,
    pub naked_single_name:         &'static str,
    pub naked_single_explanation:  &'static str,
    pub hidden_single_name:        &'static str,
    pub hidden_single_explanation: &'static str,
    pub notes_hint_name:           &'static str,
    pub notes_hint_explanation:    &'static str,
    // ... one name + explanation per strategy
    pub reveal_name:               &'static str,
    pub reveal_explanation:        &'static str,
}
```

Explanations use `{row}`, `{col}`, `{box}`, `{digit}` placeholders resolved at runtime.

Language selection:

```rust
pub fn hint_strings(lang: Language) -> &'static HintStrings {
    match lang {
        Language::De => &DE_HINTS,
        _            => &EN_HINTS,
    }
}
```

---

## 6. File Map

| File | Change |
|---|---|
| `src/hint/mod.rs` | **new** — `Strategy` trait, `Hint` struct, registry, resolution logic |
| `src/hint/strategies/tier1.rs` | **new** — Full House through Box-Line Reduction |
| `src/hint/strategies/tier2.rs` | **new** — Naked Triples through Swordfish (later iterations) |
| `src/i18n/hint_strings.rs` | **new** — `HintStrings`, EN + DE texts |
| `src/tui/colors.rs` | add `hint_cause_border`, `hint_elim_border`, `hint_target_bg` to all three themes |
| `src/tui/anim.rs` | add `hint_blink` flag; yellow↔blue phase logic |
| `src/tui/mod.rs` | `hint_count` in `GameStats`; `h` key handler; active hint state |
| `src/tui/render/grid.rs` | coloured border segments for cause/elim cells; blinking target |
| `src/tui/render/status_bar.rs` | expand panel to 36 inner chars; hint text area; full control labels |
| `src/main.rs` | terminal width check at startup (≥ 117 cols) |

**Unchanged:** `solver::backtracking`, `puzzle::GameState`, `puzzle::Grid`, `render_info_overlay`

---

## 7. Out of Scope (later)

- Extreme difficulty level (requires generator tuning for puzzles needing Tier 2 strategies)
- Designer / pattern Sudokus
- Hexadoku (16×16)
- Hint statistics in database
- Tier 2 strategy implementations
