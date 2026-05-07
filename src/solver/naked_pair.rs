use crate::puzzle::Grid;
use crate::solver::candidates::{CandidateGrid, Elimination, Strategy};

pub fn find_naked_pairs(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut elims = vec![];

    let check_house = |cells: &[(usize, usize)], elims: &mut Vec<Elimination>| {
        let pairs: Vec<(usize, usize, u16)> = cells
            .iter()
            .filter_map(|&(r, c)| {
                let m = cands.mask(r, c);
                if m.count_ones() == 2 {
                    Some((r, c, m))
                } else {
                    None
                }
            })
            .collect();
        for i in 0..pairs.len() {
            for j in (i + 1)..pairs.len() {
                if pairs[i].2 == pairs[j].2 {
                    let mask = pairs[i].2;
                    let digits: Vec<u8> = (1u8..=9).filter(|&d| mask & (1 << d) != 0).collect();
                    for &(r, c) in cells {
                        if (r, c) == (pairs[i].0, pairs[i].1) {
                            continue;
                        }
                        if (r, c) == (pairs[j].0, pairs[j].1) {
                            continue;
                        }
                        for &d in &digits {
                            if cands.has(r, c, d) {
                                elims.push(Elimination {
                                    row: r,
                                    col: c,
                                    digit: d,
                                    strategy: Strategy::NakedPair,
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
        let elims = find_naked_pairs(&cands);
        for e in &elims {
            assert!(e.digit >= 1 && e.digit <= 9);
            assert!(
                cands.has(e.row, e.col, e.digit),
                "elimination targets cell without that candidate"
            );
        }
    }
}
