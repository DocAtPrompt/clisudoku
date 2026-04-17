use crate::puzzle::Grid;
use crate::solver::candidates::{CandidateGrid, SolveStep, Strategy};

pub fn find_naked_singles(grid: &Grid, cands: &CandidateGrid) -> Vec<SolveStep> {
    let mut steps = vec![];
    for r in 0..9 {
        for c in 0..9 {
            if grid.get(r, c).is_empty() && cands.count(r, c) == 1 {
                let digit = cands.digits(r, c)[0];
                steps.push(SolveStep {
                    row: r, col: c, digit,
                    strategy: Strategy::NakedSingle,
                    source_cells: vec![],
                });
            }
        }
    }
    steps
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Grid;
    use crate::solver::candidates::CandidateGrid;

    #[test]
    fn finds_naked_single() {
        let mut grid = Grid::from_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
        ).unwrap();
        // Place all of row 0 except (0,5): solution is 5,3,4,6,7,8,9,1,2
        grid.set_filled(0, 2, 4);
        grid.set_filled(0, 3, 6);
        grid.set_filled(0, 6, 9);
        grid.set_filled(0, 7, 1);
        grid.set_filled(0, 8, 2);
        let cands = CandidateGrid::from_grid(&grid);
        assert_eq!(cands.count(0, 5), 1, "expected naked single at (0,5)");
        let steps = find_naked_singles(&grid, &cands);
        assert!(!steps.is_empty());
        let step = steps.iter().find(|s| s.row == 0 && s.col == 5);
        assert!(step.is_some(), "naked single at (0,5) not found");
        assert_eq!(step.unwrap().digit, 8);
    }

    #[test]
    fn no_panic_on_fresh_puzzle() {
        let grid = Grid::from_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
        ).unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        let steps = find_naked_singles(&grid, &cands);
        for s in &steps {
            assert_eq!(cands.count(s.row, s.col), 1);
        }
    }
}
