use crate::puzzle::Grid;
use crate::solver::candidates::{CandidateGrid, SolveStep, Strategy};

pub fn find_hidden_singles(_grid: &Grid, cands: &CandidateGrid) -> Vec<SolveStep> {
    let mut steps = vec![];

    // Rows
    for r in 0..9 {
        for digit in 1u8..=9 {
            let positions: Vec<usize> = (0..9).filter(|&c| cands.has(r, c, digit)).collect();
            if positions.len() == 1 {
                let c = positions[0];
                steps.push(SolveStep {
                    row: r,
                    col: c,
                    digit,
                    strategy: Strategy::HiddenSingle,
                    source_cells: vec![],
                });
            }
        }
    }

    // Cols
    for c in 0..9 {
        for digit in 1u8..=9 {
            let positions: Vec<usize> = (0..9).filter(|&r| cands.has(r, c, digit)).collect();
            if positions.len() == 1 {
                let r = positions[0];
                if !steps
                    .iter()
                    .any(|s| s.row == r && s.col == c && s.digit == digit)
                {
                    steps.push(SolveStep {
                        row: r,
                        col: c,
                        digit,
                        strategy: Strategy::HiddenSingle,
                        source_cells: vec![],
                    });
                }
            }
        }
    }

    // Boxes
    for b in 0..9 {
        let (br, bc) = crate::puzzle::Grid::box_start(b);
        for digit in 1u8..=9 {
            let positions: Vec<(usize, usize)> = (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc)))
                .filter(|&(r, c)| cands.has(r, c, digit))
                .collect();
            if positions.len() == 1 {
                let (r, c) = positions[0];
                if !steps
                    .iter()
                    .any(|s| s.row == r && s.col == c && s.digit == digit)
                {
                    steps.push(SolveStep {
                        row: r,
                        col: c,
                        digit,
                        strategy: Strategy::HiddenSingle,
                        source_cells: vec![],
                    });
                }
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
    fn finds_hidden_single_in_row() {
        let mut grid = Grid::from_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        )
        .unwrap();
        grid.set_filled(0, 3, 6);
        grid.set_filled(0, 5, 8);
        grid.set_filled(0, 6, 9);
        grid.set_filled(0, 7, 1);
        grid.set_filled(0, 8, 2);
        let cands = CandidateGrid::from_grid(&grid);
        let four_positions: Vec<_> = (0..9).filter(|&c| cands.has(0, c, 4)).collect();
        assert_eq!(
            four_positions.len(),
            1,
            "digit 4 should be hidden single in row 0"
        );
        let steps = find_hidden_singles(&grid, &cands);
        let step = steps
            .iter()
            .find(|s| s.row == 0 && s.col == 2 && s.digit == 4);
        assert!(step.is_some(), "hidden single (0,2)=4 not found");
    }

    #[test]
    fn no_panic_on_empty_grid() {
        let grid = Grid::empty();
        let cands = CandidateGrid::from_grid(&grid);
        let steps = find_hidden_singles(&grid, &cands);
        assert!(steps.is_empty());
    }
}
