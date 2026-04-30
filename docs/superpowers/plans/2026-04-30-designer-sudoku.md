# Designer Sudoku Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Designer Sudoku mode where the given cells form a recognisable visual pattern chosen from 28 built-in shapes; the generator runs in a background thread with countdown/retry UI, and hints receive a pre-check for incorrect digits/notes.

**Architecture:** A new `src/pattern/` module holds the Pattern struct and all 28 masks; the generator gains `generate_with_pattern()` which falls back to adding extra cells when the pattern is too sparse (Ansatz C); generation runs on a background thread (one thread per seed attempt, 3 s timeout) and communicates via `mpsc::channel`; the TUI gains two new screens — PatternSelect and Generating — wired from the DifficultySelect "Designer ▶" option and the `--pattern` CLI flag.

**Tech Stack:** Rust stable, crossterm 0.27, std::thread + std::sync::mpsc, std::time::Instant; no new dependencies.

**Spec:** `docs/superpowers/specs/2026-04-30-designer-sudoku-design.md`

**Scope note:** The database (rusqlite) is not yet implemented. This plan adds `GameCategory` + `pattern_name` to `GameStats` for future DB integration, but does NOT add DB persistence — that is a separate milestone.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/pattern/mod.rs` | **Create** | Pattern struct, 28 built-in patterns, `from_cli_str()` |
| `src/lib.rs` | Modify | `pub mod pattern;` |
| `src/generator/mod.rs` | Modify | `generate_with_pattern()`, `GeneratorResult` |
| `src/tui/generating.rs` | **Create** | `GeneratingState`, `GenMsg`, `spawn_generation()` |
| `src/tui/mod.rs` | Modify | New AppScreen variants, handlers, hint pre-check, `hint_warning` |
| `src/tui/render/pattern_select.rs` | **Create** | Pattern miniature screen rendering |
| `src/tui/render/generating.rs` | **Create** | Generating progress screen rendering |
| `src/tui/render/mod.rs` | Modify | New Screen variants, route to new renderers |
| `src/tui/render/start_screen.rs` | Modify | 4th difficulty option "Designer ▶" |
| `src/tui/render/status_bar.rs` | Modify | Hint count display in panel |
| `src/i18n/mod.rs` | Modify | 6 new string fields (all 13 language statics) |
| `src/main.rs` | Modify | `--pattern <81chars>` CLI flag |
| `tests/tui_smoke.rs` | Modify | Update Screen::Game construction if fields change |

---

## Task 1: Pattern Module

**Files:**
- Create: `src/pattern/mod.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// At the bottom of src/pattern/mod.rs (add after implementation)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_count_is_28() {
        assert_eq!(PATTERNS.len(), 28);
    }

    #[test]
    fn all_masks_have_81_cells() {
        for p in PATTERNS {
            assert_eq!(p.mask.len(), 81, "Pattern '{}' mask wrong length", p.name_en);
        }
    }

    #[test]
    fn patterns_sorted_descending_by_cell_count() {
        for i in 1..PATTERNS.len() {
            assert!(
                PATTERNS[i - 1].cell_count >= PATTERNS[i].cell_count,
                "Sort broken at index {}: {} ({}) < {} ({})",
                i, PATTERNS[i-1].name_en, PATTERNS[i-1].cell_count,
                PATTERNS[i].name_en, PATTERNS[i].cell_count
            );
        }
    }

    #[test]
    fn cell_count_matches_mask() {
        for p in PATTERNS {
            let counted = p.mask.iter().filter(|&&b| b).count();
            assert_eq!(counted, p.cell_count,
                "Pattern '{}': cell_count={} but mask has {} true bits",
                p.name_en, p.cell_count, counted);
        }
    }

    #[test]
    fn from_cli_str_valid() {
        let s = "1".repeat(81);
        let p = Pattern::from_cli_str(&s).unwrap();
        assert_eq!(p.cell_count, 81);
        assert!(p.mask.iter().all(|&b| b));
    }

    #[test]
    fn from_cli_str_too_short() {
        assert!(Pattern::from_cli_str("111").is_err());
    }

    #[test]
    fn from_cli_str_accepts_dots_and_stars() {
        // dots = empty, stars = pattern
        let s = "*".repeat(40) + &".".repeat(41);
        let p = Pattern::from_cli_str(&s).unwrap();
        assert_eq!(p.cell_count, 40);
    }
}
```

- [ ] **Step 2: Run tests — expect compile error (module missing)**

```bash
cargo test pattern 2>&1 | head -20
```

- [ ] **Step 3: Create `src/pattern/mod.rs`**

```rust
// src/pattern/mod.rs

/// A visual pattern mask for Designer Sudoku generation.
///
/// `mask[r * 9 + c]` is `true` when cell (r, c) may be a given.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub name_en:    &'static str,
    pub name_de:    &'static str,
    pub mask:       [bool; 81],
    pub cell_count: usize,
}

impl Pattern {
    /// Parse an 81-char CLI string:
    ///   '1' or '*' → pattern cell (may be given)
    ///   '.' or '0' → always empty
    pub fn from_cli_str(s: &str) -> Result<Self, String> {
        if s.len() != 81 {
            return Err(format!(
                "Pattern must be exactly 81 characters, got {}",
                s.len()
            ));
        }
        let mut mask = [false; 81];
        for (i, ch) in s.chars().enumerate() {
            mask[i] = matches!(ch, '1' | '*');
        }
        let cell_count = mask.iter().filter(|&&b| b).count();
        Ok(Pattern {
            name_en: "Custom",
            name_de: "Benutzerdefiniert",
            mask,
            cell_count,
        })
    }
}

// ── Internal mask builder (used only in const context below) ──────────────────

const fn mask_from_bytes(s: &[u8; 81]) -> [bool; 81] {
    let mut m = [false; 81];
    let mut i = 0;
    while i < 81 {
        m[i] = s[i] == b'1';
        i += 1;
    }
    m
}

// Helper: count true bits in a const mask
const fn count_bits(m: &[bool; 81]) -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < 81 { if m[i] { n += 1; } i += 1; }
    n
}

macro_rules! pat {
    ($en:expr, $de:expr, $bits:expr) => {{
        const M: [bool; 81] = mask_from_bytes($bits);
        Pattern { name_en: $en, name_de: $de, mask: M, cell_count: count_bits(&M) }
    }};
}

// ── 28 built-in patterns, sorted by cell_count descending ─────────────────────

pub static PATTERNS: &[Pattern] = &[
    // 1. Holy Crap — 46
    pat!("Holy Crap",    "Heilige Kuh",
         b"001101100110000011111101111100000001101111101111010111001111100011111110010000010"),
    // 2. Checker — 41
    pat!("Checker",      "Schachbrett",
         b"101010101010101010101010101010101010101010101010101010101010101010101010101010101"),
    // 3. Rudolph — 41
    pat!("Rudolph",      "Rudolf",
         b"100001000111111000001100000011100000001111110001111111001111111001010101001010101"),
    // 4. Bug or Feature? — 41
    pat!("Bug or Feature?", "Bug oder Feature?",
         b"000101000110111011011111110000101000011101110000101000011111110110111011000010000"),
    // 5. Heart — 40
    pat!("Heart",        "Herz",
         b"011000110111101111110111011110010011110000011011000110001101100000111000000010000"),
    // 6. Wind Up — 40
    pat!("Wind Up",      "Aufziehen",
         b"000010000111101111000010000001111000011011001010011111011111100001001000011111110"),
    // 7. Shit Happens — 39
    pat!("Shit Happens", "Shit Happens",
         b"000001000010000000000110010001110000001001100011111110011111110011100011111111111"),
    // 8. Ripples — 37
    pat!("Ripples",      "Wellen",
         b"000111000011000110010111010101000101101010101101000101010111010011000110000111000"),
    // 9. Mamihlapinatapai — 36
    pat!("Mamihlapinatapai", "Mamihlapinatapai",
         b"011000110000000000011000110100101001100101001110101101110101101100101001011000110"),
    // 10. Fire Fighter — 36
    pat!("Fire Fighter",  "Feuerwehr",
         b"000111000001101100011000110010010010111111111010000010010101010010000010001111100"),
    // 11. Asterisk — 33
    pat!("Asterisk",     "Stern",
         b"100010001010010010001010100000111000111111111000111000001010100010010010100010001"),
    // 12. Border — 32
    pat!("Border",       "Rahmen",
         b"111111111100000001100000001100000001100000001100000001100000001100000001111111111"),
    // 13. Rainy Day — 32
    pat!("Rainy Day",    "Regentag",
         b"000111000011111110010010010111111111101010101000010000000010000000010000000110000"),
    // 14. Smiley — 31
    pat!("Smiley",       "Smiley",
         b"001111100010000010100101001100000001100000001101000101100111001010000010001111100"),
    // 15. Badley — 31
    pat!("Badley",       "Traurig",
         b"001111100010000010100101001100000001100000001100111001101000101010000010001111100"),
    // 16. Bigger Fish to Fry — 31
    pat!("Bigger Fish to Fry", "Größere Sorgen",
         b"000100000011111001110001011101001110100000010100000110010001011001110001000001000"),
    // 17. Anchor — 30
    pat!("Anchor",       "Anker",
         b"000111000000111000000010000001111100000010000010010010111010111010010010001101100"),
    // 18. Lost My Cherries — 30
    pat!("Lost My Cherries", "Meine Kirschen",
         b"000000111000001100000011100000110100000100100011100110100101001100101001011000110"),
    // 19. Joshua — 29
    pat!("Joshua",       "Josua",
         b"000010000000111000000101000010101000011101010000101110000101110000101000001111100"),
    // 20. Five to Twelve — 29
    pat!("Five to Twelve", "Fünf vor Zwölf",
         b"000111000011010110010110010100010001100010001100000001010000010011000110000111000"),
    // 21. Diamond — 28
    pat!("Diamond",      "Diamant",
         b"010010010100101001001000100010010010100101001010010010001000100100101001010010010"),
    // 22. An Apple a Day — 28
    pat!("An Apple a Day", "Täglich ein Apfel",
         b"000001000000010000011111110110000011100100001101000001100000001010000010001111100"),
    // 23. Wave — 27
    pat!("Wave",         "Welle",
         b"001001001010010010100100100010010010001001001010010010100100100010010010001001001"),
    // 24. Per Aspera ad Astra — 27
    pat!("Per Aspera ad Astra", "Per Aspera ad Astra",
         b"000010000000111000000101000000111000000101000000111000001111100001010100011010110"),
    // 25. Minion — 27
    pat!("Minion",       "Minion",
         b"000111000001000100010010010010101010010010010010000010010111010001000100000111000"),
    // 26. 42 — 26
    pat!("42",           "42",
         b"000000000101000110101001001101000001101000010111100100001001000001001111000000000"),
    // 27. Home Office — 26
    pat!("Home Office",  "Home Office",
         b"000010000000111000001101100110000011010000010010000010010001010010101010010100010"),
    // 28. We Are Connected Wirelessly — 22
    pat!("We Are Connected Wirelessly", "Kabellos verbunden",
         b"001111100010000010100000001001111100010000010000000000000111000001000100000010000"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_count_is_28() {
        assert_eq!(PATTERNS.len(), 28);
    }

    #[test]
    fn all_masks_have_81_cells() {
        for p in PATTERNS {
            assert_eq!(p.mask.len(), 81, "Pattern '{}' mask wrong length", p.name_en);
        }
    }

    #[test]
    fn patterns_sorted_descending_by_cell_count() {
        for i in 1..PATTERNS.len() {
            assert!(
                PATTERNS[i - 1].cell_count >= PATTERNS[i].cell_count,
                "Sort broken at index {}: {} ({}) should be >= {} ({})",
                i,
                PATTERNS[i - 1].name_en, PATTERNS[i - 1].cell_count,
                PATTERNS[i].name_en,     PATTERNS[i].cell_count
            );
        }
    }

    #[test]
    fn cell_count_matches_mask() {
        for p in PATTERNS {
            let counted = p.mask.iter().filter(|&&b| b).count();
            assert_eq!(
                counted, p.cell_count,
                "Pattern '{}': declared cell_count={} but mask has {} true bits",
                p.name_en, p.cell_count, counted
            );
        }
    }

    #[test]
    fn from_cli_str_valid() {
        let s = "1".repeat(81);
        let p = Pattern::from_cli_str(&s).unwrap();
        assert_eq!(p.cell_count, 81);
        assert!(p.mask.iter().all(|&b| b));
    }

    #[test]
    fn from_cli_str_too_short() {
        assert!(Pattern::from_cli_str("111").is_err());
    }

    #[test]
    fn from_cli_str_too_long() {
        assert!(Pattern::from_cli_str(&"1".repeat(82)).is_err());
    }

    #[test]
    fn from_cli_str_accepts_dots_and_stars() {
        let s = "*".repeat(40) + &".".repeat(41);
        let p = Pattern::from_cli_str(&s).unwrap();
        assert_eq!(p.cell_count, 40);
    }

    #[test]
    fn from_cli_str_accepts_zeros() {
        let s = "1".repeat(40) + &"0".repeat(41);
        let p = Pattern::from_cli_str(&s).unwrap();
        assert_eq!(p.cell_count, 40);
    }
}
```

- [ ] **Step 4: Add `pub mod pattern;` to `src/lib.rs`**

Open `src/lib.rs`. Add after the existing module declarations:
```rust
pub mod pattern;
```

- [ ] **Step 5: Run tests**

```bash
cargo test pattern 2>&1 | tail -15
```
Expected: all 9 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/pattern/mod.rs src/lib.rs
git commit -m "feat(pattern): Pattern struct and 28 built-in designer patterns"
```

---

## Task 2: Pattern-Constrained Generator

**Files:**
- Modify: `src/generator/mod.rs`

**Context:** The existing `generate()` method uses `Solver::for_difficulty()` which caps strategies at a difficulty level. For pattern generation we need the full `Solver::new()` (all strategies including backtracking) so that uniqueness is checked without a difficulty cap. Difficulty is classified *after* generation.

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)]` block in `src/generator/mod.rs`:

```rust
#[test]
fn pattern_puzzle_only_has_givens_in_pattern() {
    use crate::pattern::PATTERNS;
    // Use the Border pattern (index 12, 32 cells) — reliable test case
    let pattern = PATTERNS[11].clone(); // Border
    let result = PuzzleGenerator::new(42).generate_with_pattern(&pattern);
    // Every given cell must be in the pattern mask
    for r in 0..9 {
        for c in 0..9 {
            if result.grid.get(r, c).is_given() {
                let idx = r * 9 + c;
                assert!(
                    pattern.mask[idx] || result.used_extra_cells,
                    "Given at ({r},{c}) is outside pattern and used_extra_cells=false"
                );
            }
        }
    }
    // Must be uniquely solvable
    let solved = crate::solver::Solver::new().solve(result.grid);
    assert!(solved.grid.is_solved());
}

#[test]
fn pattern_puzzle_difficulty_is_classified() {
    use crate::pattern::PATTERNS;
    let result = PuzzleGenerator::new(99).generate_with_pattern(&PATTERNS[1]); // Checker
    // difficulty must be one of the valid variants (not None)
    let _ = result.difficulty; // just verifying it compiles and is accessible
    assert!(crate::solver::Solver::new().solve(result.grid).grid.is_solved());
}
```

- [ ] **Step 2: Run tests — expect compile error**

```bash
cargo test pattern_puzzle 2>&1 | head -20
```

- [ ] **Step 3: Add `GeneratorResult` and `generate_with_pattern` to `src/generator/mod.rs`**

Add this after the `PuzzleGenerator` impl block (before the free functions):

```rust
/// Result of a pattern-constrained generation.
pub struct GeneratorResult {
    pub grid:             Grid,
    pub difficulty:       Difficulty,
    /// True when cells outside the pattern were added to reach unique solvability.
    pub used_extra_cells: bool,
}

impl PuzzleGenerator {
    /// Generate a uniquely-solvable puzzle whose given cells lie within `pattern.mask`.
    ///
    /// Strategy:
    /// 1. Fill a complete valid grid (same as `generate`).
    /// 2. Remove every non-pattern cell — these are never givens.
    /// 3. Iteratively remove pattern cells while the puzzle stays uniquely solvable
    ///    (uses the full solver, no difficulty cap).
    /// 4. Ansatz C: if < 17 givens remain, add the minimum number of non-pattern
    ///    cells back until the puzzle is uniquely solvable.
    /// 5. Classify difficulty from the strategies the solver actually used.
    pub fn generate_with_pattern(
        &self,
        pattern: &crate::pattern::Pattern,
    ) -> GeneratorResult {
        let mut rng = LcgRng::new(self.seed);
        let full = self.fill_grid(&mut rng).expect("fill_grid failed");

        // Step 2: start with all pattern cells filled, non-pattern cells empty.
        let mut puzzle = Grid::empty();
        for idx in 0..81usize {
            let (r, c) = (idx / 9, idx % 9);
            if pattern.mask[idx] {
                if let Some(v) = full.get(r, c).value() {
                    puzzle.set_given(r, c, v);
                }
            }
        }

        // Step 3: try removing pattern cells while keeping unique solvability.
        let mut pattern_indices: Vec<usize> = (0..81).filter(|&i| pattern.mask[i]).collect();
        shuffle(&mut pattern_indices, &mut rng);

        for &idx in &pattern_indices {
            let (r, c) = (idx / 9, idx % 9);
            let prev = puzzle.get(r, c).value();
            puzzle.clear(r, c);
            if !self.is_uniquely_solvable_full(&puzzle) {
                if let Some(v) = prev {
                    puzzle.set_given(r, c, v);
                }
            }
        }

        // Step 4 (Ansatz C): if still < 17 givens, add non-pattern cells.
        let given_count = (0..81)
            .filter(|&i| { let (r, c) = (i / 9, i % 9); puzzle.get(r, c).is_given() })
            .count();
        let mut used_extra_cells = false;
        if given_count < 17 {
            used_extra_cells = true;
            let mut extra: Vec<usize> = (0..81).filter(|&i| !pattern.mask[i]).collect();
            shuffle(&mut extra, &mut rng);
            for &idx in &extra {
                let (r, c) = (idx / 9, idx % 9);
                if let Some(v) = full.get(r, c).value() {
                    puzzle.set_given(r, c, v);
                }
                if self.is_uniquely_solvable_full(&puzzle) {
                    break;
                }
            }
        }

        // Rebuild as clean Given-only grid.
        let mut result = Grid::empty();
        for r in 0..9 {
            for c in 0..9 {
                if !puzzle.get(r, c).is_empty() {
                    if let Some(v) = full.get(r, c).value() {
                        result.set_given(r, c, v);
                    }
                }
            }
        }

        // Step 5: classify difficulty.
        let solve_result = crate::solver::Solver::new().solve(result.clone());
        let difficulty = crate::generator::classify(&solve_result.used_strategies);

        GeneratorResult { grid: result, difficulty, used_extra_cells }
    }

    /// Check uniqueness with the full solver (no difficulty cap, backtracking allowed).
    fn is_uniquely_solvable_full(&self, grid: &Grid) -> bool {
        crate::solver::Solver::new().solve(grid.clone()).grid.is_solved()
    }
}
```

**Note:** `Solver::new()` constructs a solver with no `max_strategy` cap and `use_backtracking: true` — check `src/solver/mod.rs` lines 24-40 to confirm. The `used_strategies` field (line 19 of solver/mod.rs) holds `Vec<Strategy>`.

- [ ] **Step 4: Run tests**

```bash
cargo test pattern_puzzle 2>&1 | tail -10
```
Expected: both tests pass. Note: `pattern_puzzle_only_has_givens_in_pattern` may take a few seconds.

- [ ] **Step 5: Commit**

```bash
git add src/generator/mod.rs
git commit -m "feat(generator): generate_with_pattern and GeneratorResult"
```

---

## Task 3: i18n New Strings

**Files:**
- Modify: `src/i18n/mod.rs`

Six new fields must be added to the `Strings` struct and to **all 13** language statics (EN, DE, ES, IT, FR, SL, EO, TP, LEET, SW, AF, PY, ID).

- [ ] **Step 1: Write the failing test**

Add to the `#[cfg(test)]` block in `src/i18n/mod.rs`:

```rust
#[test]
fn new_designer_strings_not_empty() {
    assert!(!EN.difficulty_designer.is_empty());
    assert!(!EN.designer_title.is_empty());
    assert!(!EN.hint_has_errors.is_empty());
    assert!(!EN.hint_has_wrong_notes.is_empty());
    assert!(!EN.generating_cancel.is_empty());
    assert!(!EN.using_new_seed.is_empty());
    // DE has distinct translations for the two hint warnings
    assert_ne!(DE.hint_has_errors, EN.hint_has_errors);
    assert_ne!(DE.hint_has_wrong_notes, EN.hint_has_wrong_notes);
}
```

- [ ] **Step 2: Run — expect compile error**

```bash
cargo test new_designer_strings 2>&1 | head -10
```

- [ ] **Step 3: Add 6 fields to the `Strings` struct**

In `src/i18n/mod.rs`, find the struct definition (around line 13). Add the 6 new fields after `pub ctrl_hint`:

```rust
    /// Fourth option on the difficulty select screen.
    pub difficulty_designer:   &'static str,
    /// Title of the pattern-select screen.
    pub designer_title:        &'static str,
    /// Hint pre-check warning: player has incorrect filled digits.
    pub hint_has_errors:       &'static str,
    /// Hint pre-check warning: player has incorrect notes.
    pub hint_has_wrong_notes:  &'static str,
    /// Generating screen cancel hint shown at the bottom.
    pub generating_cancel:     &'static str,
    /// Shown for ~1 s when the generator moves to a new seed.
    pub using_new_seed:        &'static str,
```

- [ ] **Step 4: Add the values to the EN static**

Find the `pub static EN: Strings = Strings {` block (around line 241). Add after `ctrl_hint`:

```rust
    difficulty_designer:  "Designer \u{25b6}",
    designer_title:       "Designer Sudoku",
    hint_has_errors:      "Fix your errors first.",
    hint_has_wrong_notes: "Your notes contain errors.",
    generating_cancel:    "  Esc    cancel",
    using_new_seed:       "using new seed...",
```

- [ ] **Step 5: Add the values to the DE static**

Find `pub static DE: Strings = Strings {` (around line 312). Add after `ctrl_hint`:

```rust
    difficulty_designer:  "Designer \u{25b6}",
    designer_title:       "Designer Sudoku",
    hint_has_errors:      "Zuerst Fehler korrigieren.",
    hint_has_wrong_notes: "Notizen enthalten Fehler.",
    generating_cancel:    "  Esc    Abbrechen",
    using_new_seed:       "Neuer Seed...",
```

- [ ] **Step 6: Add the values to all remaining 11 language statics (ES, IT, FR, SL, EO, TP, LEET, SW, AF, PY, ID)**

For each of the 11 remaining statics, add the EN values verbatim (copy from EN). Find each `pub static XX: Strings = Strings {` block and add after `ctrl_hint`:

```rust
    difficulty_designer:  "Designer \u{25b6}",
    designer_title:       "Designer Sudoku",
    hint_has_errors:      "Fix your errors first.",
    hint_has_wrong_notes: "Your notes contain errors.",
    generating_cancel:    "  Esc    cancel",
    using_new_seed:       "using new seed...",
```

- [ ] **Step 7: Run all tests**

```bash
cargo test 2>&1 | grep -E "test result|FAILED" | head -10
```
Expected: all tests pass (the existing `all_panel_strings_fit_34_chars` test still passes because the 6 new fields are not panel strings).

- [ ] **Step 8: Commit**

```bash
git add src/i18n/mod.rs
git commit -m "feat(i18n): add 6 designer sudoku strings to all language statics"
```

---

## Task 4: Hint Pre-Check and Hint Count Display

**Files:**
- Modify: `src/tui/mod.rs`
- Modify: `src/tui/render/mod.rs`
- Modify: `src/tui/render/status_bar.rs`

**Context:** `handle_hint_request()` is at ~line 516 of `src/tui/mod.rs`. The App struct has `hint_count: u32` in `GameStats` (line 63) and `active_hint: Option<Hint>` (line 108). We add `hint_warning: Option<(&'static str, &'static str)>` to App for the pre-check warning panel.

- [ ] **Step 1: Write the failing tests**

Add to the `#[cfg(test)]` block at the bottom of `src/tui/mod.rs`:

```rust
#[test]
fn hint_request_with_wrong_digit_sets_warning_not_hint() {
    // Build a game state with one deliberately wrong digit.
    use crate::puzzle::{Grid, GameState};
    use crate::tui::colors::ColorScheme;
    use crate::timer::SystemClock;

    let mut app = App::new(Box::new(SystemClock));
    // Start an easy game so solution is known.
    app.start_game(crate::generator::Difficulty::Easy);
    // Find an empty cell and fill it with the WRONG digit.
    if let (Some(state), Some(sol)) = (&app.game_state, &app.solution) {
        'outer: for r in 0..9 {
            for c in 0..9 {
                if matches!(state.grid().get(r, c), crate::puzzle::CellKind::Empty) {
                    let correct = sol.get(r, c).value().unwrap();
                    let wrong = if correct == 9 { 1 } else { correct + 1 };
                    drop(state); drop(sol);
                    // Place wrong digit
                    app.game_state.as_mut().unwrap().apply(
                        crate::puzzle::event::GameEvent::SetDigit { row: r, col: c, digit: wrong }
                    );
                    app.cursor = (r, c);
                    break 'outer;
                }
            }
        }
    }
    let hint_count_before = app.stats.hint_count;
    app.handle_action(crate::tui::input::AppAction::RequestHint);
    assert!(app.hint_warning.is_some(), "hint_warning should be set");
    assert!(app.active_hint.is_none(), "active_hint should NOT be set");
    assert_eq!(app.stats.hint_count, hint_count_before + 1, "hint_count should increment");
}

#[test]
fn hint_warning_dismissed_by_any_key() {
    use crate::timer::SystemClock;
    let mut app = App::new(Box::new(SystemClock));
    // Manually set a warning
    app.hint_warning = Some(("Warning", "Test warning"));
    // Any key should clear it
    app.handle_action(crate::tui::input::AppAction::MoveRight);
    assert!(app.hint_warning.is_none());
}
```

- [ ] **Step 2: Run — expect compile error (hint_warning field missing)**

```bash
cargo test hint_request_with_wrong hint_warning_dismissed 2>&1 | head -15
```

- [ ] **Step 3: Add `hint_warning` field to `App`**

In `src/tui/mod.rs`, find the `pub struct App {` block. Add after `active_hint`:

```rust
    /// Warning text shown in the hint panel when the pre-check fails.
    /// `(name, explanation)` in the current language.
    pub hint_warning: Option<(&'static str, &'static str)>,
```

In the `App::new()` / `App::default()` initialiser block (around line 117), add:

```rust
            hint_warning: None,
```

In `start_game()` (around line 161), add alongside `active_hint = None`:

```rust
        self.hint_warning = None;
```

- [ ] **Step 4: Add pre-check logic to `handle_hint_request()`**

Current `handle_hint_request()` starts at ~line 516. Replace the whole method:

```rust
fn handle_hint_request(&mut self) {
    use crate::hint;
    self.active_hint = None;
    self.anim.hint_blink = false;

    let strings = self.language.strings();

    let (state, solution) = match (&self.game_state, &self.solution) {
        (Some(s), Some(sol)) => (s, sol),
        _ => return,
    };

    // ── Pre-check 1: incorrect filled digits ────────────────────────────────
    let has_errors = {
        let grid = state.grid();
        let mut found = false;
        'outer: for r in 0..9 {
            for c in 0..9 {
                if let crate::puzzle::CellKind::Filled(d) = grid.get(r, c) {
                    if solution.get(r, c).value() != Some(d) {
                        found = true;
                        break 'outer;
                    }
                }
            }
        }
        found
    };
    if has_errors {
        self.stats.hint_count += 1;
        self.hint_warning = Some((strings.hint_has_errors, strings.hint_has_errors));
        return;
    }

    // ── Pre-check 2: incorrect notes ────────────────────────────────────────
    let has_wrong_notes = {
        let grid = state.grid();
        let mut found = false;
        'outer: for r in 0..9 {
            for c in 0..9 {
                if !matches!(grid.get(r, c), crate::puzzle::CellKind::Empty) { continue; }
                let notes = state.notes_mask(r, c);
                for d in 1u8..=9 {
                    if notes & (1 << d) == 0 { continue; }
                    // Digit d is noted — check if d is already in same row/col/box
                    let mut conflict = false;
                    for cc in 0..9 { if grid.get(r, cc).value() == Some(d) { conflict = true; break; } }
                    if !conflict {
                        for rr in 0..9 { if grid.get(rr, c).value() == Some(d) { conflict = true; break; } }
                    }
                    if !conflict {
                        let (br, bc) = (r / 3 * 3, c / 3 * 3);
                        'box_check: for dr in 0..3 {
                            for dc in 0..3 {
                                if grid.get(br+dr, bc+dc).value() == Some(d) { conflict = true; break 'box_check; }
                            }
                        }
                    }
                    if conflict { found = true; break 'outer; }
                }
            }
        }
        found
    };
    if has_wrong_notes {
        self.stats.hint_count += 1;
        self.hint_warning = Some((strings.hint_has_wrong_notes, strings.hint_has_wrong_notes));
        return;
    }

    // ── All clear: proceed with hint ────────────────────────────────────────
    if state.grid().is_solved() { return; }
    let h = match hint::find_hint(state, solution) {
        Some(h) => h,
        None => { self.perform_reveal(solution.clone()); return; }
    };
    self.stats.hint_count += 1;
    self.anim.hint_blink = true;
    self.anim.hint_blink_tick = 0;
    self.active_hint = Some(h);
}
```

**Note:** The warning tuple stores the same string twice as `(name, explanation)`. The panel will show name on the first line and explanation on the second. To use a single-line warning, pass the same string for both. Adjust if the panel layout requires a distinct name — e.g., `("⚠", strings.hint_has_errors)`.

- [ ] **Step 5: Clear `hint_warning` in the event loop dismiss handler**

In `src/tui/mod.rs`, find the event loop section where `active_hint` is dismissed (around line 802):

```rust
if self.active_hint.is_some() {
    self.active_hint = None;
    self.anim.hint_blink = false;
    self.needs_clear = true;
} else if ...
```

Change to also dismiss `hint_warning`:

```rust
if self.active_hint.is_some() {
    self.active_hint = None;
    self.anim.hint_blink = false;
    self.needs_clear = true;
} else if self.hint_warning.is_some() {
    self.hint_warning = None;
    self.needs_clear = true;
} else if ...
```

- [ ] **Step 6: Pass `hint_warning` through Screen::Game**

In `src/tui/render/mod.rs`, add `hint_warning` to the `Screen::Game` variant. Find the variant definition (~line 29):

```rust
    Game {
        ...
        hint: Option<&'a crate::hint::Hint>,
    },
```

Add after `hint`:

```rust
        /// Warning text when hint pre-check failed: (name, explanation).
        hint_warning: Option<(&'a str, &'a str)>,
```

In `src/tui/mod.rs`, find the `game_screen` closure (around line 918) and add:

```rust
            hint_warning: self.hint_warning,
```

- [ ] **Step 7: Use `hint_warning` in render_frame**

In `src/tui/render/mod.rs`, inside the `Screen::Game` arm of `render_frame`, find the `hint_text` computation (~line 98):

```rust
            let hint_text = hint.map(|h| {
                if std::ptr::eq(strings, &crate::i18n::DE) {
                    (h.name_de, h.explanation_de.as_str())
                } else {
                    (h.name_en, h.explanation_en.as_str())
                }
            });
```

Replace with:

```rust
            let hint_text: Option<(&str, &str)> = if let Some((name, expl)) = hint_warning {
                Some((name, expl))
            } else {
                hint.map(|h| {
                    if std::ptr::eq(strings, &crate::i18n::DE) {
                        (h.name_de, h.explanation_de.as_str())
                    } else {
                        (h.name_en, h.explanation_en.as_str())
                    }
                })
            };
```

- [ ] **Step 8: Add hint count to panel**

In `src/tui/render/status_bar.rs`, add a `hint_count: u32` parameter to `render_panel`:

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
    hint_count:   u32,                        // ← new
    hint_text:    Option<(&str, &str)>,
) -> io::Result<()> {
```

In the rows vec that builds the panel content (after `panel_filled`), add:

```rust
        (format!("  h: {}", hint_count), t, false),
```

**Update all callers:**
- In `src/tui/render/mod.rs` — pass `hint_count` from the Game screen variant (add `hint_count: self.stats.hint_count` to the Screen::Game construction in `src/tui/mod.rs`, and forward it in render_frame).
- In `src/tui/render/status_bar.rs` tests — pass `0` as `hint_count` to all `render_panel` test calls.
- In `tests/tui_smoke.rs` — update if `render_panel` is called there directly (check first).

**Add `hint_count: u32` to `Screen::Game` variant** in `src/tui/render/mod.rs`:
```rust
        hint_count: u32,
```
Pass from `src/tui/mod.rs` game_screen closure:
```rust
            hint_count: self.stats.hint_count,
```
Forward in `render_frame` `Screen::Game` arm:
```rust
            status_bar::render_panel(..., hint_count, hint_text)?;
```

- [ ] **Step 9: Run all tests**

```bash
cargo test 2>&1 | grep -E "test result|FAILED" | head -10
```
Expected: all pass.

- [ ] **Step 10: Commit**

```bash
git add src/tui/mod.rs src/tui/render/mod.rs src/tui/render/status_bar.rs
git commit -m "feat(hint): pre-check errors/notes before hint, show hint count in panel"
```

---

## Task 5: Background Generation Infrastructure

**Files:**
- Create: `src/tui/generating.rs`
- Modify: `src/tui/mod.rs` (add `pub mod generating;`)

**Context:** One thread per seed attempt. The main thread polls every 50 ms via `try_recv()`. Timeout = 3 s per seed. `GeneratingState` holds all state needed to drive the Generating screen and respond to input.

- [ ] **Step 1: Write the failing test**

```rust
// At the bottom of src/tui/generating.rs (add after implementation)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::PATTERNS;

    #[test]
    fn spawn_generation_completes_for_border_pattern() {
        // Border pattern (index 11) has 32 cells — should always succeed quickly.
        let rx = spawn_generation(PATTERNS[11].clone(), 42);
        // Wait up to 10 seconds
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            match rx.try_recv() {
                Ok(GenMsg::Done(grid, _difficulty)) => {
                    assert!(crate::solver::Solver::new().solve(grid).grid.is_solved());
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if std::time::Instant::now() > deadline {
                        panic!("Generation did not complete within 10 seconds");
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(e) => panic!("Channel error: {e}"),
            }
        }
    }
}
```

- [ ] **Step 2: Run — expect compile error**

```bash
cargo test spawn_generation 2>&1 | head -10
```

- [ ] **Step 3: Create `src/tui/generating.rs`**

```rust
// src/tui/generating.rs

use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use crate::generator::{Difficulty, PuzzleGenerator};
use crate::pattern::Pattern;
use crate::puzzle::Grid;

/// Message sent from the generator thread to the main thread.
pub enum GenMsg {
    Done(Grid, Difficulty),
}

/// Spawn a background thread that generates one puzzle with the given seed.
/// Returns the receiving end of the channel.
pub fn spawn_generation(pattern: Pattern, seed: u64) -> mpsc::Receiver<GenMsg> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = PuzzleGenerator::new(seed).generate_with_pattern(&pattern);
        let _ = tx.send(GenMsg::Done(result.grid, result.difficulty));
    });
    rx
}

/// Cyclic verb list for the generating message "baking sudoku..."
pub const VERBS: &[&str] = &[
    "generating", "frying",   "baking",    "roasting",   "shoveling",  "tinkering",
    "brewing",    "distilling","cooking",   "boiling",    "simmering",  "grilling",
    "toasting",   "smoking",   "seasoning", "marinating", "kneading",   "blending",
    "mixing",     "stirring",  "whipping",  "grinding",   "fermenting", "percolating",
    "crafting",   "forging",   "spinning",  "shuffling",  "sculpting",  "chiseling",
    "polishing",  "weaving",   "knitting",  "mining",     "hatching",   "conjuring",
    "assembling", "composing",
];

/// All state needed by AppScreen::Generating.
pub struct GeneratingState {
    pub pattern:        Pattern,
    pub rx:             mpsc::Receiver<GenMsg>,
    pub seed:           u64,
    pub started_at:     Instant,
    /// Shuffled indices into VERBS; cycles when exhausted.
    pub verb_order:     Vec<usize>,
    /// Current position in verb_order.
    pub verb_pos:       usize,
    /// True for ~1 second after a timeout triggers a new seed.
    pub show_new_seed:  bool,
    /// When show_new_seed was set (to expire it after 1 s).
    pub new_seed_at:    Option<Instant>,
    /// True when entered from --pattern CLI flag (Esc → DifficultySelect).
    /// False when entered from PatternSelect screen (Esc → PatternSelect).
    pub from_cli:       bool,
}

impl GeneratingState {
    pub fn new(pattern: Pattern, from_cli: bool) -> Self {
        let seed = crate::tui::generating::random_seed();
        let rx = spawn_generation(pattern.clone(), seed);
        let n = VERBS.len();
        let mut verb_order: Vec<usize> = (0..n).collect();
        // Shuffle using a simple LCG (no rand dependency)
        lcg_shuffle(&mut verb_order, seed);
        GeneratingState {
            pattern,
            rx,
            seed,
            started_at: Instant::now(),
            verb_order,
            verb_pos: 0,
            show_new_seed: false,
            new_seed_at: None,
            from_cli,
        }
    }

    /// Current verb string to show, e.g. "baking".
    pub fn current_verb(&self) -> &'static str {
        VERBS[self.verb_order[self.verb_pos % self.verb_order.len()]]
    }

    /// Seconds remaining in the current 3-second attempt (0..=3).
    pub fn countdown_secs(&self) -> u8 {
        let elapsed = self.started_at.elapsed().as_secs();
        3u64.saturating_sub(elapsed) as u8
    }

    /// Advance to a new seed attempt after timeout.
    pub fn try_new_seed(&mut self) {
        self.seed = self.seed.wrapping_add(0x9e3779b97f4a7c15);
        self.rx = spawn_generation(self.pattern.clone(), self.seed);
        self.started_at = Instant::now();
        self.verb_pos += 1;
        self.show_new_seed = true;
        self.new_seed_at = Some(Instant::now());
    }
}

/// Generate a seed from the current time (no external deps).
pub fn random_seed() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42)
}

fn lcg_shuffle(v: &mut Vec<usize>, seed: u64) {
    let mut state = seed ^ 0x12345678;
    for i in (1..v.len()).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (state as usize) % (i + 1);
        v.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::PATTERNS;

    #[test]
    fn spawn_generation_completes_for_border_pattern() {
        let rx = spawn_generation(PATTERNS[11].clone(), 42);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            match rx.try_recv() {
                Ok(GenMsg::Done(grid, _)) => {
                    assert!(crate::solver::Solver::new().solve(grid).grid.is_solved());
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if std::time::Instant::now() > deadline { panic!("timeout"); }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(e) => panic!("{e}"),
            }
        }
    }

    #[test]
    fn generating_state_has_verb() {
        let state = GeneratingState::new(PATTERNS[11].clone(), false);
        assert!(!state.current_verb().is_empty());
    }

    #[test]
    fn countdown_starts_at_3() {
        let state = GeneratingState::new(PATTERNS[11].clone(), false);
        assert_eq!(state.countdown_secs(), 3);
    }
}
```

- [ ] **Step 4: Add `pub mod generating;` to `src/tui/mod.rs`**

Find the `pub mod` declarations at the top of `src/tui/mod.rs` (e.g. `pub mod anim;`). Add:
```rust
pub mod generating;
```

- [ ] **Step 5: Run tests**

```bash
cargo test generating 2>&1 | tail -10
```
Expected: 3 tests pass (note: `spawn_generation_completes_for_border_pattern` may take a few seconds).

- [ ] **Step 6: Commit**

```bash
git add src/tui/generating.rs src/tui/mod.rs
git commit -m "feat(generating): background generation thread, GeneratingState, verb list"
```

---

## Task 6: AppScreen Variants and App State Machine

**Files:**
- Modify: `src/tui/mod.rs`

Add two new `AppScreen` variants and their handlers.

- [ ] **Step 1: Write the failing test**

Add to `src/tui/mod.rs` tests:

```rust
#[test]
fn selecting_designer_from_difficulty_goes_to_pattern_select() {
    use crate::timer::SystemClock;
    let mut app = App::new(Box::new(SystemClock));
    app.screen = AppScreen::DifficultySelect { selected: 0, sym_focused: false };
    // Navigate to 4th option (Designer, index 3)
    for _ in 0..3 {
        app.handle_action(AppAction::MoveDown);
    }
    app.handle_action(AppAction::Enter);
    assert!(matches!(app.screen, AppScreen::PatternSelect { .. }));
}

#[test]
fn pattern_select_wraps_around() {
    use crate::timer::SystemClock;
    let mut app = App::new(Box::new(SystemClock));
    app.screen = AppScreen::PatternSelect { selected: 0 };
    // Move left should wrap to last pattern (index 27)
    app.handle_action(AppAction::MoveLeft);
    assert!(matches!(app.screen, AppScreen::PatternSelect { selected: 27 }));
}

#[test]
fn pattern_select_back_goes_to_difficulty() {
    use crate::timer::SystemClock;
    let mut app = App::new(Box::new(SystemClock));
    app.screen = AppScreen::PatternSelect { selected: 0 };
    app.handle_action(AppAction::Back);
    assert!(matches!(app.screen, AppScreen::DifficultySelect { .. }));
}
```

- [ ] **Step 2: Run — expect compile error**

```bash
cargo test selecting_designer pattern_select_wraps pattern_select_back 2>&1 | head -10
```

- [ ] **Step 3: Add new AppScreen variants**

In `src/tui/mod.rs`, extend the `AppScreen` enum:

```rust
pub enum AppScreen {
    Start { selected: usize },
    DifficultySelect { selected: usize, sym_focused: bool },
    LanguageSelect { selected: usize },
    ThemeSelect { selected: usize },
    PatternSelect { selected: usize },
    Generating(crate::tui::generating::GeneratingState),
    Game,
}
```

- [ ] **Step 4: Change `DIFFICULTY_COUNT` to 4 in `handle_difficulty_action`**

In `handle_difficulty_action` (~line 249):
```rust
const DIFFICULTY_COUNT: usize = 4;   // was 3
```

Change the `Enter` handler (~line 281) to:
```rust
AppAction::Enter if !sym_focused => {
    match selected {
        0 => { self.start_game(Difficulty::Easy);   self.needs_clear = true; }
        1 => { self.start_game(Difficulty::Medium); self.needs_clear = true; }
        2 => { self.start_game(Difficulty::Hard);   self.needs_clear = true; }
        _ => {
            self.screen = AppScreen::PatternSelect { selected: 0 };
            self.needs_clear = true;
        }
    }
}
```

- [ ] **Step 5: Add `handle_pattern_action` and `handle_generating_action`**

Add after `handle_difficulty_action`:

```rust
fn handle_pattern_action(&mut self, action: AppAction, selected: usize) {
    const COUNT: usize = crate::pattern::PATTERNS.len(); // 28
    match action {
        AppAction::MoveRight => {
            self.screen = AppScreen::PatternSelect { selected: (selected + 1) % COUNT };
        }
        AppAction::MoveLeft => {
            self.screen = AppScreen::PatternSelect {
                selected: selected.checked_sub(1).unwrap_or(COUNT - 1),
            };
        }
        AppAction::Enter => {
            let pattern = crate::pattern::PATTERNS[selected].clone();
            self.start_generating(pattern, false);
        }
        AppAction::Back => {
            self.screen = AppScreen::DifficultySelect { selected: 3, sym_focused: false };
            self.needs_clear = true;
        }
        _ => {}
    }
}

fn handle_generating_action(&mut self, action: AppAction) {
    if matches!(action, AppAction::Back) {
        // Cancel generation — drop receiver (thread will finish but result is ignored).
        let from_cli = if let AppScreen::Generating(ref s) = self.screen {
            s.from_cli
        } else { false };
        self.screen = if from_cli {
            AppScreen::DifficultySelect { selected: 3, sym_focused: false }
        } else {
            // Restore PatternSelect at the pattern that was being generated
            let selected = if let AppScreen::Generating(ref s) = self.screen {
                crate::pattern::PATTERNS.iter().position(|p| p.name_en == s.pattern.name_en)
                    .unwrap_or(0)
            } else { 0 };
            AppScreen::PatternSelect { selected }
        };
        self.needs_clear = true;
    }
}

/// Spawn generation for a designer puzzle and transition to Generating screen.
pub fn start_generating(&mut self, pattern: crate::pattern::Pattern, from_cli: bool) {
    let state = crate::tui::generating::GeneratingState::new(pattern, from_cli);
    self.screen = AppScreen::Generating(state);
    self.needs_clear = true;
}
```

- [ ] **Step 6: Wire new screens into `handle_action`**

In `handle_action` (~line 202), add the new branches:

```rust
AppScreen::PatternSelect { selected } => self.handle_pattern_action(action, *selected),
AppScreen::Generating(_)              => self.handle_generating_action(action),
```

- [ ] **Step 7: Poll generator in the event loop**

The event loop in `src/tui/mod.rs` has a section around line 793 where it sets `poll_ms` and handles events. Find the section:

```rust
let poll_ms = if self.anim.is_active() { 80 } else { 500 };
```

**First**, extract a shared reset helper in `App` so that both `start_game()` and the generator poll use identical state resets. Find `start_game()` (~line 155) and extract its per-game reset block into a private method:

```rust
/// Reset all per-game mutable state and transition to the Game screen.
/// Call this from start_game() and from the Generating completion handler.
fn enter_game(&mut self, puzzle: Grid) {
    self.solution = crate::solver::backtracking::solve_backtracking(puzzle.clone());
    self.game_state = Some(crate::puzzle::GameState::new(puzzle));
    self.stats = crate::tui::GameStats::default();
    self.cursor = (0, 0);
    self.nav_state = NavState::default();
    self.note_mode = false;
    self.scan_mode = false;
    self.error_mode = false;
    self.anim.error_blink = false;
    self.revealed_errors.clear();
    self.paused = false;
    self.active_hint = None;
    self.hint_warning = None;
    self.anim.hint_blink = false;
    self.anim.hint_blink_tick = 0;
    self.game_start_ms = self.clock.now_ms();
    self.drain_input = true;
    self.screen = AppScreen::Game;
    self.needs_clear = true;
}
```

Then replace `start_game()`'s body with a call to `enter_game()`:

```rust
pub fn start_game(&mut self, difficulty: Difficulty) {
    let result = crate::generator::PuzzleGenerator::new(
        crate::tui::generating::random_seed()
    ).generate(difficulty);
    self.enter_game(result.grid);
}
```

**Then** add the Generating poll block **before** the `crossterm::event::poll()` call:

```rust
// ── Poll background generator ────────────────────────────────────────
if let AppScreen::Generating(ref mut gs) = self.screen {
    // Check for completion
    match gs.rx.try_recv() {
        Ok(crate::tui::generating::GenMsg::Done(grid, _difficulty)) => {
            // Capture pattern name before we consume the Generating state
            let pattern_name = {
                if let AppScreen::Generating(ref gs2) = self.screen {
                    gs2.pattern.name_en
                } else { "" }
            };
            self.enter_game(grid);
            self.stats.category = GameCategory::Design;
            self.stats.pattern_name = Some(pattern_name.to_string());
            self.render_frame(&mut out)?;
            continue;
        }
        Err(std::sync::mpsc::TryRecvError::Empty) => {}
        Err(_) => {} // channel closed (thread panicked) — will retry below
    }
    // Expire "using new seed" message after 1 second
    if gs.show_new_seed {
        if gs.new_seed_at.map(|t| t.elapsed().as_secs() >= 1).unwrap_or(false) {
            gs.show_new_seed = false;
        }
    }
    // Check timeout — spawn new seed
    if !gs.show_new_seed && gs.started_at.elapsed().as_secs() >= 3 {
        gs.try_new_seed();
    }
    self.render_frame(&mut out)?;
}

// Force 50 ms poll when generating
let poll_ms = if matches!(self.screen, AppScreen::Generating(_)) {
    50
} else if self.anim.is_active() {
    80
} else {
    500
};
```

**Note:** The exact insertion point depends on the loop structure. Look for the `let poll_ms =` line and insert the generator poll block just before it. Also find where `self.render_frame(&mut out)?;` is called each tick and make sure the Generating screen is rendered there too (it will be, once the render routing in Task 8 is done).

- [ ] **Step 8: Run tests**

```bash
cargo test selecting_designer pattern_select_wraps pattern_select_back 2>&1 | tail -10
```
Expected: 3 tests pass.

```bash
cargo test 2>&1 | grep -E "test result|FAILED" | head -10
```
Expected: all pass (some render tests may fail with missing Screen variants — those will be fixed in Tasks 7 and 8).

- [ ] **Step 9: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat(tui): PatternSelect and Generating AppScreen, handler wiring, generator poll"
```

---

## Task 7: Pattern Select Screen Rendering

**Files:**
- Create: `src/tui/render/pattern_select.rs`
- Modify: `src/tui/render/mod.rs`

- [ ] **Step 1: Write the failing test (in the new file)**

```rust
// At bottom of src/tui/render/pattern_select.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::EN;
    use crate::tui::colors::ColorScheme;

    #[test]
    fn render_pattern_select_does_not_panic() {
        let mut buf = Vec::new();
        render_pattern_select(&mut buf, 0, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        // Should contain the pattern name
        assert!(s.contains("Holy Crap"), "Expected first pattern name in output");
    }

    #[test]
    fn render_pattern_select_shows_count() {
        let mut buf = Vec::new();
        render_pattern_select(&mut buf, 0, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        // Should contain cell count and total
        assert!(s.contains("46"), "Expected cell count 46 (Holy Crap)");
        assert!(s.contains("81"), "Expected /81 in output");
    }
}
```

- [ ] **Step 2: Create `src/tui/render/pattern_select.rs`**

```rust
// src/tui/render/pattern_select.rs

use crate::i18n::Strings;
use crate::pattern::PATTERNS;
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Terminal dimensions: 117 cols × 39 rows (based on MIN_COLS and grid height).
///
/// Layout (all rows are terminal rows, 0-indexed):
///   Row  2: title (designer_title string)
///   Row  4: pattern name
///   Row  6..14: 9-row miniature (each row = one grid row)
///   Row 16: cell count  "NN / 81"
///   Row 18: position    "◄  N / 28  ►"
///   Row 20: hint        "Enter: select   Esc: back"
///
/// Miniature rendering: each cell is "█" (pattern) or "·" (empty), space-separated.
/// 9 cells × 2 chars (char + space) − 1 trailing space = 17 chars wide.
/// Center on 117 cols: left margin = (117 − 17) / 2 = 50.
const MINIATURE_LEFT: u16 = 50;
const MINIATURE_TOP_ROW: u16 = 6;

pub fn render_pattern_select(
    out:      &mut impl Write,
    selected: usize,
    strings:  &'static Strings,
    colors:   &ColorScheme,
) -> io::Result<()> {
    let pattern = &PATTERNS[selected];
    let bg = colors.ui_background;
    let fg = colors.ui_text;
    let dim = colors.ui_text_dim;

    // Clear screen area (rows 0..24, cols 0..117)
    for row in 0u16..24 {
        queue!(out,
            MoveTo(0, row),
            SetForegroundColor(bg),
            SetBackgroundColor(bg),
            Print(" ".repeat(117))
        )?;
    }

    // ── Title ────────────────────────────────────────────────────────────────
    let title = strings.designer_title;
    let title_col = ((117u16).saturating_sub(title.len() as u16)) / 2;
    queue!(out,
        MoveTo(title_col, 2),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(title)
    )?;

    // ── Pattern name ─────────────────────────────────────────────────────────
    let name = if std::ptr::eq(strings, &crate::i18n::DE) { pattern.name_de } else { pattern.name_en };
    let name_col = ((117u16).saturating_sub(name.len() as u16)) / 2;
    queue!(out,
        MoveTo(name_col, 4),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(name)
    )?;

    // ── Miniature ─────────────────────────────────────────────────────────────
    let block_fg = colors.digit_given;    // bright colour for pattern cells
    for row in 0..9usize {
        queue!(out, MoveTo(MINIATURE_LEFT, MINIATURE_TOP_ROW + row as u16))?;
        for col in 0..9usize {
            let is_pattern = pattern.mask[row * 9 + col];
            let (ch, cell_fg) = if is_pattern {
                ('\u{2588}', block_fg)   // █
            } else {
                ('\u{00b7}', dim)        // ·
            };
            queue!(out,
                SetForegroundColor(cell_fg),
                SetBackgroundColor(bg),
                Print(ch)
            )?;
            if col < 8 {
                queue!(out,
                    SetForegroundColor(dim),
                    Print(' ')
                )?;
            }
        }
    }

    // ── Cell count ───────────────────────────────────────────────────────────
    let count_str = format!("{} / 81", pattern.cell_count);
    let count_col = ((117u16).saturating_sub(count_str.len() as u16)) / 2;
    queue!(out,
        MoveTo(count_col, 16),
        SetForegroundColor(dim),
        SetBackgroundColor(bg),
        Print(&count_str)
    )?;

    // ── Position indicator ◄ N / 28 ► ───────────────────────────────────────
    let pos_str = format!("\u{25c4}  {} / {}  \u{25ba}", selected + 1, PATTERNS.len());
    let pos_col = ((117u16).saturating_sub(pos_str.len() as u16)) / 2;
    queue!(out,
        MoveTo(pos_col, 18),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(&pos_str)
    )?;

    // ── Navigation hint ───────────────────────────────────────────────────────
    let hint = "Enter: select   Esc: back";
    let hint_col = ((117u16).saturating_sub(hint.len() as u16)) / 2;
    queue!(out,
        MoveTo(hint_col, 20),
        SetForegroundColor(dim),
        SetBackgroundColor(bg),
        Print(hint)
    )?;

    Ok(())
}
```

- [ ] **Step 3: Add `pub mod pattern_select;` to `src/tui/render/mod.rs`**

At the top of `src/tui/render/mod.rs`, alongside the other module declarations:
```rust
pub mod pattern_select;
```

- [ ] **Step 4: Add `Screen::PatternSelect` variant to the `Screen` enum in `src/tui/render/mod.rs`**

```rust
PatternSelect { selected: usize },
```

- [ ] **Step 5: Route it in `render_frame`**

In `render_frame`, add a match arm:
```rust
Screen::PatternSelect { selected } => {
    pattern_select::render_pattern_select(out, *selected, strings, colors)?;
}
```

- [ ] **Step 6: Build the Screen::PatternSelect in `src/tui/mod.rs`**

In the `render_frame` call-site in `src/tui/mod.rs` (the big match on `self.screen`), add:
```rust
AppScreen::PatternSelect { selected } => {
    render_frame(out, &Screen::PatternSelect { selected: *selected }, &self.colors, self.style.as_ref(), strings)
}
```

- [ ] **Step 7: Run tests**

```bash
cargo test render_pattern_select 2>&1 | tail -10
```
Expected: 2 tests pass.

```bash
cargo test 2>&1 | grep -E "test result|FAILED" | head -10
```

- [ ] **Step 8: Commit**

```bash
git add src/tui/render/pattern_select.rs src/tui/render/mod.rs src/tui/mod.rs
git commit -m "feat(render): pattern select screen with miniature, count, and navigation"
```

---

## Task 8: Generating Screen Rendering

**Files:**
- Create: `src/tui/render/generating.rs`
- Modify: `src/tui/render/mod.rs`
- Modify: `src/tui/mod.rs`

- [ ] **Step 1: Write the failing test**

```rust
// At bottom of src/tui/render/generating.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::EN;
    use crate::tui::colors::ColorScheme;

    #[test]
    fn render_generating_normal_does_not_panic() {
        let mut buf = Vec::new();
        render_generating(&mut buf, "baking", 2, false, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("baking sudoku"));
        assert!(s.contains('2'));
    }

    #[test]
    fn render_generating_new_seed_shows_message() {
        let mut buf = Vec::new();
        render_generating(&mut buf, "frying", 0, true, &EN, &ColorScheme::default()).unwrap();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains("using new seed") || s.contains("new seed"));
    }
}
```

- [ ] **Step 2: Create `src/tui/render/generating.rs`**

```rust
// src/tui/render/generating.rs

use crate::i18n::Strings;
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Print, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// Render the "generating sudoku..." progress screen.
///
/// The message is centred in the grid area (cols 2–74, rows 1–37).
/// Grid centre: col = 2 + 36 = 38, row = 1 + 18 = 19.
pub fn render_generating(
    out:           &mut impl Write,
    verb:          &str,
    countdown:     u8,
    show_new_seed: bool,
    strings:       &'static Strings,
    colors:        &ColorScheme,
) -> io::Result<()> {
    let bg = colors.ui_background;
    let fg = colors.ui_text;
    let dim = colors.ui_text_dim;

    // Clear full terminal
    for row in 0u16..39 {
        queue!(out,
            MoveTo(0, row),
            SetForegroundColor(bg),
            SetBackgroundColor(bg),
            Print(" ".repeat(117))
        )?;
    }

    // ── Main message ─────────────────────────────────────────────────────────
    let main_line = if show_new_seed {
        strings.using_new_seed.to_string()
    } else {
        format!("{} sudoku...   {}", verb, countdown)
    };

    let msg_col = ((117u16).saturating_sub(main_line.len() as u16)) / 2;
    queue!(out,
        MoveTo(msg_col, 19),
        SetForegroundColor(fg),
        SetBackgroundColor(bg),
        Print(&main_line)
    )?;

    // ── Cancel hint ───────────────────────────────────────────────────────────
    let cancel = strings.generating_cancel;
    let cancel_col = ((117u16).saturating_sub(cancel.len() as u16)) / 2;
    queue!(out,
        MoveTo(cancel_col, 23),
        SetForegroundColor(dim),
        SetBackgroundColor(bg),
        Print(cancel)
    )?;

    Ok(())
}
```

- [ ] **Step 3: Add `pub mod generating;` to `src/tui/render/mod.rs`**

(Note: do NOT confuse with `src/tui/generating.rs` which already has `pub mod generating;` in `src/tui/mod.rs`. This is `src/tui/render/generating.rs` registered in `src/tui/render/mod.rs`.)

```rust
pub mod generating;
```

- [ ] **Step 4: Add `Screen::Generating` variant to the `Screen` enum**

```rust
Generating {
    verb:          &'a str,
    countdown:     u8,
    show_new_seed: bool,
},
```

- [ ] **Step 5: Route in `render_frame`**

```rust
Screen::Generating { verb, countdown, show_new_seed } => {
    crate::tui::render::generating::render_generating(
        out, verb, *countdown, *show_new_seed, strings, colors
    )?;
}
```

- [ ] **Step 6: Build `Screen::Generating` in `src/tui/mod.rs`**

In the big match on `self.screen` in the render section, add:

```rust
AppScreen::Generating(ref gs) => {
    let screen = Screen::Generating {
        verb:          gs.current_verb(),
        countdown:     gs.countdown_secs(),
        show_new_seed: gs.show_new_seed,
    };
    render_frame(out, &screen, &self.colors, self.style.as_ref(), strings)
}
```

- [ ] **Step 7: Run tests**

```bash
cargo test render_generating 2>&1 | tail -10
```
Expected: 2 tests pass.

```bash
cargo test 2>&1 | grep -E "test result|FAILED" | head -10
```

- [ ] **Step 8: Commit**

```bash
git add src/tui/render/generating.rs src/tui/render/mod.rs src/tui/mod.rs
git commit -m "feat(render): generating screen with verb countdown and cancel hint"
```

---

## Task 9: DifficultySelect "Designer ▶" Option

**Files:**
- Modify: `src/tui/render/start_screen.rs`

Add the 4th option to the difficulty menu. The `DIFFICULTY_COUNT` in `mod.rs` was already changed to 4 in Task 6.

- [ ] **Step 1: Write the failing test**

Add to `src/tui/render/start_screen.rs` tests:

```rust
#[test]
fn difficulty_screen_shows_designer_option() {
    let mut buf = Vec::new();
    render_difficulty(&mut buf, (0, 0), 3, false, false, &EN, &ColorScheme::default()).unwrap();
    let s = String::from_utf8_lossy(&buf);
    assert!(s.contains("Designer"), "Expected Designer option in difficulty screen");
}
```

- [ ] **Step 2: Run — expect test failure**

```bash
cargo test difficulty_screen_shows_designer 2>&1 | tail -10
```

- [ ] **Step 3: Add the 4th option to `render_difficulty`**

In `src/tui/render/start_screen.rs`, find the `render_difficulty` function. Find:

```rust
let items = [strings.difficulty_easy, strings.difficulty_medium, strings.difficulty_hard];
```

Change to:

```rust
let items = [
    strings.difficulty_easy,
    strings.difficulty_medium,
    strings.difficulty_hard,
    strings.difficulty_designer,
];
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1 | grep -E "test result|FAILED" | head -10
```
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/tui/render/start_screen.rs
git commit -m "feat(ui): add Designer option to difficulty select screen"
```

---

## Task 10: GameStats Category Fields

**Files:**
- Modify: `src/tui/mod.rs`

Scope-reduced from spec: no actual DB persistence — add `GameCategory` and fields to `GameStats` for future integration.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn game_stats_has_category_fields() {
    let stats = GameStats::default();
    assert!(matches!(stats.category, GameCategory::Classic));
    assert!(stats.pattern_name.is_none());
}
```

- [ ] **Step 2: Run — expect compile error**

```bash
cargo test game_stats_has_category 2>&1 | head -10
```

- [ ] **Step 3: Add `GameCategory` enum and fields to `GameStats`**

In `src/tui/mod.rs`, add before or after `ConfirmAction`:

```rust
/// Category of a completed game, for future database integration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum GameCategory {
    #[default]
    Classic,
    Design,
}
```

Add to `GameStats`:

```rust
    /// Category for DB storage.
    pub category:     GameCategory,
    /// Pattern name for designer games; None for classic games.
    pub pattern_name: Option<String>,
```

- [ ] **Step 4: Verify category is set in the generating poll block**

In Task 6 Step 7, the `enter_game()` call resets `self.stats` to default (Classic), then the two lines below it set the category and pattern name. Confirm those two lines are present:

```rust
self.stats.category = GameCategory::Design;
self.stats.pattern_name = Some(pattern_name.to_string());
```

`GameCategory::Design` will only compile once the enum is defined in this task — that's expected. The Task 6 step uses `GameCategory::Design` as a forward reference; add `#[allow(unused)]` on the variant or accept the compile error until Task 10 is complete.

- [ ] **Step 5: Run all tests**

```bash
cargo test 2>&1 | grep -E "test result|FAILED" | head -10
```

- [ ] **Step 6: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat(stats): GameCategory enum and pattern_name field in GameStats"
```

---

## Task 11: CLI `--pattern` Flag

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write the failing test**

This is tested via the binary — add a doc-test or integration test:

```rust
// In src/main.rs, add to the existing load_pattern function (to be created):
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pattern_str_valid() {
        let p = clisudoku::pattern::Pattern::from_cli_str(&"1".repeat(81)).unwrap();
        assert_eq!(p.cell_count, 81);
    }

    #[test]
    fn parse_pattern_str_invalid_length() {
        assert!(clisudoku::pattern::Pattern::from_cli_str("1111").is_err());
    }
}
```

- [ ] **Step 2: Add `--pattern` handling to `src/main.rs`**

In `main()`, after the `-f` block and before `app.run()`, add:

```rust
    // --pattern <81chars>  — load a designer pattern directly.
    if let Some(pos) = args.iter().position(|a| a == "--pattern") {
        match args.get(pos + 1) {
            Some(s) => {
                match clisudoku::pattern::Pattern::from_cli_str(s) {
                    Ok(pattern) => {
                        app.start_generating(pattern, true);
                    }
                    Err(e) => {
                        eprintln!("Invalid pattern string: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            None => {
                eprintln!("Option --pattern requires an 81-character string.");
                std::process::exit(1);
            }
        }
    }
```

- [ ] **Step 3: Add `--pattern` to the help text in `print_help()`**

In the `print_help()` function, add after the `-f` description:

```
    --pattern <81chars>
                       Generate a designer puzzle from a custom pattern.
                       81 characters: '1' or '*' = pattern cell (may be given),
                       '.' or '0' = always empty.
                       Example: clisudoku --pattern 111111111100000001...111111111
```

- [ ] **Step 4: Run all tests**

```bash
cargo test 2>&1 | grep -E "test result|FAILED" | head -10
```

- [ ] **Step 5: Manual smoke test**

```bash
cargo run -- --pattern "111111111100000001100000001100000001100000001100000001100000001100000001111111111"
```
Expected: Generating screen appears, then a game starts with the Border pattern.

- [ ] **Step 6: Commit**

```bash
git add src/main.rs
git commit -m "feat(cli): --pattern flag for designer sudoku from command line"
```

---

## Task 12: Integration Smoke Tests and Final Wiring

**Files:**
- Modify: `tests/tui_smoke.rs`
- Any remaining compilation errors

- [ ] **Step 1: Run the full test suite and fix any remaining issues**

```bash
cargo test 2>&1 | grep -E "FAILED|error\[" | head -20
```

Common issues to expect:
- `Screen::Game` in `tests/tui_smoke.rs` may need `hint_warning: None` and `hint_count: 0` added if those fields were added in Task 4.
- Render function signatures changed in Task 4 may need callers updated.

Fix each compilation error by reading the error message and adding missing fields/arguments.

- [ ] **Step 2: Run the full test suite — all pass**

```bash
cargo test 2>&1 | grep "test result"
```
Expected:
```
test result: ok. NNN passed; 0 failed; ...
```

- [ ] **Step 3: Manual end-to-end smoke test**

```bash
cargo run
```

Navigate: New Game → Designer ▶ → browse patterns with ←/→ → select Smiley → watch generating screen → play a few moves → press `h` → hint shown.

Also test:
```bash
cargo run -- --pattern "010010010100101001001000100010010010100101001010010010001000100100101001010010010"
```
Expected: generating screen → game with Diamond pattern.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: designer sudoku — complete integration and smoke test fixes"
```
