// src/hint/strategies/tier2.rs
//
// Tier-2 hint strategies: advanced eliminations and uniqueness techniques.

use crate::hint::{Hint, Strategy};
use crate::puzzle::game_state::GameState;
use crate::puzzle::{CellKind, Grid};

// ── Public strategy structs ───────────────────────────────────────────────────

pub struct NakedTriples;
pub struct HiddenTriples;
pub struct NakedQuads;
pub struct HiddenQuads;
pub struct XWing;
pub struct Swordfish;
pub struct Jellyfish;
pub struct Skyscraper;
pub struct TwoStringKite;
pub struct YWing;
pub struct XYZWing;
pub struct WWing;
pub struct UniqueRectangle;
pub struct BugPlusOne;
pub struct EmptyRectangle;
pub struct SimpleColoring;
pub struct XYChain;

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
    fn name_en(&self) -> &'static str {
        "Naked Triples"
    }
    fn name_de(&self) -> &'static str {
        "Naked Triples"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            // Candidates: empty cells with 1–3 notes.
            // A triple member may have just 1 candidate (e.g. {1,2} + {2,3} + {3})
            // — only the union of the three masks must equal exactly 3 digits.
            // Cells with 0 notes are not candidates; cells with 4+ notes cannot
            // be part of a triple (they'd push the union above 3).
            let small: Vec<(usize, usize, u16)> = unit
                .iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .filter_map(|&(r, c)| {
                    let m = state.notes_mask(r, c);
                    let n = m.count_ones();
                    if n >= 1 && n <= 3 {
                        Some((r, c, m))
                    } else {
                        None
                    }
                })
                .collect();

            for i in 0..small.len() {
                for j in (i + 1)..small.len() {
                    for k in (j + 1)..small.len() {
                        let combined = small[i].2 | small[j].2 | small[k].2;
                        if combined.count_ones() != 3 {
                            continue;
                        }

                        let triple = [
                            (small[i].0, small[i].1),
                            (small[j].0, small[j].1),
                            (small[k].0, small[k].1),
                        ];
                        let digits: Vec<u8> =
                            (1u8..=9).filter(|&d| combined & (1 << d) != 0).collect();

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

                        if elim.is_empty() {
                            continue;
                        }

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
    fn name_en(&self) -> &'static str {
        "X-Wing"
    }
    fn name_de(&self) -> &'static str {
        "X-Wing"
    }

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
                if row_cols[r1].len() != 2 {
                    continue;
                }
                for r2 in (r1 + 1)..9 {
                    if row_cols[r2] != row_cols[r1] {
                        continue;
                    }
                    let (c1, c2) = (row_cols[r1][0], row_cols[r1][1]);

                    let elim: Vec<(usize, usize)> = (0..9)
                        .filter(|&r| r != r1 && r != r2)
                        .flat_map(|r| [(r, c1), (r, c2)])
                        .filter(|&(r, c)| {
                            matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();

                    if elim.is_empty() {
                        continue;
                    }

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
                if col_rows[c1].len() != 2 {
                    continue;
                }
                for c2 in (c1 + 1)..9 {
                    if col_rows[c2] != col_rows[c1] {
                        continue;
                    }
                    let (r1, r2) = (col_rows[c1][0], col_rows[c1][1]);

                    let elim: Vec<(usize, usize)> = (0..9)
                        .filter(|&c| c != c1 && c != c2)
                        .flat_map(|c| [(r1, c), (r2, c)])
                        .filter(|&(r, c)| {
                            matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();

                    if elim.is_empty() {
                        continue;
                    }

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
    fn name_en(&self) -> &'static str {
        "Swordfish"
    }
    fn name_de(&self) -> &'static str {
        "Swordfish"
    }

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

                        let elim: Vec<(usize, usize)> = cols
                            .iter()
                            .flat_map(|&c| (0..9).map(move |r| (r, c)))
                            .filter(|&(r, c)| {
                                r != r1
                                    && r != r2
                                    && r != r3
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & (1 << digit)) != 0
                            })
                            .collect();

                        if elim.is_empty() {
                            continue;
                        }

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

                        let elim: Vec<(usize, usize)> = rows
                            .iter()
                            .flat_map(|&r| (0..9).map(move |c| (r, c)))
                            .filter(|&(r, c)| {
                                c != c1
                                    && c != c2
                                    && c != c3
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & (1 << digit)) != 0
                            })
                            .collect();

                        if elim.is_empty() {
                            continue;
                        }

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
    fn name_en(&self) -> &'static str {
        "Hidden Triples"
    }
    fn name_de(&self) -> &'static str {
        "Hidden Triples"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let empties: Vec<(usize, usize)> = unit
                .iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .copied()
                .collect();
            for d1 in 1u8..=9 {
                for d2 in (d1 + 1)..=9 {
                    for d3 in (d2 + 1)..=9 {
                        let triple_mask = (1u16 << d1) | (1u16 << d2) | (1u16 << d3);
                        // Cells that contain at least one of d1, d2, d3 in notes
                        let triple_cells: Vec<(usize, usize)> = empties
                            .iter()
                            .filter(|&&(r, c)| (state.notes_mask(r, c) & triple_mask) != 0)
                            .copied()
                            .collect();
                        if triple_cells.len() != 3 {
                            continue;
                        }
                        // All three digits must appear in the combined notes
                        let combined: u16 = triple_cells
                            .iter()
                            .fold(0u16, |acc, &(r, c)| acc | state.notes_mask(r, c));
                        if combined & triple_mask != triple_mask {
                            continue;
                        }
                        // At least one of the 3 cells must have extra notes to remove
                        let elim_cells: Vec<(usize, usize)> = triple_cells
                            .iter()
                            .filter(|&&(r, c)| state.notes_mask(r, c) & !triple_mask != 0)
                            .copied()
                            .collect();
                        if elim_cells.is_empty() {
                            continue;
                        }
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
    fn name_en(&self) -> &'static str {
        "Y-Wing"
    }
    fn name_de(&self) -> &'static str {
        "Y-Wing"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        // Collect all empty cells with exactly 2 notes.
        let bi_cells: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter(|&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
            .filter_map(|(r, c)| {
                let m = state.notes_mask(r, c);
                if m.count_ones() == 2 {
                    Some((r, c, m))
                } else {
                    None
                }
            })
            .collect();

        for &(r0, c0, m0) in &bi_cells {
            // Extract the two pivot digits a and b.
            let a = m0.trailing_zeros() as u8;
            let b = (m0 >> (a as u32 + 1)).trailing_zeros() as u8 + a + 1;

            for &(r1, c1, m1) in &bi_cells {
                if (r1, c1) == (r0, c0) {
                    continue;
                }
                if !sees(r0, c0, r1, c1) {
                    continue;
                }
                // Wing1 must share exactly 1 digit with pivot.
                let shared = m0 & m1;
                if shared.count_ones() != 1 {
                    continue;
                }
                let shared_ab = shared.trailing_zeros() as u8; // a or b
                let c_digit = (m1 & !shared).trailing_zeros() as u8; // elimination digit
                                                                     // Wing2 needs the other pivot digit and c_digit.
                let other_ab = if shared_ab == a { b } else { a };
                let needed_m2 = (1u16 << other_ab) | (1u16 << c_digit);

                for &(r2, c2, m2) in &bi_cells {
                    if (r2, c2) == (r0, c0) || (r2, c2) == (r1, c1) {
                        continue;
                    }
                    if !sees(r0, c0, r2, c2) {
                        continue;
                    }
                    if m2 != needed_m2 {
                        continue;
                    }

                    // Eliminate c_digit from cells seeing both wings.
                    let elim: Vec<(usize, usize)> = (0..9)
                        .flat_map(|r| (0..9).map(move |c| (r, c)))
                        .filter(|&(r, c)| {
                            (r, c) != (r0, c0)
                                && (r, c) != (r1, c1)
                                && (r, c) != (r2, c2)
                                && matches!(grid.get(r, c), CellKind::Empty)
                                && sees(r, c, r1, c1)
                                && sees(r, c, r2, c2)
                                && (state.notes_mask(r, c) & (1 << c_digit)) != 0
                        })
                        .collect();

                    if elim.is_empty() {
                        continue;
                    }

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
    fn name_en(&self) -> &'static str {
        "Unique Rectangle"
    }
    fn name_de(&self) -> &'static str {
        "Unique Rectangle"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        for r1 in 0..9usize {
            for r2 in (r1 + 1)..9 {
                for c1 in 0..9usize {
                    for c2 in (c1 + 1)..9 {
                        let corners = [(r1, c1), (r1, c2), (r2, c1), (r2, c2)];
                        // All corners must be empty.
                        if corners
                            .iter()
                            .any(|&(r, c)| !matches!(grid.get(r, c), CellKind::Empty))
                        {
                            continue;
                        }
                        // Must span exactly 2 different boxes.
                        let boxes: std::collections::HashSet<usize> =
                            corners.iter().map(|&(r, c)| (r / 3) * 3 + c / 3).collect();
                        if boxes.len() != 2 {
                            continue;
                        }

                        // Find pair {a,b}: both digits present in notes of all 4 corners.
                        for a in 1u8..=9 {
                            for b in (a + 1)..=9 {
                                let pair_mask = (1u16 << a) | (1u16 << b);
                                // All 4 corners must have both a and b in notes.
                                if corners
                                    .iter()
                                    .any(|&(r, c)| state.notes_mask(r, c) & pair_mask != pair_mask)
                                {
                                    continue;
                                }

                                // ── Type 1: 3 corners have ONLY {a,b}, 1 roof has extras ─────
                                let only_pair: Vec<(usize, usize)> = corners
                                    .iter()
                                    .filter(|&&(r, c)| state.notes_mask(r, c) == pair_mask)
                                    .copied()
                                    .collect();
                                let has_extras: Vec<(usize, usize)> = corners
                                    .iter()
                                    .filter(|&&(r, c)| state.notes_mask(r, c) & !pair_mask != 0)
                                    .copied()
                                    .collect();

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
                                if has_extras.len() != 2 || only_pair.len() != 2 {
                                    continue;
                                }
                                let (roof_a, roof_b) = (has_extras[0], has_extras[1]);
                                // Roofs must be on same row or same column.
                                let same_side = roof_a.0 == roof_b.0 || roof_a.1 == roof_b.1;
                                if !same_side {
                                    continue;
                                }
                                let extra_a = state.notes_mask(roof_a.0, roof_a.1) & !pair_mask;
                                let extra_b = state.notes_mask(roof_b.0, roof_b.1) & !pair_mask;
                                // Both roofs must have exactly 1 extra digit and it must match.
                                if extra_a.count_ones() != 1 || extra_a != extra_b {
                                    continue;
                                }
                                let c_digit = extra_a.trailing_zeros() as u8;

                                let elim: Vec<(usize, usize)> = (0..9)
                                    .flat_map(|r| (0..9).map(move |c| (r, c)))
                                    .filter(|&(r, c)| {
                                        !corners.contains(&(r, c))
                                            && matches!(grid.get(r, c), CellKind::Empty)
                                            && sees(r, c, roof_a.0, roof_a.1)
                                            && sees(r, c, roof_b.0, roof_b.1)
                                            && (state.notes_mask(r, c) & (1 << c_digit)) != 0
                                    })
                                    .collect();

                                if elim.is_empty() {
                                    continue;
                                }

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

// ── Naked Quads ───────────────────────────────────────────────────────────────

impl Strategy for NakedQuads {
    fn name_en(&self) -> &'static str {
        "Naked Quads"
    }
    fn name_de(&self) -> &'static str {
        "Naked Quads"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let small: Vec<(usize, usize, u16)> = unit
                .iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .filter_map(|&(r, c)| {
                    let m = state.notes_mask(r, c);
                    let n = m.count_ones();
                    if n >= 2 && n <= 4 {
                        Some((r, c, m))
                    } else {
                        None
                    }
                })
                .collect();

            for i in 0..small.len() {
                for j in (i + 1)..small.len() {
                    for k in (j + 1)..small.len() {
                        for l in (k + 1)..small.len() {
                            let combined = small[i].2 | small[j].2 | small[k].2 | small[l].2;
                            if combined.count_ones() != 4 {
                                continue;
                            }
                            let quad = [
                                (small[i].0, small[i].1),
                                (small[j].0, small[j].1),
                                (small[k].0, small[k].1),
                                (small[l].0, small[l].1),
                            ];
                            let digits: Vec<u8> =
                                (1u8..=9).filter(|&d| combined & (1 << d) != 0).collect();
                            let elim: Vec<(usize, usize)> = unit
                                .iter()
                                .filter(|&&(r, c)| {
                                    !quad.contains(&(r, c))
                                        && matches!(grid.get(r, c), CellKind::Empty)
                                        && (state.notes_mask(r, c) & combined) != 0
                                })
                                .copied()
                                .collect();
                            if elim.is_empty() {
                                continue;
                            }
                            let (d1, d2, d3, d4) = (digits[0], digits[1], digits[2], digits[3]);
                            return Some(Hint {
                                cause_cells:    quad.to_vec(),
                                elim_cells:     elim.clone(),
                                target_cell:    elim[0],
                                elim_digit:     Some(d1),
                                target_digit:   None,
                                name_en:        self.name_en(),
                                name_de:        self.name_de(),
                                explanation_en: format!(
                                    "These 4 cells hold only {}/{}/{}/{}. Remove those digits from notes in highlighted cells.",
                                    d1, d2, d3, d4
                                ),
                                explanation_de: format!(
                                    "Diese 4 Zellen enthalten nur {}/{}/{}/{}. Diese Ziffern aus den markierten Zellen streichen.",
                                    d1, d2, d3, d4
                                ),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

// ── Hidden Quads ──────────────────────────────────────────────────────────────

impl Strategy for HiddenQuads {
    fn name_en(&self) -> &'static str {
        "Hidden Quads"
    }
    fn name_de(&self) -> &'static str {
        "Hidden Quads"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let empties: Vec<(usize, usize)> = unit
                .iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .copied()
                .collect();
            for d1 in 1u8..=9 {
                for d2 in (d1 + 1)..=9 {
                    for d3 in (d2 + 1)..=9 {
                        for d4 in (d3 + 1)..=9 {
                            let mask = (1u16 << d1) | (1u16 << d2) | (1u16 << d3) | (1u16 << d4);
                            let quad_cells: Vec<(usize, usize)> = empties
                                .iter()
                                .filter(|&&(r, c)| (state.notes_mask(r, c) & mask) != 0)
                                .copied()
                                .collect();
                            if quad_cells.len() != 4 {
                                continue;
                            }
                            let combined: u16 = quad_cells
                                .iter()
                                .fold(0u16, |acc, &(r, c)| acc | state.notes_mask(r, c));
                            if (combined & mask) != mask {
                                continue;
                            }
                            let elim: Vec<(usize, usize)> = quad_cells
                                .iter()
                                .filter(|&&(r, c)| (state.notes_mask(r, c) & !mask) != 0)
                                .copied()
                                .collect();
                            if elim.is_empty() {
                                continue;
                            }
                            let target = elim[0];
                            return Some(Hint {
                                cause_cells:    quad_cells,
                                elim_cells:     elim,
                                target_cell:    target,
                                elim_digit:     Some(d1),
                                target_digit:   None,
                                name_en:        self.name_en(),
                                name_de:        self.name_de(),
                                explanation_en: format!(
                                    "Only these cells can hold {}/{}/{}/{}. Remove other notes from highlighted cells.",
                                    d1, d2, d3, d4
                                ),
                                explanation_de: format!(
                                    "Nur diese Zellen k\u{f6}nnen {}/{}/{}/{} halten. Andere Notizen aus den markierten Zellen streichen.",
                                    d1, d2, d3, d4
                                ),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

// ── Jellyfish ─────────────────────────────────────────────────────────────────

impl Strategy for Jellyfish {
    fn name_en(&self) -> &'static str {
        "Jellyfish"
    }
    fn name_de(&self) -> &'static str {
        "Jellyfish"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for digit in 1u8..=9 {
            // ── Row-based Jellyfish ───────────────────────────────────────────
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
                    n >= 2 && n <= 4
                })
                .collect();
            for i in 0..cand_rows.len() {
                for j in (i + 1)..cand_rows.len() {
                    for k in (j + 1)..cand_rows.len() {
                        for l in (k + 1)..cand_rows.len() {
                            let (r1, r2, r3, r4) =
                                (cand_rows[i], cand_rows[j], cand_rows[k], cand_rows[l]);
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
                            for &c in &row_cols[r4] {
                                cols.insert(c);
                            }
                            if cols.len() != 4 {
                                continue;
                            }
                            let cause: Vec<(usize, usize)> = [r1, r2, r3, r4]
                                .iter()
                                .flat_map(|&r| row_cols[r].iter().map(move |&c| (r, c)))
                                .collect();
                            let elim: Vec<(usize, usize)> = cols
                                .iter()
                                .flat_map(|&c| (0..9).map(move |r| (r, c)))
                                .filter(|&(r, c)| {
                                    r != r1
                                        && r != r2
                                        && r != r3
                                        && r != r4
                                        && matches!(grid.get(r, c), CellKind::Empty)
                                        && (state.notes_mask(r, c) & (1 << digit)) != 0
                                })
                                .collect();
                            if elim.is_empty() {
                                continue;
                            }
                            return Some(Hint {
                                cause_cells:    cause,
                                elim_cells:     elim.clone(),
                                target_cell:    elim[0],
                                elim_digit:     Some(digit),
                                target_digit:   None,
                                name_en:        self.name_en(),
                                name_de:        self.name_de(),
                                explanation_en: format!(
                                    "{} is locked in 4 rows across 4 columns. Remove {} from notes in highlighted cells.",
                                    digit, digit
                                ),
                                explanation_de: format!(
                                    "{} ist in 4 Zeilen auf 4 Spalten eingeschr\u{e4}nkt. {} aus den markierten Zellen streichen.",
                                    digit, digit
                                ),
                            });
                        }
                    }
                }
            }

            // ── Column-based Jellyfish ────────────────────────────────────────
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
                    n >= 2 && n <= 4
                })
                .collect();
            for i in 0..cand_cols.len() {
                for j in (i + 1)..cand_cols.len() {
                    for k in (j + 1)..cand_cols.len() {
                        for l in (k + 1)..cand_cols.len() {
                            let (c1, c2, c3, c4) =
                                (cand_cols[i], cand_cols[j], cand_cols[k], cand_cols[l]);
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
                            for &r in &col_rows[c4] {
                                rows.insert(r);
                            }
                            if rows.len() != 4 {
                                continue;
                            }
                            let cause: Vec<(usize, usize)> = [c1, c2, c3, c4]
                                .iter()
                                .flat_map(|&c| col_rows[c].iter().map(move |&r| (r, c)))
                                .collect();
                            let elim: Vec<(usize, usize)> = rows
                                .iter()
                                .flat_map(|&r| (0..9).map(move |c| (r, c)))
                                .filter(|&(r, c)| {
                                    c != c1
                                        && c != c2
                                        && c != c3
                                        && c != c4
                                        && matches!(grid.get(r, c), CellKind::Empty)
                                        && (state.notes_mask(r, c) & (1 << digit)) != 0
                                })
                                .collect();
                            if elim.is_empty() {
                                continue;
                            }
                            return Some(Hint {
                                cause_cells:    cause,
                                elim_cells:     elim.clone(),
                                target_cell:    elim[0],
                                elim_digit:     Some(digit),
                                target_digit:   None,
                                name_en:        self.name_en(),
                                name_de:        self.name_de(),
                                explanation_en: format!(
                                    "{} is locked in 4 columns across 4 rows. Remove {} from notes in highlighted cells.",
                                    digit, digit
                                ),
                                explanation_de: format!(
                                    "{} ist in 4 Spalten auf 4 Zeilen eingeschr\u{e4}nkt. {} aus den markierten Zellen streichen.",
                                    digit, digit
                                ),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

// ── Skyscraper ────────────────────────────────────────────────────────────────

impl Strategy for Skyscraper {
    fn name_en(&self) -> &'static str {
        "Skyscraper"
    }
    fn name_de(&self) -> &'static str {
        "Skyscraper"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for digit in 1u8..=9 {
            // ── Row-based Skyscraper ──────────────────────────────────────────
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
                if row_cols[r1].len() != 2 {
                    continue;
                }
                for r2 in (r1 + 1)..9 {
                    if row_cols[r2].len() != 2 {
                        continue;
                    }
                    let shared: Vec<usize> = row_cols[r1]
                        .iter()
                        .filter(|c| row_cols[r2].contains(c))
                        .copied()
                        .collect();
                    if shared.len() != 1 {
                        continue;
                    }
                    let c_shared = shared[0];
                    let ca = *row_cols[r1].iter().find(|&&c| c != c_shared).unwrap();
                    let cb = *row_cols[r2].iter().find(|&&c| c != c_shared).unwrap();
                    let cause = vec![(r1, c_shared), (r1, ca), (r2, c_shared), (r2, cb)];
                    let elim: Vec<(usize, usize)> = (0..9)
                        .flat_map(|r| (0..9).map(move |c| (r, c)))
                        .filter(|&(r, c)| {
                            (r, c) != (r1, ca)
                                && (r, c) != (r2, cb)
                                && sees(r, c, r1, ca)
                                && sees(r, c, r2, cb)
                                && matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();
                    if elim.is_empty() {
                        continue;
                    }
                    return Some(Hint {
                        cause_cells:    cause,
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "Skyscraper on {}: the two tips force out {}. Remove {} from notes in highlighted cells.",
                            digit, digit, digit
                        ),
                        explanation_de: format!(
                            "Skyscraper auf {}: die zwei Spitzen schließen {} aus. {} aus den markierten Zellen streichen.",
                            digit, digit, digit
                        ),
                    });
                }
            }

            // ── Column-based Skyscraper ───────────────────────────────────────
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
                if col_rows[c1].len() != 2 {
                    continue;
                }
                for c2 in (c1 + 1)..9 {
                    if col_rows[c2].len() != 2 {
                        continue;
                    }
                    let shared: Vec<usize> = col_rows[c1]
                        .iter()
                        .filter(|r| col_rows[c2].contains(r))
                        .copied()
                        .collect();
                    if shared.len() != 1 {
                        continue;
                    }
                    let r_shared = shared[0];
                    let ra = *col_rows[c1].iter().find(|&&r| r != r_shared).unwrap();
                    let rb = *col_rows[c2].iter().find(|&&r| r != r_shared).unwrap();
                    let cause = vec![(r_shared, c1), (ra, c1), (r_shared, c2), (rb, c2)];
                    let elim: Vec<(usize, usize)> = (0..9)
                        .flat_map(|r| (0..9).map(move |c| (r, c)))
                        .filter(|&(r, c)| {
                            (r, c) != (ra, c1)
                                && (r, c) != (rb, c2)
                                && sees(r, c, ra, c1)
                                && sees(r, c, rb, c2)
                                && matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();
                    if elim.is_empty() {
                        continue;
                    }
                    return Some(Hint {
                        cause_cells:    cause,
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "Skyscraper on {}: the two tips force out {}. Remove {} from notes in highlighted cells.",
                            digit, digit, digit
                        ),
                        explanation_de: format!(
                            "Skyscraper auf {}: die zwei Spitzen schließen {} aus. {} aus den markierten Zellen streichen.",
                            digit, digit, digit
                        ),
                    });
                }
            }
        }
        None
    }
}

// ── 2-String Kite ─────────────────────────────────────────────────────────────

impl Strategy for TwoStringKite {
    fn name_en(&self) -> &'static str {
        "2-String Kite"
    }
    fn name_de(&self) -> &'static str {
        "2-String Kite"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for digit in 1u8..=9 {
            for row in 0..9usize {
                let row_cells: Vec<usize> = (0..9)
                    .filter(|&c| {
                        matches!(grid.get(row, c), CellKind::Empty)
                            && (state.notes_mask(row, c) & (1 << digit)) != 0
                    })
                    .collect();
                if row_cells.len() != 2 {
                    continue;
                }
                let (rc1, rc2) = (row_cells[0], row_cells[1]);

                for col in 0..9usize {
                    let col_cells: Vec<usize> = (0..9)
                        .filter(|&r| {
                            matches!(grid.get(r, col), CellKind::Empty)
                                && (state.notes_mask(r, col) & (1 << digit)) != 0
                        })
                        .collect();
                    if col_cells.len() != 2 {
                        continue;
                    }
                    let (cr1, cr2) = (col_cells[0], col_cells[1]);

                    let row_pair = [(row, rc1), (row, rc2)];
                    let col_pair = [(cr1, col), (cr2, col)];

                    for &(r_int, c_int) in &row_pair {
                        for &(r_col_int, c_col_int) in &col_pair {
                            if (r_int, c_int) == (r_col_int, c_col_int) {
                                continue;
                            }
                            if r_int / 3 != r_col_int / 3 || c_int / 3 != c_col_int / 3 {
                                continue;
                            }
                            let tip1 = *row_pair.iter().find(|&&rc| rc != (r_int, c_int)).unwrap();
                            let tip2 = *col_pair
                                .iter()
                                .find(|&&rc| rc != (r_col_int, c_col_int))
                                .unwrap();
                            let cause = vec![(r_int, c_int), (r_col_int, c_col_int), tip1, tip2];
                            let elim: Vec<(usize, usize)> = (0..9)
                                .flat_map(|r| (0..9).map(move |c| (r, c)))
                                .filter(|&(r, c)| {
                                    (r, c) != tip1
                                        && (r, c) != tip2
                                        && sees(r, c, tip1.0, tip1.1)
                                        && sees(r, c, tip2.0, tip2.1)
                                        && matches!(grid.get(r, c), CellKind::Empty)
                                        && (state.notes_mask(r, c) & (1 << digit)) != 0
                                })
                                .collect();
                            if elim.is_empty() {
                                continue;
                            }
                            return Some(Hint {
                                cause_cells:    cause,
                                elim_cells:     elim.clone(),
                                target_cell:    elim[0],
                                elim_digit:     Some(digit),
                                target_digit:   None,
                                name_en:        self.name_en(),
                                name_de:        self.name_de(),
                                explanation_en: format!(
                                    "2-String Kite on {}: a row and column share a box, forcing {} out. Remove {} from notes in highlighted cells.",
                                    digit, digit, digit
                                ),
                                explanation_de: format!(
                                    "2-String Kite auf {}: Zeile und Spalte teilen eine Box und schließen {} aus. {} aus den markierten Zellen streichen.",
                                    digit, digit, digit
                                ),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

// ── XYZ-Wing ──────────────────────────────────────────────────────────────────

impl Strategy for XYZWing {
    fn name_en(&self) -> &'static str {
        "XYZ-Wing"
    }
    fn name_de(&self) -> &'static str {
        "XYZ-Wing"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        let bivalue: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter_map(|(r, c)| {
                if !matches!(grid.get(r, c), CellKind::Empty) {
                    return None;
                }
                let m = state.notes_mask(r, c);
                if m.count_ones() == 2 {
                    Some((r, c, m))
                } else {
                    None
                }
            })
            .collect();

        let trivalue: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter_map(|(r, c)| {
                if !matches!(grid.get(r, c), CellKind::Empty) {
                    return None;
                }
                let m = state.notes_mask(r, c);
                if m.count_ones() == 3 {
                    Some((r, c, m))
                } else {
                    None
                }
            })
            .collect();

        for &(pr, pc, pm) in &trivalue {
            let digits: Vec<u8> = (1u8..=9).filter(|&d| pm & (1 << d) != 0).collect();
            if digits.len() != 3 {
                continue;
            }
            let (da, db, dc) = (digits[0], digits[1], digits[2]);

            for &c_digit in &[da, db, dc] {
                let others: Vec<u8> = [da, db, dc]
                    .iter()
                    .filter(|&&d| d != c_digit)
                    .copied()
                    .collect();
                let wing_masks = [
                    (1u16 << others[0]) | (1u16 << c_digit),
                    (1u16 << others[1]) | (1u16 << c_digit),
                ];

                let wings_for: [Vec<(usize, usize)>; 2] = [
                    bivalue
                        .iter()
                        .filter(|&&(r, c, m)| {
                            (r, c) != (pr, pc) && sees(r, c, pr, pc) && m == wing_masks[0]
                        })
                        .map(|&(r, c, _)| (r, c))
                        .collect(),
                    bivalue
                        .iter()
                        .filter(|&&(r, c, m)| {
                            (r, c) != (pr, pc) && sees(r, c, pr, pc) && m == wing_masks[1]
                        })
                        .map(|&(r, c, _)| (r, c))
                        .collect(),
                ];

                for &(w1r, w1c) in &wings_for[0] {
                    for &(w2r, w2c) in &wings_for[1] {
                        if (w1r, w1c) == (w2r, w2c) {
                            continue;
                        }
                        let elim: Vec<(usize, usize)> = (0..9)
                            .flat_map(|r| (0..9).map(move |c| (r, c)))
                            .filter(|&(r, c)| {
                                (r, c) != (pr, pc)
                                    && (r, c) != (w1r, w1c)
                                    && (r, c) != (w2r, w2c)
                                    && sees(r, c, pr, pc)
                                    && sees(r, c, w1r, w1c)
                                    && sees(r, c, w2r, w2c)
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & (1 << c_digit)) != 0
                            })
                            .collect();
                        if elim.is_empty() {
                            continue;
                        }
                        return Some(Hint {
                            cause_cells:    vec![(pr, pc), (w1r, w1c), (w2r, w2c)],
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(c_digit),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "XYZ-Wing: pivot has 3 candidates, all three share {}. Remove {} from notes in highlighted cells.",
                                c_digit, c_digit
                            ),
                            explanation_de: format!(
                                "XYZ-Wing: Pivot hat 3 Kandidaten, alle drei teilen {}. {} aus den markierten Zellen streichen.",
                                c_digit, c_digit
                            ),
                        });
                    }
                }
            }
        }
        None
    }
}

// ── W-Wing ────────────────────────────────────────────────────────────────────

impl Strategy for WWing {
    fn name_en(&self) -> &'static str {
        "W-Wing"
    }
    fn name_de(&self) -> &'static str {
        "W-Wing"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        let bivalue: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter_map(|(r, c)| {
                if !matches!(grid.get(r, c), CellKind::Empty) {
                    return None;
                }
                let m = state.notes_mask(r, c);
                if m.count_ones() == 2 {
                    Some((r, c, m))
                } else {
                    None
                }
            })
            .collect();

        // Iterate over all possible pair masks {a, b}
        for pair_mask in (1u16..512).filter(|m| m.count_ones() == 2) {
            let a = pair_mask.trailing_zeros() as u8;
            let b = (pair_mask >> (a as u32 + 1)).trailing_zeros() as u8 + a + 1;

            let pairs: Vec<(usize, usize)> = bivalue
                .iter()
                .filter(|&&(_, _, m)| m == pair_mask)
                .map(|&(r, c, _)| (r, c))
                .collect();
            if pairs.len() < 2 {
                continue;
            }

            // For each unit with digit `a` in exactly 2 cells → strong link
            for unit in all_units() {
                let unit_a: Vec<(usize, usize)> = unit
                    .iter()
                    .filter(|&&(r, c)| {
                        matches!(grid.get(r, c), CellKind::Empty)
                            && (state.notes_mask(r, c) & (1 << a)) != 0
                    })
                    .copied()
                    .collect();
                if unit_a.len() != 2 {
                    continue;
                }
                let (e1, e2) = (unit_a[0], unit_a[1]);

                for &p1 in &pairs {
                    if p1 == e1 {
                        continue;
                    }
                    if !sees(p1.0, p1.1, e1.0, e1.1) {
                        continue;
                    }
                    for &p2 in &pairs {
                        if p2 == p1 || p2 == e2 {
                            continue;
                        }
                        if !sees(p2.0, p2.1, e2.0, e2.1) {
                            continue;
                        }
                        if sees(p1.0, p1.1, p2.0, p2.1) {
                            continue;
                        }
                        let elim: Vec<(usize, usize)> = (0..9)
                            .flat_map(|r| (0..9).map(move |c| (r, c)))
                            .filter(|&(r, c)| {
                                (r, c) != p1
                                    && (r, c) != p2
                                    && sees(r, c, p1.0, p1.1)
                                    && sees(r, c, p2.0, p2.1)
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & (1 << b)) != 0
                            })
                            .collect();
                        if elim.is_empty() {
                            continue;
                        }
                        return Some(Hint {
                            cause_cells:    vec![p1, e1, e2, p2],
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(b),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "W-Wing: two {}/{} cells linked by strong link on {}. Remove {} from notes in highlighted cells.",
                                a, b, a, b
                            ),
                            explanation_de: format!(
                                "W-Wing: Zwei {}/{}-Zellen verbunden durch starke Verbindung auf {}. {} aus den markierten Zellen streichen.",
                                a, b, a, b
                            ),
                        });
                    }
                }
            }
        }
        None
    }
}

// ── BUG+1 ─────────────────────────────────────────────────────────────────────

impl Strategy for BugPlusOne {
    fn name_en(&self) -> &'static str {
        "BUG+1"
    }
    fn name_de(&self) -> &'static str {
        "BUG+1"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        let mut trivalue_cell: Option<(usize, usize)> = None;
        for r in 0..9 {
            for c in 0..9 {
                if !matches!(grid.get(r, c), CellKind::Empty) {
                    continue;
                }
                let n = state.notes_mask(r, c).count_ones();
                if n == 0 {
                    return None;
                }
                if n == 3 {
                    if trivalue_cell.is_some() {
                        return None;
                    }
                    trivalue_cell = Some((r, c));
                } else if n != 2 {
                    return None;
                }
            }
        }
        let (br, bc) = trivalue_cell?;
        let mask = state.notes_mask(br, bc);
        let digits: Vec<u8> = (1u8..=9).filter(|&d| mask & (1 << d) != 0).collect();
        if digits.len() != 3 {
            return None;
        }

        for &d in &digits {
            let row_count = (0..9)
                .filter(|&c| {
                    c != bc
                        && matches!(grid.get(br, c), CellKind::Empty)
                        && (state.notes_mask(br, c) & (1 << d)) != 0
                })
                .count();
            let col_count = (0..9)
                .filter(|&r| {
                    r != br
                        && matches!(grid.get(r, bc), CellKind::Empty)
                        && (state.notes_mask(r, bc) & (1 << d)) != 0
                })
                .count();
            let box_br = (br / 3) * 3;
            let box_bc = (bc / 3) * 3;
            let box_count = (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (box_br + dr, box_bc + dc)))
                .filter(|&(r, c)| {
                    (r, c) != (br, bc)
                        && matches!(grid.get(r, c), CellKind::Empty)
                        && (state.notes_mask(r, c) & (1 << d)) != 0
                })
                .count();

            if row_count % 2 == 1 && col_count % 2 == 1 && box_count % 2 == 1 {
                return Some(Hint {
                    cause_cells:    vec![],
                    elim_cells:     vec![],
                    target_cell:    (br, bc),
                    elim_digit:     None,
                    target_digit:   Some(d),
                    name_en:        self.name_en(),
                    name_de:        self.name_de(),
                    explanation_en: format!(
                        "BUG+1: all other cells are bivalue. Place {} here to restore balance everywhere.",
                        d
                    ),
                    explanation_de: format!(
                        "BUG+1: Alle anderen Zellen sind bivalue. {} hier setzen stellt die Balance \u{fc}berall wieder her.",
                        d
                    ),
                });
            }
        }
        None
    }
}

// ── Empty Rectangle ───────────────────────────────────────────────────────────

impl Strategy for EmptyRectangle {
    fn name_en(&self) -> &'static str {
        "Empty Rectangle"
    }
    fn name_de(&self) -> &'static str {
        "Empty Rectangle"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        for digit in 1u8..=9 {
            // Iterate all 9 boxes as possible ER boxes
            for box_idx in 0..9usize {
                let box_row = (box_idx / 3) * 3; // first row of box
                let box_col = (box_idx % 3) * 3; // first col of box

                // Collect cells in this box that have `digit` in notes
                let box_cells: Vec<(usize, usize)> = (0..3)
                    .flat_map(|dr| (0..3).map(move |dc| (box_row + dr, box_col + dc)))
                    .filter(|&(r, c)| {
                        matches!(grid.get(r, c), CellKind::Empty)
                            && (state.notes_mask(r, c) & (1 << digit)) != 0
                    })
                    .collect();

                // Need at least 2 cells (a single cell is a naked single, handled earlier)
                if box_cells.len() < 2 {
                    continue;
                }

                // ── ER on a row: all box cells with digit are in the same row ──
                let er_row = box_cells[0].0;
                if box_cells.iter().all(|&(r, _)| r == er_row) {
                    // Find a conjugate pair on `digit` in some column outside box B
                    for c_conj in 0..9usize {
                        if c_conj / 3 == box_col / 3 {
                            continue;
                        } // skip cols in box's band
                        let col_cells: Vec<usize> = (0..9)
                            .filter(|&r| {
                                matches!(grid.get(r, c_conj), CellKind::Empty)
                                    && (state.notes_mask(r, c_conj) & (1 << digit)) != 0
                            })
                            .collect();
                        if col_cells.len() != 2 {
                            continue;
                        }
                        let (r_a, r_b) = (col_cells[0], col_cells[1]);

                        // Check both orientations
                        let (r_er_end, r_other) = if r_a == er_row {
                            (r_a, r_b)
                        } else if r_b == er_row {
                            (r_b, r_a)
                        } else {
                            continue;
                        };
                        let _ = r_er_end;

                        // Eliminate digit from cells in row r_other that are in box B's columns
                        for c_er in box_col..(box_col + 3) {
                            if c_er == c_conj {
                                continue;
                            }
                            if !matches!(grid.get(r_other, c_er), CellKind::Empty) {
                                continue;
                            }
                            if (state.notes_mask(r_other, c_er) & (1 << digit)) == 0 {
                                continue;
                            }

                            let mut cause = box_cells.clone();
                            cause.push((r_a, c_conj));
                            cause.push((r_b, c_conj));
                            let elim = vec![(r_other, c_er)];
                            return Some(Hint {
                                cause_cells:    cause,
                                elim_cells:     elim.clone(),
                                target_cell:    elim[0],
                                elim_digit:     Some(digit),
                                target_digit:   None,
                                name_en:        self.name_en(),
                                name_de:        self.name_de(),
                                explanation_en: format!(
                                    "Empty Rectangle: {} in the box is confined to one row. Remove {} from notes in highlighted cells.",
                                    digit, digit
                                ),
                                explanation_de: format!(
                                    "Empty Rectangle: {} in der Box ist auf eine Zeile beschr\u{e4}nkt. {} aus den markierten Zellen streichen.",
                                    digit, digit
                                ),
                            });
                        }
                    }
                }

                // ── ER on a column: all box cells with digit are in the same col ──
                let er_col = box_cells[0].1;
                if box_cells.iter().all(|&(_, c)| c == er_col) {
                    // Find a conjugate pair on `digit` in some row outside box B
                    for r_conj in 0..9usize {
                        if r_conj / 3 == box_row / 3 {
                            continue;
                        } // skip rows in box's band
                        let row_cells: Vec<usize> = (0..9)
                            .filter(|&c| {
                                matches!(grid.get(r_conj, c), CellKind::Empty)
                                    && (state.notes_mask(r_conj, c) & (1 << digit)) != 0
                            })
                            .collect();
                        if row_cells.len() != 2 {
                            continue;
                        }
                        let (c_a, c_b) = (row_cells[0], row_cells[1]);

                        let (c_er_end, c_other) = if c_a == er_col {
                            (c_a, c_b)
                        } else if c_b == er_col {
                            (c_b, c_a)
                        } else {
                            continue;
                        };
                        let _ = c_er_end;

                        // Eliminate digit from cells in col c_other that are in box B's rows
                        for r_er in box_row..(box_row + 3) {
                            if r_er == r_conj {
                                continue;
                            }
                            if !matches!(grid.get(r_er, c_other), CellKind::Empty) {
                                continue;
                            }
                            if (state.notes_mask(r_er, c_other) & (1 << digit)) == 0 {
                                continue;
                            }

                            let mut cause = box_cells.clone();
                            cause.push((r_conj, c_a));
                            cause.push((r_conj, c_b));
                            let elim = vec![(r_er, c_other)];
                            return Some(Hint {
                                cause_cells:    cause,
                                elim_cells:     elim.clone(),
                                target_cell:    elim[0],
                                elim_digit:     Some(digit),
                                target_digit:   None,
                                name_en:        self.name_en(),
                                name_de:        self.name_de(),
                                explanation_en: format!(
                                    "Empty Rectangle: {} in the box is confined to one column. Remove {} from notes in highlighted cells.",
                                    digit, digit
                                ),
                                explanation_de: format!(
                                    "Empty Rectangle: {} in der Box ist auf eine Spalte beschr\u{e4}nkt. {} aus den markierten Zellen streichen.",
                                    digit, digit
                                ),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

// ── Simple Coloring ───────────────────────────────────────────────────────────

impl Strategy for SimpleColoring {
    fn name_en(&self) -> &'static str {
        "Simple Coloring"
    }
    fn name_de(&self) -> &'static str {
        "Simple Coloring"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        use std::collections::HashMap;

        let grid = state.grid();

        for digit in 1u8..=9 {
            // Build strong-link adjacency list
            let mut links: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();
            for unit in all_units() {
                let cells_d: Vec<(usize, usize)> = unit
                    .iter()
                    .filter(|&&(r, c)| {
                        matches!(grid.get(r, c), CellKind::Empty)
                            && (state.notes_mask(r, c) & (1 << digit)) != 0
                    })
                    .copied()
                    .collect();
                if cells_d.len() == 2 {
                    links.entry(cells_d[0]).or_default().push(cells_d[1]);
                    links.entry(cells_d[1]).or_default().push(cells_d[0]);
                }
            }

            if links.is_empty() {
                continue;
            }

            // BFS to find connected components and 2-color them
            let mut color_map: HashMap<(usize, usize), u8> = HashMap::new();

            let all_linked_cells: Vec<(usize, usize)> = links.keys().copied().collect();

            for &start in &all_linked_cells {
                if color_map.contains_key(&start) {
                    continue;
                }

                // BFS
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

                // Separate component into two colors
                let color0: Vec<(usize, usize)> = component
                    .iter()
                    .filter(|&&c| color_map[&c] == 0)
                    .copied()
                    .collect();
                let color1: Vec<(usize, usize)> = component
                    .iter()
                    .filter(|&&c| color_map[&c] == 1)
                    .copied()
                    .collect();

                // Color Wrap: same-color cells see each other → that color is impossible
                for col_idx in 0..2u8 {
                    let same_color = if col_idx == 0 { &color0 } else { &color1 };
                    'outer: for i in 0..same_color.len() {
                        for j in (i + 1)..same_color.len() {
                            let (r1, c1) = same_color[i];
                            let (r2, c2) = same_color[j];
                            if sees(r1, c1, r2, c2) {
                                // Eliminate digit from all cells of this color
                                let elim: Vec<(usize, usize)> = same_color
                                    .iter()
                                    .filter(|&&(r, c)| {
                                        matches!(grid.get(r, c), CellKind::Empty)
                                            && (state.notes_mask(r, c) & (1 << digit)) != 0
                                    })
                                    .copied()
                                    .collect();
                                if elim.is_empty() {
                                    break 'outer;
                                }
                                return Some(Hint {
                                    cause_cells:    same_color.clone(),
                                    elim_cells:     elim.clone(),
                                    target_cell:    elim[0],
                                    elim_digit:     Some(digit),
                                    target_digit:   None,
                                    name_en:        self.name_en(),
                                    name_de:        self.name_de(),
                                    explanation_en: format!(
                                        "Two same-color cells see each other \u{2014} that color is impossible. Remove {} from notes in highlighted cells.",
                                        digit
                                    ),
                                    explanation_de: format!(
                                        "Zwei gleichfarbige Zellen sehen sich \u{2014} diese Farbe ist unm\u{f6}glich. {} aus den markierten Zellen streichen.",
                                        digit
                                    ),
                                });
                            }
                        }
                    }
                }

                // Color Trap: a cell outside the component sees both colors
                for r in 0..9usize {
                    for c in 0..9usize {
                        if color_map.contains_key(&(r, c)) {
                            continue;
                        }
                        if !matches!(grid.get(r, c), CellKind::Empty) {
                            continue;
                        }
                        if (state.notes_mask(r, c) & (1 << digit)) == 0 {
                            continue;
                        }

                        let seen0 = color0.iter().find(|&&(r2, c2)| sees(r, c, r2, c2)).copied();
                        let seen1 = color1.iter().find(|&&(r2, c2)| sees(r, c, r2, c2)).copied();

                        if let (Some(s0), Some(s1)) = (seen0, seen1) {
                            return Some(Hint {
                                cause_cells:    vec![s0, s1],
                                elim_cells:     vec![(r, c)],
                                target_cell:    (r, c),
                                elim_digit:     Some(digit),
                                target_digit:   None,
                                name_en:        self.name_en(),
                                name_de:        self.name_de(),
                                explanation_en: format!(
                                    "This cell sees both colors of the {} chain. Remove {} from notes in highlighted cells.",
                                    digit, digit
                                ),
                                explanation_de: format!(
                                    "Diese Zelle sieht beide Farben der {}-Kette. {} aus den markierten Zellen streichen.",
                                    digit, digit
                                ),
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

// ── XY-Chain ──────────────────────────────────────────────────────────────────

impl Strategy for XYChain {
    fn name_en(&self) -> &'static str {
        "XY-Chain"
    }
    fn name_de(&self) -> &'static str {
        "XY-Chain"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        // Collect all bivalue cells
        let bivalue: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter_map(|(r, c)| {
                if !matches!(grid.get(r, c), CellKind::Empty) {
                    return None;
                }
                let m = state.notes_mask(r, c);
                if m.count_ones() == 2 {
                    Some((r, c, m))
                } else {
                    None
                }
            })
            .collect();

        const MAX_DEPTH: usize = 8;

        // Try each bivalue cell as chain start
        for &(sr, sc, sm) in &bivalue {
            // Extract the two digits: elim_d (first) and x (second link digit)
            let elim_d = sm.trailing_zeros() as u8;
            let x = (sm >> (elim_d as u32 + 1)).trailing_zeros() as u8 + elim_d + 1;

            // DFS: chain = list of cells, incoming = digit the next cell must contain
            // We start chain with (sr, sc), need next cell to contain `x`
            let mut chain: Vec<(usize, usize)> = vec![(sr, sc)];
            if let Some(h) = xy_chain_dfs(
                &mut chain,
                x,
                elim_d,
                &bivalue,
                grid,
                state,
                MAX_DEPTH,
                self.name_en(),
                self.name_de(),
            ) {
                return Some(h);
            }
        }
        None
    }
}

fn xy_chain_dfs(
    chain: &mut Vec<(usize, usize)>,
    incoming: u8,
    elim_d: u8,
    bivalue: &[(usize, usize, u16)],
    grid: &crate::puzzle::Grid,
    state: &GameState,
    max_depth: usize,
    name_en: &'static str,
    name_de: &'static str,
) -> Option<Hint> {
    if chain.len() >= max_depth {
        return None;
    }

    let &(cur_r, cur_c) = chain.last().unwrap();

    for &(nr, nc, nm) in bivalue {
        if chain.contains(&(nr, nc)) {
            continue;
        }
        if !sees(cur_r, cur_c, nr, nc) {
            continue;
        }
        // The new cell must contain `incoming`
        if (nm & (1 << incoming)) == 0 {
            continue;
        }

        // The "outgoing" digit of this cell (the other one)
        let other = if nm.trailing_zeros() as u8 == incoming {
            (nm >> (incoming as u32 + 1)).trailing_zeros() as u8 + incoming + 1
        } else {
            nm.trailing_zeros() as u8
        };

        chain.push((nr, nc));

        // If the outgoing digit equals elim_d AND chain length >= 3 (so at least 3 cells)
        if other == elim_d && chain.len() >= 3 {
            let (start_r, start_c) = chain[0];
            // Find cells that see both chain start and chain end and have elim_d in notes
            let elim: Vec<(usize, usize)> = (0..9)
                .flat_map(|r| (0..9).map(move |c| (r, c)))
                .filter(|&(r, c)| {
                    !chain.contains(&(r, c))
                        && matches!(grid.get(r, c), CellKind::Empty)
                        && sees(r, c, start_r, start_c)
                        && sees(r, c, nr, nc)
                        && (state.notes_mask(r, c) & (1 << elim_d)) != 0
                })
                .collect();

            if !elim.is_empty() {
                let hint = Hint {
                    cause_cells:    chain.clone(),
                    elim_cells:     elim.clone(),
                    target_cell:    elim[0],
                    elim_digit:     Some(elim_d),
                    target_digit:   None,
                    name_en,
                    name_de,
                    explanation_en: format!(
                        "XY-Chain: {} is forced out of one chain end. Remove {} from notes in highlighted cells.",
                        elim_d, elim_d
                    ),
                    explanation_de: format!(
                        "XY-Chain: {} wird aus einem Kettenende herausgezwungen. {} aus den markierten Zellen streichen.",
                        elim_d, elim_d
                    ),
                };
                chain.pop();
                return Some(hint);
            }
        }

        // Continue DFS with outgoing digit as new incoming (only if outgoing != elim_d,
        // to avoid revisiting a completed chain without eliminations)
        if other != elim_d {
            if let Some(h) = xy_chain_dfs(
                chain, other, elim_d, bivalue, grid, state, max_depth, name_en, name_de,
            ) {
                chain.pop();
                return Some(h);
            }
        }

        chain.pop();
    }
    None
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hint::Strategy;
    use crate::puzzle::{GameState, Grid};

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
    fn naked_triples_with_one_candidate_cell() {
        // Regression: a triple member with exactly 1 note was previously excluded
        // from the candidate list, causing valid triples to be missed.
        //
        // Construct row 0 so that cells (0,6), (0,7), (0,8) are empty and hold:
        //   (0,6): {1,2}   (0,7): {2,3}   (0,8): {3}
        // Union = {1,2,3} — a valid naked triple even though (0,8) has only 1 note.
        // Cells (0,0)–(0,5) are filled to simplify (no peer eliminations needed
        // there); add a 4th empty cell in the row, (0,5), with note {1} to give
        // the strategy an elimination target.
        //
        // Puzzle: first row = 4,5,6,7,8,_,_,_,_  (col 5-8 empty)
        //                       givens: 4 5 6 7 8 0 0 0 0
        // Fill remaining rows to avoid conflicts (rows 1-8 irrelevant, just need
        // a valid grid structure; use the standard test puzzle for rows 1-8).
        // Simplest: use an all-zero puzzle (no givens) and manually set notes.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            // Row 0 has cols 0-4 filled with 4,5,6,7,8; cols 5-8 empty.
            // Rows 1-8: use zeros (all empty) — we only test row 0.
            "456780000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap(); // solution content irrelevant here

        // (0,5): note {1} — an elimination target for the triple
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 1 });
        // (0,6): notes {1,2}
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 2 });
        // (0,7): notes {2,3}
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 3 });
        // (0,8): note {3}  ← this cell has exactly 1 note; was previously excluded
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 3 });

        // The triple {(0,6),(0,7),(0,8)} covers {1,2,3}.
        // Cell (0,5) has note {1} which must be eliminated.
        // Both (0,5)+(0,6)+(0,7) and (0,6)+(0,7)+(0,8) are valid naked triples;
        // the strategy may return either one.  The key assertion is that a triple
        // IS found — previously this returned None because 1-note cells were
        // excluded from the candidate list.
        assert!(
            NakedTriples.find(&state, &sol).is_some(),
            "NakedTriples should detect a triple where one member has exactly 1 note"
        );
    }

    #[test]
    fn hidden_triples_finds_triple_and_eliminates() {
        // Row 0: cols 0-5 filled with 4,5,6,7,8,9; cols 6,7,8 empty.
        // Digits 1,2,3 are confined to exactly those 3 empty cells.
        // (0,6): notes {1,2} + extra note 5  → elim target
        // (0,7): notes {2,3} + extra note 6  → elim target
        // (0,8): notes {1,3}                 → no extras
        // HiddenTriples must find d1/d2/d3 = 1/2/3 and report elim_cells.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "456789000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // (0,6): {1,2,5} — extra 5 must be eliminated
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 5 });
        // (0,7): {2,3,6} — extra 6 must be eliminated
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 6 });
        // (0,8): {1,3} — no extras; included in triple
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 3 });

        let hint = HiddenTriples
            .find(&state, &sol)
            .expect("HiddenTriples should detect triple {1,2,3} in row 0");
        assert_eq!(hint.name_en, "Hidden Triples");
        // Both (0,6) and (0,7) have extra notes and must appear as elim targets.
        assert!(
            hint.elim_cells.contains(&(0, 6)),
            "elim_cells should include (0,6) which has extra note 5"
        );
        assert!(
            hint.elim_cells.contains(&(0, 7)),
            "elim_cells should include (0,7) which has extra note 6"
        );
    }

    #[test]
    fn naked_quads_finds_quad_and_eliminates() {
        // Row 0: cols 0-3 filled with 5,6,7,8; cols 4-8 empty.
        // The 4 quad cells (0,4)–(0,7) together hold only {1,2,3,4}.
        // Cell (0,8) holds {1,3} — overlaps the quad, so its notes are eliminated.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "567800000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // Quad cells — union = {1,2,3,4}
        state.apply(GameEvent::ToggleNote { row: 0, col: 4, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 4, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 4 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 4 });
        // (0,8): {1,3} — outside the quad; notes 1 and 3 must be eliminated
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 3 });

        let hint = NakedQuads
            .find(&state, &sol)
            .expect("NakedQuads should detect quad {1,2,3,4} in row 0");
        assert_eq!(hint.name_en, "Naked Quads");
        assert!(
            hint.elim_cells.contains(&(0, 8)),
            "elim_cells should contain (0,8) which shares notes with the quad"
        );
    }

    #[test]
    fn hidden_quads_finds_quad_and_eliminates() {
        // Row 0: cols 0-4 filled with 5,6,7,8,9; cols 5-8 empty.
        // Digits 1,2,3,4 are confined to exactly those 4 empty cells.
        // (0,5): {1,2,5}  — extra note 5 must be eliminated
        // (0,6): {2,3,6}  — extra note 6 must be eliminated
        // (0,7): {3,4,7}  — extra note 7 must be eliminated
        // (0,8): {1,4}    — no extras
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "567890000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // (0,5): {1,2,5}
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 5 });
        // (0,6): {2,3,6}
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 6 });
        // (0,7): {3,4,7}
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 4 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 7 });
        // (0,8): {1,4}
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 4 });

        let hint = HiddenQuads
            .find(&state, &sol)
            .expect("HiddenQuads should detect quad {1,2,3,4} in row 0");
        assert_eq!(hint.name_en, "Hidden Quads");
        assert!(
            hint.elim_cells.contains(&(0, 5)),
            "elim_cells should include (0,5) which has extra note 5"
        );
        assert!(
            hint.elim_cells.contains(&(0, 6)),
            "elim_cells should include (0,6) which has extra note 6"
        );
        assert!(
            hint.elim_cells.contains(&(0, 7)),
            "elim_cells should include (0,7) which has extra note 7"
        );
    }

    #[test]
    fn x_wing_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(XWing.find(&state, &sol).is_none());
    }

    #[test]
    fn x_wing_finds_row_based_and_eliminates() {
        // Digit 5 is confined to exactly cols 0 and 8 in both row 2 and row 6
        // (X-Wing pattern). Cell (4,0) also carries note 5 and must be eliminated.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // X-Wing base rows: digit 5 only in cols 0 and 8
        state.apply(GameEvent::ToggleNote { row: 2, col: 0, digit: 5 });
        state.apply(GameEvent::ToggleNote { row: 2, col: 8, digit: 5 });
        state.apply(GameEvent::ToggleNote { row: 6, col: 0, digit: 5 });
        state.apply(GameEvent::ToggleNote { row: 6, col: 8, digit: 5 });
        // Elimination target outside the two base rows
        state.apply(GameEvent::ToggleNote { row: 4, col: 0, digit: 5 });

        let hint = XWing
            .find(&state, &sol)
            .expect("XWing should detect the row-based pattern for digit 5");
        assert_eq!(hint.name_en, "X-Wing");
        assert!(
            hint.elim_cells.contains(&(4, 0)),
            "elim_cells should contain (4,0) which sees both X-Wing columns"
        );
    }

    #[test]
    fn swordfish_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(Swordfish.find(&state, &sol).is_none());
    }

    #[test]
    fn swordfish_finds_row_based_and_eliminates() {
        // Digit 7 spans exactly 3 rows (0, 3, 6), each with 2 occurrences.
        // The union of their columns is {0, 3, 6} — exactly 3 columns → Swordfish.
        // Cell (1,0) also has note 7 and must be eliminated.
        //
        // Row 0: cols {0, 3}   Row 3: cols {0, 6}   Row 6: cols {3, 6}
        // Union = {0, 3, 6} ✓
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // Swordfish base rows
        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 7 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 3, digit: 7 });
        state.apply(GameEvent::ToggleNote { row: 3, col: 0, digit: 7 });
        state.apply(GameEvent::ToggleNote { row: 3, col: 6, digit: 7 });
        state.apply(GameEvent::ToggleNote { row: 6, col: 3, digit: 7 });
        state.apply(GameEvent::ToggleNote { row: 6, col: 6, digit: 7 });
        // Elimination target: row 1, col 0 (outside the three base rows, inside col 0)
        state.apply(GameEvent::ToggleNote { row: 1, col: 0, digit: 7 });

        let hint = Swordfish
            .find(&state, &sol)
            .expect("Swordfish should detect the row-based pattern for digit 7");
        assert_eq!(hint.name_en, "Swordfish");
        assert!(
            hint.elim_cells.contains(&(1, 0)),
            "elim_cells should contain (1,0) which is in a Swordfish column"
        );
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
    fn y_wing_finds_pattern_and_eliminates() {
        // Classic Y-Wing:
        //   Pivot  (0,0): {1,2}
        //   Wing1  (0,5): {1,3}  — sees pivot (same row)
        //   Wing2  (5,0): {2,3}  — sees pivot (same col)
        //   Elimination digit: 3 (shared between the two wings but not pivot)
        //   Target (5,5): note 3 — sees Wing1 via col 5, sees Wing2 via row 5
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000\
             000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // Pivot
        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 2 });
        // Wing1: shares digit 1 with pivot, carries elimination digit 3
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 3 });
        // Wing2: carries the other pivot digit (2) and elimination digit 3
        state.apply(GameEvent::ToggleNote { row: 5, col: 0, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 0, digit: 3 });
        // Elimination target: sees Wing1 (col 5) and Wing2 (row 5)
        state.apply(GameEvent::ToggleNote { row: 5, col: 5, digit: 3 });

        let hint = YWing
            .find(&state, &sol)
            .expect("YWing should detect the pivot/wing pattern");
        assert_eq!(hint.name_en, "Y-Wing");
        assert!(
            hint.elim_cells.contains(&(5, 5)),
            "elim_cells should contain (5,5) which sees both wings"
        );
    }

    #[test]
    fn naked_quads_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(NakedQuads.find(&state, &sol).is_none());
    }

    #[test]
    fn hidden_quads_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(HiddenQuads.find(&state, &sol).is_none());
    }

    #[test]
    fn jellyfish_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(Jellyfish.find(&state, &sol).is_none());
    }

    #[test]
    fn jellyfish_finds_row_based_and_eliminates() {
        // Digit 3 spans 4 rows (0,2,5,7), each with 2 occurrences.
        // Union of columns = {0,2,5,7} — exactly 4 → Jellyfish.
        // Row 0:{0,2}  Row 2:{0,5}  Row 5:{2,7}  Row 7:{5,7}
        // Cell (1,0) has note 3 outside the 4 base rows → eliminated.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 2, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 2, col: 0, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 2, col: 5, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 2, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 7, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 7, col: 5, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 7, col: 7, digit: 3 });
        // Elimination target: outside the 4 base rows, inside a Jellyfish column
        state.apply(GameEvent::ToggleNote { row: 1, col: 0, digit: 3 });

        let hint = Jellyfish
            .find(&state, &sol)
            .expect("Jellyfish should detect the row-based pattern for digit 3");
        assert_eq!(hint.name_en, "Jellyfish");
        assert!(
            hint.elim_cells.contains(&(1, 0)),
            "elim_cells should contain (1,0) inside a Jellyfish column"
        );
    }

    #[test]
    fn skyscraper_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(Skyscraper.find(&state, &sol).is_none());
    }

    #[test]
    fn skyscraper_finds_row_based_and_eliminates() {
        // Digit 4: two rows each with exactly 2 occurrences, sharing column 2.
        // Row 0: cols {2, 6}   Row 5: cols {2, 8}   shared col = 2
        // Tips: (0,6) and (5,8).
        // Elimination target (3,6):
        //   sees tip (0,6) via col 6, sees tip (5,8) via box 5 (rows 3-5, cols 6-8).
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // Row 0 base
        state.apply(GameEvent::ToggleNote { row: 0, col: 2, digit: 4 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 4 });
        // Row 5 base
        state.apply(GameEvent::ToggleNote { row: 5, col: 2, digit: 4 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 8, digit: 4 });
        // Elimination target: sees tip (0,6) via col 6, sees tip (5,8) via box 5
        state.apply(GameEvent::ToggleNote { row: 3, col: 6, digit: 4 });

        let hint = Skyscraper
            .find(&state, &sol)
            .expect("Skyscraper should detect the row-based pattern for digit 4");
        assert_eq!(hint.name_en, "Skyscraper");
        assert!(
            hint.elim_cells.contains(&(3, 6)),
            "elim_cells should contain (3,6) which sees both Skyscraper tips"
        );
    }

    #[test]
    fn two_string_kite_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(TwoStringKite.find(&state, &sol).is_none());
    }

    #[test]
    fn two_string_kite_finds_and_eliminates() {
        // Digit 4: row 0 has exactly cols {3,7}; col 5 has exactly rows {1,7}.
        // (0,3) and (1,5) are both in box 1 → they form the kite's intersection.
        // Tips: (0,7) and (7,5).
        // (7,7): sees (0,7) via col 7, sees (7,5) via row 7 → eliminated.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // Row string
        state.apply(GameEvent::ToggleNote { row: 0, col: 3, digit: 4 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 4 });
        // Col string
        state.apply(GameEvent::ToggleNote { row: 1, col: 5, digit: 4 });
        state.apply(GameEvent::ToggleNote { row: 7, col: 5, digit: 4 });
        // Elimination target
        state.apply(GameEvent::ToggleNote { row: 7, col: 7, digit: 4 });

        let hint = TwoStringKite
            .find(&state, &sol)
            .expect("TwoStringKite should detect the kite pattern for digit 4");
        assert_eq!(hint.name_en, "2-String Kite");
        assert!(
            hint.elim_cells.contains(&(7, 7)),
            "elim_cells should contain (7,7) which sees both kite tips"
        );
    }

    #[test]
    fn xyz_wing_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(XYZWing.find(&state, &sol).is_none());
    }

    #[test]
    fn xyz_wing_finds_and_eliminates() {
        // Pivot  (0,0): {1,2,3}   — trivalue
        // Wing1  (0,5): {1,3}     — sees pivot (same row), mask = {1, c_digit=3}
        // Wing2  (0,8): {2,3}     — sees pivot (same row), mask = {2, c_digit=3}
        // All three are in row 0.  Elimination digit = 3.
        // Target (0,3): note 3 — sees all three via row 0 → eliminated.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // Pivot
        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 3 });
        // Wing1: {1,3}
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 3 });
        // Wing2: {2,3}
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 3 });
        // Elimination target: sees pivot, wing1, wing2 via row 0
        state.apply(GameEvent::ToggleNote { row: 0, col: 3, digit: 3 });

        let hint = XYZWing
            .find(&state, &sol)
            .expect("XYZWing should detect the pivot+wing pattern");
        assert_eq!(hint.name_en, "XYZ-Wing");
        assert!(
            hint.elim_cells.contains(&(0, 3)),
            "elim_cells should contain (0,3) which sees all three XYZ-Wing cells"
        );
    }

    #[test]
    fn w_wing_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(WWing.find(&state, &sol).is_none());
    }

    #[test]
    fn w_wing_finds_and_eliminates() {
        // W-Wing on {1,2} (a=1, b=2):
        //   P1 = (0,1): bivalue {1,2}  — sees strong-link cell (5,1) via col 1
        //   P2 = (8,8): bivalue {1,2}  — sees strong-link cell (5,8) via col 8
        //   P1 and P2 do NOT see each other.
        //   Strong link: row 5 has digit 1 in exactly (5,1) and (5,8).
        //   Eliminate b=2 from cells seeing both P1 and P2:
        //   (0,8): sees (0,1) via row 0, sees (8,8) via col 8 → eliminated.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // Bivalue W-Wing endpoints
        state.apply(GameEvent::ToggleNote { row: 0, col: 1, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 1, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 8, col: 8, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 8, col: 8, digit: 2 });
        // Strong link on digit 1 in row 5
        state.apply(GameEvent::ToggleNote { row: 5, col: 1, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 8, digit: 1 });
        // Elimination target: sees P1=(0,1) via row 0, sees P2=(8,8) via col 8
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 2 });

        let hint = WWing
            .find(&state, &sol)
            .expect("WWing should detect the W-Wing pattern for {1,2}");
        assert_eq!(hint.name_en, "W-Wing");
        assert!(
            hint.elim_cells.contains(&(0, 8)),
            "elim_cells should contain (0,8) which sees both W-Wing endpoints"
        );
    }

    #[test]
    fn unique_rectangle_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(UniqueRectangle.find(&state, &sol).is_none());
    }

    #[test]
    fn unique_rectangle_type1_finds_and_eliminates() {
        // Rectangle corners in 2 boxes (box 0 and box 1):
        //   (0,0),(0,3),(1,0),(1,3) — rows {0,1}, cols {0,3}
        // 3 corners locked to {1,2}; roof (1,3) has extra note 5.
        // Unique Rectangle Type 1: eliminate {1,2} from the roof.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // Three floor corners: only {1,2}
        for &(r, c) in &[(0usize, 0usize), (0, 3), (1, 0)] {
            state.apply(GameEvent::ToggleNote { row: r, col: c, digit: 1 });
            state.apply(GameEvent::ToggleNote { row: r, col: c, digit: 2 });
        }
        // Roof corner: {1,2} + extra 5
        state.apply(GameEvent::ToggleNote { row: 1, col: 3, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 1, col: 3, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 1, col: 3, digit: 5 });

        let hint = UniqueRectangle
            .find(&state, &sol)
            .expect("UniqueRectangle should detect the Type-1 pattern");
        assert_eq!(hint.name_en, "Unique Rectangle");
        assert_eq!(
            hint.target_cell,
            (1, 3),
            "target_cell should be the roof corner (1,3)"
        );
    }

    #[test]
    fn bug_plus_one_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(BugPlusOne.find(&state, &sol).is_none());
    }

    #[test]
    fn bug_plus_one_finds_and_places() {
        // Board: 4 empty cells, all others given (digit 5 as filler).
        // (0,4): {1,3}   (0,8): {1,2,3}  ← the BUG+1 trivalue cell
        // (2,6): {1,5}   (5,8): {1,4}
        //
        // Digit 1 at (0,8):
        //   row 0 peers with note 1: (0,4)           → count = 1 (odd) ✓
        //   col 8 peers with note 1: (5,8)            → count = 1 (odd) ✓
        //   box 2 peers with note 1: (2,6)            → count = 1 (odd) ✓
        // → BUG+1 places digit 1 at (0,8).
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            // (0,4)=0, (0,8)=0, (2,6)=0, (5,8)=0; all others = 5 (given)
            "555505550\
             555555555\
             555555055\
             555555555\
             555555555\
             555555550\
             555555555\
             555555555\
             555555555",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        state.apply(GameEvent::ToggleNote { row: 0, col: 4, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 4, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 2, col: 6, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 2, col: 6, digit: 5 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 8, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 8, digit: 4 });

        let hint = BugPlusOne
            .find(&state, &sol)
            .expect("BugPlusOne should detect the trivalue BUG+1 cell");
        assert_eq!(hint.name_en, "BUG+1");
        assert_eq!(hint.target_cell, (0, 8), "target_cell should be the trivalue cell");
        assert_eq!(hint.target_digit, Some(1), "digit 1 restores balance in all three units");
    }

    #[test]
    fn empty_rectangle_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(EmptyRectangle.find(&state, &sol).is_none());
    }

    #[test]
    fn empty_rectangle_finds_column_confinement_and_eliminates() {
        // Box 0 (rows 0-2, cols 0-2): digit 4 only in col 0 → (0,0) and (2,0).
        // Conjugate pair (row outside box band): row 5 has digit 4 exactly at
        // (5,0) and (5,7).  c_er_end=0 matches er_col=0; c_other=7.
        // Eliminate 4 from (r_er, 7) where r_er in {0,1,2}: (0,7) has note 4.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // ER cells (box 0, confined to col 0)
        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 4 });
        state.apply(GameEvent::ToggleNote { row: 2, col: 0, digit: 4 });
        // Conjugate pair in row 5
        state.apply(GameEvent::ToggleNote { row: 5, col: 0, digit: 4 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 7, digit: 4 });
        // Elimination target
        state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit: 4 });

        let hint = EmptyRectangle
            .find(&state, &sol)
            .expect("EmptyRectangle should detect column confinement in box 0");
        assert_eq!(hint.name_en, "Empty Rectangle");
        assert!(
            hint.elim_cells.contains(&(0, 7)),
            "elim_cells should contain (0,7)"
        );
    }

    #[test]
    fn simple_coloring_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(SimpleColoring.find(&state, &sol).is_none());
    }

    #[test]
    fn simple_coloring_finds_color_trap_and_eliminates() {
        // Digit 7 — Color Wrap (two same-color cells see each other).
        // 4-link chain with strong links:
        //   row 0 : (0,0) c0 ↔ (0,6) c1   [exactly 2 in row 0]
        //   col 6 : (0,6) c1 ↔ (3,6) c0   [exactly 2 in col 6]
        //   row 3 : (3,6) c0 ↔ (3,3) c1   [exactly 2 in row 3]
        //   col 3 : (3,3) c1 ↔ (0,3) c0   [exactly 2 in col 3]
        // c0 = {(0,0), (3,6), (0,3)}.
        // (0,0) and (0,3) are both c0 and share row 0 → Color Wrap fires.
        // All c0 cells are eliminated → elim_cells contains (0,0).
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 7 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit: 7 });
        state.apply(GameEvent::ToggleNote { row: 3, col: 6, digit: 7 });
        state.apply(GameEvent::ToggleNote { row: 3, col: 3, digit: 7 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 3, digit: 7 });

        let hint = SimpleColoring
            .find(&state, &sol)
            .expect("SimpleColoring should detect the Color Wrap");
        assert_eq!(hint.name_en, "Simple Coloring");
        assert!(
            hint.elim_cells.contains(&(0, 0))
                || hint.elim_cells.contains(&(3, 6))
                || hint.elim_cells.contains(&(0, 3)),
            "elim_cells should contain at least one c0 cell eliminated by Color Wrap"
        );
    }

    #[test]
    fn xy_chain_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(XYChain.find(&state, &sol).is_none());
    }

    #[test]
    fn xy_chain_finds_and_eliminates() {
        // 3-cell XY-Chain:  (0,0){1,2} — (0,5){2,3} — (5,5){3,1}
        // elim_d = 1 (first digit of start cell).
        // Chain ends share digit 1; cell (5,0) sees start (0,0) via col 0
        // and end (5,5) via row 5, and has note 1 → eliminated.
        use crate::puzzle::event::GameEvent;
        let grid = Grid::from_str(
            "000000000000000000000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(SOL).unwrap();

        // Chain cells
        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 1 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 0, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 2 });
        state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 5, digit: 3 });
        state.apply(GameEvent::ToggleNote { row: 5, col: 5, digit: 1 });
        // Elimination target
        state.apply(GameEvent::ToggleNote { row: 5, col: 0, digit: 1 });

        let hint = XYChain
            .find(&state, &sol)
            .expect("XYChain should detect the 3-cell chain");
        assert_eq!(hint.name_en, "XY-Chain");
        assert!(
            hint.elim_cells.contains(&(5, 0)),
            "elim_cells should contain (5,0) which sees both chain ends"
        );
    }
}
