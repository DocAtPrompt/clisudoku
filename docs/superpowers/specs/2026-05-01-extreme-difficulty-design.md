# Extreme Difficulty Design

**Date:** 2026-05-01
**Status:** Approved

---

## Goal

Add an Extreme difficulty level to clisudoku. Extreme puzzles require the Swordfish strategy to solve — a step beyond Hard (which caps at X-Wing). Backtracking, previously classified as Hard, moves to Extreme.

---

## Background

The solver currently has 9 strategies in ascending difficulty order:

1. NakedSingle
2. HiddenSingle
3. NakedPair
4. PointingPair
5. NakedTriple
6. HiddenPair
7. BoxLineReduction
8. XWing
9. Backtracking

Hard puzzles are generated with `max_strategy = XWing, use_backtracking = false`. The `classify()` function currently maps Backtracking → Hard, which is inconsistent with generation (no Hard puzzle ever requires Backtracking). This inconsistency is resolved as part of this feature.

---

## Design

### 1. Swordfish Strategy

**File:** `src/solver/swordfish.rs` (new)

Swordfish is the 3-line generalisation of X-Wing. For rows: if a candidate digit appears in exactly 2 or 3 columns in each of exactly 3 rows, and those columns are the same set across all 3 rows, then the digit can be eliminated from all other cells in those 3 columns. The column variant is symmetric.

**Interface:**
```rust
pub fn find_swordfish(candidates: &CandidateGrid) -> Vec<Elimination>
```

Both the row and column directions are checked in a single call.

**Registration — two required edits in `src/solver/mod.rs`:**
1. Add `Strategy::Swordfish` to the `strategy_order()` array between `XWing` and `Backtracking`.
2. Add an `apply_elims!(swordfish::find_swordfish, Strategy::Swordfish)` call in the `solve()` loop body, between the `x_wing` and backtracking invocations. Without this second edit the strategy is correctly ordered and gated but is never actually invoked — the puzzle appears unsolvable at the Swordfish level.

**`Strategy` enum update** (`src/solver/candidates.rs`):
```rust
pub enum Strategy {
    NakedSingle, HiddenSingle, NakedPair, PointingPair,
    NakedTriple, HiddenPair, BoxLineReduction, XWing,
    Swordfish,       // new — between XWing and Backtracking
    Backtracking,
}
```

### 2. Difficulty::Extreme

**File:** `src/generator/difficulty.rs`

```rust
pub enum Difficulty { Easy, Medium, Hard, Extreme }
```

**`Solver::for_difficulty` update** (`src/solver/mod.rs`):
```rust
Difficulty::Extreme => Self { max_strategy: Some(Strategy::Swordfish), use_backtracking: false },
```

**`classify()` update:**
| Difficulty | Strategies |
|---|---|
| Easy | only NakedSingle / HiddenSingle |
| Medium | NakedPair or PointingPair |
| Hard | NakedTriple, HiddenPair, BoxLineReduction, or XWing |
| Extreme | **Swordfish or Backtracking** |

Backtracking moves from Hard → Extreme. Hard becomes consistent: every Hard puzzle is solvable by XWing strategies without T&E.

### 3. UI

**Start screen** (`src/tui/render/start_screen.rs`):

Extreme is inserted as the 4th difficulty item, between Hard and Designer ▶:
```
Easy  Medium  Hard  Extreme  Designer ▶
```

**`DIFFICULTY_COUNT`** (`src/tui/mod.rs`): Note — this is a local `const` declared *inside* `handle_difficulty_action()`, not a module-level constant. Search for `const DIFFICULTY_COUNT` within that function and change the value from `4` to `5`.

**DifficultySelect Enter handler** (`src/tui/mod.rs`): The current Enter arms are `0` → Easy, `1` → Medium, `2` → Hard, `3` → PatternSelect (Designer). Insert a new arm `3` → `start_game(Difficulty::Extreme)` and shift the old `3` arm (PatternSelect) to `4`.

**PatternSelect back-navigation** (`src/tui/mod.rs`): There are two places that return to `DifficultySelect { selected: 3, … }` when coming back from PatternSelect — one in `handle_pattern_action` (Back arm) and one in `handle_generating_action`. Both must be updated to `selected: 4` so the cursor lands on Designer ▶ after the user presses Esc, not on Extreme.

**i18n** (`src/i18n/mod.rs`): new field `difficulty_extreme` added after `difficulty_hard` in the `Strings` struct and in all 13 language statics. Value: `"Extreme"` in all languages (the word is internationally understood; German "Extrem" would drop the final e which looks odd in context).

### 4. Out of Scope

- No changes to the hint system beyond what falls out naturally from the solver reporting Swordfish steps. Hint display of Swordfish source cells works the same as for X-Wing.
- No new Y-Wing, XY-Chain, or other strategies in this milestone.
- No database persistence changes (deferred, same as Designer Sudoku).

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/solver/swordfish.rs` | **Create** | `find_swordfish()` — row and column variants |
| `src/solver/candidates.rs` | Modify | Add `Strategy::Swordfish` between XWing and Backtracking |
| `src/solver/mod.rs` | Modify | (1) Add `Strategy::Swordfish` to `strategy_order()`; (2) add `apply_elims!(swordfish::find_swordfish, Strategy::Swordfish)` in `solve()` loop; (3) add `Difficulty::Extreme` config in `for_difficulty()` |
| `src/generator/difficulty.rs` | Modify | Add `Difficulty::Extreme`; update `classify()` |
| `src/i18n/mod.rs` | Modify | Add `difficulty_extreme` field to all 13 language statics |
| `src/tui/render/start_screen.rs` | Modify | Insert Extreme as 4th difficulty option |
| `src/tui/mod.rs` | Modify | `DIFFICULTY_COUNT` 4 → 5; new Enter arm for Extreme; Designer shifts to index 4 |

---

## Testing

- Unit tests for `find_swordfish`: known Swordfish position (fixture), column variant, no false positives
- `classify()` tests: Swordfish → Extreme, Backtracking → Extreme, XWing → Hard (regression)
- Generator integration test: `generate(Difficulty::Extreme, false)` produces a puzzle classified as Extreme
- `Solver::for_difficulty(Extreme)` correctly applies Swordfish but not Backtracking
- Start screen render test: asserts "Extreme" appears in output
- All existing tests continue to pass
