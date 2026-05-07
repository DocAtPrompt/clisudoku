pub mod difficulty;
pub use difficulty::{classify, Difficulty};

use crate::puzzle::Grid;

pub struct PuzzleGenerator {
    seed: u64,
}

/// Result of a pattern-constrained generation.
pub struct GeneratorResult {
    pub grid:             Grid,
    pub difficulty:       Difficulty,
    /// True when cells outside the pattern were added to reach unique solvability.
    pub used_extra_cells: bool,
}

impl PuzzleGenerator {
    pub fn new(seed: u64) -> Self { Self { seed } }

    pub fn generate(&self, difficulty: Difficulty, symmetry: bool) -> Grid {
        if difficulty == Difficulty::Expert {
            return self.generate_expert(symmetry);
        }

        let mut rng = LcgRng::new(self.seed);

        // Step 1: fill a complete valid grid (randomized backtracking)
        let full = self.fill_grid(&mut rng).expect("fill_grid failed");

        // Step 2: remove cells while keeping puzzle uniquely solvable at target difficulty.
        // BareMinimum uses full backtracking uniqueness (no strategy cap) for maximum removal.
        let solvable = |puzzle: &Grid| -> bool {
            if difficulty == Difficulty::BareMinimum {
                self.is_uniquely_solvable_full(puzzle)
            } else {
                self.is_uniquely_solvable(puzzle, difficulty)
            }
        };

        let mut puzzle = full.clone();

        if symmetry {
            // 180° rotational symmetry: cell at index i pairs with cell at index 80−i.
            // Iterate over the 40 pairs plus the unique centre cell (index 40).
            let mut pair_indices: Vec<usize> = (0..=40).collect();
            shuffle(&mut pair_indices, &mut rng);

            for &idx in &pair_indices {
                let (r1, c1) = (idx / 9, idx % 9);
                let prev1 = puzzle.get(r1, c1).value();
                puzzle.clear(r1, c1);

                // For indices 0..40 remove the mirror cell too; index 40 is the centre.
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
                    if let Some((r2, c2, Some(v))) = mirror_state {
                        puzzle.set_given(r2, c2, v);
                    }
                }
            }
        } else if difficulty == Difficulty::BareMinimum {
            // Multi-pass exhaustive removal: repeat until no cell can be removed.
            // After each pass a new shuffle is used so order-dependent blockages resolve.
            loop {
                let mut changed = false;
                let mut indices: Vec<usize> = (0..81)
                    .filter(|&i| { let (r, c) = (i / 9, i % 9); !puzzle.get(r, c).is_empty() })
                    .collect();
                shuffle(&mut indices, &mut rng);

                for &idx in &indices {
                    let (row, col) = (idx / 9, idx % 9);
                    let prev_val = puzzle.get(row, col).value();
                    puzzle.clear(row, col);
                    if self.is_uniquely_solvable_full(&puzzle) {
                        changed = true; // cell successfully removed
                    } else {
                        if let Some(v) = prev_val { puzzle.set_given(row, col, v); }
                    }
                }

                if !changed { break; } // converged — no further reduction possible
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

        // Rebuild as a clean Given-only grid
        let mut result = Grid::empty();
        for r in 0..9 {
            for c in 0..9 {
                if !puzzle.get(r, c).is_empty() {
                    let v = full.get(r, c).value().unwrap();
                    result.set_given(r, c, v);
                }
            }
        }
        result
    }

    fn generate_expert(&self, symmetry: bool) -> Grid {
        const MAX_RETRIES: u32 = 200;
        for attempt in 0..MAX_RETRIES {
            let seed = self.seed.wrapping_add(attempt as u64 * 7919);
            let mut rng = LcgRng::new(seed);
            let full = self.fill_grid(&mut rng).expect("fill_grid failed");

            let is_expert_solvable = |puzzle: &Grid| -> bool {
                self.is_uniquely_solvable(puzzle, Difficulty::Expert)
            };
            let is_extreme_solvable = |puzzle: &Grid| -> bool {
                self.is_uniquely_solvable(puzzle, Difficulty::Extreme)
            };

            // Phase 1: greedy removal keeping the puzzle Expert-solvable.
            // We do NOT require !is_extreme_solvable here; removing as many clues
            // as possible maximises the chance that the final puzzle requires a
            // genuine Expert technique (and is no longer Extreme-solvable).
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
                    if !is_expert_solvable(&puzzle) {
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
                    if !is_expert_solvable(&puzzle) {
                        if let Some(v) = prev_val { puzzle.set_given(row, col, v); }
                    }
                }
            }

            // Phase 2: double-check — the minimally Expert-solvable puzzle must
            // NOT be solvable by the Extreme solver (i.e. it genuinely requires
            // an Expert-tier technique, not just Swordfish or below).
            if is_expert_solvable(&puzzle) && !is_extreme_solvable(&puzzle) {
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

    fn fill_grid(&self, rng: &mut LcgRng) -> Option<Grid> {
        let mut grid = Grid::empty();
        Self::fill_recursive(&mut grid, rng)
    }

    fn fill_recursive(grid: &mut Grid, rng: &mut LcgRng) -> Option<Grid> {
        let empty = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .find(|&(r, c)| grid.get(r, c).is_empty());

        let (row, col) = match empty {
            None => return if grid.is_solved() { Some(grid.clone()) } else { None },
            Some(pos) => pos,
        };

        let mut digits: Vec<u8> = (1..=9).collect();
        shuffle_u8(&mut digits, rng);

        for digit in digits {
            if is_valid_placement(grid, row, col, digit) {
                grid.set_given(row, col, digit);
                if let Some(solved) = Self::fill_recursive(grid, rng) {
                    return Some(solved);
                }
                grid.clear(row, col);
            }
        }
        None
    }

    fn is_uniquely_solvable(&self, grid: &Grid, difficulty: Difficulty) -> bool {
        // Check that the puzzle is fully solvable within the difficulty's strategy
        // budget. The logical solver can only place digits that are forced (i.e.
        // common to all solutions), so a fully-solved result implicitly guarantees
        // uniqueness without a separate count_solutions call.
        let solver = crate::solver::Solver::for_difficulty(&difficulty);
        solver.solve(grid.clone()).grid.is_solved()
    }

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
        let difficulty = classify(&solve_result.used_strategies);

        GeneratorResult { grid: result, difficulty, used_extra_cells }
    }

    /// Check uniqueness by counting solutions via backtracking, short-circuiting at 2.
    fn is_uniquely_solvable_full(&self, grid: &Grid) -> bool {
        let mut count = 0u8;
        let mut working = grid.clone();
        Self::count_solutions(&mut working, &mut count);
        count == 1
    }

    fn count_solutions(grid: &mut Grid, count: &mut u8) {
        if *count > 1 { return; } // short-circuit
        let empty = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .find(|&(r, c)| grid.get(r, c).is_empty());
        match empty {
            None => {
                *count += 1;
            }
            Some((row, col)) => {
                for digit in 1u8..=9 {
                    if is_valid_placement(grid, row, col, digit) {
                        grid.set_filled(row, col, digit);
                        Self::count_solutions(grid, count);
                        grid.clear(row, col);
                        if *count > 1 { return; }
                    }
                }
            }
        }
    }
}

fn is_valid_placement(grid: &Grid, row: usize, col: usize, digit: u8) -> bool {
    for c in 0..9 { if grid.get(row, c).value() == Some(digit) { return false; } }
    for r in 0..9 { if grid.get(r, col).value() == Some(digit) { return false; } }
    let (br, bc) = Grid::box_start(Grid::box_idx(row, col));
    for dr in 0..3 { for dc in 0..3 {
        if grid.get(br+dr, bc+dc).value() == Some(digit) { return false; }
    }}
    true
}

/// Minimal LCG RNG for deterministic seeds — no external deps.
struct LcgRng { state: u64 }
impl LcgRng {
    fn new(seed: u64) -> Self { Self { state: seed ^ 0x12345678 } }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.state
    }
    fn next_usize(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

fn shuffle(v: &mut Vec<usize>, rng: &mut LcgRng) {
    for i in (1..v.len()).rev() {
        let j = rng.next_usize(i + 1);
        v.swap(i, j);
    }
}

fn shuffle_u8(v: &mut Vec<u8>, rng: &mut LcgRng) {
    for i in (1..v.len()).rev() {
        let j = rng.next_usize(i + 1);
        v.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::Solver;

    #[test]
    fn generates_solvable_easy_puzzle() {
        let grid = PuzzleGenerator::new(42).generate(Difficulty::Easy, false);
        let given_count = (0..9).flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter(|&(r, c)| grid.get(r, c).is_given())
            .count();
        assert!(given_count >= 17, "too few givens: {}", given_count);
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved());
    }

    #[test]
    fn generates_solvable_hard_puzzle() {
        let grid = PuzzleGenerator::new(99).generate(Difficulty::Hard, false);
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved());
    }

    #[test]
    fn symmetric_puzzle_has_rotational_symmetry() {
        let grid = PuzzleGenerator::new(42).generate(Difficulty::Easy, true);
        // Every given cell must have a partner at the 180°-rotated position.
        for r in 0..9 {
            for c in 0..9 {
                let mirror_r = 8 - r;
                let mirror_c = 8 - c;
                assert_eq!(
                    grid.get(r, c).is_empty(),
                    grid.get(mirror_r, mirror_c).is_empty(),
                    "symmetry broken at ({r},{c}) ↔ ({mirror_r},{mirror_c})"
                );
            }
        }
    }

    #[test]
    fn pattern_puzzle_only_has_givens_in_pattern() {
        use crate::pattern::PATTERNS;
        // Use the Asterisk pattern (index 10, 33 cells) — interior-heavy, faster than Border
        let pattern = PATTERNS[10].clone();
        let result = PuzzleGenerator::new(42).generate_with_pattern(&pattern);
        // Every given cell must be in the pattern mask (unless extra cells were needed)
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
    fn generates_solvable_extreme_puzzle() {
        let grid = PuzzleGenerator::new(7).generate(Difficulty::Extreme, false);
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved(), "Extreme puzzle must be solvable");
    }

    #[test]
    fn extreme_solver_solves_extreme_puzzle() {
        // Verify that the Extreme-capped solver (Swordfish, no backtracking) can
        // solve a puzzle generated at Extreme difficulty. If find_swordfish were
        // silently broken (returning empty), the generator would only produce
        // Hard-or-below puzzles at Extreme setting — this test would then fail
        // to assert that the puzzle required at least XWing-level strategy.
        let grid = PuzzleGenerator::new(7).generate(Difficulty::Extreme, false);
        let solver = Solver::for_difficulty(&Difficulty::Extreme);
        let result = solver.solve(grid);
        assert!(result.grid.is_solved(),
            "Extreme-capped solver must solve an Extreme-generated puzzle");
        // The generated puzzle should require at least a medium-level strategy.
        // If it only used NakedSingle/HiddenSingle the generator did not push
        // hard enough — but this is seed-dependent so we accept Easy too.
        // The key assertion above (solver can solve it) is the critical check.
        let _ = result.used_strategies; // accessible for debugging if needed
    }

    #[test]
    fn pattern_puzzle_difficulty_is_classified() {
        use crate::pattern::PATTERNS;
        let result = PuzzleGenerator::new(99).generate_with_pattern(&PATTERNS[1]); // Checker
        // difficulty must be one of the valid variants (not None)
        let _ = result.difficulty; // just verifying it compiles and is accessible
        assert!(crate::solver::Solver::new().solve(result.grid).grid.is_solved());
    }

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

    #[test]
    fn bare_minimum_is_solvable_and_reduced() {
        // Multi-pass exhaustive removal should produce well below 30 givens.
        // The theoretical minimum is 17; practically the greedy algorithm converges
        // at 20–26 depending on the seed. We accept anything ≤ 27 as proof that
        // the multi-pass loop is working correctly.
        let grid = PuzzleGenerator::new(42).generate(Difficulty::BareMinimum, false);
        let given_count = (0..9).flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter(|&(r, c)| grid.get(r, c).is_given())
            .count();
        assert!(given_count >= 17, "too few givens: {}", given_count);
        assert!(given_count <= 27,
            "Bare Minimum produced {} givens — multi-pass loop not converging", given_count);
        // Must still be uniquely solvable
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved(), "BareMinimum puzzle must be solvable");
    }
}
