// src/hint/strategies/tier2.rs
//
// Tier-2 hint strategies: Naked Triples, X-Wing, Swordfish.
// All three work exclusively from the player's notes masks — the same
// source of truth as the Tier-1 pair strategies.

use crate::hint::{Hint, Strategy};
use crate::puzzle::{CellKind, Grid};
use crate::puzzle::game_state::GameState;

// ── Public strategy structs ───────────────────────────────────────────────────

pub struct NakedTriples;
pub struct XWing;
pub struct Swordfish;
pub struct HiddenTriples;
pub struct YWing;
pub struct UniqueRectangle;

// ── Helper ────────────────────────────────────────────────────────────────────

/// Iterate the 27 units (9 rows, 9 cols, 9 boxes) as lists of (row, col).
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
        ); // box i
    }
    units
}

/// True if cells (r1,c1) and (r2,c2) share a row, column, or box.
fn sees(r1: usize, c1: usize, r2: usize, c2: usize) -> bool {
    r1 == r2 || c1 == c2 || (r1 / 3 == r2 / 3 && c1 / 3 == c2 / 3)
}

// ── Naked Triples ─────────────────────────────────────────────────────────────

impl Strategy for NakedTriples {
    fn name_en(&self) -> &'static str { "Naked Triples" }
    fn name_de(&self) -> &'static str { "Naked Triples" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            // Candidates: empty cells with 2 or 3 notes (cells with 1 note are naked
            // singles and handled earlier; cells with 0 notes skip cleanly).
            let small: Vec<(usize, usize, u16)> = unit
                .iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .filter_map(|&(r, c)| {
                    let m = state.notes_mask(r, c);
                    let n = m.count_ones();
                    if n == 2 || n == 3 { Some((r, c, m)) } else { None }
                })
                .collect();

            for i in 0..small.len() {
                for j in (i + 1)..small.len() {
                    for k in (j + 1)..small.len() {
                        let combined = small[i].2 | small[j].2 | small[k].2;
                        if combined.count_ones() != 3 { continue; }

                        let triple = [
                            (small[i].0, small[i].1),
                            (small[j].0, small[j].1),
                            (small[k].0, small[k].1),
                        ];
                        let digits: Vec<u8> = (1u8..=9)
                            .filter(|&d| combined & (1 << d) != 0)
                            .collect();

                        // Elimination targets: other cells in the unit that still
                        // carry at least one of the three triple digits.
                        let elim: Vec<(usize, usize)> = unit
                            .iter()
                            .filter(|&&(r, c)| {
                                !triple.contains(&(r, c))
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & combined) != 0
                            })
                            .copied()
                            .collect();

                        if elim.is_empty() { continue; }

                        let (d1, d2, d3) = (digits[0], digits[1], digits[2]);
                        return Some(Hint {
                            cause_cells:    triple.to_vec(),
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(d1),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "These 3 cells hold only {}/{}/{}. Remove those digits from notes in highlighted cells.",
                                d1, d2, d3
                            ),
                            explanation_de: format!(
                                "Diese 3 Zellen enthalten nur {}/{}/{}. Diese Ziffern aus den markierten Zellen streichen.",
                                d1, d2, d3
                            ),
                        });
                    }
                }
            }
        }
        None
    }
}

// ── X-Wing ────────────────────────────────────────────────────────────────────

impl Strategy for XWing {
    fn name_en(&self) -> &'static str { "X-Wing" }
    fn name_de(&self) -> &'static str { "X-Wing" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        for digit in 1u8..=9 {
            // ── Row-based: digit in exactly 2 cols in each of 2 rows ──────────
            let row_cols: Vec<Vec<usize>> = (0..9)
                .map(|r| {
                    (0..9)
                        .filter(|&c| {
                            matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect()
                })
                .collect();

            for r1 in 0..9 {
                if row_cols[r1].len() != 2 { continue; }
                for r2 in (r1 + 1)..9 {
                    if row_cols[r2] != row_cols[r1] { continue; }
                    let (c1, c2) = (row_cols[r1][0], row_cols[r1][1]);

                    let elim: Vec<(usize, usize)> = (0..9)
                        .filter(|&r| r != r1 && r != r2)
                        .flat_map(|r| [(r, c1), (r, c2)])
                        .filter(|&(r, c)| {
                            matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();

                    if elim.is_empty() { continue; }

                    return Some(Hint {
                        cause_cells:    vec![(r1, c1), (r1, c2), (r2, c1), (r2, c2)],
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "{} appears in exactly 2 columns in both rows. Remove {} from notes in those columns.",
                            digit, digit
                        ),
                        explanation_de: format!(
                            "{} ist in beiden Zeilen auf dieselben 2 Spalten beschr\u{e4}nkt. {} aus diesen Spalten streichen.",
                            digit, digit
                        ),
                    });
                }
            }

            // ── Column-based: digit in exactly 2 rows in each of 2 cols ──────
            let col_rows: Vec<Vec<usize>> = (0..9)
                .map(|c| {
                    (0..9)
                        .filter(|&r| {
                            matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect()
                })
                .collect();

            for c1 in 0..9 {
                if col_rows[c1].len() != 2 { continue; }
                for c2 in (c1 + 1)..9 {
                    if col_rows[c2] != col_rows[c1] { continue; }
                    let (r1, r2) = (col_rows[c1][0], col_rows[c1][1]);

                    let elim: Vec<(usize, usize)> = (0..9)
                        .filter(|&c| c != c1 && c != c2)
                        .flat_map(|c| [(r1, c), (r2, c)])
                        .filter(|&(r, c)| {
                            matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();

                    if elim.is_empty() { continue; }

                    return Some(Hint {
                        cause_cells:    vec![(r1, c1), (r1, c2), (r2, c1), (r2, c2)],
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "{} appears in exactly 2 rows in both columns. Remove {} from notes in those rows.",
                            digit, digit
                        ),
                        explanation_de: format!(
                            "{} ist in beiden Spalten auf dieselben 2 Zeilen beschr\u{e4}nkt. {} aus diesen Zeilen streichen.",
                            digit, digit
                        ),
                    });
                }
            }
        }
        None
    }
}

// ── Swordfish ─────────────────────────────────────────────────────────────────

impl Strategy for Swordfish {
    fn name_en(&self) -> &'static str { "Swordfish" }
    fn name_de(&self) -> &'static str { "Swordfish" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        for digit in 1u8..=9 {
            // ── Row-based Swordfish ───────────────────────────────────────────
            let row_cols: Vec<Vec<usize>> = (0..9)
                .map(|r| {
                    (0..9)
                        .filter(|&c| {
                            matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect()
                })
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
                        let (r1, r2, r3) =
                            (cand_rows[i], cand_rows[j], cand_rows[k]);

                        let mut cols = std::collections::BTreeSet::new();
                        for &c in &row_cols[r1] { cols.insert(c); }
                        for &c in &row_cols[r2] { cols.insert(c); }
                        for &c in &row_cols[r3] { cols.insert(c); }
                        if cols.len() != 3 { continue; }

                        let elim: Vec<(usize, usize)> = cols
                            .iter()
                            .flat_map(|&c| (0..9).map(move |r| (r, c)))
                            .filter(|&(r, c)| {
                                r != r1 && r != r2 && r != r3
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & (1 << digit)) != 0
                            })
                            .collect();

                        if elim.is_empty() { continue; }

                        let cause: Vec<(usize, usize)> = [r1, r2, r3]
                            .iter()
                            .flat_map(|&r| row_cols[r].iter().map(move |&c| (r, c)))
                            .collect();

                        return Some(Hint {
                            cause_cells:    cause,
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(digit),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "{} spans 3 rows, all confined to 3 columns. Remove {} from notes in those columns.",
                                digit, digit
                            ),
                            explanation_de: format!(
                                "{} erstreckt sich \u{fc}ber 3 Zeilen auf 3 Spalten. {} aus diesen Spalten streichen.",
                                digit, digit
                            ),
                        });
                    }
                }
            }

            // ── Column-based Swordfish ────────────────────────────────────────
            let col_rows: Vec<Vec<usize>> = (0..9)
                .map(|c| {
                    (0..9)
                        .filter(|&r| {
                            matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect()
                })
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
                        let (c1, c2, c3) =
                            (cand_cols[i], cand_cols[j], cand_cols[k]);

                        let mut rows = std::collections::BTreeSet::new();
                        for &r in &col_rows[c1] { rows.insert(r); }
                        for &r in &col_rows[c2] { rows.insert(r); }
                        for &r in &col_rows[c3] { rows.insert(r); }
                        if rows.len() != 3 { continue; }

                        let elim: Vec<(usize, usize)> = rows
                            .iter()
                            .flat_map(|&r| (0..9).map(move |c| (r, c)))
                            .filter(|&(r, c)| {
                                c != c1 && c != c2 && c != c3
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & (1 << digit)) != 0
                            })
                            .collect();

                        if elim.is_empty() { continue; }

                        let cause: Vec<(usize, usize)> = [c1, c2, c3]
                            .iter()
                            .flat_map(|&c| col_rows[c].iter().map(move |&r| (r, c)))
                            .collect();

                        return Some(Hint {
                            cause_cells:    cause,
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(digit),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "{} spans 3 columns, all confined to 3 rows. Remove {} from notes in those rows.",
                                digit, digit
                            ),
                            explanation_de: format!(
                                "{} erstreckt sich \u{fc}ber 3 Spalten auf 3 Zeilen. {} aus diesen Zeilen streichen.",
                                digit, digit
                            ),
                        });
                    }
                }
            }
        }
        None
    }
}

// ── Hidden Triples ────────────────────────────────────────────────────────────

impl Strategy for HiddenTriples {
    fn name_en(&self) -> &'static str { "Hidden Triples" }
    fn name_de(&self) -> &'static str { "Hidden Triples" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let empties: Vec<(usize, usize)> = unit.iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .copied().collect();
            for d1 in 1u8..=9 {
                for d2 in (d1 + 1)..=9 {
                    for d3 in (d2 + 1)..=9 {
                        let triple_mask = (1u16 << d1) | (1u16 << d2) | (1u16 << d3);
                        // Cells that contain at least one of d1, d2, d3 in notes
                        let triple_cells: Vec<(usize, usize)> = empties.iter()
                            .filter(|&&(r, c)| (state.notes_mask(r, c) & triple_mask) != 0)
                            .copied().collect();
                        if triple_cells.len() != 3 { continue; }
                        // All three digits must appear in the combined notes
                        let combined: u16 = triple_cells.iter()
                            .fold(0u16, |acc, &(r, c)| acc | state.notes_mask(r, c));
                        if combined & triple_mask != triple_mask { continue; }
                        // At least one of the 3 cells must have extra notes to remove
                        let elim_cells: Vec<(usize, usize)> = triple_cells.iter()
                            .filter(|&&(r, c)| state.notes_mask(r, c) & !triple_mask != 0)
                            .copied().collect();
                        if elim_cells.is_empty() { continue; }
                        let target = elim_cells[0];
                        return Some(Hint {
                            cause_cells:    triple_cells,
                            elim_cells,
                            target_cell:    target,
                            elim_digit:     Some(d1),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "Only these 3 cells can hold {}/{}/{}. Remove all other notes from them.",
                                d1, d2, d3
                            ),
                            explanation_de: format!(
                                "Nur diese 3 Zellen k\u{f6}nnen {}/{}/{} enthalten. Alle anderen Notizen daraus streichen.",
                                d1, d2, d3
                            ),
                        });
                    }
                }
            }
        }
        None
    }
}

// ── Y-Wing ────────────────────────────────────────────────────────────────────

impl Strategy for YWing {
    fn name_en(&self) -> &'static str { "Y-Wing" }
    fn name_de(&self) -> &'static str { "Y-Wing" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        // Collect all empty cells with exactly 2 notes.
        let bi_cells: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter(|&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
            .filter_map(|(r, c)| {
                let m = state.notes_mask(r, c);
                if m.count_ones() == 2 { Some((r, c, m)) } else { None }
            })
            .collect();

        for &(r0, c0, m0) in &bi_cells {
            // Extract the two pivot digits a and b.
            let a = m0.trailing_zeros() as u8;
            let b = (m0 >> (a as u32 + 1)).trailing_zeros() as u8 + a + 1;

            for &(r1, c1, m1) in &bi_cells {
                if (r1, c1) == (r0, c0) { continue; }
                if !sees(r0, c0, r1, c1) { continue; }
                // Wing1 must share exactly 1 digit with pivot.
                let shared = m0 & m1;
                if shared.count_ones() != 1 { continue; }
                let shared_ab = shared.trailing_zeros() as u8; // a or b
                let c_digit = (m1 & !shared).trailing_zeros() as u8; // elimination digit
                // Wing2 needs the other pivot digit and c_digit.
                let other_ab = if shared_ab == a { b } else { a };
                let needed_m2 = (1u16 << other_ab) | (1u16 << c_digit);

                for &(r2, c2, m2) in &bi_cells {
                    if (r2, c2) == (r0, c0) || (r2, c2) == (r1, c1) { continue; }
                    if !sees(r0, c0, r2, c2) { continue; }
                    if m2 != needed_m2 { continue; }

                    // Eliminate c_digit from cells seeing both wings.
                    let elim: Vec<(usize, usize)> = (0..9)
                        .flat_map(|r| (0..9).map(move |c| (r, c)))
                        .filter(|&(r, c)| {
                            (r, c) != (r0, c0) && (r, c) != (r1, c1) && (r, c) != (r2, c2)
                                && matches!(grid.get(r, c), CellKind::Empty)
                                && sees(r, c, r1, c1)
                                && sees(r, c, r2, c2)
                                && (state.notes_mask(r, c) & (1 << c_digit)) != 0
                        })
                        .collect();

                    if elim.is_empty() { continue; }

                    return Some(Hint {
                        cause_cells:    vec![(r0, c0), (r1, c1), (r2, c2)],
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(c_digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "Y-Wing: the pivot forces {} into one of the wings. Remove {} from notes in highlighted cells.",
                            c_digit, c_digit
                        ),
                        explanation_de: format!(
                            "Y-Wing: Der Pivot zwingt {} in einen der Fl\u{fc}gel. {} aus den markierten Zellen streichen.",
                            c_digit, c_digit
                        ),
                    });
                }
            }
        }
        None
    }
}

// ── Unique Rectangle ──────────────────────────────────────────────────────────

impl Strategy for UniqueRectangle {
    fn name_en(&self) -> &'static str { "Unique Rectangle" }
    fn name_de(&self) -> &'static str { "Unique Rectangle" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        for r1 in 0..9usize {
            for r2 in (r1 + 1)..9 {
                for c1 in 0..9usize {
                    for c2 in (c1 + 1)..9 {
                        let corners = [(r1,c1),(r1,c2),(r2,c1),(r2,c2)];
                        // All corners must be empty.
                        if corners.iter().any(|&(r,c)| !matches!(grid.get(r,c), CellKind::Empty)) {
                            continue;
                        }
                        // Must span exactly 2 different boxes.
                        let boxes: std::collections::HashSet<usize> = corners.iter()
                            .map(|&(r,c)| (r/3)*3 + c/3)
                            .collect();
                        if boxes.len() != 2 { continue; }

                        // Find pair {a,b}: both digits present in notes of all 4 corners.
                        for a in 1u8..=9 {
                            for b in (a+1)..=9 {
                                let pair_mask = (1u16<<a)|(1u16<<b);
                                // All 4 corners must have both a and b in notes.
                                if corners.iter().any(|&(r,c)| state.notes_mask(r,c) & pair_mask != pair_mask) {
                                    continue;
                                }

                                // ── Type 1: 3 corners have ONLY {a,b}, 1 roof has extras ─────
                                let only_pair: Vec<(usize,usize)> = corners.iter()
                                    .filter(|&&(r,c)| state.notes_mask(r,c) == pair_mask)
                                    .copied().collect();
                                let has_extras: Vec<(usize,usize)> = corners.iter()
                                    .filter(|&&(r,c)| state.notes_mask(r,c) & !pair_mask != 0)
                                    .copied().collect();

                                if only_pair.len() == 3 && has_extras.len() == 1 {
                                    let roof = has_extras[0];
                                    return Some(Hint {
                                        cause_cells:    only_pair,
                                        elim_cells:     vec![roof],
                                        target_cell:    roof,
                                        elim_digit:     Some(a),
                                        target_digit:   None,
                                        name_en:        self.name_en(),
                                        name_de:        self.name_de(),
                                        explanation_en: format!(
                                            "3 corners are locked to {}/{}. Placing either here would create 2 solutions — remove {}/{} from notes here.",
                                            a, b, a, b
                                        ),
                                        explanation_de: format!(
                                            "3 Ecken sind auf {}/{} festgelegt. Beides hier zu setzen erg\u{e4}be 2 L\u{f6}sungen — {}/{} aus den Notizen streichen.",
                                            a, b, a, b
                                        ),
                                    });
                                }

                                // ── Type 2: 2 floors {a,b}, 2 roofs {a,b,c} on same side ────
                                // Find the extra candidate c shared by both roofs.
                                if has_extras.len() != 2 || only_pair.len() != 2 { continue; }
                                let (roof_a, roof_b) = (has_extras[0], has_extras[1]);
                                // Roofs must be on same row or same column.
                                let same_side = roof_a.0 == roof_b.0 || roof_a.1 == roof_b.1;
                                if !same_side { continue; }
                                let extra_a = state.notes_mask(roof_a.0, roof_a.1) & !pair_mask;
                                let extra_b = state.notes_mask(roof_b.0, roof_b.1) & !pair_mask;
                                // Both roofs must have exactly 1 extra digit and it must match.
                                if extra_a.count_ones() != 1 || extra_a != extra_b { continue; }
                                let c_digit = extra_a.trailing_zeros() as u8;

                                let elim: Vec<(usize,usize)> = (0..9)
                                    .flat_map(|r| (0..9).map(move |c| (r,c)))
                                    .filter(|&(r,c)| {
                                        !corners.contains(&(r,c))
                                            && matches!(grid.get(r,c), CellKind::Empty)
                                            && sees(r, c, roof_a.0, roof_a.1)
                                            && sees(r, c, roof_b.0, roof_b.1)
                                            && (state.notes_mask(r,c) & (1<<c_digit)) != 0
                                    })
                                    .collect();

                                if elim.is_empty() { continue; }

                                return Some(Hint {
                                    cause_cells:    corners.to_vec(),
                                    elim_cells:     elim.clone(),
                                    target_cell:    elim[0],
                                    elim_digit:     Some(c_digit),
                                    target_digit:   None,
                                    name_en:        self.name_en(),
                                    name_de:        self.name_de(),
                                    explanation_en: format!(
                                        "The floor cells lock {}/{}. {} must go in one roof cell — remove {} from notes in highlighted cells.",
                                        a, b, c_digit, c_digit
                                    ),
                                    explanation_de: format!(
                                        "Die Boden-Zellen sperren {}/{}. {} muss in eine Dach-Zelle — {} aus den markierten Zellen streichen.",
                                        a, b, c_digit, c_digit
                                    ),
                                });
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hint::Strategy;
    use crate::puzzle::{Grid, GameState};

    fn state_from(s: &str) -> GameState {
        let grid = Grid::from_str(s).unwrap();
        GameState::new(grid)
    }

    const PUZZLE: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const SOL: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn naked_triples_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(NakedTriples.find(&state, &sol).is_none());
    }

    #[test]
    fn x_wing_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(XWing.find(&state, &sol).is_none());
    }

    #[test]
    fn swordfish_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(Swordfish.find(&state, &sol).is_none());
    }

    #[test]
    fn hidden_triples_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(HiddenTriples.find(&state, &sol).is_none());
    }

    #[test]
    fn y_wing_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(YWing.find(&state, &sol).is_none());
    }

    #[test]
    fn unique_rectangle_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(UniqueRectangle.find(&state, &sol).is_none());
    }
}
