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

`strategy_order()` inserts `Expert` between `Swordfish` and `Backtracking`.

`for_difficulty(Expert)` returns:
```rust
Self { max_strategy: Some(Strategy::Expert), use_backtracking: false }
```

`solve()` loop gets a new `apply_elims!` block that calls all 14 expert functions in order,
under the single `Strategy::Expert` guard.

### `Difficulty` enum (`src/generator/difficulty.rs`)

New variant between `Extreme` and `BareMinimum`:

```rust
pub enum Difficulty {
    Easy, Medium, Hard, Extreme,
    Expert,      // NEW
    BareMinimum,
}
```

`classify()` extended:

```rust
if needs(Strategy::Expert)      { Difficulty::Expert  }
else if needs(Strategy::Swordfish) || needs(Strategy::Backtracking) { Difficulty::Extreme }
// … rest unchanged …
```

### Generator (`src/generator/mod.rs`)

Expert uses a **double solvability check** — the closure passed to the removal loop becomes:

```rust
Difficulty::Expert => |puzzle: &Grid| {
    // 1. Expert solver can uniquely solve it
    self.is_uniquely_solvable(puzzle, Difficulty::Expert)
    // 2. Extreme solver cannot (puzzle genuinely needs Expert techniques)
    && !self.is_uniquely_solvable(puzzle, Difficulty::Extreme)
}
```

A cell is removed only when both conditions hold. This guarantees every generated puzzle
requires at least one Expert technique. Generation takes longer than Extreme due to the
double check; the existing "Generating…" screen with progress feedback handles the wait.

Symmetry mode (180° rotation) works unchanged — it uses the same closure.

### TUI (`src/tui/mod.rs`)

`DifficultySelect` gets a sixth entry between Extreme and BareMinimum.
The handler maps `selected == 4` to `Difficulty::Expert` (indices shift for BareMinimum).

### i18n (`src/i18n/mod.rs`)

Two new string fields added to the `Strings` struct and filled for both languages:

| Key | EN | DE |
|---|---|---|
| `difficulty_expert` | `"Expert"` | `"Experte"` |
| `difficulty_expert_desc` | `"Requires advanced techniques (Skyscraper, XY-Chain, …)"` | `"Erfordert fortgeschrittene Techniken (Skyscraper, XY-Kette, …)"` |

## Files Changed

| File | Change |
|---|---|
| `src/solver/expert.rs` | NEW — 14 solver functions |
| `src/solver/candidates.rs` | Add `Strategy::Expert` |
| `src/solver/mod.rs` | strategy_order, for_difficulty, apply_elims block |
| `src/generator/difficulty.rs` | Add `Difficulty::Expert`, extend classify() |
| `src/generator/mod.rs` | Expert generation logic (double solvability check) |
| `src/tui/mod.rs` | Expert entry in DifficultySelect |
| `src/i18n/mod.rs` | 2 new strings |

## Testing

- Unit tests for every `find_X` function in `expert.rs` (positive: detects pattern,
  negative: returns empty on blank grid).
- `classify()` test: Expert strategies → `Difficulty::Expert`.
- `Solver::for_difficulty(Expert)` integration test: solves a known Expert puzzle without
  backtracking and without Extreme being sufficient.
- Generator smoke test: generated Expert puzzle passes double solvability check.

## What This Is Not

- **BareMinimum** = fewest possible givens (~17), solved via backtracking, structural
  curiosity. Challenge through clue scarcity.
- **Expert** = normal given count, purely logical, requires Tier-2 pattern recognition.
  Challenge through technique complexity.
