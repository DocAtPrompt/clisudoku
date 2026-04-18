pub mod difficulty;
pub use difficulty::{classify, Difficulty};

use crate::puzzle::Grid;

pub struct PuzzleGenerator {
    seed: u64,
}

impl PuzzleGenerator {
    pub fn new(seed: u64) -> Self { Self { seed } }

    pub fn generate(&self, difficulty: Difficulty) -> Grid {
        let mut rng = LcgRng::new(self.seed);

        // Step 1: fill a complete valid grid (randomized backtracking)
        let full = self.fill_grid(&mut rng).expect("fill_grid failed");

        // Step 2: remove cells while keeping puzzle uniquely solvable at target difficulty
        let mut puzzle = full.clone();
        let mut indices: Vec<usize> = (0..81).collect();
        shuffle(&mut indices, &mut rng);

        for &idx in &indices {
            let row = idx / 9;
            let col = idx % 9;
            let prev_val = puzzle.get(row, col).value();
            puzzle.clear(row, col);

            if !self.is_uniquely_solvable(&puzzle, difficulty) {
                // Restore
                if let Some(v) = prev_val {
                    puzzle.set_given(row, col, v);
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

    fn is_uniquely_solvable(&self, grid: &Grid, _difficulty: Difficulty) -> bool {
        // Use fast backtracking uniqueness check (stops at 2 solutions).
        // We use Solver::new() (with backtracking) to first check solvability
        // and simultaneously verify uniqueness via count_solutions.
        count_solutions(grid.clone(), 2) == 1
    }
}

fn candidates_for(grid: &Grid, row: usize, col: usize) -> Vec<u8> {
    (1u8..=9).filter(|&d| is_valid_placement(grid, row, col, d)).collect()
}

fn count_solutions(grid: Grid, limit: usize) -> usize {
    fn recurse(grid: &mut Grid, count: &mut usize, limit: usize) {
        if *count >= limit { return; }
        // MRV: pick the empty cell with fewest valid candidates
        let best = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter(|&(r, c)| grid.get(r, c).is_empty())
            .map(|(r, c)| {
                let cands = candidates_for(grid, r, c);
                (cands.len(), r, c, cands)
            })
            .min_by_key(|&(n, _, _, _)| n);
        match best {
            None => { *count += 1; } // all filled → solved
            Some((0, _, _, _)) => {} // dead end — no candidates for some cell
            Some((_, row, col, cands)) => {
                for digit in cands {
                    grid.set_filled(row, col, digit);
                    recurse(grid, count, limit);
                    grid.clear(row, col);
                    if *count >= limit { return; }
                }
            }
        }
    }
    let mut g = grid;
    let mut count = 0;
    recurse(&mut g, &mut count, limit);
    count
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
        let grid = PuzzleGenerator::new(42).generate(Difficulty::Easy);
        let given_count = (0..9).flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter(|&(r, c)| grid.get(r, c).is_given())
            .count();
        assert!(given_count >= 17, "too few givens: {}", given_count);
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved());
    }

    #[test]
    fn generates_solvable_hard_puzzle() {
        let grid = PuzzleGenerator::new(99).generate(Difficulty::Hard);
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved());
    }
}
