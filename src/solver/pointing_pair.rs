use crate::puzzle::Grid;
use crate::solver::candidates::{CandidateGrid, Elimination, Strategy};

pub fn find_pointing_pairs(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut elims = vec![];

    for b in 0..9 {
        let (br, bc) = Grid::box_start(b);
        for digit in 1u8..=9 {
            let positions: Vec<(usize, usize)> = (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc)))
                .filter(|&(r, c)| cands.has(r, c, digit))
                .collect();

            if positions.len() < 2 || positions.len() > 3 { continue; }

            let rows: std::collections::HashSet<usize> = positions.iter().map(|p| p.0).collect();
            if rows.len() == 1 {
                let row = *rows.iter().next().unwrap();
                for c in 0..9 {
                    if c < bc || c >= bc + 3 {
                        if cands.has(row, c, digit) {
                            elims.push(Elimination { row, col: c, digit, strategy: Strategy::PointingPair });
                        }
                    }
                }
            }

            let cols: std::collections::HashSet<usize> = positions.iter().map(|p| p.1).collect();
            if cols.len() == 1 {
                let col = *cols.iter().next().unwrap();
                for r in 0..9 {
                    if r < br || r >= br + 3 {
                        if cands.has(r, col, digit) {
                            elims.push(Elimination { row: r, col, digit, strategy: Strategy::PointingPair });
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Grid;
    use crate::solver::candidates::CandidateGrid;

    #[test]
    fn no_panic_no_false_positives() {
        let grid = Grid::from_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
        ).unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        let elims = find_pointing_pairs(&cands);
        for e in &elims {
            assert!(e.digit >= 1 && e.digit <= 9);
            assert!(cands.has(e.row, e.col, e.digit), "elimination targets cell without candidate");
        }
    }
}
