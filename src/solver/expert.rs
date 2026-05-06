// src/solver/expert.rs
//
// Expert-level solver functions operating on CandidateGrid.
// These are pure elimination/placement finders — no display logic.
//
// Translation rule from hint/strategies/tier2.rs:
//   state.notes_mask(r, c)              →  cands.mask(r, c)
//   matches!(grid.get(r,c), CellKind::Empty)  →  cands.mask(r, c) != 0
//
// Each find_* function returns Vec<Elimination> (all eliminations found).
// find_bug_plus_one_step returns Option<SolveStep> (a placement, not elimination).

use crate::solver::candidates::{CandidateGrid, Elimination, SolveStep, Strategy};

// ── Internal helpers ──────────────────────────────────────────────────────────

fn all_units() -> Vec<Vec<(usize, usize)>> {
    let mut units = Vec::with_capacity(27);
    for i in 0..9 {
        units.push((0..9).map(|c| (i, c)).collect()); // row i
        units.push((0..9).map(|r| (r, i)).collect()); // col i
        let br = (i / 3) * 3;
        let bc = (i % 3) * 3;
        units.push(
            (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc)))
                .collect(),
        );
    }
    units
}

fn sees(r1: usize, c1: usize, r2: usize, c2: usize) -> bool {
    r1 == r2 || c1 == c2 || (r1 / 3 == r2 / 3 && c1 / 3 == c2 / 3)
}

fn elim(row: usize, col: usize, digit: u8) -> Elimination {
    Elimination { row, col, digit, strategy: Strategy::Expert }
}

// ── Jellyfish ─────────────────────────────────────────────────────────────────

pub fn find_jellyfish(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        // Row-based
        let row_cols: Vec<Vec<usize>> = (0..9)
            .map(|r| (0..9).filter(|&c| cands.mask(r, c) != 0 && cands.has(r, c, digit)).collect())
            .collect();
        let cand_rows: Vec<usize> = (0..9)
            .filter(|&r| { let n = row_cols[r].len(); n >= 2 && n <= 4 })
            .collect();
        for i in 0..cand_rows.len() {
            for j in (i + 1)..cand_rows.len() {
                for k in (j + 1)..cand_rows.len() {
                    for l in (k + 1)..cand_rows.len() {
                        let (r1, r2, r3, r4) = (cand_rows[i], cand_rows[j], cand_rows[k], cand_rows[l]);
                        let mut cols = std::collections::BTreeSet::new();
                        for &c in &row_cols[r1] { cols.insert(c); }
                        for &c in &row_cols[r2] { cols.insert(c); }
                        for &c in &row_cols[r3] { cols.insert(c); }
                        for &c in &row_cols[r4] { cols.insert(c); }
                        if cols.len() != 4 { continue; }
                        for &c in &cols {
                            for r in 0..9 {
                                if r != r1 && r != r2 && r != r3 && r != r4
                                    && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                                {
                                    result.push(elim(r, c, digit));
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
        // Column-based
        let col_rows: Vec<Vec<usize>> = (0..9)
            .map(|c| (0..9).filter(|&r| cands.mask(r, c) != 0 && cands.has(r, c, digit)).collect())
            .collect();
        let cand_cols: Vec<usize> = (0..9)
            .filter(|&c| { let n = col_rows[c].len(); n >= 2 && n <= 4 })
            .collect();
        for i in 0..cand_cols.len() {
            for j in (i + 1)..cand_cols.len() {
                for k in (j + 1)..cand_cols.len() {
                    for l in (k + 1)..cand_cols.len() {
                        let (c1, c2, c3, c4) = (cand_cols[i], cand_cols[j], cand_cols[k], cand_cols[l]);
                        let mut rows = std::collections::BTreeSet::new();
                        for &r in &col_rows[c1] { rows.insert(r); }
                        for &r in &col_rows[c2] { rows.insert(r); }
                        for &r in &col_rows[c3] { rows.insert(r); }
                        for &r in &col_rows[c4] { rows.insert(r); }
                        if rows.len() != 4 { continue; }
                        for &r in &rows {
                            for c in 0..9 {
                                if c != c1 && c != c2 && c != c3 && c != c4
                                    && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                                {
                                    result.push(elim(r, c, digit));
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
    }
    result
}

// ── Naked Quad ────────────────────────────────────────────────────────────────

pub fn find_naked_quad(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for unit in all_units() {
        let small: Vec<(usize, usize, u16)> = unit
            .iter()
            .filter_map(|&(r, c)| {
                let m = cands.mask(r, c);
                let n = m.count_ones();
                if n >= 2 && n <= 4 { Some((r, c, m)) } else { None }
            })
            .collect();
        for i in 0..small.len() {
            for j in (i + 1)..small.len() {
                for k in (j + 1)..small.len() {
                    for l in (k + 1)..small.len() {
                        let combined = small[i].2 | small[j].2 | small[k].2 | small[l].2;
                        if combined.count_ones() != 4 { continue; }
                        let quad = [
                            (small[i].0, small[i].1),
                            (small[j].0, small[j].1),
                            (small[k].0, small[k].1),
                            (small[l].0, small[l].1),
                        ];
                        for &(r, c) in &unit {
                            if !quad.contains(&(r, c)) && cands.mask(r, c) != 0
                                && (cands.mask(r, c) & combined) != 0
                            {
                                for d in 1u8..=9 {
                                    if (combined & (1 << d)) != 0 && cands.has(r, c, d) {
                                        result.push(elim(r, c, d));
                                    }
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
    }
    result
}

// ── Hidden Triple ─────────────────────────────────────────────────────────────

pub fn find_hidden_triple(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for unit in all_units() {
        let empties: Vec<(usize, usize)> = unit
            .iter()
            .filter(|&&(r, c)| cands.mask(r, c) != 0)
            .copied()
            .collect();
        for d1 in 1u8..=9 {
            for d2 in (d1 + 1)..=9 {
                for d3 in (d2 + 1)..=9 {
                    let triple_mask = (1u16 << d1) | (1u16 << d2) | (1u16 << d3);
                    let triple_cells: Vec<(usize, usize)> = empties
                        .iter()
                        .filter(|&&(r, c)| (cands.mask(r, c) & triple_mask) != 0)
                        .copied()
                        .collect();
                    if triple_cells.len() != 3 { continue; }
                    let combined: u16 = triple_cells
                        .iter()
                        .fold(0u16, |acc, &(r, c)| acc | cands.mask(r, c));
                    if combined & triple_mask != triple_mask { continue; }
                    for &(r, c) in &triple_cells {
                        let extra = cands.mask(r, c) & !triple_mask;
                        for d in 1u8..=9 {
                            if (extra & (1 << d)) != 0 {
                                result.push(elim(r, c, d));
                            }
                        }
                    }
                    if !result.is_empty() { return result; }
                }
            }
        }
    }
    result
}

// ── Hidden Quad ───────────────────────────────────────────────────────────────

pub fn find_hidden_quad(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for unit in all_units() {
        let empties: Vec<(usize, usize)> = unit
            .iter()
            .filter(|&&(r, c)| cands.mask(r, c) != 0)
            .copied()
            .collect();
        for d1 in 1u8..=9 {
            for d2 in (d1 + 1)..=9 {
                for d3 in (d2 + 1)..=9 {
                    for d4 in (d3 + 1)..=9 {
                        let mask = (1u16 << d1) | (1u16 << d2) | (1u16 << d3) | (1u16 << d4);
                        let quad_cells: Vec<(usize, usize)> = empties
                            .iter()
                            .filter(|&&(r, c)| (cands.mask(r, c) & mask) != 0)
                            .copied()
                            .collect();
                        if quad_cells.len() != 4 { continue; }
                        let combined: u16 = quad_cells
                            .iter()
                            .fold(0u16, |acc, &(r, c)| acc | cands.mask(r, c));
                        if (combined & mask) != mask { continue; }
                        for &(r, c) in &quad_cells {
                            let extra = cands.mask(r, c) & !mask;
                            for d in 1u8..=9 {
                                if (extra & (1 << d)) != 0 {
                                    result.push(elim(r, c, d));
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
    }
    result
}

// ── Skyscraper ────────────────────────────────────────────────────────────────

pub fn find_skyscraper(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        // Row-based
        let row_cols: Vec<Vec<usize>> = (0..9)
            .map(|r| (0..9).filter(|&c| cands.mask(r, c) != 0 && cands.has(r, c, digit)).collect())
            .collect();
        for r1 in 0..9 {
            if row_cols[r1].len() != 2 { continue; }
            for r2 in (r1 + 1)..9 {
                if row_cols[r2].len() != 2 { continue; }
                let shared: Vec<usize> = row_cols[r1].iter()
                    .filter(|c| row_cols[r2].contains(c)).copied().collect();
                if shared.len() != 1 { continue; }
                let c_shared = shared[0];
                let ca = *row_cols[r1].iter().find(|&&c| c != c_shared).unwrap();
                let cb = *row_cols[r2].iter().find(|&&c| c != c_shared).unwrap();
                for r in 0..9 {
                    for c in 0..9 {
                        if (r, c) != (r1, ca) && (r, c) != (r2, cb)
                            && sees(r, c, r1, ca) && sees(r, c, r2, cb)
                            && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                        {
                            result.push(elim(r, c, digit));
                        }
                    }
                }
                if !result.is_empty() { return result; }
            }
        }
        // Column-based
        let col_rows: Vec<Vec<usize>> = (0..9)
            .map(|c| (0..9).filter(|&r| cands.mask(r, c) != 0 && cands.has(r, c, digit)).collect())
            .collect();
        for c1 in 0..9 {
            if col_rows[c1].len() != 2 { continue; }
            for c2 in (c1 + 1)..9 {
                if col_rows[c2].len() != 2 { continue; }
                let shared: Vec<usize> = col_rows[c1].iter()
                    .filter(|r| col_rows[c2].contains(r)).copied().collect();
                if shared.len() != 1 { continue; }
                let r_shared = shared[0];
                let ra = *col_rows[c1].iter().find(|&&r| r != r_shared).unwrap();
                let rb = *col_rows[c2].iter().find(|&&r| r != r_shared).unwrap();
                for r in 0..9 {
                    for c in 0..9 {
                        if (r, c) != (ra, c1) && (r, c) != (rb, c2)
                            && sees(r, c, ra, c1) && sees(r, c, rb, c2)
                            && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                        {
                            result.push(elim(r, c, digit));
                        }
                    }
                }
                if !result.is_empty() { return result; }
            }
        }
    }
    result
}

// ── 2-String Kite ─────────────────────────────────────────────────────────────

pub fn find_two_string_kite(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        for row in 0..9usize {
            let row_cs: Vec<usize> = (0..9)
                .filter(|&c| cands.mask(row, c) != 0 && cands.has(row, c, digit))
                .collect();
            if row_cs.len() != 2 { continue; }
            let (rc1, rc2) = (row_cs[0], row_cs[1]);
            for col in 0..9usize {
                let col_rs: Vec<usize> = (0..9)
                    .filter(|&r| cands.mask(r, col) != 0 && cands.has(r, col, digit))
                    .collect();
                if col_rs.len() != 2 { continue; }
                let (cr1, cr2) = (col_rs[0], col_rs[1]);
                let row_pair = [(row, rc1), (row, rc2)];
                let col_pair = [(cr1, col), (cr2, col)];
                for &(r_int, c_int) in &row_pair {
                    for &(r_col_int, c_col_int) in &col_pair {
                        if (r_int, c_int) == (r_col_int, c_col_int) { continue; }
                        if r_int / 3 != r_col_int / 3 || c_int / 3 != c_col_int / 3 { continue; }
                        let tip1 = *row_pair.iter().find(|&&rc| rc != (r_int, c_int)).unwrap();
                        let tip2 = *col_pair.iter().find(|&&rc| rc != (r_col_int, c_col_int)).unwrap();
                        for r in 0..9 {
                            for c in 0..9 {
                                if (r, c) != tip1 && (r, c) != tip2
                                    && sees(r, c, tip1.0, tip1.1) && sees(r, c, tip2.0, tip2.1)
                                    && cands.mask(r, c) != 0 && cands.has(r, c, digit)
                                {
                                    result.push(elim(r, c, digit));
                                }
                            }
                        }
                        if !result.is_empty() { return result; }
                    }
                }
            }
        }
    }
    result
}

// ── Y-Wing ────────────────────────────────────────────────────────────────────

pub fn find_y_wing(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    let bi_cells: Vec<(usize, usize, u16)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter_map(|(r, c)| {
            let m = cands.mask(r, c);
            if m.count_ones() == 2 { Some((r, c, m)) } else { None }
        })
        .collect();
    for &(r0, c0, m0) in &bi_cells {
        let a = m0.trailing_zeros() as u8;
        let b = (m0 >> (a as u32 + 1)).trailing_zeros() as u8 + a + 1;
        for &(r1, c1, m1) in &bi_cells {
            if (r1, c1) == (r0, c0) { continue; }
            if !sees(r0, c0, r1, c1) { continue; }
            let shared = m0 & m1;
            if shared.count_ones() != 1 { continue; }
            let shared_ab = shared.trailing_zeros() as u8;
            let c_digit = (m1 & !shared).trailing_zeros() as u8;
            let other_ab = if shared_ab == a { b } else { a };
            let needed_m2 = (1u16 << other_ab) | (1u16 << c_digit);
            for &(r2, c2, m2) in &bi_cells {
                if (r2, c2) == (r0, c0) || (r2, c2) == (r1, c1) { continue; }
                if !sees(r0, c0, r2, c2) { continue; }
                if m2 != needed_m2 { continue; }
                for r in 0..9 {
                    for c in 0..9 {
                        if (r, c) != (r0, c0) && (r, c) != (r1, c1) && (r, c) != (r2, c2)
                            && cands.mask(r, c) != 0
                            && sees(r, c, r1, c1) && sees(r, c, r2, c2)
                            && cands.has(r, c, c_digit)
                        {
                            result.push(elim(r, c, c_digit));
                        }
                    }
                }
                if !result.is_empty() { return result; }
            }
        }
    }
    result
}

// ── XYZ-Wing ──────────────────────────────────────────────────────────────────

pub fn find_xyz_wing(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    let bivalue: Vec<(usize, usize, u16)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter_map(|(r, c)| {
            let m = cands.mask(r, c);
            if m.count_ones() == 2 { Some((r, c, m)) } else { None }
        })
        .collect();
    let trivalue: Vec<(usize, usize, u16)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter_map(|(r, c)| {
            let m = cands.mask(r, c);
            if m.count_ones() == 3 { Some((r, c, m)) } else { None }
        })
        .collect();
    for &(pr, pc, pm) in &trivalue {
        let digits: Vec<u8> = (1u8..=9).filter(|&d| pm & (1 << d) != 0).collect();
        if digits.len() != 3 { continue; }
        let (da, db, dc) = (digits[0], digits[1], digits[2]);
        for &c_digit in &[da, db, dc] {
            let others: Vec<u8> = [da, db, dc].iter().filter(|&&d| d != c_digit).copied().collect();
            let wing_masks = [
                (1u16 << others[0]) | (1u16 << c_digit),
                (1u16 << others[1]) | (1u16 << c_digit),
            ];
            let wings: [Vec<(usize, usize)>; 2] = [
                bivalue.iter()
                    .filter(|&&(r, c, m)| m == wing_masks[0] && sees(r, c, pr, pc))
                    .map(|&(r, c, _)| (r, c)).collect(),
                bivalue.iter()
                    .filter(|&&(r, c, m)| m == wing_masks[1] && sees(r, c, pr, pc))
                    .map(|&(r, c, _)| (r, c)).collect(),
            ];
            for &w1 in &wings[0] {
                for &w2 in &wings[1] {
                    if w1 == w2 { continue; }
                    // Eliminate c_digit from cells seeing all three: pivot, w1, w2
                    for r in 0..9 {
                        for c in 0..9 {
                            if (r, c) == (pr, pc) || (r, c) == w1 || (r, c) == w2 { continue; }
                            if cands.mask(r, c) == 0 { continue; }
                            if !cands.has(r, c, c_digit) { continue; }
                            if sees(r, c, pr, pc) && sees(r, c, w1.0, w1.1) && sees(r, c, w2.0, w2.1) {
                                result.push(elim(r, c, c_digit));
                            }
                        }
                    }
                    if !result.is_empty() { return result; }
                }
            }
        }
    }
    result
}

// ── W-Wing ────────────────────────────────────────────────────────────────────

pub fn find_w_wing(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    let pairs: Vec<(usize, usize)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter(|&(r, c)| cands.mask(r, c).count_ones() == 2)
        .collect();
    for &p1 in &pairs {
        for &p2 in &pairs {
            if p1 >= p2 { continue; }
            if cands.mask(p1.0, p1.1) != cands.mask(p2.0, p2.1) { continue; }
            if sees(p1.0, p1.1, p2.0, p2.1) { continue; }
            let pair_mask = cands.mask(p1.0, p1.1);
            let a = pair_mask.trailing_zeros() as u8;
            let b = (pair_mask >> (a as u32 + 1)).trailing_zeros() as u8 + a + 1;
            // Need strong link on `a` connecting p1 and p2 through a unit
            for unit in all_units() {
                let unit_a: Vec<(usize, usize)> = unit.iter()
                    .filter(|&&(r, c)| cands.mask(r, c) != 0 && cands.has(r, c, a))
                    .copied().collect();
                if unit_a.len() != 2 { continue; }
                let (e1, e2) = (unit_a[0], unit_a[1]);
                // p1 sees e1 and p2 sees e2 (or vice versa)
                for &(ea, eb) in &[(e1, e2), (e2, e1)] {
                    if ea == p1 || eb == p2 { continue; }
                    if !sees(p1.0, p1.1, ea.0, ea.1) { continue; }
                    if !sees(p2.0, p2.1, eb.0, eb.1) { continue; }
                    // Eliminate b from cells seeing both p1 and p2
                    for r in 0..9 {
                        for c in 0..9 {
                            if (r, c) == p1 || (r, c) == p2 { continue; }
                            if cands.mask(r, c) == 0 { continue; }
                            if !cands.has(r, c, b) { continue; }
                            if sees(r, c, p1.0, p1.1) && sees(r, c, p2.0, p2.1) {
                                result.push(elim(r, c, b));
                            }
                        }
                    }
                    if !result.is_empty() { return result; }
                }
            }
        }
    }
    result
}

// ── Unique Rectangle ──────────────────────────────────────────────────────────

pub fn find_unique_rectangle(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for r1 in 0..9usize {
        for r2 in (r1 + 1)..9 {
            for c1 in 0..9usize {
                for c2 in (c1 + 1)..9 {
                    let corners = [(r1, c1), (r1, c2), (r2, c1), (r2, c2)];
                    // All corners must be empty (non-zero mask)
                    if corners.iter().any(|&(r, c)| cands.mask(r, c) == 0) { continue; }
                    // Must span exactly 2 different boxes
                    let boxes: std::collections::HashSet<usize> =
                        corners.iter().map(|&(r, c)| (r / 3) * 3 + c / 3).collect();
                    if boxes.len() != 2 { continue; }
                    for a in 1u8..=9 {
                        for b in (a + 1)..=9 {
                            let pair_mask = (1u16 << a) | (1u16 << b);
                            if corners.iter().any(|&(r, c)| cands.mask(r, c) & pair_mask != pair_mask) {
                                continue;
                            }
                            // Type 1: 3 corners have ONLY {a,b}, 1 roof has extras
                            let only_pair: Vec<(usize, usize)> = corners.iter()
                                .filter(|&&(r, c)| cands.mask(r, c) == pair_mask)
                                .copied().collect();
                            let has_extras: Vec<(usize, usize)> = corners.iter()
                                .filter(|&&(r, c)| cands.mask(r, c) & !pair_mask != 0)
                                .copied().collect();
                            if only_pair.len() == 3 && has_extras.len() == 1 {
                                let (rr, cc) = has_extras[0];
                                for d in [a, b] {
                                    if cands.has(rr, cc, d) {
                                        result.push(elim(rr, cc, d));
                                    }
                                }
                                if !result.is_empty() { return result; }
                            }
                            // Type 2: 2 floors {a,b}, 2 roofs {a,b,c} on same side
                            if has_extras.len() != 2 || only_pair.len() != 2 { continue; }
                            let (roof_a, roof_b) = (has_extras[0], has_extras[1]);
                            let same_side = roof_a.0 == roof_b.0 || roof_a.1 == roof_b.1;
                            if !same_side { continue; }
                            let extra_a = cands.mask(roof_a.0, roof_a.1) & !pair_mask;
                            let extra_b = cands.mask(roof_b.0, roof_b.1) & !pair_mask;
                            if extra_a.count_ones() != 1 || extra_a != extra_b { continue; }
                            let c_digit = extra_a.trailing_zeros() as u8;
                            for r in 0..9 {
                                for c in 0..9 {
                                    if corners.contains(&(r, c)) { continue; }
                                    if cands.mask(r, c) == 0 { continue; }
                                    if !cands.has(r, c, c_digit) { continue; }
                                    if sees(r, c, roof_a.0, roof_a.1) && sees(r, c, roof_b.0, roof_b.1) {
                                        result.push(elim(r, c, c_digit));
                                    }
                                }
                            }
                            if !result.is_empty() { return result; }
                        }
                    }
                }
            }
        }
    }
    result
}

// ── BUG+1 ─────────────────────────────────────────────────────────────────────
// Returns a placement (SolveStep), not an elimination.

pub fn find_bug_plus_one_step(cands: &CandidateGrid) -> Option<SolveStep> {
    let mut trivalue_cell: Option<(usize, usize)> = None;
    for r in 0..9 {
        for c in 0..9 {
            let n = cands.mask(r, c).count_ones();
            if n == 0 { continue; } // filled cell
            if n == 3 {
                if trivalue_cell.is_some() { return None; } // more than 1 trivalue
                trivalue_cell = Some((r, c));
            } else if n != 2 {
                return None; // non-bivalue non-trivalue cell
            }
        }
    }
    let (br, bc) = trivalue_cell?;
    let mask = cands.mask(br, bc);
    let digits: Vec<u8> = (1u8..=9).filter(|&d| mask & (1 << d) != 0).collect();
    if digits.len() != 3 { return None; }
    for &d in &digits {
        let row_count = (0..9).filter(|&c| c != bc && cands.mask(br, c) != 0 && cands.has(br, c, d)).count();
        let col_count = (0..9).filter(|&r| r != br && cands.mask(r, bc) != 0 && cands.has(r, bc, d)).count();
        let box_br = (br / 3) * 3;
        let box_bc = (bc / 3) * 3;
        let box_count = (0..3).flat_map(|dr| (0..3).map(move |dc| (box_br + dr, box_bc + dc)))
            .filter(|&(r, c)| (r, c) != (br, bc) && cands.mask(r, c) != 0 && cands.has(r, c, d))
            .count();
        if row_count % 2 == 1 && col_count % 2 == 1 && box_count % 2 == 1 {
            return Some(SolveStep {
                row: br,
                col: bc,
                digit: d,
                strategy: Strategy::Expert,
                source_cells: vec![],
            });
        }
    }
    None
}

// ── Empty Rectangle ───────────────────────────────────────────────────────────

pub fn find_empty_rectangle(cands: &CandidateGrid) -> Vec<Elimination> {
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        for box_idx in 0..9usize {
            let box_row = (box_idx / 3) * 3;
            let box_col = (box_idx % 3) * 3;
            let box_cells: Vec<(usize, usize)> = (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (box_row + dr, box_col + dc)))
                .filter(|&(r, c)| cands.mask(r, c) != 0 && cands.has(r, c, digit))
                .collect();
            if box_cells.len() < 2 { continue; }
            // ER on a row: all cells in the same row
            let er_row = box_cells[0].0;
            if box_cells.iter().all(|&(r, _)| r == er_row) {
                for c_conj in 0..9usize {
                    if c_conj / 3 == box_col / 3 { continue; }
                    let col_cells: Vec<usize> = (0..9)
                        .filter(|&r| cands.mask(r, c_conj) != 0 && cands.has(r, c_conj, digit))
                        .collect();
                    if col_cells.len() != 2 { continue; }
                    let (r_a, r_b) = (col_cells[0], col_cells[1]);
                    let r_other = if r_a == er_row { r_b } else if r_b == er_row { r_a } else { continue };
                    for c_er in box_col..(box_col + 3) {
                        if c_er == c_conj { continue; }
                        if cands.mask(r_other, c_er) == 0 { continue; }
                        if !cands.has(r_other, c_er, digit) { continue; }
                        result.push(elim(r_other, c_er, digit));
                    }
                    if !result.is_empty() { return result; }
                }
            }
            // ER on a column: all cells in the same col
            let er_col = box_cells[0].1;
            if box_cells.iter().all(|&(_, c)| c == er_col) {
                for r_conj in 0..9usize {
                    if r_conj / 3 == box_row / 3 { continue; }
                    let row_cells: Vec<usize> = (0..9)
                        .filter(|&c| cands.mask(r_conj, c) != 0 && cands.has(r_conj, c, digit))
                        .collect();
                    if row_cells.len() != 2 { continue; }
                    let (c_a, c_b) = (row_cells[0], row_cells[1]);
                    let c_other = if c_a == er_col { c_b } else if c_b == er_col { c_a } else { continue };
                    for r_er in box_row..(box_row + 3) {
                        if r_er == r_conj { continue; }
                        if cands.mask(r_er, c_other) == 0 { continue; }
                        if !cands.has(r_er, c_other, digit) { continue; }
                        result.push(elim(r_er, c_other, digit));
                    }
                    if !result.is_empty() { return result; }
                }
            }
        }
    }
    result
}

// ── Simple Coloring ───────────────────────────────────────────────────────────

pub fn find_simple_coloring(cands: &CandidateGrid) -> Vec<Elimination> {
    use std::collections::HashMap;
    let mut result = Vec::new();
    for digit in 1u8..=9 {
        let mut links: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();
        for unit in all_units() {
            let cells_d: Vec<(usize, usize)> = unit.iter()
                .filter(|&&(r, c)| cands.mask(r, c) != 0 && cands.has(r, c, digit))
                .copied().collect();
            if cells_d.len() == 2 {
                links.entry(cells_d[0]).or_default().push(cells_d[1]);
                links.entry(cells_d[1]).or_default().push(cells_d[0]);
            }
        }
        if links.is_empty() { continue; }
        let mut color_map: HashMap<(usize, usize), u8> = HashMap::new();
        let all_linked: Vec<(usize, usize)> = links.keys().copied().collect();
        for &start in &all_linked {
            if color_map.contains_key(&start) { continue; }
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(start);
            color_map.insert(start, 0);
            let mut component: Vec<(usize, usize)> = vec![start];
            while let Some(cell) = queue.pop_front() {
                let cell_color = color_map[&cell];
                let next_color = 1 - cell_color;
                if let Some(neighbors) = links.get(&cell) {
                    for &nb in neighbors {
                        if !color_map.contains_key(&nb) {
                            color_map.insert(nb, next_color);
                            component.push(nb);
                            queue.push_back(nb);
                        }
                    }
                }
            }
            let color0: Vec<(usize, usize)> = component.iter()
                .filter(|&&c| color_map[&c] == 0).copied().collect();
            let color1: Vec<(usize, usize)> = component.iter()
                .filter(|&&c| color_map[&c] == 1).copied().collect();
            // Color Wrap: two same-color cells see each other
            for col_group in [&color0, &color1] {
                let mut wrap_found = false;
                'outer: for i in 0..col_group.len() {
                    for j in (i + 1)..col_group.len() {
                        let (r1, c1) = col_group[i];
                        let (r2, c2) = col_group[j];
                        if sees(r1, c1, r2, c2) {
                            wrap_found = true;
                            break 'outer;
                        }
                    }
                }
                if wrap_found {
                    for &(r, c) in col_group {
                        if cands.mask(r, c) != 0 && cands.has(r, c, digit) {
                            result.push(elim(r, c, digit));
                        }
                    }
                    if !result.is_empty() { return result; }
                }
            }
            // Color Trap: external cell sees both colors
            for r in 0..9usize {
                for c in 0..9usize {
                    if color_map.contains_key(&(r, c)) { continue; }
                    if cands.mask(r, c) == 0 { continue; }
                    if !cands.has(r, c, digit) { continue; }
                    let seen0 = color0.iter().any(|&(r2, c2)| sees(r, c, r2, c2));
                    let seen1 = color1.iter().any(|&(r2, c2)| sees(r, c, r2, c2));
                    if seen0 && seen1 {
                        result.push(elim(r, c, digit));
                    }
                }
            }
            if !result.is_empty() { return result; }
        }
    }
    result
}

// ── XY-Chain ──────────────────────────────────────────────────────────────────

pub fn find_xy_chain(cands: &CandidateGrid) -> Vec<Elimination> {
    let bivalue: Vec<(usize, usize, u16)> = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter_map(|(r, c)| {
            let m = cands.mask(r, c);
            if m.count_ones() == 2 { Some((r, c, m)) } else { None }
        })
        .collect();
    const MAX_DEPTH: usize = 8;
    for &(sr, sc, sm) in &bivalue {
        let elim_d = sm.trailing_zeros() as u8;
        let x = (sm >> (elim_d as u32 + 1)).trailing_zeros() as u8 + elim_d + 1;
        let mut chain: Vec<(usize, usize)> = vec![(sr, sc)];
        if let Some(r) = xy_chain_dfs(&mut chain, x, elim_d, &bivalue, cands, MAX_DEPTH) {
            return r;
        }
    }
    vec![]
}

fn xy_chain_dfs(
    chain: &mut Vec<(usize, usize)>,
    incoming: u8,
    elim_d: u8,
    bivalue: &[(usize, usize, u16)],
    cands: &CandidateGrid,
    max_depth: usize,
) -> Option<Vec<Elimination>> {
    if chain.len() >= max_depth { return None; }
    let &(cur_r, cur_c) = chain.last().unwrap();
    for &(nr, nc, nm) in bivalue {
        if chain.contains(&(nr, nc)) { continue; }
        if !sees(cur_r, cur_c, nr, nc) { continue; }
        if (nm & (1 << incoming)) == 0 { continue; }
        let other = if nm.trailing_zeros() as u8 == incoming {
            (nm >> (incoming as u32 + 1)).trailing_zeros() as u8 + incoming + 1
        } else {
            nm.trailing_zeros() as u8
        };
        chain.push((nr, nc));
        if other == elim_d && chain.len() >= 3 {
            let (start_r, start_c) = chain[0];
            let mut result = Vec::new();
            for r in 0..9 {
                for c in 0..9 {
                    if chain.contains(&(r, c)) { continue; }
                    if cands.mask(r, c) == 0 { continue; }
                    if !cands.has(r, c, elim_d) { continue; }
                    if sees(r, c, start_r, start_c) && sees(r, c, nr, nc) {
                        result.push(elim(r, c, elim_d));
                    }
                }
            }
            if !result.is_empty() {
                chain.pop();
                return Some(result);
            }
        }
        if let Some(r) = xy_chain_dfs(chain, other, elim_d, bivalue, cands, max_depth) {
            chain.pop();
            return Some(r);
        }
        chain.pop();
    }
    None
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::candidates::CandidateGrid;
    use crate::puzzle::Grid;

    // Helper: blank CandidateGrid with all 9 candidates in every cell.
    fn blank_cands() -> CandidateGrid {
        CandidateGrid::from_grid(
            &Grid::from_str("000000000000000000000000000000000000000000000000000000000000000000000000000000000")
                .unwrap(),
        )
    }

    // Helper: remove all candidates except `keep` from a cell.
    fn keep_only(c: &mut CandidateGrid, row: usize, col: usize, keep: &[u8]) {
        for d in 1u8..=9 {
            if !keep.contains(&d) {
                c.remove(row, col, d);
            }
        }
    }

    // Helper: remove digit `d` from every cell except those in `except`.
    fn remove_digit_except(c: &mut CandidateGrid, d: u8, except: &[(usize, usize)]) {
        for r in 0..9 {
            for col in 0..9 {
                if !except.contains(&(r, col)) {
                    c.remove(r, col, d);
                }
            }
        }
    }

    // ── Blank grid returns empty (negative tests) ──────────────────────────────

    #[test]
    fn jellyfish_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_jellyfish(&c).is_empty());
    }

    #[test]
    fn naked_quad_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_naked_quad(&c).is_empty());
    }

    #[test]
    fn hidden_triple_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_hidden_triple(&c).is_empty());
    }

    #[test]
    fn hidden_quad_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_hidden_quad(&c).is_empty());
    }

    #[test]
    fn skyscraper_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_skyscraper(&c).is_empty());
    }

    #[test]
    fn two_string_kite_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_two_string_kite(&c).is_empty());
    }

    #[test]
    fn y_wing_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_y_wing(&c).is_empty());
    }

    #[test]
    fn xyz_wing_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_xyz_wing(&c).is_empty());
    }

    #[test]
    fn w_wing_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_w_wing(&c).is_empty());
    }

    #[test]
    fn unique_rectangle_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_unique_rectangle(&c).is_empty());
    }

    #[test]
    fn bug_plus_one_returns_none_on_blank_grid() {
        let c = blank_cands();
        assert!(find_bug_plus_one_step(&c).is_none());
    }

    #[test]
    fn empty_rectangle_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_empty_rectangle(&c).is_empty());
    }

    #[test]
    fn simple_coloring_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_simple_coloring(&c).is_empty());
    }

    #[test]
    fn xy_chain_returns_empty_on_blank_grid() {
        let c = blank_cands();
        assert!(find_xy_chain(&c).is_empty());
    }

    // ── Positive tests ─────────────────────────────────────────────────────────

    #[test]
    fn jellyfish_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        let fish_rows = [0usize, 2, 5, 7];
        let fish_cols = [1usize, 4, 6, 8];
        for r in 0..9 {
            for col in 0..9 {
                if fish_rows.contains(&r) {
                    if !fish_cols.contains(&col) { c.remove(r, col, 3); }
                } else {
                    if !(r == 3 && col == 1) { c.remove(r, col, 3); }
                }
            }
        }
        let elims = find_jellyfish(&c);
        assert!(!elims.is_empty(), "jellyfish should find eliminations");
        assert!(elims.iter().any(|e| e.digit == 3));
    }

    #[test]
    fn naked_quad_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        for col in 0..4usize { keep_only(&mut c, 0, col, &[1, 2, 3, 4]); }
        for col in 5..9usize { for d in [1u8,2,3,4] { c.remove(0, col, d); } }
        let elims = find_naked_quad(&c);
        assert!(!elims.is_empty(), "naked_quad should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col == 4));
    }

    #[test]
    fn hidden_triple_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        for col in 3..9usize { for d in [1u8,2,3] { c.remove(0, col, d); } }
        for col in 0..3usize { for d in [4u8,6,7,8,9] { c.remove(0, col, d); } }
        let elims = find_hidden_triple(&c);
        assert!(!elims.is_empty(), "hidden_triple should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col < 3 && e.digit == 5));
    }

    #[test]
    fn hidden_quad_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        for col in 4..9usize { for d in [1u8,2,3,4] { c.remove(0, col, d); } }
        for col in 0..4usize { for d in [6u8,7,8,9] { c.remove(0, col, d); } }
        let elims = find_hidden_quad(&c);
        assert!(!elims.is_empty(), "hidden_quad should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col < 4 && e.digit == 5));
    }

    #[test]
    fn skyscraper_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        remove_digit_except(&mut c, 5, &[(0,2),(0,6),(5,2),(5,8),(3,6)]);
        for &(r,col) in &[(0usize,2),(0,6),(5,2),(5,8)] { keep_only(&mut c, r, col, &[5]); }
        let elims = find_skyscraper(&c);
        assert!(!elims.is_empty(), "skyscraper should find eliminations");
        assert!(elims.iter().any(|e| e.row == 3 && e.col == 6 && e.digit == 5));
    }

    #[test]
    fn two_string_kite_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        remove_digit_except(&mut c, 2, &[(0,0),(0,3),(2,0),(2,3)]);
        keep_only(&mut c, 0, 0, &[2]);
        keep_only(&mut c, 0, 3, &[2]);
        keep_only(&mut c, 2, 0, &[2]);
        let elims = find_two_string_kite(&c);
        assert!(!elims.is_empty(), "two_string_kite should find eliminations");
        assert!(elims.iter().any(|e| e.digit == 2));
    }

    #[test]
    fn y_wing_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 4, 4, &[1, 3]);
        keep_only(&mut c, 4, 0, &[1, 2]);
        keep_only(&mut c, 0, 4, &[2, 3]);
        remove_digit_except(&mut c, 2, &[(4,0),(0,4),(0,0)]);
        let elims = find_y_wing(&c);
        assert!(!elims.is_empty(), "y_wing should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col == 0 && e.digit == 2));
    }

    #[test]
    fn xyz_wing_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 2, 2, &[1, 2, 3]);
        keep_only(&mut c, 2, 0, &[1, 3]);
        keep_only(&mut c, 0, 2, &[2, 3]);
        remove_digit_except(&mut c, 3, &[(2,2),(2,0),(0,2),(0,0)]);
        let elims = find_xyz_wing(&c);
        assert!(!elims.is_empty(), "xyz_wing should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col == 0 && e.digit == 3));
    }

    #[test]
    fn w_wing_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 0, 0, &[1, 2]);
        keep_only(&mut c, 5, 5, &[1, 2]);
        keep_only(&mut c, 0, 3, &[1]);
        keep_only(&mut c, 5, 3, &[1]);
        remove_digit_except(&mut c, 1, &[(0,0),(5,5),(0,3),(5,3)]);
        remove_digit_except(&mut c, 2, &[(0,0),(5,5),(0,5)]);
        let elims = find_w_wing(&c);
        assert!(!elims.is_empty(), "w_wing should find eliminations");
        assert!(elims.iter().any(|e| e.row == 0 && e.col == 5 && e.digit == 2));
    }

    #[test]
    fn unique_rectangle_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 0, 0, &[1, 2]);
        keep_only(&mut c, 0, 3, &[1, 2]);
        keep_only(&mut c, 6, 0, &[1, 2]);
        keep_only(&mut c, 6, 3, &[1, 2, 5]);
        for r in 0..9 { for col in 0..9 {
            if ![(0usize,0),(0,3),(6,0),(6,3)].contains(&(r,col)) {
                c.remove(r, col, 1); c.remove(r, col, 2);
            }
        }}
        let elims = find_unique_rectangle(&c);
        assert!(!elims.is_empty(), "unique_rectangle should find eliminations");
        assert!(elims.iter().any(|e| e.row == 6 && e.col == 3));
    }

    #[test]
    fn empty_rectangle_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        remove_digit_except(&mut c, 7, &[(0,0),(0,1),(0,6),(4,6),(4,1)]);
        keep_only(&mut c, 0, 0, &[7]);
        keep_only(&mut c, 0, 1, &[7]);
        keep_only(&mut c, 0, 6, &[7]);
        keep_only(&mut c, 4, 6, &[7]);
        let elims = find_empty_rectangle(&c);
        assert!(!elims.is_empty(), "empty_rectangle should find eliminations");
        assert!(elims.iter().any(|e| e.digit == 7));
    }

    #[test]
    fn simple_coloring_detects_color_wrap_and_eliminates() {
        let mut c = blank_cands();
        let chain = [(0usize,0),(3,0),(3,6),(0,6),(0,3)];
        remove_digit_except(&mut c, 7, &chain);
        for &(r,col) in &chain { keep_only(&mut c, r, col, &[7]); }
        let elims = find_simple_coloring(&c);
        assert!(!elims.is_empty(), "simple_coloring should find eliminations");
        assert!(elims.iter().all(|e| e.digit == 7));
        let cells: Vec<_> = elims.iter().map(|e| (e.row, e.col)).collect();
        for &cell in &[(0usize,0),(3,6),(0,3)] {
            assert!(cells.contains(&cell), "expected {:?} in elims, got {:?}", cell, cells);
        }
    }

    #[test]
    fn xy_chain_detects_pattern_and_eliminates() {
        let mut c = blank_cands();
        keep_only(&mut c, 0, 0, &[1, 2]);
        keep_only(&mut c, 0, 3, &[2, 3]);
        keep_only(&mut c, 4, 3, &[3, 1]);
        remove_digit_except(&mut c, 1, &[(0,0),(4,3),(4,0)]);
        let elims = find_xy_chain(&c);
        assert!(!elims.is_empty(), "xy_chain should find eliminations");
        assert!(elims.iter().any(|e| e.row == 4 && e.col == 0 && e.digit == 1));
    }

    #[test]
    fn bug_plus_one_returns_none_on_non_bug_board() {
        let mut c = blank_cands();
        for r in 0..9 { for col in 0..9 {
            for d in 3u8..=9 {
                if (r, col) == (4, 4) && d == 3 { continue; }
                c.remove(r, col, d);
            }
        }}
        assert!(find_bug_plus_one_step(&c).is_none());
    }
}
