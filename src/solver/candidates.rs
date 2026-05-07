use crate::puzzle::Grid;
use serde::{Deserialize, Serialize};

/// Which strategy produced this step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    Expert,
    Backtracking,
}

/// A single placement: set `digit` at (row, col), found by `strategy`.
/// `source_cells` highlights cells that caused this deduction (for hint display).
#[derive(Debug, Clone)]
pub struct SolveStep {
    pub row: usize,
    pub col: usize,
    pub digit: u8,
    pub strategy: Strategy,
    pub source_cells: Vec<(usize, usize)>,
}

/// Remove `digit` as a candidate from cell (row, col).
/// Used by pair/triple/wing strategies.
#[derive(Debug, Clone)]
pub struct Elimination {
    pub row: usize,
    pub col: usize,
    pub digit: u8,
    pub strategy: Strategy,
}

/// Per-cell candidate bitmask. Bit d (1-indexed, 1-9) set → digit d is a candidate.
/// Bit 0 is unused. Filled/Given cells have mask = 0.
#[derive(Debug, Clone)]
pub struct CandidateGrid {
    masks: [u16; 81],
}

impl CandidateGrid {
    fn idx(row: usize, col: usize) -> usize {
        row * 9 + col
    }

    pub fn from_grid(grid: &Grid) -> Self {
        let mut masks = [0u16; 81];
        // Initialize empty cells with all 9 candidates (bits 1-9)
        for r in 0..9 {
            for c in 0..9 {
                if grid.get(r, c).is_empty() {
                    masks[Self::idx(r, c)] = 0b1111111110u16; // bits 1-9
                }
            }
        }
        // Eliminate based on placed values
        for r in 0..9 {
            for c in 0..9 {
                if let Some(v) = grid.get(r, c).value() {
                    let bit = !(1u16 << v);
                    for cc in 0..9 {
                        masks[Self::idx(r, cc)] &= bit;
                    }
                    for rr in 0..9 {
                        masks[Self::idx(rr, c)] &= bit;
                    }
                    let (br, bc) = Grid::box_start(Grid::box_idx(r, c));
                    for dr in 0..3 {
                        for dc in 0..3 {
                            masks[Self::idx(br + dr, bc + dc)] &= bit;
                        }
                    }
                }
            }
        }
        Self { masks }
    }

    pub fn has(&self, row: usize, col: usize, digit: u8) -> bool {
        self.masks[Self::idx(row, col)] & (1 << digit) != 0
    }

    pub fn remove(&mut self, row: usize, col: usize, digit: u8) {
        self.masks[Self::idx(row, col)] &= !(1u16 << digit);
    }

    pub fn count(&self, row: usize, col: usize) -> u32 {
        self.masks[Self::idx(row, col)].count_ones()
    }

    pub fn digits(&self, row: usize, col: usize) -> Vec<u8> {
        (1u8..=9).filter(|&d| self.has(row, col, d)).collect()
    }

    pub fn mask(&self, row: usize, col: usize) -> u16 {
        self.masks[Self::idx(row, col)]
    }

    /// After placing `digit` at (row, col): clear that cell's mask and remove
    /// digit from all peers in the same row, col, and box.
    pub fn eliminate_from_peers(&mut self, row: usize, col: usize, digit: u8) {
        self.masks[Self::idx(row, col)] = 0;
        let bit = !(1u16 << digit);
        for cc in 0..9 {
            if cc != col {
                self.masks[Self::idx(row, cc)] &= bit;
            }
        }
        for rr in 0..9 {
            if rr != row {
                self.masks[Self::idx(rr, col)] &= bit;
            }
        }
        let (br, bc) = Grid::box_start(Grid::box_idx(row, col));
        for dr in 0..3 {
            for dc in 0..3 {
                let (rr, cc) = (br + dr, bc + dc);
                if rr != row || cc != col {
                    self.masks[Self::idx(rr, cc)] &= bit;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Grid;

    const EASY: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    #[test]
    fn empty_cell_has_no_given_digit_as_candidate() {
        let grid = Grid::from_str(EASY).unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        // (0,2) is empty; 5 and 3 are in same row → not candidates
        assert!(!cands.has(0, 2, 5));
        assert!(!cands.has(0, 2, 3));
    }

    #[test]
    fn given_cell_has_no_candidates() {
        let grid = Grid::from_str(EASY).unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        // (0,0) = Given(5) → no candidates
        assert_eq!(cands.count(0, 0), 0);
    }

    #[test]
    fn candidates_count_plausible() {
        let grid = Grid::from_str(EASY).unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        let c = cands.count(0, 2);
        assert!(c > 0 && c <= 9);
    }

    #[test]
    fn remove_candidate() {
        let grid = Grid::from_str(EASY).unwrap();
        let mut cands = CandidateGrid::from_grid(&grid);
        let row = 0;
        let col = 2;
        let before = cands.count(row, col);
        let digit = cands.digits(row, col)[0];
        cands.remove(row, col, digit);
        assert!(!cands.has(row, col, digit));
        assert_eq!(cands.count(row, col), before - 1);
    }

    #[test]
    fn eliminate_digit_from_peers() {
        let grid = Grid::from_str(EASY).unwrap();
        let mut cands = CandidateGrid::from_grid(&grid);
        cands.eliminate_from_peers(0, 2, 4);
        for c in 0..9 {
            if c != 2 {
                assert!(!cands.has(0, c, 4), "row peer still has 4 at col {}", c);
            }
        }
        for r in 0..9 {
            if r != 0 {
                assert!(!cands.has(r, 2, 4), "col peer still has 4 at row {}", r);
            }
        }
    }

    #[test]
    fn strategy_expert_exists_and_is_distinct() {
        // Ensure Expert exists and is not equal to Swordfish or Backtracking
        assert_ne!(Strategy::Expert, Strategy::Swordfish);
        assert_ne!(Strategy::Expert, Strategy::Backtracking);
    }
}
