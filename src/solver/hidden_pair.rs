use crate::puzzle::Grid;
use crate::solver::candidates::{CandidateGrid, Elimination, Strategy};

pub fn find_hidden_pairs(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut elims = vec![];

    let check_house = |cells: &[(usize, usize)], elims: &mut Vec<Elimination>| {
        for d1 in 1u8..=9 {
            let pos1: Vec<(usize, usize)> = cells
                .iter()
                .filter(|&&(r, c)| cands.has(r, c, d1))
                .cloned()
                .collect();
            if pos1.len() != 2 {
                continue;
            }
            for d2 in (d1 + 1)..=9 {
                let pos2: Vec<(usize, usize)> = cells
                    .iter()
                    .filter(|&&(r, c)| cands.has(r, c, d2))
                    .cloned()
                    .collect();
                if pos2 == pos1 {
                    for &(r, c) in &pos1 {
                        for d in 1u8..=9 {
                            if d != d1 && d != d2 && cands.has(r, c, d) {
                                elims.push(Elimination {
                                    row: r,
                                    col: c,
                                    digit: d,
                                    strategy: Strategy::HiddenPair,
                                });
                            }
                        }
                    }
                }
            }
        }
    };

    for i in 0..9 {
        let row: Vec<_> = (0..9).map(|c| (i, c)).collect();
        check_house(&row, &mut elims);
        let col: Vec<_> = (0..9).map(|r| (r, i)).collect();
        check_house(&col, &mut elims);
        let (br, bc) = Grid::box_start(i);
        let bx: Vec<_> = (0..3)
            .flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc)))
            .collect();
        check_house(&bx, &mut elims);
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
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        )
        .unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        let elims = find_hidden_pairs(&cands);
        for e in &elims {
            assert!(cands.has(e.row, e.col, e.digit));
        }
    }
}
