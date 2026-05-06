# Expert Difficulty — Design Spec

## Goal

Add a new `Expert` difficulty level that generates puzzles logically solvable only with
advanced techniques (Tier 2: Jellyfish, Skyscraper, XY-Chain, …). The generator
guarantees that every Expert puzzle truly requires at least one such technique — a
simpler solver cannot crack it.

## Architecture

### New solver module: `src/solver/expert.rs`

Pure `CandidateGrid`-based functions — no display logic, no coupling to the hint system.
Each function signature follows the existing pattern:

```rust
pub fn find_X(cands: &CandidateGrid) -> Vec<Elimination>
```

Functions to implement (algorithms mirror `src/hint/strategies/tier2.rs`, stripped of
explanation text and cause-cell tracking):

| Function | Based on |
|---|---|
| `find_jellyfish` | Fish N=4, generalised from `swordfish.rs` |
| `find_naked_quad` | Generalised from `naked_triple.rs` |
| `find_hidden_triple` | New |
| `find_hidden_quad` | New |
| `find_skyscraper` | New |
| `find_two_string_kite` | New |
| `find_y_wing` | New |
| `find_xyz_wing` | New |
| `find_w_wing` | New |
| `find_unique_rectangle` | New |
| `find_bug_plus_one` | New |
| `find_empty_rectangle` | New |
| `find_simple_coloring` | New |
| `find_xy_chain` | New |

The hint system (`src/hint/`) is not touched. The two sides remain deliberately separate.

### `Strategy` enum (`src/solver/candidates.rs`)

One new variant:

```rust
pub enum Strategy {
    // … existing …
    Expert,      // covers all Tier-2 expert techniques
    Backtracking,
}
```

A single variant suffices — all expert techniques map to the same difficulty level.

### `Solver` (`src/solver/mod.rs`)

`pub mod expert;` declaration is added alongside the existing strategy module declarations.

`strategy_order()` inserts `Expert` between `Swordfish` and `Backtracking`:

```rust
&[NakedSingle, HiddenSingle, NakedPair, PointingPair, NakedTriple, HiddenPair,
  BoxLineReduction, XWing, Swordfish, Expert, Backtracking]
```

`Backtracking` remains special-cased in `allowed()` via early return and is not looked up
in `strategy_order()`. This is unchanged and intentional: `for_difficulty(Expert)` sets
`use_backtracking: false`, so `allowed(Backtracking)` correctly returns false without any
ordering logic.

`for_difficulty(Expert)` returns:
```rust
Self { max_strategy: Some(Strategy::Expert), use_backtracking: false }
```

`solve()` loop gets a new `apply_elims!` block immediately after Swordfish that calls all
14 expert functions in order, under the single `Strategy::Expert` guard.

### `Difficulty` enum (`src/generator/difficulty.rs`)

New variant between `Extreme` and `BareMinimum`:

```rust
pub enum Difficulty {
    Easy, Medium, Hard, Extreme,
    Expert,      // NEW
    BareMinimum,
}
```

`classify()` extended — Expert is checked first since it is the new highest logical tier:

```rust
if needs(Strategy::Expert)                                          { Difficulty::Expert  }
else if needs(Strategy::Swordfish) || needs(Strategy::Backtracking){ Difficulty::Extreme }
// … rest unchanged …
```

`generate_with_pattern()` uses an uncapped `Solver::new()` for post-hoc classification.
Once `Expert` is in `strategy_order()` and `solve()` applies it, pattern puzzles may
classify as Expert. This is correct and intentional — the pattern generator uses the
real, uncapped solver and reflects whatever difficulty it actually requires.

### Generator (`src/generator/mod.rs`)

Expert uses a **double solvability check** — the closure passed to the removal loop:

```rust
Difficulty::Expert => |puzzle: &Grid| {
    // 1. Expert solver can logically solve it to completion (implies unique solution)
    self.is_uniquely_solvable(puzzle, Difficulty::Expert)
    // 2. Extreme solver cannot (puzzle genuinely needs an Expert technique)
    && !self.is_uniquely_solvable(puzzle, Difficulty::Extreme)
}
```

**Why `is_uniquely_solvable` (logical solver) rather than `is_uniquely_solvable_full`
(backtracking) for condition 1:** The logical solver only places digits that are forced
across all solutions. A full logical solve therefore implies a unique solution. This is
the same reasoning used for all other non-BareMinimum difficulties — no change in
contract.

A cell is removed only when both conditions hold. This guarantees every generated puzzle
requires at least one Expert technique.

**Fallback — degenerate seed:** If the single-pass removal loop finishes with a puzzle
that still fails the double check (e.g., the full grid is already Extreme-solvable before
any removals), `generate()` retries internally with `seed + 1`. The retry loop lives
inside `generate()` itself — the TUI calling path for standard difficulties is
synchronous (`start_game()` calls `generate()` directly) and has no retry mechanism at
the TUI layer. A bounded retry (e.g., up to 64 attempts) prevents an infinite loop; in
practice Expert puzzles emerge reliably from the first or second attempt.

**Known puzzle for tests:** At least one 81-character puzzle string that requires an
Expert technique must be found during implementation (generate one, verify the Extreme
solver fails on it) and hardcoded in the test. The implementer is responsible for finding
this string before writing the negative assertion.

Symmetry mode (180° rotation) works unchanged — it uses the same `solvable` closure.

### TUI (`src/tui/mod.rs`)

`DifficultySelect` gets a sixth content entry (seventh total including Designer/Pattern)
inserted between Extreme and BareMinimum. Index mapping after the change:

| selected | Difficulty |
|---|---|
| 0 | Easy |
| 1 | Medium |
| 2 | Hard |
| 3 | Extreme |
| 4 | Expert ← NEW |
| 5 | BareMinimum |
| 6 | Designer (PatternSelect) |

`DIFFICULTY_COUNT` (the clamp constant in `handle_difficulty_action`) must be updated
from 6 to 7.

### Render (`src/tui/render/start_screen.rs`)

The `items` array in the difficulty-select render function gains a seventh entry for
Expert between Extreme and BareMinimum. Only the label string is needed — no description
text is displayed (no `difficulty_expert_desc` field is added to `Strings`).

### i18n (`src/i18n/mod.rs`)

One new string field added to the `Strings` struct and filled for both languages:

| Key | EN | DE |
|---|---|---|
| `difficulty_expert` | `"Expert"` | `"Experte"` |

## Files Changed

| File | Change |
|---|---|
| `src/solver/expert.rs` | NEW — 14 solver functions |
| `src/solver/candidates.rs` | Add `Strategy::Expert` |
| `src/solver/mod.rs` | `pub mod expert`, strategy_order, for_difficulty, apply_elims block |
| `src/generator/difficulty.rs` | Add `Difficulty::Expert`, extend classify() |
| `src/generator/mod.rs` | Expert generation logic (double solvability check + seed retry) |
| `src/tui/mod.rs` | Expert entry + DIFFICULTY_COUNT 6→7 |
| `src/tui/render/start_screen.rs` | 7th entry in items array |
| `src/i18n/mod.rs` | 1 new string (`difficulty_expert`) |

## Testing

- Unit tests for every `find_X` function in `expert.rs`: positive (detects pattern and
  returns ≥1 elimination) and negative (returns empty vec on a blank CandidateGrid).
- `classify()` test: `Strategy::Expert` in used set → `Difficulty::Expert`.
- `Solver::for_difficulty(Expert)` integration test: solves a hardcoded known-Expert
  puzzle string (found during implementation), does not use `Strategy::Backtracking`,
  and the same puzzle string fails to solve with `Solver::for_difficulty(Extreme)`.
- Generator smoke test: `PuzzleGenerator::new(seed).generate(Difficulty::Expert, false)`
  returns a puzzle that passes the double solvability check.

## What This Is Not

- **BareMinimum** = fewest possible givens (~17), solved via backtracking, structural
  curiosity. Challenge through clue scarcity.
- **Expert** = normal given count, purely logical, requires Tier-2 pattern recognition.
  Challenge through technique complexity.
