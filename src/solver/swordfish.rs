use crate::solver::candidates::{CandidateGrid, Elimination, Strategy};

pub fn find_swordfish(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut elims = vec![];

    for digit in 1u8..=9 {
        // ── Row-based Swordfish ───────────────────────────────────────────────
        let row_cols: Vec<Vec<usize>> = (0..9)
            .map(|r| (0..9).filter(|&c| cands.has(r, c, digit)).collect())
            .collect();

        let cand_rows: Vec<usize> = (0..9)
            .filter(|&r| {
                let n = row_cols[r].len();
                n == 2 || n == 3
            })
            .collect();

        for i in 0..cand_rows.len() {
            for j in (i + 1)..cand_rows.len() {
                for k in (j + 1)..cand_rows.len() {
                    let (r1, r2, r3) = (cand_rows[i], cand_rows[j], cand_rows[k]);
                    let mut cols = std::collections::BTreeSet::new();
                    for &c in &row_cols[r1] {
                        cols.insert(c);
                    }
                    for &c in &row_cols[r2] {
                        cols.insert(c);
                    }
                    for &c in &row_cols[r3] {
                        cols.insert(c);
                    }
                    if cols.len() != 3 {
                        continue;
                    }
                    for &c in &cols {
                        for r in 0..9 {
                            if r == r1 || r == r2 || r == r3 {
                                continue;
                            }
                            if cands.has(r, c, digit) {
                                elims.push(Elimination {
                                    row: r,
                                    col: c,
                                    digit,
                                    strategy: Strategy::Swordfish,
                                });
                            }
                        }
                    }
                }
            }
        }

        // ── Column-based Swordfish (symmetric) ───────────────────────────────
        let col_rows: Vec<Vec<usize>> = (0..9)
            .map(|c| (0..9).filter(|&r| cands.has(r, c, digit)).collect())
            .collect();

        let cand_cols: Vec<usize> = (0..9)
            .filter(|&c| {
                let n = col_rows[c].len();
                n == 2 || n == 3
            })
            .collect();

        for i in 0..cand_cols.len() {
            for j in (i + 1)..cand_cols.len() {
                for k in (j + 1)..cand_cols.len() {
                    let (c1, c2, c3) = (cand_cols[i], cand_cols[j], cand_cols[k]);
                    let mut rows = std::collections::BTreeSet::new();
                    for &r in &col_rows[c1] {
                        rows.insert(r);
                    }
                    for &r in &col_rows[c2] {
                        rows.insert(r);
                    }
                    for &r in &col_rows[c3] {
                        rows.insert(r);
                    }
                    if rows.len() != 3 {
                        continue;
                    }
                    for &r in &rows {
                        for c in 0..9 {
                            if c == c1 || c == c2 || c == c3 {
                                continue;
                            }
                            if cands.has(r, c, digit) {
                                elims.push(Elimination {
                                    row: r,
                                    col: c,
                                    digit,
                                    strategy: Strategy::Swordfish,
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

    // All returned eliminations must be actual candidates (no false positives).
    #[test]
    fn no_panic_no_false_positives() {
        let grid = Grid::from_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        )
        .unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        let elims = find_swordfish(&cands);
        for e in &elims {
            assert!(
                cands.has(e.row, e.col, e.digit),
                "false positive: ({},{}) digit {} is not a candidate",
                e.row,
                e.col,
                e.digit
            );
        }
    }

    // Swordfish should not fire on an X-Wing (2-row) pattern — only on 3-row.
    #[test]
    fn all_eliminations_carry_swordfish_strategy() {
        let grid = Grid::from_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        )
        .unwrap();
        let cands = CandidateGrid::from_grid(&grid);
        let elims = find_swordfish(&cands);
        for e in &elims {
            assert_eq!(e.strategy, Strategy::Swordfish);
        }
    }
}
