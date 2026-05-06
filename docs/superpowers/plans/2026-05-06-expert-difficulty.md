# Expert Difficulty Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an `Expert` difficulty level that generates puzzles requiring at least one Tier-2 strategy (Jellyfish, Skyscraper, XY-Chain, etc.) — puzzles the Extreme solver cannot crack.

**Architecture:** A new `src/solver/expert.rs` module holds 13 elimination functions + 1 placement function, all operating on `CandidateGrid`. The solver integrates them as a single `Strategy::Expert` block between Swordfish and Backtracking. The generator uses a double solvability check (Expert solver CAN solve it AND Extreme solver CANNOT) with up to 64 seed-retry attempts inside `generate()`.

**Tech Stack:** Rust, existing `CandidateGrid`/`Elimination`/`SolveStep` types, crossterm for TUI.

---

## Translation rule (tier2.rs → expert.rs)

Every function in `src/hint/strategies/tier2.rs` uses:
- `state.notes_mask(r, c)` → replace with `cands.mask(r, c)`
- `matches!(grid.get(r, c), CellKind::Empty)` → replace with `cands.mask(r, c) != 0`
- No `grid` parameter needed — only `cands: &CandidateGrid`
- Strip all `Hint` construction, `name_*`, `explanation_*`, `cause_cells`, `target_cell` fields
- Return `Vec<Elimination>` instead of `Option<Hint>` (collect ALL eliminations, not just first)
- Each `Elimination` sets `strategy: Strategy::Expert`

**Special case — BUG+1:** Returns `Option<SolveStep>` (a placement, not elimination). The solver calls it like NakedSingle, before the elimination loop.

---

## Files

| File | Change |
|---|---|
| `src/solver/expert.rs` | NEW — 14 functions |
| `src/solver/candidates.rs` | Add `Strategy::Expert` variant |
| `src/solver/mod.rs` | `pub mod expert`, strategy_order, for_difficulty, Expert solve block |
| `src/generator/difficulty.rs` | Add `Difficulty::Expert`, extend `classify()` |
| `src/generator/mod.rs` | Expert arm in `generate()` with double check + retry |
| `src/tui/generating.rs` | `spawn_expert()` + `new_expert()` constructor, `expert: bool` field |
| `src/tui/mod.rs` | Expert entry in match, `DIFFICULTY_COUNT` 6→7, route Expert via Generating screen |
| `src/tui/render/start_screen.rs` | 7th entry in `items` array |
| `src/i18n/mod.rs` | New `difficulty_expert` field, fill all languages |

---

## Task 1: Add `Strategy::Expert` to candidates.rs

**Files:**
- Modify: `src/solver/candidates.rs`

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)]` section at the bottom of `src/solver/candidates.rs`:

```rust
#[test]
fn strategy_expert_exists_and_is_distinct() {
    // Ensure Expert exists and is not equal to Swordfish or Backtracking
    assert_ne!(Strategy::Expert, Strategy::Swordfish);
    assert_ne!(Strategy::Expert, Strategy::Backtracking);
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd /Users/alexandererben/Tresors/OrdiSync/6_Entwicklung/Claude/SudokuCLI
cargo test strategy_expert_exists_and_is_distinct 2>&1 | tail -5
```

Expected: compile error "no variant named `Expert` found for enum `Strategy`"

- [ ] **Step 3: Add the variant**

In `src/solver/candidates.rs`, add `Expert` between `Swordfish` and `Backtracking`:

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
    Swordfish,
    Expert,       // NEW — covers all Tier-2 expert techniques
    Backtracking,
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cargo test strategy_expert_exists_and_is_distinct 2>&1 | tail -5
```

Expected: `test result: ok. 1 passed`

- [ ] **Step 5: Run all tests (must stay green)**

```bash
cargo test 2>&1 | grep -E "^test result"
```

Expected: all `ok`, no failures.

- [ ] **Step 6: Commit**

```bash
git add src/solver/candidates.rs
git commit -m "feat: add Strategy::Expert variant"
```

---

## Task 2: Add `Difficulty::Expert` and update `classify()`

**Files:**
- Modify: `src/generator/difficulty.rs`

- [ ] **Step 1: Write the failing tests**

Add to `#[cfg(test)]` in `src/generator/difficulty.rs`:

```rust
#[test]
fn expert_strategy_classifies_as_expert() {
    let used = vec![Strategy::NakedSingle, Strategy::Expert];
    assert_eq!(classify(&used), Difficulty::Expert);
}

#[test]
fn swordfish_without_expert_still_classifies_as_extreme() {
    let used = vec![Strategy::NakedSingle, Strategy::Swordfish];
    assert_eq!(classify(&used), Difficulty::Extreme);
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test expert_strategy_classifies_as_expert swordfish_without_expert_still_classifies_as_extreme 2>&1 | tail -10
```

Expected: compile error — `Difficulty::Expert` does not exist yet.

- [ ] **Step 3: Add `Difficulty::Expert` variant and update `classify()`**

Replace the contents of `src/generator/difficulty.rs` with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
    /// Requires at least one Tier-2 expert technique (Jellyfish, Skyscraper, XY-Chain, …).
    /// The Expert solver can solve it; the Extreme solver cannot.
    Expert,
    /// Maximally-reduced puzzle: as few givens as possible (targeting 17),
    /// solved using full backtracking — no strategy cap.
    BareMinimum,
}

use crate::solver::Strategy;

pub fn classify(used: &[Strategy]) -> Difficulty {
    let needs = |s: Strategy| used.contains(&s);
    if needs(Strategy::Expert) {
        Difficulty::Expert
    } else if needs(Strategy::Swordfish) || needs(Strategy::Backtracking) {
        Difficulty::Extreme
    } else if needs(Strategy::XWing)
        || needs(Strategy::HiddenPair)
        || needs(Strategy::NakedTriple)
        || needs(Strategy::BoxLineReduction)
    {
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

    #[test]
    fn expert_strategy_classifies_as_expert() {
        let used = vec![Strategy::NakedSingle, Strategy::Expert];
        assert_eq!(classify(&used), Difficulty::Expert);
    }

    #[test]
    fn swordfish_without_expert_still_classifies_as_extreme() {
        let used = vec![Strategy::NakedSingle, Strategy::Swordfish];
        assert_eq!(classify(&used), Difficulty::Extreme);
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test -p clisudoku --lib generator::difficulty 2>&1 | tail -5
```

Expected: `test result: ok. 8 passed`

- [ ] **Step 5: Run all tests (expect compile errors from files not yet updated)**

```bash
cargo build 2>&1 | grep "^error" | head -20
```

Expected: compile errors in `src/tui/mod.rs` and `src/generator/mod.rs` about non-exhaustive match on `Difficulty`. That is expected — they will be fixed in Tasks 5 and 6.

- [ ] **Step 6: Commit**

```bash
git add src/generator/difficulty.rs
git commit -m "feat: add Difficulty::Expert and update classify()"
```

---

## Task 3: Create `src/solver/expert.rs` — all 14 functions

**Files:**
- Create: `src/solver/expert.rs`

This is the largest task. The functions are direct translations of `src/hint/strategies/tier2.rs` with the API stripped down to `CandidateGrid` only.

**Translation rule recap:**
- `state.notes_mask(r, c)` → `cands.mask(r, c)`
- `matches!(grid.get(r, c), CellKind::Empty)` → `cands.mask(r, c) != 0`
- Return `Vec<Elimination>` (collect ALL eliminations, not just the first one)
- `BugPlusOne` returns `Option<SolveStep>` instead

**Note on TDD for a new file with `pub mod` dependency:** `expert.rs` cannot be compiled as part of the crate until Task 4 adds `pub mod expert;` to `solver/mod.rs`. To still write tests before the implementation, Step 1 creates the file with stubs (functions that return empty) and the full test module. Step 3 then replaces the stubs with real implementations. This guarantees all positive tests fail with stubs and pass with the real code — the TDD structure is in place even though the test run happens in Task 4 Step 4.

- [ ] **Step 1: Create `expert.rs` with stubs and tests**

Create `src/solver/expert.rs` with the following stub content (stubs return empty — all positive tests will fail):

```rust
// src/solver/expert.rs — STUBS
// Step 3 will replace these stubs with real implementations.
// The test module below is the specification: positive tests must all pass after Step 3.

use crate::solver::candidates::{CandidateGrid, Elimination, SolveStep, Strategy};

fn all_units() -> Vec<Vec<(usize, usize)>> { vec![] }
fn sees(_r1: usize, _c1: usize, _r2: usize, _c2: usize) -> bool { false }
fn elim(row: usize, col: usize, digit: u8) -> Elimination {
    Elimination { row, col, digit, strategy: Strategy::Expert }
}

pub fn find_jellyfish(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_naked_quad(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_hidden_triple(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_hidden_quad(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_skyscraper(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_two_string_kite(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_y_wing(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_xyz_wing(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_w_wing(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_unique_rectangle(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_bug_plus_one_step(_cands: &CandidateGrid) -> Option<SolveStep> { None }
pub fn find_empty_rectangle(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_simple_coloring(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
pub fn find_xy_chain(_cands: &CandidateGrid) -> Vec<Elimination> { vec![] }
```

Then append the complete test module below the stubs (copy from Step 3's full file — the `#[cfg(test)] mod tests { … }` block is identical in both). The test module is listed in full in Step 3; paste it here as well so the stubs file has the complete tests.

- [ ] **Step 2: Verify stubs compile (within the file itself)**

```bash
cd /Users/alexandererben/Tresors/OrdiSync/6_Entwicklung/Claude/SudokuCLI
rustc --edition 2021 --crate-type lib src/solver/expert.rs 2>&1 | head -10
```

Expected: errors about unknown types (`CandidateGrid`, `Elimination`, `SolveStep`, `Strategy`) because the file isn't in the crate module tree yet — that is OK. The important check is that there are NO syntax errors in the stubs or test module themselves (all errors should reference unresolved names, not parse failures). If you see `error: expected` or `error: this file contains an unclosed delimiter`, fix the syntax before continuing.

- [ ] **Step 3: Replace stubs with the full `src/solver/expert.rs` implementation**

Overwrite `src/solver/expert.rs` with the complete content below (real implementations + same test module):

Create the file with the following content:

```rust
// src/solver/expert.rs
//
// Expert-level solver functions operating on CandidateGrid.
// These are pure elimination/placement finders — no display logic.
//
// Translation rule from hint/strategies/tier2.rs:
//   state.notes_mask(r, c)              →  cands.mask(r, c)
//   matches!(grid.get(r,c), CellKind::Empty)  →  cands.mask(r, c) != 0
//
// Each find_* function returns Vec<Elimination> (all eliminations found).
// find_bug_plus_one_step returns Option<SolveStep> (a placement, not elimination).

use crate::solver::candidates::{CandidateGrid, Elimination, SolveStep, Strategy};

// ── Internal helpers ──────────────────────────────────────────────────────────

fn all_units() -> Vec<Vec<(usize, usize)>> {
    let mut units = Vec::with_capacity(27);
    for i in 0..9 {
        units.push((0..9).map(|c| (i, c)).collect()); // row i
        units.push((0..9).map(|r| (r, i)).collect()); // col i
        let br = (i / 3) * 3;
        let bc = (i % 3) * 3;
        units.push(
            (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc)))
                .collect(),
        );
    }
    units
}

fn sees(r1: usize, c1: usize, r2: usize, c2: usize) -> bool {
    r1 == r2 || c1 == c2 || (r1 / 3 == r2 / 3 && c1 / 3 == c2 / 3)
}

fn elim(row: usize, col: usize, digit: u8) -> Elimination {
    Elimination { row, col, digit, strategy: Strategy::Expert }
}

// ── Jellyfish ─────────────────────────────────────────────────────────────────

pub fn find_jellyfish(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        // Row-based
        let row_cols: Vec<Vec<usize>> = (0..9)
            .map(|r| (0..9).filter(|&c| cands.mask(r, c) != 0 && cands.has(r, c, digit)).collect())
            .collect();
        let cand_rows: Vec<usize> = (0..9)
            .filter(|&r| { let n = row_cols[r].len(); n >= 2 && n <= 4 })
            .collect();
        for i in 0..cand_rows.len() {
            for j in (i + 1)..cand_rows.len() {
                for k in (j + 1)..cand_rows.len() {
                    for l in (k + 1)..cand_rows.len() {
                        let (r1, r2, r3, r4) = (cand_rows[i], cand_rows[j], cand_rows[k], cand_rows[l]);
                        let mut cols = std::collections::BTreeSet::new();
                        for &c in &row_cols[r1] { cols.insert(c); }
                        for &c in &row_cols[r2] { cols.insert(c); }
                        for &c in &row_cols[r3] { cols.insert(c); }
                        for &c in &row_cols[r4] { cols.insert(c); }
                        if cols.len() != 4 { continue; }
                        for &c in &cols {
                            for r in 0..9 {
                                if r != r1 && r != r2 && r != r3 && r != r4
                                    && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                                {
                                    result.push(elim(r, c, digit));
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
        // Column-based
        let col_rows: Vec<Vec<usize>> = (0..9)
            .map(|c| (0..9).filter(|&r| cands.mask(r, c) != 0 && cands.has(r, c, digit)).collect())
            .collect();
        let cand_cols: Vec<usize> = (0..9)
            .filter(|&c| { let n = col_rows[c].len(); n >= 2 && n <= 4 })
            .collect();
        for i in 0..cand_cols.len() {
            for j in (i + 1)..cand_cols.len() {
                for k in (j + 1)..cand_cols.len() {
                    for l in (k + 1)..cand_cols.len() {
                        let (c1, c2, c3, c4) = (cand_cols[i], cand_cols[j], cand_cols[k], cand_cols[l]);
                        let mut rows = std::collections::BTreeSet::new();
                        for &r in &col_rows[c1] { rows.insert(r); }
                        for &r in &col_rows[c2] { rows.insert(r); }
                        for &r in &col_rows[c3] { rows.insert(r); }
                        for &r in &col_rows[c4] { rows.insert(r); }
                        if rows.len() != 4 { continue; }
                        for &r in &rows {
                            for c in 0..9 {
                                if c != c1 && c != c2 && c != c3 && c != c4
                                    && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                                {
                                    result.push(elim(r, c, digit));
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
    }
    result
}

// ── Naked Quad ────────────────────────────────────────────────────────────────

pub fn find_naked_quad(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for unit in all_units() {
        let small: Vec<(usize, usize, u16)> = unit
            .iter()
            .filter_map(|&(r, c)| {
                let m = cands.mask(r, c);
                let n = m.count_ones();
                if n >= 2 && n <= 4 { Some((r, c, m)) } else { None }
            })
            .collect();
        for i in 0..small.len() {
            for j in (i + 1)..small.len() {
                for k in (j + 1)..small.len() {
                    for l in (k + 1)..small.len() {
                        let combined = small[i].2 | small[j].2 | small[k].2 | small[l].2;
                        if combined.count_ones() != 4 { continue; }
                        let quad = [
                            (small[i].0, small[i].1),
                            (small[j].0, small[j].1),
                            (small[k].0, small[k].1),
                            (small[l].0, small[l].1),
                        ];
                        for &(r, c) in &unit {
                            if !quad.contains(&(r, c)) && cands.mask(r, c) != 0
                                && (cands.mask(r, c) & combined) != 0
                            {
                                for d in 1u8..=9 {
                                    if (combined & (1 << d)) != 0 && cands.has(r, c, d) {
                                        result.push(elim(r, c, d));
                                    }
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
    }
    result
}

// ── Hidden Triple ─────────────────────────────────────────────────────────────

pub fn find_hidden_triple(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for unit in all_units() {
        let empties: Vec<(usize, usize)> = unit
            .iter()
            .filter(|&&(r, c)| cands.mask(r, c) != 0)
            .copied()
            .collect();
        for d1 in 1u8..=9 {
            for d2 in (d1 + 1)..=9 {
                for d3 in (d2 + 1)..=9 {
                    let triple_mask = (1u16 << d1) | (1u16 << d2) | (1u16 << d3);
                    let triple_cells: Vec<(usize, usize)> = empties
                        .iter()
                        .filter(|&&(r, c)| (cands.mask(r, c) & triple_mask) != 0)
                        .copied()
                        .collect();
                    if triple_cells.len() != 3 { continue; }
                    let combined: u16 = triple_cells
                        .iter()
                        .fold(0u16, |acc, &(r, c)| acc | cands.mask(r, c));
                    if combined & triple_mask != triple_mask { continue; }
                    for &(r, c) in &triple_cells {
                        let extra = cands.mask(r, c) & !triple_mask;
                        for d in 1u8..=9 {
                            if (extra & (1 << d)) != 0 {
                                result.push(elim(r, c, d));
                            }
                        }
                    }
                    if !result.is_empty() { return result; }
                }
            }
        }
    }
    result
}

// ── Hidden Quad ───────────────────────────────────────────────────────────────

pub fn find_hidden_quad(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for unit in all_units() {
        let empties: Vec<(usize, usize)> = unit
            .iter()
            .filter(|&&(r, c)| cands.mask(r, c) != 0)
            .copied()
            .collect();
        for d1 in 1u8..=9 {
            for d2 in (d1 + 1)..=9 {
                for d3 in (d2 + 1)..=9 {
                    for d4 in (d3 + 1)..=9 {
                        let mask = (1u16 << d1) | (1u16 << d2) | (1u16 << d3) | (1u16 << d4);
                        let quad_cells: Vec<(usize, usize)> = empties
                            .iter()
                            .filter(|&&(r, c)| (cands.mask(r, c) & mask) != 0)
                            .copied()
                            .collect();
                        if quad_cells.len() != 4 { continue; }
                        let combined: u16 = quad_cells
                            .iter()
                            .fold(0u16, |acc, &(r, c)| acc | cands.mask(r, c));
                        if (combined & mask) != mask { continue; }
                        for &(r, c) in &quad_cells {
                            let extra = cands.mask(r, c) & !mask;
                            for d in 1u8..=9 {
                                if (extra & (1 << d)) != 0 {
                                    result.push(elim(r, c, d));
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
    }
    result
}

// ── Skyscraper ────────────────────────────────────────────────────────────────

pub fn find_skyscraper(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        // Row-based
        let row_cols: Vec<Vec<usize>> = (0..9)
            .map(|r| (0..9).filter(|&c| cands.mask(r, c) != 0 && cands.has(r, c, digit)).collect())
            .collect();
        for r1 in 0..9 {
            if row_cols[r1].len() != 2 { continue; }
            for r2 in (r1 + 1)..9 {
                if row_cols[r2].len() != 2 { continue; }
                let shared: Vec<usize> = row_cols[r1].iter()
                    .filter(|c| row_cols[r2].contains(c)).copied().collect();
                if shared.len() != 1 { continue; }
                let c_shared = shared[0];
                let ca = *row_cols[r1].iter().find(|&&c| c != c_shared).unwrap();
                let cb = *row_cols[r2].iter().find(|&&c| c != c_shared).unwrap();
                for r in 0..9 {
                    for c in 0..9 {
                        if (r, c) != (r1, ca) && (r, c) != (r2, cb)
                            && sees(r, c, r1, ca) && sees(r, c, r2, cb)
                            && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                        {
                            result.push(elim(r, c, digit));
                        }
                    }
                }
                if !result.is_empty() { return result; }
            }
        }
        // Column-based
        let col_rows: Vec<Vec<usize>> = (0..9)
            .map(|c| (0..9).filter(|&r| cands.mask(r, c) != 0 && cands.has(r, c, digit)).collect())
            .collect();
        for c1 in 0..9 {
            if col_rows[c1].len() != 2 { continue; }
            for c2 in (c1 + 1)..9 {
                if col_rows[c2].len() != 2 { continue; }
                let shared: Vec<usize> = col_rows[c1].iter()
                    .filter(|r| col_rows[c2].contains(r)).copied().collect();
                if shared.len() != 1 { continue; }
                let r_shared = shared[0];
                let ra = *col_rows[c1].iter().find(|&&r| r != r_shared).unwrap();
                let rb = *col_rows[c2].iter().find(|&&r| r != r_shared).unwrap();
                for r in 0..9 {
                    for c in 0..9 {
                        if (r, c) != (ra, c1) && (r, c) != (rb, c2)
                            && sees(r, c, ra, c1) && sees(r, c, rb, c2)
                            && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                        {
                            result.push(elim(r, c, digit));
                        }
                    }
                }
                if !result.is_empty() { return result; }
            }
        }
    }
    result
}

// ── 2-String Kite ─────────────────────────────────────────────────────────────

pub fn find_two_string_kite(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        for row in 0..9usize {
            let row_cs: Vec<usize> = (0..9)
                .filter(|&c| cands.mask(row, c) != 0 && cands.has(row, c, digit))
                .collect();
            if row_cs.len() != 2 { continue; }
            let (rc1, rc2) = (row_cs[0], row_cs[1]);
            for col in 0..9usize {
                let col_rs: Vec<usize> = (0..9)
                    .filter(|&r| cands.mask(r, col) != 0 && cands.has(r, col, digit))
                    .collect();
                if col_rs.len() != 2 { continue; }
                let (cr1, cr2) = (col_rs[0], col_rs[1]);
                let row_pair = [(row, rc1), (row, rc2)];
                let col_pair = [(cr1, col), (cr2, col)];
                for &(r_int, c_int) in &row_pair {
                    for &(r_col_int, c_col_int) in &col_pair {
                        if (r_int, c_int) == (r_col_int, c_col_int) { continue; }
                        if r_int / 3 != r_col_int / 3 || c_int / 3 != c_col_int / 3 { continue; }
                        let tip1 = *row_pair.iter().find(|&&rc| rc != (r_int, c_int)).unwrap();
                        let tip2 = *col_pair.iter().find(|&&rc| rc != (r_col_int, c_col_int)).unwrap();
                        for r in 0..9 {
                            for c in 0..9 {
                                if (r, c) != tip1 && (r, c) != tip2
                                    && sees(r, c, tip1.0, tip1.1) && sees(r, c, tip2.0, tip2.1)
                                    && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                                {
                                    result.push(elim(r, c, digit));
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
    }
    result
}

// ── Y-Wing ────────────────────────────────────────────────────────────────────

pub fn find_y_wing(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    let bi_cells: Vec<(usize, usize, u16)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter_map(|(r, c)| {
            let m = cands.mask(r, c);
            if m.count_ones() == 2 { Some((r, c, m)) } else { None }
        })
        .collect();
    for &(r0, c0, m0) in &bi_cells {
        let a = m0.trailing_zeros() as u8;
        let b = (m0 >> (a as u32 + 1)).trailing_zeros() as u8 + a + 1;
        for &(r1, c1, m1) in &bi_cells {
            if (r1, c1) == (r0, c0) { continue; }
            if !sees(r0, c0, r1, c1) { continue; }
            let shared = m0 & m1;
            if shared.count_ones() != 1 { continue; }
            let shared_ab = shared.trailing_zeros() as u8;
            let c_digit = (m1 & !shared).trailing_zeros() as u8;
            let other_ab = if shared_ab == a { b } else { a };
            let needed_m2 = (1u16 << other_ab) | (1u16 << c_digit);
            for &(r2, c2, m2) in &bi_cells {
                if (r2, c2) == (r0, c0) || (r2, c2) == (r1, c1) { continue; }
                if !sees(r0, c0, r2, c2) { continue; }
                if m2 != needed_m2 { continue; }
                for r in 0..9 {
                    for c in 0..9 {
                        if (r, c) != (r0, c0) && (r, c) != (r1, c1) && (r, c) != (r2, c2)
                            && cands.mask(r, c) != 0
                            && sees(r, c, r1, c1) && sees(r, c, r2, c2)
                            && cands.has(r, c, c_digit)
                        {
                            result.push(elim(r, c, c_digit));
                        }
                    }
                }
                if !result.is_empty() { return result; }
            }
        }
    }
    result
}

// ── XYZ-Wing ──────────────────────────────────────────────────────────────────

pub fn find_xyz_wing(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    let bivalue: Vec<(usize, usize, u16)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter_map(|(r, c)| {
            let m = cands.mask(r, c);
            if m.count_ones() == 2 { Some((r, c, m)) } else { None }
        })
        .collect();
    let trivalue: Vec<(usize, usize, u16)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter_map(|(r, c)| {
            let m = cands.mask(r, c);
            if m.count_ones() == 3 { Some((r, c, m)) } else { None }
        })
        .collect();
    for &(pr, pc, pm) in &trivalue {
        let digits: Vec<u8> = (1u8..=9).filter(|&d| pm & (1 << d) != 0).collect();
        if digits.len() != 3 { continue; }
        let (da, db, dc) = (digits[0], digits[1], digits[2]);
        for &c_digit in &[da, db, dc] {
            let others: Vec<u8> = [da, db, dc].iter().filter(|&&d| d != c_digit).copied().collect();
            let wing_masks = [
                (1u16 << others[0]) | (1u16 << c_digit),
                (1u16 << others[1]) | (1u16 << c_digit),
            ];
            let wings: [Vec<(usize, usize)>; 2] = [
                bivalue.iter()
                    .filter(|&&(r, c, m)| m == wing_masks[0] && sees(r, c, pr, pc))
                    .map(|&(r, c, _)| (r, c)).collect(),
                bivalue.iter()
                    .filter(|&&(r, c, m)| m == wing_masks[1] && sees(r, c, pr, pc))
                    .map(|&(r, c, _)| (r, c)).collect(),
            ];
            for &w1 in &wings[0] {
                for &w2 in &wings[1] {
                    if w1 == w2 { continue; }
                    // Eliminate c_digit from cells seeing all three: pivot, w1, w2
                    for r in 0..9 {
                        for c in 0..9 {
                            if (r, c) == (pr, pc) || (r, c) == w1 || (r, c) == w2 { continue; }
                            if cands.mask(r, c) == 0 { continue; }
                            if !cands.has(r, c, c_digit) { continue; }
                            if sees(r, c, pr, pc) && sees(r, c, w1.0, w1.1) && sees(r, c, w2.0, w2.1) {
                                result.push(elim(r, c, c_digit));
                            }
                        }
                    }
                    if !result.is_empty() { return result; }
                }
            }
        }
    }
    result
}

// ── W-Wing ────────────────────────────────────────────────────────────────────

pub fn find_w_wing(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    let pairs: Vec<(usize, usize)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter(|&(r, c)| cands.mask(r, c).count_ones() == 2)
        .collect();
    for &p1 in &pairs {
        for &p2 in &pairs {
            if p1 >= p2 { continue; }
            if cands.mask(p1.0, p1.1) != cands.mask(p2.0, p2.1) { continue; }
            if sees(p1.0, p1.1, p2.0, p2.1) { continue; }
            let pair_mask = cands.mask(p1.0, p1.1);
            let a = pair_mask.trailing_zeros() as u8;
            let b = (pair_mask >> (a as u32 + 1)).trailing_zeros() as u8 + a + 1;
            // Need strong link on `a` connecting p1 and p2 through a unit
            for unit in all_units() {
                let unit_a: Vec<(usize, usize)> = unit.iter()
                    .filter(|&&(r, c)| cands.mask(r, c) != 0 && cands.has(r, c, a))
                    .copied().collect();
                if unit_a.len() != 2 { continue; }
                let (e1, e2) = (unit_a[0], unit_a[1]);
                // p1 sees e1 and p2 sees e2 (or vice versa)
                for &(ea, eb) in &[(e1, e2), (e2, e1)] {
                    if ea == p1 || eb == p2 { continue; }
                    if !sees(p1.0, p1.1, ea.0, ea.1) { continue; }
                    if !sees(p2.0, p2.1, eb.0, eb.1) { continue; }
                    // Eliminate b from cells seeing both p1 and p2
                    for r in 0..9 {
                        for c in 0..9 {
                            if (r, c) == p1 || (r, c) == p2 { continue; }
                            if cands.mask(r, c) == 0 { continue; }
                            if !cands.has(r, c, b) { continue; }
                            if sees(r, c, p1.0, p1.1) && sees(r, c, p2.0, p2.1) {
                                result.push(elim(r, c, b));
                            }
                        }
                    }
                    if !result.is_empty() { return result; }
                }
            }
        }
    }
    result
}

// ── Unique Rectangle ──────────────────────────────────────────────────────────

pub fn find_unique_rectangle(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for r1 in 0..9usize {
        for r2 in (r1 + 1)..9 {
            for c1 in 0..9usize {
                for c2 in (c1 + 1)..9 {
                    let corners = [(r1, c1), (r1, c2), (r2, c1), (r2, c2)];
                    // All corners must be empty (non-zero mask)
                    if corners.iter().any(|&(r, c)| cands.mask(r, c) == 0) { continue; }
                    // Must span exactly 2 different boxes
                    let boxes: std::collections::HashSet<usize> =
                        corners.iter().map(|&(r, c)| (r / 3) * 3 + c / 3).collect();
                    if boxes.len() != 2 { continue; }
                    for a in 1u8..=9 {
                        for b in (a + 1)..=9 {
                            let pair_mask = (1u16 << a) | (1u16 << b);
                            if corners.iter().any(|&(r, c)| cands.mask(r, c) & pair_mask != pair_mask) {
                                continue;
                            }
                            // Type 1: 3 corners have ONLY {a,b}, 1 roof has extras
                            let only_pair: Vec<(usize, usize)> = corners.iter()
                                .filter(|&&(r, c)| cands.mask(r, c) == pair_mask)
                                .copied().collect();
                            let has_extras: Vec<(usize, usize)> = corners.iter()
                                .filter(|&&(r, c)| cands.mask(r, c) & !pair_mask != 0)
                                .copied().collect();
                            if only_pair.len() == 3 && has_extras.len() == 1 {
                                let (rr, cc) = has_extras[0];
                                for d in [a, b] {
                                    if cands.has(rr, cc, d) {
                                        result.push(elim(rr, cc, d));
                                    }
                                }
                                if !result.is_empty() { return result; }
                            }
                            // Type 2: 2 floors {a,b}, 2 roofs {a,b,c} on same side
                            if has_extras.len() != 2 || only_pair.len() != 2 { continue; }
                            let (roof_a, roof_b) = (has_extras[0], has_extras[1]);
                            let same_side = roof_a.0 == roof_b.0 || roof_a.1 == roof_b.1;
                            if !same_side { continue; }
                            let extra_a = cands.mask(roof_a.0, roof_a.1) & !pair_mask;
                            let extra_b = cands.mask(roof_b.0, roof_b.1) & !pair_mask;
                            if extra_a.count_ones() != 1 || extra_a != extra_b { continue; }
                            let c_digit = extra_a.trailing_zeros() as u8;
                            for r in 0..9 {
                                for c in 0..9 {
                                    if corners.contains(&(r, c)) { continue; }
                                    if cands.mask(r, c) == 0 { continue; }
                                    if !cands.has(r, c, c_digit) { continue; }
                                    if sees(r, c, roof_a.0, roof_a.1) && sees(r, c, roof_b.0, roof_b.1) {
                                        result.push(elim(r, c, c_digit));
                                    }
                                }
                            }
                            if !result.is_empty() { return result; }
                        }
                    }
                }
            }
        }
    }
    result
}

// ── BUG+1 ─────────────────────────────────────────────────────────────────────
// Returns a placement (SolveStep), not an elimination.

pub fn find_bug_plus_one_step(cands: &CandidateGrid) -> Option<SolveStep> {
    let mut trivalue_cell: Option<(usize, usize)> = None;
    for r in 0..9 {
        for c in 0..9 {
            let n = cands.mask(r, c).count_ones();
            if n == 0 { continue; } // filled cell
            if n == 3 {
                if trivalue_cell.is_some() { return None; } // more than 1 trivalue
                trivalue_cell = Some((r, c));
            } else if n != 2 {
                return None; // non-bivalue non-trivalue cell
            }
        }
    }
    let (br, bc) = trivalue_cell?;
    let mask = cands.mask(br, bc);
    let digits: Vec<u8> = (1u8..=9).filter(|&d| mask & (1 << d) != 0).collect();
    if digits.len() != 3 { return None; }
    for &d in &digits {
        let row_count = (0..9).filter(|&c| c != bc && cands.mask(br, c) != 0 && cands.has(br, c, d)).count();
        let col_count = (0..9).filter(|&r| r != br && cands.mask(r, bc) != 0 && cands.has(r, bc, d)).count();
        let box_br = (br / 3) * 3;
        let box_bc = (bc / 3) * 3;
        let box_count = (0..3).flat_map(|dr| (0..3).map(move |dc| (box_br + dr, box_bc + dc)))
            .filter(|&(r, c)| (r, c) != (br, bc) && cands.mask(r, c) != 0 && cands.has(r, c, d))
            .count();
        if row_count % 2 == 1 && col_count % 2 == 1 && box_count % 2 == 1 {
            return Some(SolveStep {
                row: br,
                col: bc,
                digit: d,
                strategy: Strategy::Expert,
                source_cells: vec![],
            });
        }
    }
    None
}

// ── Empty Rectangle ───────────────────────────────────────────────────────────

pub fn find_empty_rectangle(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        for box_idx in 0..9usize {
            let box_row = (box_idx / 3) * 3;
            let box_col = (box_idx % 3) * 3;
            let box_cells: Vec<(usize, usize)> = (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (box_row + dr, box_col + dc)))
                .filter(|&(r, c)| cands.mask(r, c) != 0 && cands.has(r, c, digit))
                .collect();
            if box_cells.len() < 2 { continue; }
            // ER on a row: all cells in the same row
            let er_row = box_cells[0].0;
            if box_cells.iter().all(|&(r, _)| r == er_row) {
                for c_conj in 0..9usize {
                    if c_conj / 3 == box_col / 3 { continue; }
                    let col_cells: Vec<usize> = (0..9)
                        .filter(|&r| cands.mask(r, c_conj) != 0 && cands.has(r, c_conj, digit))
                        .collect();
                    if col_cells.len() != 2 { continue; }
                    let (r_a, r_b) = (col_cells[0], col_cells[1]);
                    let r_other = if r_a == er_row { r_b } else if r_b == er_row { r_a } else { continue };
                    for c_er in box_col..(box_col + 3) {
                        if c_er == c_conj { continue; }
                        if cands.mask(r_other, c_er) == 0 { continue; }
                        if !cands.has(r_other, c_er, digit) { continue; }
                        result.push(elim(r_other, c_er, digit));
                    }
                    if !result.is_empty() { return result; }
                }
            }
            // ER on a column: all cells in the same col
            let er_col = box_cells[0].1;
            if box_cells.iter().all(|&(_, c)| c == er_col) {
                for r_conj in 0..9usize {
                    if r_conj / 3 == box_row / 3 { continue; }
                    let row_cells: Vec<usize> = (0..9)
                        .filter(|&c| cands.mask(r_conj, c) != 0 && cands.has(r_conj, c, digit))
                        .collect();
                    if row_cells.len() != 2 { continue; }
                    let (c_a, c_b) = (row_cells[0], row_cells[1]);
                    let c_other = if c_a == er_col { c_b } else if c_b == er_col { c_a } else { continue };
                    for r_er in box_row..(box_row + 3) {
                        if r_er == r_conj { continue; }
                        if cands.mask(r_er, c_other) == 0 { continue; }
                        if !cands.has(r_er, c_other, digit) { continue; }
                        result.push(elim(r_er, c_other, digit));
                    }
                    if !result.is_empty() { return result; }
                }
            }
        }
    }
    result
}

// ── Simple Coloring ───────────────────────────────────────────────────────────

pub fn find_simple_coloring(cands: &CandidateGrid) -> Vec<Elimination> {
    use std::collections::HashMap;
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        let mut links: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();
        for unit in all_units() {
            let cells_d: Vec<(usize, usize)> = unit.iter()
                .filter(|&&(r, c)| cands.mask(r, c) != 0 && cands.has(r, c, digit))
                .copied().collect();
            if cells_d.len() == 2 {
                links.entry(cells_d[0]).or_default().push(cells_d[1]);
                links.entry(cells_d[1]).or_default().push(cells_d[0]);
            }
        }
        if links.is_empty() { continue; }
        let mut color_map: HashMap<(usize, usize), u8> = HashMap::new();
        let all_linked: Vec<(usize, usize)> = links.keys().copied().collect();
        for &start in &all_linked {
            if color_map.contains_key(&start) { continue; }
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(start);
            color_map.insert(start, 0);
            let mut component: Vec<(usize, usize)> = vec![start];
            while let Some(cell) = queue.pop_front() {
                let cell_color = color_map[&cell];
                let next_color = 1 - cell_color;
                if let Some(neighbors) = links.get(&cell) {
                    for &nb in neighbors {
                        if !color_map.contains_key(&nb) {
                            color_map.insert(nb, next_color);
                            component.push(nb);
                            queue.push_back(nb);
                        }
                    }
                }
            }
            let color0: Vec<(usize, usize)> = component.iter()
                .filter(|&&c| color_map[&c] == 0).copied().collect();
            let color1: Vec<(usize, usize)> = component.iter()
                .filter(|&&c| color_map[&c] == 1).copied().collect();
            // Color Wrap: two same-color cells see each other
            for col_group in [&color0, &color1] {
                let mut wrap_found = false;
                'outer: for i in 0..col_group.len() {
                    for j in (i + 1)..col_group.len() {
                        let (r1, c1) = col_group[i];
                        let (r2, c2) = col_group[j];
                        if sees(r1, c1, r2, c2) {
                            wrap_found = true;
                            break 'outer;
                        }
                    }
                }
                if wrap_found {
                    for &(r, c) in col_group {
                        if cands.mask(r, c) != 0 && cands.has(r, c, digit) {
                            result.push(elim(r, c, digit));
                        }
                    }
                    if !result.is_empty() { return result; }
                }
            }
            // Color Trap: external cell sees both colors
            for r in 0..9usize {
                for c in 0..9usize {
                    if color_map.contains_key(&(r, c)) { continue; }
                    if cands.mask(r, c) == 0 { continue; }
                    if !cands.has(r, c, digit) { continue; }
                    let seen0 = color0.iter().any(|&(r2, c2)| sees(r, c, r2, c2));
                    let seen1 = color1.iter().any(|&(r2, c2)| sees(r, c, r2, c2));
                    if seen0 && seen1 {
                        result.push(elim(r, c, digit));
                    }
                }
            }
            if !result.is_empty() { return result; }
        }
    }
    result
}

// ── XY-Chain ──────────────────────────────────────────────────────────────────

pub fn find_xy_chain(cands: &CandidateGrid) -> Vec<Elimination> {
    let bivalue: Vec<(usize, usize, u16)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter_map(|(r, c)| {
            let m = cands.mask(r, c);
            if m.count_ones() == 2 { Some((r, c, m)) } else { None }
        })
        .collect();
    const MAX_DEPTH: usize = 8;
    for &(sr, sc, sm) in &bivalue {
        let elim_d = sm.trailing_zeros() as u8;
        let x = (sm >> (elim_d as u32 + 1)).trailing_zeros() as u8 + elim_d + 1;
        let mut chain: Vec<(usize, usize)> = vec![(sr, sc)];
        if let Some(r) = xy_chain_dfs(&mut chain, x, elim_d, &bivalue, cands, MAX_DEPTH) {
            return r;
        }
    }
    vec![]
}

fn xy_chain_dfs(
    chain: &mut Vec<(usize, usize)>,
    incoming: u8,
    elim_d: u8,
    bivalue: &[(usize, usize, u16)],
    cands: &CandidateGrid,
    max_depth: usize,
) -> Option<Vec<Elimination>> {
    if chain.len() >= max_depth { return None; }
    let &(cur_r, cur_c) = chain.last().unwrap();
    for &(nr, nc, nm) in bivalue {
        if chain.contains(&(nr, nc)) { continue; }
        if !sees(cur_r, cur_c, nr, nc) { continue; }
        if (nm & (1 << incoming)) == 0 { continue; }
        let other = if nm.trailing_zeros() as u8 == incoming {
            (nm >> (incoming as u32 + 1)).trailing_zeros() as u8 + incoming + 1
        } else {
            nm.trailing_zeros() as u8
        };
        chain.push((nr, nc));
        if other == elim_d && chain.len() >= 3 {
            let (start_r, start_c) = chain[0];
            let mut result = Vec::new();
            for r in 0..9 {
                for c in 0..9 {
                    if chain.contains(&(r, c)) { continue; }
                    if cands.mask(r, c) == 0 { continue; }
                    if !cands.has(r, c, elim_d) { continue; }
                    if sees(r, c, start_r, start_c) && sees(r, c, nr, nc) {
                        result.push(elim(r, c, elim_d));
                    }
                }
            }
            if !result.is_empty() {
                chain.pop();
                return Some(result);
            }
        }
        if let Some(r) = xy_chain_dfs(chain, other, elim_d, bivalue, cands, max_depth) {
            chain.pop();
            return Some(r);
        }
        chain.pop();
    }
    None
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::candidates::CandidateGrid;
    use crate::puzzle::Grid;

    // Helper: blank CandidateGrid with all 9 candidates in every cell.
    fn blank_cands() -> CandidateGrid {
        CandidateGrid::from_grid(
            &Grid::from_str("000000000000000000000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        )
    }

    // Helper: remove all candidates except `keep` from a cell.
    fn keep_only(c: &mut CandidateGrid, row: usize, col: usize, keep: &[u8]) {
        for d in 1u8..=9 {
            if !keep.contains(&d) {
                c.remove(row, col, d);
            }
        }
    }

    // Helper: remove digit `d` from every cell except those in `except`.
    fn remove_digit_except(c: &mut CandidateGrid, d: u8, except: &[(usize, usize)]) {
        for r in 0..9 {
            for col in 0..9 {
                if !except.contains(&(r, col)) {
                    c.remove(r, col, d);
                }
            }
        }
    }

    // ── Blank grid returns empty (negative tests) ──────────────────────────────

    #[test]
    fn jellyfish_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_jellyfish(&c).is_empty());
    }

    #[test]
    fn naked_quad_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_naked_quad(&c).is_empty());
    }

    #[test]
    fn hidden_triple_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_hidden_triple(&c).is_empty());
    }

    #[test]
    fn hidden_quad_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_hidden_quad(&c).is_empty());
    }

    #[test]
    fn skyscraper_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_skyscraper(&c).is_empty());
    }

    #[test]
    fn two_string_kite_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_two_string_kite(&c).is_empty());
    }

    #[test]
    fn y_wing_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_y_wing(&c).is_empty());
    }

    #[test]
    fn xyz_wing_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_xyz_wing(&c).is_empty());
    }

    #[test]
    fn w_wing_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_w_wing(&c).is_empty());
    }

    #[test]
    fn unique_rectangle_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_unique_rectangle(&c).is_empty());
    }

    #[test]
    fn bug_plus_one_returns_none_on_blank_grid() {
        let c = blank_cands();
        assert!(find_bug_plus_one_step(&c).is_none());
    }

    #[test]
    fn empty_rectangle_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_empty_rectangle(&c).is_empty());
    }

    #[test]
    fn simple_coloring_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_simple_coloring(&c).is_empty());
    }

    #[test]
    fn xy_chain_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_xy_chain(&c).is_empty());
    }

    // ── Positive tests ─────────────────────────────────────────────────────────
    //
    // Each test constructs a minimal CandidateGrid where exactly one strategy
    // pattern is present, then asserts ≥1 elimination is returned.
    // All setups start from blank_cands() and use remove_digit_except / keep_only.

    // ── Positive tests: Jellyfish ─────────────────────────────────────────────

    // Digit 3 in rows 0,2,5,7 confined to cols {1,4,6,8} — 4 rows × 4 cols.
    // Row 3 col 1 is the victim (not in the 4 fish rows).
    #[test]
    fn jellyfish_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        let fish_rows = [0usize, 2, 5, 7];
        let fish_cols = [1usize, 4, 6, 8];
        for r in 0..9 {
            for col in 0..9 {
                if fish_rows.contains(&r) {
                    if !fish_cols.contains(&col) { c.remove(r, col, 3); }
                } else {
                    if !(r == 3 && col == 1) { c.remove(r, col, 3); }
                }
            }
        }
        let elims = find_jellyfish(&c);
        assert!(!elims.is_empty(), "jellyfish should find eliminations");
        assert!(elims.iter().any(|e| e.digit == 3));
    }

    // ── Positive tests: Naked Quad ────────────────────────────────────────────

    // Row 0 cells (0,0)–(0,3): candidates restricted to {1,2,3,4}.
    // Cell (0,4) still has digits 1–4 → they should be eliminated.
    #[test]
    fn naked_quad_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        for col in 0..4usize { keep_only(&mut c, 0, col, &[1, 2, 3, 4]); }
        for col in 5..9usize { for d in [1u8,2,3,4] { c.remove(0, col, d); } }
        // (0,4) still has all 9 candidates including 1–4
        let elims = find_naked_quad(&c);
        assert!(!elims.is_empty(), "naked_quad should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col == 4));
    }

    // ── Positive tests: Hidden Triple ─────────────────────────────────────────

    // Row 0: digits {1,2,3} only in cells (0,0),(0,1),(0,2) which also have digit 5.
    // Digit 5 must be eliminated from the triple cells.
    #[test]
    fn hidden_triple_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        // Remove digits 1,2,3 from cols 3–8 of row 0
        for col in 3..9usize { for d in [1u8,2,3] { c.remove(0, col, d); } }
        // Triple cells have {1,2,3,5} only
        for col in 0..3usize { for d in [4u8,6,7,8,9] { c.remove(0, col, d); } }
        let elims = find_hidden_triple(&c);
        assert!(!elims.is_empty(), "hidden_triple should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col < 3 && e.digit == 5));
    }

    // ── Positive tests: Hidden Quad ───────────────────────────────────────────

    // Row 0: digits {1,2,3,4} only in cells (0,0)–(0,3) which also have digit 5.
    #[test]
    fn hidden_quad_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        for col in 4..9usize { for d in [1u8,2,3,4] { c.remove(0, col, d); } }
        for col in 0..4usize { for d in [6u8,7,8,9] { c.remove(0, col, d); } }
        // cells (0,0)–(0,3) now have {1,2,3,4,5}; digit 5 is the extra to eliminate
        let elims = find_hidden_quad(&c);
        assert!(!elims.is_empty(), "hidden_quad should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col < 4 && e.digit == 5));
    }

    // ── Positive tests: Skyscraper ────────────────────────────────────────────

    // Digit 5: rows 0 and 5 share col 2 (trunk). Tips (0,6) and (5,8).
    // Victim (3,6) sees (0,6) via col 6 and (5,8) via box 5.
    #[test]
    fn skyscraper_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        remove_digit_except(&mut c, 5, &[(0,2),(0,6),(5,2),(5,8),(3,6)]);
        for &(r,col) in &[(0usize,2),(0,6),(5,2),(5,8)] { keep_only(&mut c, r, col, &[5]); }
        let elims = find_skyscraper(&c);
        assert!(!elims.is_empty(), "skyscraper should find eliminations");
        assert!(elims.iter().any(|e| e.row == 3 && e.col == 6 && e.digit == 5));
    }

    // ── Positive tests: 2-String Kite ────────────────────────────────────────

    // Digit 2. Row 0 strong link (0,0)↔(0,3). Col 0 strong link (0,0)↔(2,0).
    // Intersection at (0,0). Tips: (0,3) and (2,0).
    // Victim (2,3) sees (0,3) via col 3 and (2,0) via row 2.
    #[test]
    fn two_string_kite_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        remove_digit_except(&mut c, 2, &[(0,0),(0,3),(2,0),(2,3)]);
        keep_only(&mut c, 0, 0, &[2]);
        keep_only(&mut c, 0, 3, &[2]);
        keep_only(&mut c, 2, 0, &[2]);
        let elims = find_two_string_kite(&c);
        assert!(!elims.is_empty(), "two_string_kite should find eliminations");
        assert!(elims.iter().any(|e| e.digit == 2));
    }

    // ── Positive tests: Y-Wing ────────────────────────────────────────────────

    // Pivot (4,4){1,3}, wing1 (4,0){1,2}, wing2 (0,4){2,3}. Victim (0,0) loses 2.
    #[test]
    fn y_wing_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 4, 4, &[1, 3]);
        keep_only(&mut c, 4, 0, &[1, 2]);
        keep_only(&mut c, 0, 4, &[2, 3]);
        remove_digit_except(&mut c, 2, &[(4,0),(0,4),(0,0)]);
        let elims = find_y_wing(&c);
        assert!(!elims.is_empty(), "y_wing should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col == 0 && e.digit == 2));
    }

    // ── Positive tests: XYZ-Wing ──────────────────────────────────────────────

    // Pivot (2,2){1,2,3}. Wing1 (2,0){1,3} same row. Wing2 (0,2){2,3} same col.
    // Victim (0,0) sees pivot via box 0, wing1 via col 0, wing2 via row 0. Loses digit 3.
    #[test]
    fn xyz_wing_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 2, 2, &[1, 2, 3]);
        keep_only(&mut c, 2, 0, &[1, 3]);
        keep_only(&mut c, 0, 2, &[2, 3]);
        remove_digit_except(&mut c, 3, &[(2,2),(2,0),(0,2),(0,0)]);
        let elims = find_xyz_wing(&c);
        assert!(!elims.is_empty(), "xyz_wing should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col == 0 && e.digit == 3));
    }

    // ── Positive tests: W-Wing ────────────────────────────────────────────────

    // p1=(0,0){1,2}, p2=(5,5){1,2} (don't see each other).
    // Strong link on digit 1 in col 3: e1=(0,3), e2=(5,3).
    // p1 sees e1 via row 0. p2 sees e2 via row 5.
    // Victim (0,5) sees p1 via row 0 and p2 via col 5. Loses digit 2.
    #[test]
    fn w_wing_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 0, 0, &[1, 2]);
        keep_only(&mut c, 5, 5, &[1, 2]);
        keep_only(&mut c, 0, 3, &[1]);
        keep_only(&mut c, 5, 3, &[1]);
        remove_digit_except(&mut c, 1, &[(0,0),(5,5),(0,3),(5,3)]);
        remove_digit_except(&mut c, 2, &[(0,0),(5,5),(0,5)]);
        let elims = find_w_wing(&c);
        assert!(!elims.is_empty(), "w_wing should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col == 5 && e.digit == 2));
    }

    // ── Positive tests: Unique Rectangle ─────────────────────────────────────

    // Type 1: corners (0,0),(0,3),(6,0),(6,3) span 2 boxes.
    // 3 corners {1,2} only; roof (6,3) has {1,2,5}. Eliminate 1 and 2 from roof.
    #[test]
    fn unique_rectangle_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 0, 0, &[1, 2]);
        keep_only(&mut c, 0, 3, &[1, 2]);
        keep_only(&mut c, 6, 0, &[1, 2]);
        keep_only(&mut c, 6, 3, &[1, 2, 5]);
        for r in 0..9 { for col in 0..9 {
            if ![(0usize,0),(0,3),(6,0),(6,3)].contains(&(r,col)) {
                c.remove(r, col, 1); c.remove(r, col, 2);
            }
        }}
        let elims = find_unique_rectangle(&c);
        assert!(!elims.is_empty(), "unique_rectangle should find eliminations");
        assert!(elims.iter().any(|e| e.row == 6 && e.col == 3));
    }

    // ── Positive tests: Empty Rectangle ──────────────────────────────────────

    // Box 0 (rows 0-2, cols 0-2): digit 7 confined to row 0 at cols 0 and 1.
    // Conjugate pair in col 6: (0,6)↔(4,6). r_other = row 4.
    // Eliminate digit 7 from (4,0) or (4,1) — within box 0 cols, row 4.
    #[test]
    fn empty_rectangle_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        remove_digit_except(&mut c, 7, &[(0,0),(0,1),(0,6),(4,6),(4,1)]);
        keep_only(&mut c, 0, 0, &[7]);
        keep_only(&mut c, 0, 1, &[7]);
        keep_only(&mut c, 0, 6, &[7]);
        keep_only(&mut c, 4, 6, &[7]);
        let elims = find_empty_rectangle(&c);
        assert!(!elims.is_empty(), "empty_rectangle should find eliminations");
        assert!(elims.iter().any(|e| e.digit == 7));
    }

    // ── Positive tests: Simple Coloring ──────────────────────────────────────

    // Color Wrap: digit 7. Chain (0,0)-(3,0)-(3,6)-(0,6)-(0,3).
    // c0=(0,0),(3,6),(0,3); c1=(3,0),(0,6). (0,0)&(0,3) both c0 share row 0 → Wrap.
    #[test]
    fn simple_coloring_detects_color_wrap_and_eliminates() {
        let mut c = blank_cands();
        let chain = [(0usize,0),(3,0),(3,6),(0,6),(0,3)];
        remove_digit_except(&mut c, 7, &chain);
        for &(r,col) in &chain { keep_only(&mut c, r, col, &[7]); }
        let elims = find_simple_coloring(&c);
        assert!(!elims.is_empty(), "simple_coloring should find eliminations");
        assert!(elims.iter().all(|e| e.digit == 7));
        let cells: Vec<_> = elims.iter().map(|e| (e.row, e.col)).collect();
        for &cell in &[(0usize,0),(3,6),(0,3)] {
            assert!(cells.contains(&cell), "expected {:?} in elims, got {:?}", cell, cells);
        }
    }

    // ── Positive tests: XY-Chain ──────────────────────────────────────────────

    // 3-cell chain: (0,0){1,2}→(0,3){2,3}→(4,3){3,1}. Victim (4,0) loses digit 1.
    #[test]
    fn xy_chain_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 0, 0, &[1, 2]);
        keep_only(&mut c, 0, 3, &[2, 3]);
        keep_only(&mut c, 4, 3, &[3, 1]);
        remove_digit_except(&mut c, 1, &[(0,0),(4,3),(4,0)]);
        let elims = find_xy_chain(&c);
        assert!(!elims.is_empty(), "xy_chain should find eliminations");
        assert!(elims.iter().any(|e| e.row == 4 && e.col == 0 && e.digit == 1));
    }

    // ── BUG+1 ─────────────────────────────────────────────────────────────────

    // BUG+1 requires a near-complete board that cannot be constructed from scratch
    // via remove-only CandidateGrid API. This test verifies the negative case;
    // the positive case is covered by the integration test in Task 7.
    #[test]
    fn bug_plus_one_returns_none_on_non_bug_board() {
        let mut c = blank_cands();
        for r in 0..9 { for col in 0..9 {
            for d in 3u8..=9 {
                if (r, col) == (4, 4) && d == 3 { continue; }
                c.remove(r, col, d);
            }
        }}
        assert!(find_bug_plus_one_step(&c).is_none());
    }
}
```

- [ ] **Step 4: Verify `expert.rs` is staged correctly**

Note: `expert.rs` won't compile as part of the crate until Task 4 adds `pub mod expert;` to `solver/mod.rs`. That's fine — commit the file now and the compilation happens as part of Task 4.

```bash
cd /Users/alexandererben/Tresors/OrdiSync/6_Entwicklung/Claude/SudokuCLI
cargo build 2>&1 | grep "^error" | head -5
```

Expected: one error "file not found for module `expert`" — this is fixed in Task 4.

- [ ] **Step 5: Commit**

```bash
git add src/solver/expert.rs
git commit -m "feat: add src/solver/expert.rs with 14 expert solver functions and unit tests"
```

---
---

## Task 4: Update `src/solver/mod.rs`

**Files:**
- Modify: `src/solver/mod.rs`

Changes:
1. Add `pub mod expert;` at top
2. Add `Strategy::Expert` to `strategy_order()`
3. Add `for_difficulty(Expert)` arm
4. Add Expert block in `solve()` after Swordfish

- [ ] **Step 1: Write failing tests**

Add to the `#[cfg(test)]` section in `src/solver/mod.rs`:

```rust
// This puzzle string must be found during implementation (Task 7).
// Placeholder — replace KNOWN_EXPERT_PUZZLE with actual string once found.
// const KNOWN_EXPERT_PUZZLE: &str = "TODO";

#[test]
fn for_difficulty_expert_has_correct_config() {
    let solver = Solver::for_difficulty(&crate::generator::difficulty::Difficulty::Expert);
    assert_eq!(solver.max_strategy, Some(Strategy::Expert));
    assert!(!solver.use_backtracking);
}

#[test]
fn expert_comes_after_swordfish_in_strategy_order() {
    let order = Solver::strategy_order();
    let swordfish_pos = order.iter().position(|&s| s == Strategy::Swordfish).unwrap();
    let expert_pos = order.iter().position(|&s| s == Strategy::Expert).unwrap();
    assert!(expert_pos > swordfish_pos, "Expert must come after Swordfish");
}

#[test]
fn expert_comes_before_backtracking_in_strategy_order() {
    let order = Solver::strategy_order();
    let expert_pos = order.iter().position(|&s| s == Strategy::Expert).unwrap();
    let back_pos = order.iter().position(|&s| s == Strategy::Backtracking).unwrap();
    assert!(expert_pos < back_pos, "Expert must come before Backtracking");
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test for_difficulty_expert_has_correct_config expert_comes_after_swordfish expert_comes_before_backtracking 2>&1 | tail -10
```

Expected: compile errors.

- [ ] **Step 3: Apply all changes to `src/solver/mod.rs`**

Replace the entire file with:

```rust
pub mod backtracking;
pub mod box_line_reduction;
pub mod candidates;
pub mod expert;
pub mod hidden_pair;
pub mod hidden_single;
pub mod naked_pair;
pub mod naked_single;
pub mod naked_triple;
pub mod pointing_pair;
pub mod swordfish;
pub mod x_wing;

pub use candidates::{CandidateGrid, Elimination, SolveStep, Strategy};

use crate::puzzle::Grid;
use std::collections::HashSet;

pub struct SolveResult {
    pub grid: Grid,
    pub used_strategies: Vec<Strategy>,
    pub steps: Vec<SolveStep>,
}

pub struct Solver {
    pub max_strategy: Option<Strategy>,
    pub use_backtracking: bool,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            max_strategy: None,
            use_backtracking: true,
        }
    }

    pub fn for_difficulty(difficulty: &crate::generator::difficulty::Difficulty) -> Self {
        use crate::generator::difficulty::Difficulty;
        match difficulty {
            Difficulty::Easy => Self {
                max_strategy: Some(Strategy::HiddenSingle),
                use_backtracking: false,
            },
            Difficulty::Medium => Self {
                max_strategy: Some(Strategy::PointingPair),
                use_backtracking: false,
            },
            Difficulty::Hard => Self {
                max_strategy: Some(Strategy::XWing),
                use_backtracking: false,
            },
            Difficulty::Extreme => Self {
                max_strategy: Some(Strategy::Swordfish),
                use_backtracking: false,
            },
            Difficulty::Expert => Self {
                max_strategy: Some(Strategy::Expert),
                use_backtracking: false,
            },
            Difficulty::BareMinimum => Self {
                max_strategy: None,
                use_backtracking: true,
            },
        }
    }

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
            Strategy::Swordfish,
            Strategy::Expert,
            Strategy::Backtracking,
        ]
    }

    fn allowed(&self, s: Strategy) -> bool {
        if s == Strategy::Backtracking {
            return self.use_backtracking;
        }
        match self.max_strategy {
            None => true,
            Some(max) => {
                let order = Self::strategy_order();
                let pos_s = order.iter().position(|&x| x == s).unwrap_or(usize::MAX);
                let pos_max = order.iter().position(|&x| x == max).unwrap_or(usize::MAX);
                pos_s <= pos_max
            }
        }
    }

    pub fn solve(&self, mut grid: Grid) -> SolveResult {
        let mut used: HashSet<Strategy> = HashSet::new();
        let mut steps: Vec<SolveStep> = vec![];

        let mut cands = CandidateGrid::from_grid(&grid);

        'outer: loop {
            // Apply ONE naked single then restart
            if self.allowed(Strategy::NakedSingle) {
                if let Some(step) = naked_single::find_naked_singles(&grid, &cands)
                    .into_iter()
                    .next()
                {
                    cands.eliminate_from_peers(step.row, step.col, step.digit);
                    grid.set_filled(step.row, step.col, step.digit);
                    used.insert(Strategy::NakedSingle);
                    steps.push(step);
                    continue 'outer;
                }
            }

            // Apply ONE hidden single then restart
            if self.allowed(Strategy::HiddenSingle) {
                if let Some(step) = hidden_single::find_hidden_singles(&grid, &cands)
                    .into_iter()
                    .next()
                {
                    cands.eliminate_from_peers(step.row, step.col, step.digit);
                    grid.set_filled(step.row, step.col, step.digit);
                    used.insert(Strategy::HiddenSingle);
                    steps.push(step);
                    continue 'outer;
                }
            }

            macro_rules! apply_elims {
                ($find_fn:expr, $strat:expr) => {
                    if self.allowed($strat) {
                        let elims = $find_fn(&cands);
                        if !elims.is_empty() {
                            for e in &elims {
                                cands.remove(e.row, e.col, e.digit);
                            }
                            used.insert($strat);
                            continue 'outer;
                        }
                    }
                };
            }

            apply_elims!(naked_pair::find_naked_pairs, Strategy::NakedPair);
            apply_elims!(pointing_pair::find_pointing_pairs, Strategy::PointingPair);
            apply_elims!(naked_triple::find_naked_triples, Strategy::NakedTriple);
            apply_elims!(hidden_pair::find_hidden_pairs, Strategy::HiddenPair);
            apply_elims!(
                box_line_reduction::find_box_line_reductions,
                Strategy::BoxLineReduction
            );
            apply_elims!(x_wing::find_x_wings, Strategy::XWing);
            apply_elims!(swordfish::find_swordfish, Strategy::Swordfish);

            // Expert block: BUG+1 placement first, then 13 elimination functions.
            if self.allowed(Strategy::Expert) {
                // BUG+1 is a placement, handled like NakedSingle
                if let Some(step) = expert::find_bug_plus_one_step(&cands) {
                    cands.eliminate_from_peers(step.row, step.col, step.digit);
                    grid.set_filled(step.row, step.col, step.digit);
                    used.insert(Strategy::Expert);
                    steps.push(step);
                    continue 'outer;
                }
                // 13 elimination functions
                let expert_fns: &[fn(&CandidateGrid) -> Vec<Elimination>] = &[
                    expert::find_jellyfish,
                    expert::find_naked_quad,
                    expert::find_hidden_triple,
                    expert::find_hidden_quad,
                    expert::find_skyscraper,
                    expert::find_two_string_kite,
                    expert::find_y_wing,
                    expert::find_xyz_wing,
                    expert::find_w_wing,
                    expert::find_unique_rectangle,
                    expert::find_empty_rectangle,
                    expert::find_simple_coloring,
                    expert::find_xy_chain,
                ];
                for f in expert_fns {
                    let e = f(&cands);
                    if !e.is_empty() {
                        for elim in &e {
                            cands.remove(elim.row, elim.col, elim.digit);
                        }
                        used.insert(Strategy::Expert);
                        continue 'outer;
                    }
                }
            }

            // Backtracking fallback
            if self.use_backtracking && !grid.is_solved() {
                if let Some(solved) = backtracking::solve_backtracking(grid.clone()) {
                    used.insert(Strategy::Backtracking);
                    grid = solved;
                }
            }

            break;
        }

        SolveResult {
            grid,
            used_strategies: used.into_iter().collect(),
            steps,
        }
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Grid;

    const EASY: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const EASY_SOL: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    const MEDIUM: &str =
        "000000000904607000076804100309701080008000300050308702007502610000403208000000000";
    const MEDIUM_SOL: &str =
        "583219467914637825276854139349721586728965341651348792497582613165493278832176954";

    #[test]
    fn solves_easy_with_logic_only() {
        let grid = Grid::from_str(EASY).unwrap();
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved());
        assert_eq!(result.grid.to_str(), EASY_SOL);
        assert!(!result.used_strategies.contains(&Strategy::Backtracking));
    }

    #[test]
    fn solves_medium_with_elimination_strategies() {
        let grid = Grid::from_str(MEDIUM).unwrap();
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved());
        assert_eq!(result.grid.to_str(), MEDIUM_SOL);
        assert!(!result.used_strategies.contains(&Strategy::Backtracking));
    }

    #[test]
    fn restricted_solver_stops_at_max_strategy() {
        let grid = Grid::from_str(EASY).unwrap();
        let mut solver = Solver::new();
        solver.max_strategy = Some(Strategy::NakedSingle);
        solver.use_backtracking = false;
        let result = solver.solve(grid);
        assert!(!result.used_strategies.contains(&Strategy::HiddenSingle));
        assert!(!result.used_strategies.contains(&Strategy::Backtracking));
    }

    #[test]
    fn for_difficulty_expert_has_correct_config() {
        let solver = Solver::for_difficulty(&crate::generator::difficulty::Difficulty::Expert);
        assert_eq!(solver.max_strategy, Some(Strategy::Expert));
        assert!(!solver.use_backtracking);
    }

    #[test]
    fn expert_comes_after_swordfish_in_strategy_order() {
        let order = Solver::strategy_order();
        let swordfish_pos = order.iter().position(|&s| s == Strategy::Swordfish).unwrap();
        let expert_pos = order.iter().position(|&s| s == Strategy::Expert).unwrap();
        assert!(expert_pos > swordfish_pos);
    }

    #[test]
    fn expert_comes_before_backtracking_in_strategy_order() {
        let order = Solver::strategy_order();
        let expert_pos = order.iter().position(|&s| s == Strategy::Expert).unwrap();
        let back_pos = order.iter().position(|&s| s == Strategy::Backtracking).unwrap();
        assert!(expert_pos < back_pos);
    }
}
```

- [ ] **Step 4: Compile and run all solver tests**

```bash
cargo test --lib 2>&1 | tail -15
```

Expected: all solver tests pass. Fix any compile errors before proceeding.

- [ ] **Step 5: Run all project tests**

```bash
cargo test 2>&1 | grep -E "^test result|FAILED"
```

Expected: all `ok`. There may still be compile errors in `src/tui/mod.rs` and `src/generator/mod.rs` due to the new `Difficulty::Expert` variant — these are fixed in Tasks 5 and 6.

- [ ] **Step 6: Commit**

```bash
git add src/solver/mod.rs
git commit -m "feat: integrate Expert strategy block into solver"
```

---

## Task 5: Update `src/generator/mod.rs` — Expert generation

**Files:**
- Modify: `src/generator/mod.rs`

The Expert arm in `generate()` uses a double solvability check with up to 64 seed-retry attempts.

- [ ] **Step 1: Write the failing test**

Add to `#[cfg(test)]` in `src/generator/mod.rs`:

```rust
#[test]
#[ignore = "slow: generates Expert puzzle (30–120 s); run with -- --include-ignored"]
fn generates_expert_puzzle_passing_double_check() {
    use crate::solver::Solver;
    // Generate an Expert puzzle
    let grid = PuzzleGenerator::new(42).generate(Difficulty::Expert, false);
    // Condition 1: Expert solver can solve it
    let expert_solver = Solver::for_difficulty(&Difficulty::Expert);
    let expert_result = expert_solver.solve(grid.clone());
    assert!(expert_result.grid.is_solved(),
        "Expert solver must solve an Expert-generated puzzle");
    // Condition 2: Extreme solver cannot solve it
    let extreme_solver = Solver::for_difficulty(&Difficulty::Extreme);
    let extreme_result = extreme_solver.solve(grid.clone());
    assert!(!extreme_result.grid.is_solved(),
        "Extreme solver must NOT solve an Expert-generated puzzle");
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test generates_expert_puzzle_passing_double_check -- --include-ignored 2>&1 | tail -10
```

Expected: compile error — non-exhaustive pattern for `Difficulty::Expert` in `generate()`.

- [ ] **Step 3: Update `src/generator/mod.rs`**

Find the `generate()` method. It currently has:
```rust
if difficulty == Difficulty::BareMinimum {
    // multi-pass loop ...
} else {
    let mut indices: Vec<usize> = (0..81).collect();
    // single-pass loop ...
}
```

Replace the top-level structure with:

```rust
pub fn generate(&self, difficulty: Difficulty, symmetry: bool) -> Grid {
    if difficulty == Difficulty::Expert {
        return self.generate_expert(symmetry);
    }
    // ... rest of existing generate() unchanged ...
}

fn generate_expert(&self, symmetry: bool) -> Grid {
    const MAX_RETRIES: u32 = 64;
    for attempt in 0..MAX_RETRIES {
        let seed = self.seed.wrapping_add(attempt as u64);
        let mut rng = LcgRng::new(seed);
        let full = self.fill_grid(&mut rng).expect("fill_grid failed");

        let is_expert_solvable = |puzzle: &Grid| -> bool {
            self.is_uniquely_solvable(puzzle, Difficulty::Expert)
        };
        let is_extreme_solvable = |puzzle: &Grid| -> bool {
            self.is_uniquely_solvable(puzzle, Difficulty::Extreme)
        };

        let solvable = |puzzle: &Grid| -> bool {
            is_expert_solvable(puzzle) && !is_extreme_solvable(puzzle)
        };

        let mut puzzle = full.clone();

        if symmetry {
            let mut pair_indices: Vec<usize> = (0..=40).collect();
            shuffle(&mut pair_indices, &mut rng);
            for &idx in &pair_indices {
                let (r1, c1) = (idx / 9, idx % 9);
                let prev1 = puzzle.get(r1, c1).value();
                puzzle.clear(r1, c1);
                let mirror_state = if idx < 40 {
                    let m = 80 - idx;
                    let (r2, c2) = (m / 9, m % 9);
                    let prev2 = puzzle.get(r2, c2).value();
                    puzzle.clear(r2, c2);
                    Some((r2, c2, prev2))
                } else {
                    None
                };
                if !solvable(&puzzle) {
                    if let Some(v) = prev1 { puzzle.set_given(r1, c1, v); }
                    if let Some((r2, c2, Some(v))) = mirror_state { puzzle.set_given(r2, c2, v); }
                }
            }
        } else {
            let mut indices: Vec<usize> = (0..81).collect();
            shuffle(&mut indices, &mut rng);
            for &idx in &indices {
                let row = idx / 9;
                let col = idx % 9;
                let prev_val = puzzle.get(row, col).value();
                puzzle.clear(row, col);
                if !solvable(&puzzle) {
                    if let Some(v) = prev_val { puzzle.set_given(row, col, v); }
                }
            }
        }

        // Verify the final puzzle passes the double check
        if solvable(&puzzle) {
            let mut result = Grid::empty();
            for r in 0..9 {
                for c in 0..9 {
                    if !puzzle.get(r, c).is_empty() {
                        let v = full.get(r, c).value().unwrap();
                        result.set_given(r, c, v);
                    }
                }
            }
            return result;
        }
        // Else retry with next seed
    }
    panic!("Expert puzzle generation failed after {} attempts", MAX_RETRIES);
}
```

- [ ] **Step 4: Run the new test**

```bash
cargo test generates_expert_puzzle_passing_double_check -- --include-ignored --nocapture 2>&1 | tail -10
```

Note: This test generates an Expert puzzle which can take 30–120 seconds. Expected: `test result: ok. 1 passed`.

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1 | grep -E "^test result|FAILED"
```

Expected: all `ok`. At this point only `src/tui/mod.rs` and `src/tui/render/start_screen.rs` and `src/i18n/mod.rs` may have compile errors (Task 6).

- [ ] **Step 6: Commit**

```bash
git add src/generator/mod.rs
git commit -m "feat: add Expert generation with double solvability check and 64-attempt retry"
```

---

## Task 6: Update TUI, render, and i18n

**Files:**
- Modify: `src/tui/generating.rs`
- Modify: `src/i18n/mod.rs`
- Modify: `src/tui/mod.rs`
- Modify: `src/tui/render/start_screen.rs`

- [ ] **Step 0: Verify that the build fails with the expected errors (the failing tests)**

The additions from Tasks 1–5 introduced `Difficulty::Expert` and `Strategy::Expert` — these cause compile errors in the TUI files that have non-exhaustive match arms. Confirm them before making changes:

```bash
cargo build 2>&1 | grep "^error" | head -10
```

Expected: errors about non-exhaustive patterns on `Difficulty` in `src/tui/mod.rs` and missing struct fields or match arms. If `cargo build` passes without errors, something was missed in a prior task — do not proceed until the failures are present.

- [ ] **Step 1: Update `src/tui/generating.rs`**

Expert generation takes 30–120 seconds and must run on a background thread — exactly like BareMinimum. The generating module already has a `spawn_bare_minimum()` function and a `bare_minimum: bool` field on `GeneratingState`. Mirror that pattern for Expert.

First, read the current file to understand the structure:
```bash
grep -n "bare_minimum\|spawn_bare\|GeneratingState\|pub fn spawn\|pub fn new" /Users/alexandererben/Tresors/OrdiSync/6_Entwicklung/Claude/SudokuCLI/src/tui/generating.rs | head -30
```

Then add the following to `src/tui/generating.rs`:

**1a. Add `expert: bool` field to `GeneratingState`:**

The struct is in `src/tui/generating.rs`. It currently ends with:
```rust
    pub bare_minimum: bool,
    pub bm_done: usize,
    pub bm_total: usize,
    pub bm_best_count: usize,
}
```

Add `expert: bool` after `bare_minimum`:
```rust
    pub bare_minimum: bool,
    pub expert: bool,           // NEW
    pub bm_done: usize,
    pub bm_total: usize,
    pub bm_best_count: usize,
}
```

Also add `expert: false` to every existing `GeneratingState { … }` constructor literal (`new()` and `new_bare_minimum()`), and `bare_minimum: false` to the new `new_expert()` below. The compiler will flag every constructor literal that is missing the new field.

**1b. Add `GeneratingState::new_expert()` constructor:**

Add this function immediately after `new_bare_minimum()`:

```rust
/// Create a state for Expert single-attempt generation.
/// No pattern is involved; back navigation returns to DifficultySelect (index 4).
pub fn new_expert(symmetry: bool) -> Self {
    let seed = random_seed();
    let rx = spawn_expert(seed, symmetry);
    let n = VERBS.len();
    let mut verb_order: Vec<usize> = (0..n).collect();
    lcg_shuffle(&mut verb_order, seed);
    let dummy = Pattern {
        name_en: "",
        mask: [false; 81],
        cell_count: 0,
    };
    GeneratingState {
        pattern: dummy,
        rx,
        seed,
        started_at: Instant::now(),
        verb_order,
        verb_pos: 0,
        show_new_seed: false,
        new_seed_at: None,
        from_cli: false,
        bare_minimum: false,
        expert: true,
        bm_done: 0,
        bm_total: 0,
        bm_best_count: 0,
    }
}
```

**1c. Add `spawn_expert()` function:**

Add this function immediately after `spawn_bare_minimum()`:

```rust
/// Spawn a background thread that generates one Expert puzzle.
pub fn spawn_expert(seed: u64, symmetry: bool) -> mpsc::Receiver<GenMsg> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let grid = PuzzleGenerator::new(seed)
            .generate(Difficulty::Expert, symmetry);
        let _ = tx.send(GenMsg::Done(grid));
    });
    rx
}
```

- [ ] **Step 2: Verify `src/tui/generating.rs` compiles**

```bash
cargo build 2>&1 | grep "^error" | head -5
```

Expected: no errors from `generating.rs` (there may still be errors from other files about the missing `Difficulty::Expert` variant — those are fixed in later steps).

- [ ] **Step 3: Update `src/i18n/mod.rs`**

Add `difficulty_expert` field to the `Strings` struct, between `difficulty_extreme` and `difficulty_bare_minimum`:

In `src/i18n/mod.rs`, locate:
```rust
    pub difficulty_extreme: &'static str,
    pub difficulty_bare_minimum: &'static str,
```

Replace with:
```rust
    pub difficulty_extreme: &'static str,
    pub difficulty_expert: &'static str,
    pub difficulty_bare_minimum: &'static str,
```

Then find every language constant (they're `const` values of type `Strings`). Search for the pattern `difficulty_extreme:` in the file and after each one add `difficulty_expert:` with the appropriate value:
- English: `difficulty_expert: "Expert",`
- German: `difficulty_expert: "Experte",`
- All other languages: `difficulty_expert: "Expert",`

```bash
grep -n "difficulty_extreme:" /Users/alexandererben/Tresors/OrdiSync/6_Entwicklung/Claude/SudokuCLI/src/i18n/mod.rs
```

Use that output to find all language constants and add the field after each `difficulty_extreme:` line.

- [ ] **Step 4: Update `src/tui/render/start_screen.rs`**

In `src/tui/render/start_screen.rs`, find:
```rust
    let items = [
        strings.difficulty_easy,
        strings.difficulty_medium,
        strings.difficulty_hard,
        strings.difficulty_extreme,
        strings.difficulty_bare_minimum,
        strings.difficulty_designer,
    ];
```

Replace with:
```rust
    let items = [
        strings.difficulty_easy,
        strings.difficulty_medium,
        strings.difficulty_hard,
        strings.difficulty_extreme,
        strings.difficulty_expert,
        strings.difficulty_bare_minimum,
        strings.difficulty_designer,
    ];
```

- [ ] **Step 5: Update `src/tui/mod.rs`**

**5a. Update `DIFFICULTY_COUNT`:**

Find:
```rust
        const DIFFICULTY_COUNT: usize = 6;
```
Replace with:
```rust
        const DIFFICULTY_COUNT: usize = 7;
```

**5b. Update the `Enter` match arm in `handle_difficulty_action()`:**

Find:
```rust
            AppAction::Enter if !sym_focused => match selected {
                0 => {
                    self.start_game(Difficulty::Easy);
                    self.needs_clear = true;
                }
                1 => {
                    self.start_game(Difficulty::Medium);
                    self.needs_clear = true;
                }
                2 => {
                    self.start_game(Difficulty::Hard);
                    self.needs_clear = true;
                }
                3 => {
                    self.start_game(Difficulty::Extreme);
                    self.needs_clear = true;
                }
                4 => {
                    self.start_game(Difficulty::BareMinimum);
                    self.needs_clear = true;
                }
                5 => {
                    self.screen = AppScreen::PatternSelect { selected: 0 };
                    self.needs_clear = true;
                }
                _ => {}
            },
```

Replace with:
```rust
            AppAction::Enter if !sym_focused => match selected {
                0 => {
                    self.start_game(Difficulty::Easy);
                    self.needs_clear = true;
                }
                1 => {
                    self.start_game(Difficulty::Medium);
                    self.needs_clear = true;
                }
                2 => {
                    self.start_game(Difficulty::Hard);
                    self.needs_clear = true;
                }
                3 => {
                    self.start_game(Difficulty::Extreme);
                    self.needs_clear = true;
                }
                4 => {
                    self.start_game(Difficulty::Expert);
                    self.needs_clear = true;
                }
                5 => {
                    self.start_game(Difficulty::BareMinimum);
                    self.needs_clear = true;
                }
                6 => {
                    self.screen = AppScreen::PatternSelect { selected: 0 };
                    self.needs_clear = true;
                }
                _ => {}
            },
```

**5c. Route Expert through the Generating screen in `start_game()`:**

The existing `start_game()` currently routes `BareMinimum` to the Generating screen and all other difficulties synchronously. Expert generation is similarly slow — it must take the same async path.

Locate the section in `start_game()` that handles `BareMinimum`:
```rust
if difficulty == Difficulty::BareMinimum {
    let state = crate::tui::generating::GeneratingState::new_bare_minimum(self.symmetry);
    self.screen = AppScreen::Generating(state);
    self.needs_clear = true;
    self.drain_input = true;
    return;
}
```

Add an analogous block for Expert immediately before or after the BareMinimum block:
```rust
if difficulty == Difficulty::Expert {
    let state = crate::tui::generating::GeneratingState::new_expert(self.symmetry);
    self.screen = AppScreen::Generating(state);
    self.needs_clear = true;
    self.drain_input = true;
    return;
}
```

**5d. Fix `handle_generating_action()` — Back handler and BareMinimum index:**

When the user presses Back on the Generating screen, the handler returns to `DifficultySelect` with a hardcoded `selected` index. Two changes are needed:

1. Add detection for the Expert case (using `gs.expert`)  
2. Fix BareMinimum's `selected` index: before this change BareMinimum was at index 4; after adding Expert it moves to index 5.

Find the Back handler in `handle_generating_action()`. It currently looks approximately like:
```rust
AppAction::Back => {
    self.screen = AppScreen::DifficultySelect {
        selected: 4,  // BareMinimum was at index 4
        sym_focused: false,
    };
    self.needs_clear = true;
}
```

Replace with:
```rust
AppAction::Back => {
    if let AppScreen::Generating(ref gs) = self.screen {
        let selected = if gs.expert {
            4  // Expert is now at index 4
        } else {
            5  // BareMinimum is now at index 5
        };
        self.screen = AppScreen::DifficultySelect {
            selected,
            sym_focused: false,
        };
    }
    self.needs_clear = true;
}
```

**5e. Check for any other Difficulty match arms in the file** that need updating (e.g., classify result display, game category detection). Run:

```bash
grep -n "Difficulty::" /Users/alexandererben/Tresors/OrdiSync/6_Entwicklung/Claude/SudokuCLI/src/tui/mod.rs
```

Any non-exhaustive match on `Difficulty` will be caught by the compiler. Fix each one by adding an `Expert` arm (same behavior as `Extreme` for most UI purposes — Expert is not a `GameCategory::Design` puzzle).

- [ ] **Step 6: Compile and run all tests**

```bash
cargo test 2>&1 | grep -E "^test result|FAILED|^error"
```

Expected: all `ok`, no errors.

- [ ] **Step 7: Commit**

```bash
git add src/tui/generating.rs src/i18n/mod.rs src/tui/mod.rs src/tui/render/start_screen.rs
git commit -m "feat: add Expert difficulty to TUI, generating screen, render, and i18n"
```

---

## Task 7: Integration tests and final validation

**Files:**
- Modify: `src/solver/mod.rs` (add known-Expert-puzzle test)

**The known Expert puzzle string:** During this task, generate a puzzle, verify it passes the double check, and hardcode it.

- [ ] **Step 1: Find a known Expert puzzle string**

```bash
cd /Users/alexandererben/Tresors/OrdiSync/6_Entwicklung/Claude/SudokuCLI
cargo test generates_expert_puzzle_passing_double_check -- --include-ignored --nocapture 2>&1 | head -30
```

Then write a small test helper to print the generated puzzle:

```bash
cargo test -- --nocapture 2>&1 | grep -A5 "expert"
```

Or write a temporary binary in `src/bin/find_expert.rs`:

```rust
fn main() {
    use clisudoku::generator::{Difficulty, PuzzleGenerator};
    use clisudoku::solver::Solver;
    let grid = PuzzleGenerator::new(42).generate(Difficulty::Expert, false);
    println!("Expert puzzle: {}", grid.to_str());
    let expert = Solver::for_difficulty(&Difficulty::Expert);
    let extreme = Solver::for_difficulty(&Difficulty::Extreme);
    println!("Expert solves: {}", expert.solve(grid.clone()).grid.is_solved());
    println!("Extreme solves: {}", extreme.solve(grid.clone()).grid.is_solved());
}
```

```bash
cargo run --bin find_expert 2>&1
```

Take the printed puzzle string — that is `KNOWN_EXPERT_PUZZLE`.

- [ ] **Step 2: Add the integration test to `src/solver/mod.rs`**

Add to the `#[cfg(test)]` section (replace the placeholder string with the actual one found in Step 1):

```rust
// Hardcoded known Expert puzzle — found by running PuzzleGenerator::new(42).generate(Expert, false).
// The Expert solver solves it; the Extreme solver cannot.
const KNOWN_EXPERT_PUZZLE: &str = "<REPLACE_WITH_ACTUAL_81_CHAR_STRING>";

#[test]
fn expert_solver_solves_known_expert_puzzle() {
    let grid = Grid::from_str(KNOWN_EXPERT_PUZZLE).unwrap();
    let result = Solver::for_difficulty(&crate::generator::difficulty::Difficulty::Expert).solve(grid);
    assert!(result.grid.is_solved(),
        "Expert solver must solve the known Expert puzzle");
    assert!(result.used_strategies.contains(&Strategy::Expert),
        "Must have used Strategy::Expert");
    assert!(!result.used_strategies.contains(&Strategy::Backtracking),
        "Must not use backtracking");
}

#[test]
fn extreme_solver_cannot_solve_known_expert_puzzle() {
    let grid = Grid::from_str(KNOWN_EXPERT_PUZZLE).unwrap();
    let result = Solver::for_difficulty(&crate::generator::difficulty::Difficulty::Extreme).solve(grid);
    assert!(!result.grid.is_solved(),
        "Extreme solver must NOT solve the known Expert puzzle");
}
```

- [ ] **Step 3: Run the integration tests**

```bash
cargo test expert_solver_solves_known_expert_puzzle extreme_solver_cannot_solve_known_expert_puzzle 2>&1 | tail -10
```

Expected: `test result: ok. 2 passed`

- [ ] **Step 4: Clean up the temporary binary if created**

```bash
rm -f src/bin/find_expert.rs
```

- [ ] **Step 5: Run the full test suite**

```bash
cargo test 2>&1 | grep -E "^test result|FAILED"
```

Expected: all `test result: ok.` lines, zero failures.

- [ ] **Step 6: Commit**

```bash
git add src/solver/mod.rs
git commit -m "test: add integration tests for Expert difficulty with known puzzle string"
```

---

## Notes for the Implementer

### Key gotchas

1. **BUG+1 cannot be easily tested positively via CandidateGrid manipulation** because the public API only has `remove()` — you can't add candidates back. The integration test in Task 7 covers BUG+1 implicitly if the known Expert puzzle happens to require it. The unit test in Task 3 verifies the negative case only.

2. **Expert generation is slow** (30–120 seconds per puzzle). The smoke test in Task 5 (`generates_expert_puzzle_passing_double_check`) is marked `#[ignore]` so it doesn't slow the normal test run. To run it explicitly: `cargo test generates_expert_puzzle -- --include-ignored --nocapture`.

3. **Difficulty::Expert in other match arms**: `src/tui/mod.rs` may have other places that match on `Difficulty` (e.g., the generating screen label, the game category assignment). The compiler will flag them — fix each by adding `Difficulty::Expert` with appropriate behavior (same as `Extreme` for most UI purposes).

4. **The `find_hidden_triple` function** in expert.rs handles the case where the hidden triple is a subset of `all_units()`. The tier2.rs version uses `NakedTriples` for this same pattern. Verify the output against the hint system's `HiddenTriples` tests to confirm correctness.

5. **`cargo test` order**: Tasks 1→2→3→4→5→6→7. Do not run Task 5 tests before Task 4 is complete (the compile errors from non-exhaustive `Difficulty` matches will block it).
