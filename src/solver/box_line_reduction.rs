use crate::puzzle::Grid;
use crate::solver::candidates::{CandidateGrid, Elimination, Strategy};
use std::collections::HashSet;

pub fn find_box_line_reductions(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut elims = vec![];

    // Rows: if all candidates for a digit in a row are in the same box → eliminate from rest of box
    for r in 0..9 {
        for digit in 1u8..=9 {
            let positions: Vec<usize> = (0..9).filter(|&c| cands.has(r, c, digit)).collect();
            if positions.is_empty() { continue; }
            let boxes: HashSet<usize> = positions.iter().map(|&c| Grid::box_idx(r, c)).collect();
            if boxes.len() == 1 {
                let b = *boxes.iter().next().unwrap();
                let (br, bc) = Grid::box_start(b);
                for dr in 0..3 {
                    for dc in 0..3 {
                        let (rr, cc) = (br + dr, bc + dc);
                        if rr != r && cands.has(rr, cc, digit) {
                            elims.push(Elimination {
                                row: rr,
                                col: cc,
                                digit,
                                strategy: Strategy::BoxLineReduction,
                            });
                        }
                    }
                }
            }
        }
    }

    // Cols: if all candidates for a digit in a col are in the same box → eliminate from rest of box
    for c in 0..9 {
        for digit in 1u8..=9 {
            let positions: Vec<usize> = (0..9).filter(|&r| cands.has(r, c, digit)).collect();
            if positions.is_empty() { continue; }
            let boxes: HashSet<usize> = positions.iter().map(|&r| Grid::box_idx(r, c)).collect();
            if boxes.len() == 1 {
                let b = *boxes.iter().next().unwrap();
                let (br, bc) = Grid::box_start(b);
                for dr in 0..3 {
                    for dc in 0..3 {
                        let (rr, cc) = (br + dr, bc + dc);
                        if cc != c && cands.has(rr, cc, digit) {
                            elims.push(Elimination {
                                row: rr,
                                col: cc,
                                digit,
                                strategy: Strategy::BoxLineReduction,
                            });
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
        let elims = find_box_line_reductions(&cands);
        for e in &elims { assert!(cands.has(e.row, e.col, e.digit)); }
    }
}
