# Extreme Difficulty Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an Extreme difficulty level backed by a new Swordfish strategy, and reclassify Backtracking-required puzzles from Hard → Extreme.

**Architecture:** A new `src/solver/swordfish.rs` implements the Swordfish elimination strategy (3-row/3-column generalisation of X-Wing). The solver registers it between XWing and Backtracking. `Difficulty::Extreme` is added to the enum; `classify()` maps Swordfish/Backtracking → Extreme and XWing → Hard (cleanup). The UI gains a 5th difficulty slot between Hard and Designer ▶.

**Tech Stack:** Rust stable, existing solver/generator/TUI framework — no new dependencies.

**Spec:** `docs/superpowers/specs/2026-05-01-extreme-difficulty-design.md`

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/solver/swordfish.rs` | **Create** | `find_swordfish()` — row and column Swordfish elimination |
| `src/solver/candidates.rs` | Modify | Add `Strategy::Swordfish` between XWing and Backtracking |
| `src/solver/mod.rs` | Modify | `pub mod swordfish`; add Swordfish to `strategy_order()`; add `apply_elims!` call in `solve()`; add `Difficulty::Extreme` arm in `for_difficulty()` |
| `src/generator/difficulty.rs` | Modify | Add `Difficulty::Extreme`; update `classify()`: Swordfish/Backtracking → Extreme, XWing only → Hard |
| `src/i18n/mod.rs` | Modify | Add `difficulty_extreme: &'static str` after `difficulty_hard` in struct; add `difficulty_extreme: "Extreme"` to all 13 language statics |
| `src/tui/render/start_screen.rs` | Modify | Expand `items` array from 4 to 5 entries, inserting Extreme between Hard and Designer ▶; add smoke test assertion |
| `src/tui/mod.rs` | Modify | `DIFFICULTY_COUNT` 4 → 5; add `3 => start_game(Difficulty::Extreme)` arm; shift Designer arm to `4`; fix back-navigation from PatternSelect/Generating to `selected: 4` |

---

## Task 1: Swordfish Strategy

**Files:**
- Create: `src/solver/swordfish.rs`
- Modify: `src/solver/candidates.rs`
- Modify: `src/solver/mod.rs`

- [ ] **Step 1: Add `Strategy::Swordfish` to the enum and write the failing test**

In `src/solver/candidates.rs`, insert `Swordfish` between `XWing` and `Backtracking`:

```rust
pub enum Strategy {
    NakedSingle,
    HiddenSingle,
    NakedPair,
    PointingPair,
    NakedTriple,
    HiddenPair,
    BoxLineReduction,
    XWing,
    Swordfish,       // ← new
    Backtracking,
}
```

Then write the failing test in `src/solver/swordfish.rs` (the file doesn't exist yet — create it):

```rust
use crate::solver::candidates::{CandidateGrid, Elimination, Strategy};

pub fn find_swordfish(_cands: &CandidateGrid) -> Vec<Elimination> {
    vec![]   // stub
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Grid;
    use crate::solver::candidates::CandidateGrid;

    // All returned eliminations must be actual candidates (no false positives).
    #[test]
    fn no_panic_no_false_positives() {
        let grid = Grid::from_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
        ).unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        let elims = find_swordfish(&cands);
        for e in &elims {
            assert!(cands.has(e.row, e.col, e.digit),
                "false positive: ({},{}) digit {} is not a candidate", e.row, e.col, e.digit);
        }
    }

    // Swordfish should not fire on an X-Wing (2-row) pattern — only on 3-row.
    // If the function returns nothing on a simple grid, that's fine.
    // The real firing test is in Task 2 (integration via Solver::for_difficulty).
    #[test]
    fn all_eliminations_carry_swordfish_strategy() {
        let grid = Grid::from_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
        ).unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        let elims = find_swordfish(&cands);
        for e in &elims {
            assert_eq!(e.strategy, Strategy::Swordfish);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test swordfish 2>&1 | head -20
```

Expected: compile error (module not declared yet) or test failures.

- [ ] **Step 3: Register the module and implement `find_swordfish`**

In `src/solver/mod.rs`, add `pub mod swordfish;` after `pub mod x_wing;` (line 10):

```rust
pub mod x_wing;
pub mod swordfish;     // ← add this line
```

Then replace the stub in `src/solver/swordfish.rs` with the full implementation:

```rust
use crate::solver::candidates::{CandidateGrid, Elimination, Strategy};

pub fn find_swordfish(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut elims = vec![];

    for digit in 1u8..=9 {
        // ── Row-based Swordfish ───────────────────────────────────────────────
        // Collect columns where `digit` appears in each row.
        let row_cols: Vec<Vec<usize>> = (0..9)
            .map(|r| (0..9).filter(|&c| cands.has(r, c, digit)).collect())
            .collect();

        // Only rows with exactly 2 or 3 candidate positions qualify.
        let cand_rows: Vec<usize> = (0..9)
            .filter(|&r| { let n = row_cols[r].len(); n == 2 || n == 3 })
            .collect();

        for i in 0..cand_rows.len() {
            for j in (i + 1)..cand_rows.len() {
                for k in (j + 1)..cand_rows.len() {
                    let (r1, r2, r3) = (cand_rows[i], cand_rows[j], cand_rows[k]);
                    // Union of the three rows' column sets must be exactly 3 columns.
                    let mut cols = std::collections::BTreeSet::new();
                    for &c in &row_cols[r1] { cols.insert(c); }
                    for &c in &row_cols[r2] { cols.insert(c); }
                    for &c in &row_cols[r3] { cols.insert(c); }
                    if cols.len() != 3 { continue; }
                    // Eliminate `digit` from all other rows in those 3 columns.
                    for &c in &cols {
                        for r in 0..9 {
                            if r == r1 || r == r2 || r == r3 { continue; }
                            if cands.has(r, c, digit) {
                                elims.push(Elimination { row: r, col: c, digit, strategy: Strategy::Swordfish });
                            }
                        }
                    }
                }
            }
        }

        // ── Column-based Swordfish (symmetric) ───────────────────────────────
        let col_rows: Vec<Vec<usize>> = (0..9)
            .map(|c| (0..9).filter(|&r| cands.has(r, c, digit)).collect())
            .collect();

        let cand_cols: Vec<usize> = (0..9)
            .filter(|&c| { let n = col_rows[c].len(); n == 2 || n == 3 })
            .collect();

        for i in 0..cand_cols.len() {
            for j in (i + 1)..cand_cols.len() {
                for k in (j + 1)..cand_cols.len() {
                    let (c1, c2, c3) = (cand_cols[i], cand_cols[j], cand_cols[k]);
                    let mut rows = std::collections::BTreeSet::new();
                    for &r in &col_rows[c1] { rows.insert(r); }
                    for &r in &col_rows[c2] { rows.insert(r); }
                    for &r in &col_rows[c3] { rows.insert(r); }
                    if rows.len() != 3 { continue; }
                    for &r in &rows {
                        for c in 0..9 {
                            if c == c1 || c == c2 || c == c3 { continue; }
                            if cands.has(r, c, digit) {
                                elims.push(Elimination { row: r, col: c, digit, strategy: Strategy::Swordfish });
                            }
                        }
                    }
                }
            }
        }
    }

    elims.sort_by_key(|e| (e.row, e.col, e.digit));
    elims.dedup_by_key(|e| (e.row, e.col, e.digit));
    elims
}
```

Keep the `#[cfg(test)]` block from Step 1 at the bottom of the file.

- [ ] **Step 4: Wire Swordfish into the solve loop**

In `src/solver/mod.rs`, make **two edits**:

**Edit 1:** Add `Strategy::Swordfish` to `strategy_order()` between XWing and Backtracking (currently lines 51–52):

```rust
fn strategy_order() -> &'static [Strategy] {
    &[
        Strategy::NakedSingle,
        Strategy::HiddenSingle,
        Strategy::NakedPair,
        Strategy::PointingPair,
        Strategy::NakedTriple,
        Strategy::HiddenPair,
        Strategy::BoxLineReduction,
        Strategy::XWing,
        Strategy::Swordfish,       // ← add
        Strategy::Backtracking,
    ]
}
```

**Edit 2:** Add `apply_elims!` call in `solve()` between the XWing and backtracking lines (currently line 123):

```rust
apply_elims!(x_wing::find_x_wings,                      Strategy::XWing);
apply_elims!(swordfish::find_swordfish,                  Strategy::Swordfish);   // ← add

// Backtracking fallback
```

Without this second edit, `strategy_order()` knows about Swordfish but `solve()` never calls `find_swordfish` — the strategy is silently skipped.

- [ ] **Step 5: Run tests**

```bash
cargo test swordfish 2>&1 | grep -E "test result|FAILED|error"
```

Expected:
```
test result: ok. 2 passed; 0 failed; ...
```

- [ ] **Step 6: Commit**

```bash
git add src/solver/swordfish.rs src/solver/candidates.rs src/solver/mod.rs
git commit -m "feat(solver): add Swordfish strategy between X-Wing and Backtracking"
```

---

## Task 2: Difficulty::Extreme

**Files:**
- Modify: `src/generator/difficulty.rs`
- Modify: `src/solver/mod.rs`

- [ ] **Step 1: Write the failing tests for classify()**

Add to the `tests` module in `src/generator/difficulty.rs`:

```rust
    #[test]
    fn swordfish_classifies_as_extreme() {
        let used = vec![Strategy::NakedSingle, Strategy::Swordfish];
        assert_eq!(classify(&used), Difficulty::Extreme);
    }

    #[test]
    fn backtracking_classifies_as_extreme() {
        let used = vec![Strategy::NakedSingle, Strategy::Backtracking];
        assert_eq!(classify(&used), Difficulty::Extreme);
    }

    #[test]
    fn x_wing_alone_classifies_as_hard() {
        // Regression: XWing must remain Hard (not Extreme) after the reclassification.
        let used = vec![Strategy::NakedSingle, Strategy::XWing];
        assert_eq!(classify(&used), Difficulty::Hard);
    }
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test classify 2>&1 | grep -E "FAILED|error\[|test result"
```

Expected: compile error (`Difficulty::Extreme` doesn't exist yet) or test failure.

- [ ] **Step 3: Add `Difficulty::Extreme` and update `classify()`**

Replace the entire `src/generator/difficulty.rs` with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
}

use crate::solver::Strategy;

pub fn classify(used: &[Strategy]) -> Difficulty {
    let needs = |s: Strategy| used.contains(&s);
    if needs(Strategy::Swordfish) || needs(Strategy::Backtracking) {
        Difficulty::Extreme
    } else if needs(Strategy::XWing) || needs(Strategy::HiddenPair)
        || needs(Strategy::NakedTriple) || needs(Strategy::BoxLineReduction) {
        Difficulty::Hard
    } else if needs(Strategy::NakedPair) || needs(Strategy::PointingPair) {
        Difficulty::Medium
    } else {
        Difficulty::Easy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::Strategy;

    #[test]
    fn easy_uses_only_singles() {
        let used = vec![Strategy::NakedSingle, Strategy::HiddenSingle];
        assert_eq!(classify(&used), Difficulty::Easy);
    }

    #[test]
    fn medium_uses_naked_pair() {
        let used = vec![Strategy::NakedSingle, Strategy::NakedPair];
        assert_eq!(classify(&used), Difficulty::Medium);
    }

    #[test]
    fn hard_uses_x_wing() {
        let used = vec![Strategy::NakedSingle, Strategy::XWing];
        assert_eq!(classify(&used), Difficulty::Hard);
    }

    #[test]
    fn swordfish_classifies_as_extreme() {
        let used = vec![Strategy::NakedSingle, Strategy::Swordfish];
        assert_eq!(classify(&used), Difficulty::Extreme);
    }

    #[test]
    fn backtracking_classifies_as_extreme() {
        let used = vec![Strategy::NakedSingle, Strategy::Backtracking];
        assert_eq!(classify(&used), Difficulty::Extreme);
    }

    #[test]
    fn x_wing_alone_classifies_as_hard() {
        let used = vec![Strategy::NakedSingle, Strategy::XWing];
        assert_eq!(classify(&used), Difficulty::Hard);
    }
}
```

- [ ] **Step 4: Add `Difficulty::Extreme` arm to `Solver::for_difficulty()`**

In `src/solver/mod.rs`, find the `for_difficulty` function (currently lines 33–40) and add the `Extreme` arm:

```rust
pub fn for_difficulty(difficulty: &crate::generator::difficulty::Difficulty) -> Self {
    use crate::generator::difficulty::Difficulty;
    match difficulty {
        Difficulty::Easy    => Self { max_strategy: Some(Strategy::HiddenSingle),  use_backtracking: false },
        Difficulty::Medium  => Self { max_strategy: Some(Strategy::PointingPair),   use_backtracking: false },
        Difficulty::Hard    => Self { max_strategy: Some(Strategy::XWing),          use_backtracking: false },
        Difficulty::Extreme => Self { max_strategy: Some(Strategy::Swordfish),      use_backtracking: false },
    }
}
```

- [ ] **Step 5: Fix any compile errors from adding the new enum variant**

The `Difficulty` enum is used in `generate()`, `is_uniquely_solvable()`, `start_game()`, and `handle_action()` match arms. Run `cargo check` to see all errors:

```bash
cargo check 2>&1 | grep "error\[" | head -10
```

For any non-exhaustive match on `Difficulty`, add a `Difficulty::Extreme` arm. In `src/generator/mod.rs`, `generate()` already delegates to `is_uniquely_solvable()` which calls `Solver::for_difficulty()` — no extra match needed there.

In `src/tui/mod.rs`, `start_game()` calls `PuzzleGenerator::new(seed).generate(difficulty, symmetry)` — no match needed. Any explicit match on `Difficulty` (e.g. in display code) needs `Difficulty::Extreme => "Extreme"` or similar.

- [ ] **Step 6: Write the generator integration test**

Add to `src/generator/mod.rs` `tests` module:

```rust
    #[test]
    fn generates_solvable_extreme_puzzle() {
        let grid = PuzzleGenerator::new(7).generate(Difficulty::Extreme, false);
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved(), "Extreme puzzle must be solvable");
    }
```

- [ ] **Step 7: Run the tests**

```bash
cargo test 2>&1 | grep "test result"
```

Expected: all test results ok, 0 failed.

- [ ] **Step 8: Commit**

```bash
git add src/generator/difficulty.rs src/solver/mod.rs src/generator/mod.rs
git commit -m "feat(difficulty): add Extreme difficulty; Swordfish/Backtracking → Extreme"
```

---

## Task 3: i18n, Start Screen, and Navigation

**Files:**
- Modify: `src/i18n/mod.rs`
- Modify: `src/tui/render/start_screen.rs`
- Modify: `src/tui/mod.rs`

- [ ] **Step 1: Write the failing test for start screen**

Add to `src/tui/render/start_screen.rs` tests (after `difficulty_screen_shows_designer_option`):

```rust
    #[test]
    fn difficulty_screen_shows_extreme_option() {
        let mut buf = Vec::new();
        render_difficulty(&mut buf, (0, 0), 3, false, true, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("Extreme"), "Expected Extreme option in difficulty screen");
    }
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test difficulty_screen_shows_extreme_option 2>&1 | grep -E "FAILED|error\[|test result"
```

Expected: compile error (missing `difficulty_extreme` field) or assertion failure.

- [ ] **Step 3: Add `difficulty_extreme` to the `Strings` struct and all 13 language statics**

In `src/i18n/mod.rs`, add the field to the struct after `difficulty_hard` (line 22):

```rust
    pub difficulty_hard:       &'static str,
    pub difficulty_extreme:    &'static str,   // ← add
    pub symmetry_label:        &'static str,
```

Then add `difficulty_extreme: "Extreme",` to every language static, immediately after `difficulty_hard`. The value is `"Extreme"` for all 13 languages (the word is internationally understood).

For **EN** (after `difficulty_hard: "Hard",`):
```rust
    difficulty_extreme:   "Extreme",
```

For **DE** (after `difficulty_hard: "Schwer",`):
```rust
    difficulty_extreme:   "Extreme",
```

For the remaining 11 statics (ES, IT, FR, SL, EO, TP, L3, SW, AF, ZH, ID) — each has `difficulty_hard: "..."` followed by `symmetry_label`. Insert after each:
```rust
    difficulty_extreme:   "Extreme",
```

There are 13 statics total (EN, DE, ES, IT, FR, SL, EO, TP, L3/Leet, SW, AF, ZH, ID). Use `grep -n "difficulty_hard" src/i18n/mod.rs` to find each location.

- [ ] **Step 4: Insert Extreme into the start screen items array**

In `src/tui/render/start_screen.rs`, find the `items` array (currently lines 98–103):

```rust
    let items = [
        strings.difficulty_easy,
        strings.difficulty_medium,
        strings.difficulty_hard,
        strings.difficulty_designer,
    ];
```

Replace with:

```rust
    let items = [
        strings.difficulty_easy,
        strings.difficulty_medium,
        strings.difficulty_hard,
        strings.difficulty_extreme,        // ← new
        strings.difficulty_designer,
    ];
```

- [ ] **Step 5: Run the start screen test**

```bash
cargo test difficulty_screen_shows_extreme_option 2>&1 | grep -E "FAILED|ok|test result"
```

Expected: `test result: ok. 1 passed; 0 failed`.

- [ ] **Step 6: Update navigation in `src/tui/mod.rs`**

Three edits are needed:

**Edit 1 — `DIFFICULTY_COUNT`:** Find `const DIFFICULTY_COUNT: usize = 4;` inside `handle_difficulty_action()` and change to `5`:

```rust
const DIFFICULTY_COUNT: usize = 5;
```

**Edit 2 — Enter handler:** Find the Enter match block (currently lines 296–306) and insert the Extreme arm:

```rust
AppAction::Enter if !sym_focused => {
    match selected {
        0 => { self.start_game(Difficulty::Easy);    self.needs_clear = true; }
        1 => { self.start_game(Difficulty::Medium);  self.needs_clear = true; }
        2 => { self.start_game(Difficulty::Hard);    self.needs_clear = true; }
        3 => { self.start_game(Difficulty::Extreme); self.needs_clear = true; }   // ← new
        4 => {                                                                      // ← was 3
            self.screen = AppScreen::PatternSelect { selected: 0 };
            self.needs_clear = true;
        }
        _ => {}
    }
}
```

**Edit 3 — PatternSelect/Generating back-navigation:** There are exactly two places that return to `DifficultySelect { selected: 3, … }` — the `Back` arm in `handle_pattern_action()` and the `from_cli` branch in `handle_generating_action()`. Both must change from `selected: 3` to `selected: 4`:

In `handle_pattern_action()` (currently at line 394):
```rust
AppAction::Back => {
    self.screen = AppScreen::DifficultySelect { selected: 4, sym_focused: false };   // was 3
    self.needs_clear = true;
}
```

In `handle_generating_action()` (currently at line 412 — the `from_cli` branch):
```rust
self.screen = if from_cli {
    AppScreen::DifficultySelect { selected: 4, sym_focused: false }   // was 3
} else {
    AppScreen::PatternSelect { selected: pat_selected }
```

- [ ] **Step 7: Run the full test suite**

```bash
cargo test 2>&1 | grep "test result"
```

Expected: all `ok`, 0 failed.

- [ ] **Step 8: Commit**

```bash
git add src/i18n/mod.rs src/tui/render/start_screen.rs src/tui/mod.rs
git commit -m "feat(ui): add Extreme to difficulty select screen"
```

---

## Task 4: Integration and Final Wiring

**Files:**
- Any remaining compilation issues

- [ ] **Step 1: Run the full test suite and fix any remaining issues**

```bash
cargo test 2>&1 | grep -E "FAILED|error\[" | head -20
```

Common issues to expect:
- Any `match difficulty` on `Difficulty` in files not yet updated will be a compile error — add the `Difficulty::Extreme` arm.
- `tests/tui_smoke.rs` should not need changes (Screen::Game doesn't use Difficulty directly).
- If any display code formats `Difficulty` as a string, add the Extreme case.

Check with:
```bash
cargo check 2>&1 | grep "error\[" | head -10
```

- [ ] **Step 2: Run the full test suite — all pass**

```bash
cargo test 2>&1 | grep "test result"
```

Expected:
```
test result: ok. NNN passed; 0 failed; ...
```

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: Extreme difficulty with Swordfish strategy — complete integration"
```
