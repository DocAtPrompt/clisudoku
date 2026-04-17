use crate::solver::candidates::{CandidateGrid, Elimination, Strategy};

pub fn find_x_wings(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut elims = vec![];

    for digit in 1u8..=9 {
        // Row-based X-Wing
        let row_cols: Vec<Vec<usize>> = (0..9)
            .map(|r| (0..9).filter(|&c| cands.has(r, c, digit)).collect())
            .collect();

        for r1 in 0..9 {
            if row_cols[r1].len() != 2 { continue; }
            for r2 in (r1 + 1)..9 {
                if row_cols[r2] == row_cols[r1] {
                    let c1 = row_cols[r1][0];
                    let c2 = row_cols[r1][1];
                    for r in 0..9 {
                        if r == r1 || r == r2 { continue; }
                        for &c in &[c1, c2] {
                            if cands.has(r, c, digit) {
                                elims.push(Elimination {
                                    row: r,
                                    col: c,
                                    digit,
                                    strategy: Strategy::XWing,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Col-based X-Wing
        let col_rows: Vec<Vec<usize>> = (0..9)
            .map(|c| (0..9).filter(|&r| cands.has(r, c, digit)).collect())
            .collect();

        for c1 in 0..9 {
            if col_rows[c1].len() != 2 { continue; }
            for c2 in (c1 + 1)..9 {
                if col_rows[c2] == col_rows[c1] {
                    let r1 = col_rows[c1][0];
                    let r2 = col_rows[c1][1];
                    for c in 0..9 {
                        if c == c1 || c == c2 { continue; }
                        for &r in &[r1, r2] {
                            if cands.has(r, c, digit) {
                                elims.push(Elimination {
                                    row: r,
                                    col: c,
                                    digit,
                                    strategy: Strategy::XWing,
                                });
                            }
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
        let elims = find_x_wings(&cands);
        for e in &elims { assert!(cands.has(e.row, e.col, e.digit)); }
    }
}
