// src/hint/strategies/tier2.rs
// Tier-2 hint strategies: from NakedTriples / HiddenTriples through BUG+1.

use crate::hint::{Hint, Strategy};
use crate::puzzle::{CellKind, Grid};
use crate::puzzle::game_state::GameState;

// ── Public structs ─────────────────────────────────────────────────────────────

pub struct NotesValidator;
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

// ── Shared helpers ─────────────────────────────────────────────────────────────

/// Iterate the 27 units (9 rows, 9 cols, 9 boxes) as lists of (row,col).
fn all_units() -> Vec<Vec<(usize, usize)>> {
    let mut units = Vec::with_capacity(27);
    for i in 0..9 {
        units.push((0..9).map(|c| (i, c)).collect::<Vec<_>>()); // row i
        units.push((0..9).map(|r| (r, i)).collect::<Vec<_>>()); // col i
        let br = (i / 3) * 3;
        let bc = (i % 3) * 3;
        units.push(
            (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc)))
                .collect::<Vec<_>>(),
        ); // box
    }
    units
}

/// Returns true if (r1,c1) and (r2,c2) share a row, column, or 3×3 box.
#[inline]
fn sees(r1: usize, c1: usize, r2: usize, c2: usize) -> bool {
    r1 == r2 || c1 == c2 || (r1 / 3 == r2 / 3 && c1 / 3 == c2 / 3)
}

// ── NotesValidator ─────────────────────────────────────────────────────────────

impl Strategy for NotesValidator {
    fn name_en(&self) -> &'static str { "Notes Error" }
    fn name_de(&self) -> &'static str { "Notizen-Fehler" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for r in 0..9 {
            for c in 0..9 {
                if !matches!(grid.get(r, c), CellKind::Empty) { continue; }
                let mask = state.notes_mask(r, c);
                if mask == 0 { continue; }
                // Check each note against placed digits in peers
                for d in 1u8..=9 {
                    if mask & (1 << d) == 0 { continue; }
                    // Is this digit already placed in the same row/col/box?
                    let mut conflict = false;
                    for cc in 0..9 {
                        if grid.get(r, cc).value() == Some(d) { conflict = true; break; }
                    }
                    if !conflict {
                        for rr in 0..9 {
                            if grid.get(rr, c).value() == Some(d) { conflict = true; break; }
                        }
                    }
                    if !conflict {
                        let br = (r / 3) * 3;
                        let bc = (c / 3) * 3;
                        'outer: for dr in 0..3 {
                            for dc in 0..3 {
                                if grid.get(br + dr, bc + dc).value() == Some(d) {
                                    conflict = true;
                                    break 'outer;
                                }
                            }
                        }
                    }
                    if conflict {
                        return Some(Hint {
                            cause_cells:    vec![],
                            elim_cells:     vec![(r, c)],
                            target_cell:    (r, c),
                            elim_digit:     Some(d),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!("Note {} in this cell is invalid — {} is already placed in a peer.", d, d),
                            explanation_de: format!("Notiz {} in dieser Zelle ist ung\u{fc}ltig — {} ist bereits in einer Nachbarzelle.", d, d),
                        });
                    }
                }
            }
        }
        None
    }
}

// ── NakedTriples ──────────────────────────────────────────────────────────────

impl Strategy for NakedTriples {
    fn name_en(&self) -> &'static str { "Naked Triples" }
    fn name_de(&self) -> &'static str { "Naked Triples" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let small: Vec<(usize, usize, u16)> = unit.iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .filter_map(|&(r, c)| {
                    let m = state.notes_mask(r, c);
                    let n = m.count_ones();
                    if n >= 2 && n <= 3 { Some((r, c, m)) } else { None }
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
                        let elim: Vec<(usize, usize)> = unit.iter()
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

// ── HiddenTriples ─────────────────────────────────────────────────────────────

impl Strategy for HiddenTriples {
    fn name_en(&self) -> &'static str { "Hidden Triples" }
    fn name_de(&self) -> &'static str { "Hidden Triples" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let empties: Vec<(usize, usize)> = unit.iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .copied()
                .collect();
            for d1 in 1u8..=9 {
                for d2 in (d1 + 1)..=9 {
                    for d3 in (d2 + 1)..=9 {
                        let mask = (1u16 << d1) | (1u16 << d2) | (1u16 << d3);
                        // Cells in this unit that contain at least one of d1/d2/d3
                        let triple_cells: Vec<(usize, usize)> = empties.iter()
                            .filter(|&&(r, c)| (state.notes_mask(r, c) & mask) != 0)
                            .copied()
                            .collect();
                        if triple_cells.len() != 3 { continue; }
                        // All three digits must appear in combined notes of those 3 cells
                        let combined: u16 = triple_cells.iter()
                            .fold(0u16, |acc, &(r, c)| acc | state.notes_mask(r, c));
                        if (combined & mask) != mask { continue; }
                        // At least one cell must have extra candidates to eliminate
                        let elim: Vec<(usize, usize)> = triple_cells.iter()
                            .filter(|&&(r, c)| (state.notes_mask(r, c) & !mask) != 0)
                            .copied()
                            .collect();
                        if elim.is_empty() { continue; }
                        let target = elim[0];
                        return Some(Hint {
                            cause_cells:    triple_cells,
                            elim_cells:     elim,
                            target_cell:    target,
                            elim_digit:     Some(d1),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "Only these cells can hold {}/{}/{}. Remove other notes from highlighted cells.",
                                d1, d2, d3
                            ),
                            explanation_de: format!(
                                "Nur diese Zellen k\u{f6}nnen {}/{}/{} halten. Andere Notizen aus den markierten Zellen streichen.",
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

// ── NakedQuads ────────────────────────────────────────────────────────────────

impl Strategy for NakedQuads {
    fn name_en(&self) -> &'static str { "Naked Quads" }
    fn name_de(&self) -> &'static str { "Naked Quads" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let small: Vec<(usize, usize, u16)> = unit.iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .filter_map(|&(r, c)| {
                    let m = state.notes_mask(r, c);
                    let n = m.count_ones();
                    if n >= 2 && n <= 4 { Some((r, c, m)) } else { None }
                })
                .collect();

            for i in 0..small.len() {
                for j in (i + 1)..small.len() {
                    for k in (j + 1)..small.len() {
                        for l in (k + 1)..small.len() {
                            let combined =
                                small[i].2 | small[j].2 | small[k].2 | small[l].2;
                            if combined.count_ones() != 4 { continue; }
                            let quad = [
                                (small[i].0, small[i].1),
                                (small[j].0, small[j].1),
                                (small[k].0, small[k].1),
                                (small[l].0, small[l].1),
                            ];
                            let digits: Vec<u8> = (1u8..=9)
                                .filter(|&d| combined & (1 << d) != 0)
                                .collect();
                            let elim: Vec<(usize, usize)> = unit.iter()
                                .filter(|&&(r, c)| {
                                    !quad.contains(&(r, c))
                                        && matches!(grid.get(r, c), CellKind::Empty)
                                        && (state.notes_mask(r, c) & combined) != 0
                                })
                                .copied()
                                .collect();
                            if elim.is_empty() { continue; }
                            let (d1, d2, d3, d4) =
                                (digits[0], digits[1], digits[2], digits[3]);
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

// ── HiddenQuads ───────────────────────────────────────────────────────────────

impl Strategy for HiddenQuads {
    fn name_en(&self) -> &'static str { "Hidden Quads" }
    fn name_de(&self) -> &'static str { "Hidden Quads" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let empties: Vec<(usize, usize)> = unit.iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .copied()
                .collect();
            for d1 in 1u8..=9 {
                for d2 in (d1 + 1)..=9 {
                    for d3 in (d2 + 1)..=9 {
                        for d4 in (d3 + 1)..=9 {
                            let mask = (1u16 << d1) | (1u16 << d2)
                                | (1u16 << d3) | (1u16 << d4);
                            // Cells containing at least one of the four digits
                            let quad_cells: Vec<(usize, usize)> = empties.iter()
                                .filter(|&&(r, c)| {
                                    (state.notes_mask(r, c) & mask) != 0
                                })
                                .copied()
                                .collect();
                            if quad_cells.len() != 4 { continue; }
                            // All four digits must appear in combined notes
                            let combined: u16 = quad_cells.iter()
                                .fold(0u16, |acc, &(r, c)| acc | state.notes_mask(r, c));
                            if (combined & mask) != mask { continue; }
                            // At least one cell must have extra candidates
                            let elim: Vec<(usize, usize)> = quad_cells.iter()
                                .filter(|&&(r, c)| {
                                    (state.notes_mask(r, c) & !mask) != 0
                                })
                                .copied()
                                .collect();
                            if elim.is_empty() { continue; }
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

// ── XWing ─────────────────────────────────────────────────────────────────────

impl Strategy for XWing {
    fn name_en(&self) -> &'static str { "X-Wing" }
    fn name_de(&self) -> &'static str { "X-Wing" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for digit in 1u8..=9 {
            // Row-based X-Wing
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
                    let c1 = row_cols[r1][0];
                    let c2 = row_cols[r1][1];
                    let cause: Vec<(usize, usize)> = vec![(r1, c1), (r1, c2), (r2, c1), (r2, c2)];
                    let elim: Vec<(usize, usize)> = (0..9)
                        .flat_map(|r| [(r, c1), (r, c2)])
                        .filter(|&(r, c)| {
                            r != r1 && r != r2
                                && matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();
                    if elim.is_empty() { continue; }
                    return Some(Hint {
                        cause_cells:    cause,
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "{} is locked in 2 rows across 2 columns. Remove {} from notes in highlighted cells.",
                            digit, digit
                        ),
                        explanation_de: format!(
                            "{} ist in 2 Zeilen auf 2 Spalten eingeschr\u{e4}nkt. {} aus den markierten Zellen streichen.",
                            digit, digit
                        ),
                    });
                }
            }

            // Column-based X-Wing
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
                    let r1 = col_rows[c1][0];
                    let r2 = col_rows[c1][1];
                    let cause: Vec<(usize, usize)> = vec![(r1, c1), (r2, c1), (r1, c2), (r2, c2)];
                    let elim: Vec<(usize, usize)> = (0..9)
                        .flat_map(|c| [(r1, c), (r2, c)])
                        .filter(|&(r, c)| {
                            c != c1 && c != c2
                                && matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();
                    if elim.is_empty() { continue; }
                    return Some(Hint {
                        cause_cells:    cause,
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "{} is locked in 2 columns across 2 rows. Remove {} from notes in highlighted cells.",
                            digit, digit
                        ),
                        explanation_de: format!(
                            "{} ist in 2 Spalten auf 2 Zeilen eingeschr\u{e4}nkt. {} aus den markierten Zellen streichen.",
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
            // Row-based Swordfish
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
                .filter(|&r| { let n = row_cols[r].len(); n == 2 || n == 3 })
                .collect();

            for i in 0..cand_rows.len() {
                for j in (i + 1)..cand_rows.len() {
                    for k in (j + 1)..cand_rows.len() {
                        let (r1, r2, r3) = (cand_rows[i], cand_rows[j], cand_rows[k]);
                        let mut cols = std::collections::BTreeSet::new();
                        for &c in &row_cols[r1] { cols.insert(c); }
                        for &c in &row_cols[r2] { cols.insert(c); }
                        for &c in &row_cols[r3] { cols.insert(c); }
                        if cols.len() != 3 { continue; }
                        let cause: Vec<(usize, usize)> = [r1, r2, r3].iter()
                            .flat_map(|&r| row_cols[r].iter().map(move |&c| (r, c)))
                            .collect();
                        let elim: Vec<(usize, usize)> = cols.iter()
                            .flat_map(|&c| (0..9).map(move |r| (r, c)))
                            .filter(|&(r, c)| {
                                r != r1 && r != r2 && r != r3
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & (1 << digit)) != 0
                            })
                            .collect();
                        if elim.is_empty() { continue; }
                        return Some(Hint {
                            cause_cells:    cause,
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(digit),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "{} is locked in 3 rows across 3 columns. Remove {} from notes in highlighted cells.",
                                digit, digit
                            ),
                            explanation_de: format!(
                                "{} ist in 3 Zeilen auf 3 Spalten eingeschr\u{e4}nkt. {} aus den markierten Zellen streichen.",
                                digit, digit
                            ),
                        });
                    }
                }
            }

            // Column-based Swordfish
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
                .filter(|&c| { let n = col_rows[c].len(); n == 2 || n == 3 })
                .collect();

            for i in 0..cand_cols.len() {
                for j in (i + 1)..cand_cols.len() {
                    for k in (j + 1)..cand_cols.len() {
                        let (c1, c2, c3) = (cand_cols[i], cand_cols[j], cand_cols[k]);
                        let mut rows = std::collections::BTreeSet::new();
                        for &r in &col_rows[c1] { rows.insert(r); }
                        for &r in &col_rows[c2] { rows.insert(r); }
                        for &r in &col_rows[c3] { rows.insert(r); }
                        if rows.len() != 3 { continue; }
                        let cause: Vec<(usize, usize)> = [c1, c2, c3].iter()
                            .flat_map(|&c| col_rows[c].iter().map(move |&r| (r, c)))
                            .collect();
                        let elim: Vec<(usize, usize)> = rows.iter()
                            .flat_map(|&r| (0..9).map(move |c| (r, c)))
                            .filter(|&(r, c)| {
                                c != c1 && c != c2 && c != c3
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & (1 << digit)) != 0
                            })
                            .collect();
                        if elim.is_empty() { continue; }
                        return Some(Hint {
                            cause_cells:    cause,
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(digit),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "{} is locked in 3 columns across 3 rows. Remove {} from notes in highlighted cells.",
                                digit, digit
                            ),
                            explanation_de: format!(
                                "{} ist in 3 Spalten auf 3 Zeilen eingeschr\u{e4}nkt. {} aus den markierten Zellen streichen.",
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

// ── Jellyfish ─────────────────────────────────────────────────────────────────

impl Strategy for Jellyfish {
    fn name_en(&self) -> &'static str { "Jellyfish" }
    fn name_de(&self) -> &'static str { "Jellyfish" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for digit in 1u8..=9 {
            // Row-based Jellyfish
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
                .filter(|&r| { let n = row_cols[r].len(); n >= 2 && n <= 4 })
                .collect();

            for i in 0..cand_rows.len() {
                for j in (i + 1)..cand_rows.len() {
                    for k in (j + 1)..cand_rows.len() {
                        for l in (k + 1)..cand_rows.len() {
                            let (r1, r2, r3, r4) = (
                                cand_rows[i], cand_rows[j], cand_rows[k], cand_rows[l],
                            );
                            let mut cols = std::collections::BTreeSet::new();
                            for &c in &row_cols[r1] { cols.insert(c); }
                            for &c in &row_cols[r2] { cols.insert(c); }
                            for &c in &row_cols[r3] { cols.insert(c); }
                            for &c in &row_cols[r4] { cols.insert(c); }
                            if cols.len() != 4 { continue; }
                            let cause: Vec<(usize, usize)> = [r1, r2, r3, r4].iter()
                                .flat_map(|&r| row_cols[r].iter().map(move |&c| (r, c)))
                                .collect();
                            let elim: Vec<(usize, usize)> = cols.iter()
                                .flat_map(|&c| (0..9).map(move |r| (r, c)))
                                .filter(|&(r, c)| {
                                    r != r1 && r != r2 && r != r3 && r != r4
                                        && matches!(grid.get(r, c), CellKind::Empty)
                                        && (state.notes_mask(r, c) & (1 << digit)) != 0
                                })
                                .collect();
                            if elim.is_empty() { continue; }
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

            // Column-based Jellyfish
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
                .filter(|&c| { let n = col_rows[c].len(); n >= 2 && n <= 4 })
                .collect();

            for i in 0..cand_cols.len() {
                for j in (i + 1)..cand_cols.len() {
                    for k in (j + 1)..cand_cols.len() {
                        for l in (k + 1)..cand_cols.len() {
                            let (c1, c2, c3, c4) = (
                                cand_cols[i], cand_cols[j], cand_cols[k], cand_cols[l],
                            );
                            let mut rows = std::collections::BTreeSet::new();
                            for &r in &col_rows[c1] { rows.insert(r); }
                            for &r in &col_rows[c2] { rows.insert(r); }
                            for &r in &col_rows[c3] { rows.insert(r); }
                            for &r in &col_rows[c4] { rows.insert(r); }
                            if rows.len() != 4 { continue; }
                            let cause: Vec<(usize, usize)> = [c1, c2, c3, c4].iter()
                                .flat_map(|&c| col_rows[c].iter().map(move |&r| (r, c)))
                                .collect();
                            let elim: Vec<(usize, usize)> = rows.iter()
                                .flat_map(|&r| (0..9).map(move |c| (r, c)))
                                .filter(|&(r, c)| {
                                    c != c1 && c != c2 && c != c3 && c != c4
                                        && matches!(grid.get(r, c), CellKind::Empty)
                                        && (state.notes_mask(r, c) & (1 << digit)) != 0
                                })
                                .collect();
                            if elim.is_empty() { continue; }
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
    fn name_en(&self) -> &'static str { "Skyscraper" }
    fn name_de(&self) -> &'static str { "Skyscraper" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for digit in 1u8..=9 {
            // Row-based Skyscraper:
            // Two rows each have the digit in exactly 2 columns.
            // They share exactly one column (conjugate). Eliminate from cells
            // that see BOTH non-shared column endpoints.
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
                    if row_cols[r2].len() != 2 { continue; }
                    // Find shared column
                    let shared: Vec<usize> = row_cols[r1].iter()
                        .filter(|c| row_cols[r2].contains(c))
                        .copied()
                        .collect();
                    if shared.len() != 1 { continue; }
                    let c_shared = shared[0];
                    let ca = *row_cols[r1].iter().find(|&&c| c != c_shared).unwrap();
                    let cb = *row_cols[r2].iter().find(|&&c| c != c_shared).unwrap();
                    // Eliminate from cells seeing both (r1, ca) and (r2, cb)
                    let cause = vec![(r1, c_shared), (r1, ca), (r2, c_shared), (r2, cb)];
                    let elim: Vec<(usize, usize)> = (0..9)
                        .flat_map(|r| (0..9).map(move |c| (r, c)))
                        .filter(|&(r, c)| {
                            (r, c) != (r1, ca) && (r, c) != (r2, cb)
                                && sees(r, c, r1, ca) && sees(r, c, r2, cb)
                                && matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();
                    if elim.is_empty() { continue; }
                    return Some(Hint {
                        cause_cells:    cause,
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "Skyscraper on {}. Remove {} from notes in highlighted cells.",
                            digit, digit
                        ),
                        explanation_de: format!(
                            "Skyscraper auf {}. {} aus den markierten Zellen streichen.",
                            digit, digit
                        ),
                    });
                }
            }

            // Column-based Skyscraper
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
                    if col_rows[c2].len() != 2 { continue; }
                    let shared: Vec<usize> = col_rows[c1].iter()
                        .filter(|r| col_rows[c2].contains(r))
                        .copied()
                        .collect();
                    if shared.len() != 1 { continue; }
                    let r_shared = shared[0];
                    let ra = *col_rows[c1].iter().find(|&&r| r != r_shared).unwrap();
                    let rb = *col_rows[c2].iter().find(|&&r| r != r_shared).unwrap();
                    let cause = vec![(r_shared, c1), (ra, c1), (r_shared, c2), (rb, c2)];
                    let elim: Vec<(usize, usize)> = (0..9)
                        .flat_map(|r| (0..9).map(move |c| (r, c)))
                        .filter(|&(r, c)| {
                            (r, c) != (ra, c1) && (r, c) != (rb, c2)
                                && sees(r, c, ra, c1) && sees(r, c, rb, c2)
                                && matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << digit)) != 0
                        })
                        .collect();
                    if elim.is_empty() { continue; }
                    return Some(Hint {
                        cause_cells:    cause,
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "Skyscraper on {}. Remove {} from notes in highlighted cells.",
                            digit, digit
                        ),
                        explanation_de: format!(
                            "Skyscraper auf {}. {} aus den markierten Zellen streichen.",
                            digit, digit
                        ),
                    });
                }
            }
        }
        None
    }
}

// ── TwoStringKite ─────────────────────────────────────────────────────────────

impl Strategy for TwoStringKite {
    fn name_en(&self) -> &'static str { "2-String Kite" }
    fn name_de(&self) -> &'static str { "2-String Kite" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for digit in 1u8..=9 {
            // For each row with exactly 2 cells having the digit
            for row in 0..9usize {
                let row_cells: Vec<usize> = (0..9)
                    .filter(|&c| {
                        matches!(grid.get(row, c), CellKind::Empty)
                            && (state.notes_mask(row, c) & (1 << digit)) != 0
                    })
                    .collect();
                if row_cells.len() != 2 { continue; }
                let (rc1, rc2) = (row_cells[0], row_cells[1]);

                // For each column with exactly 2 cells having the digit
                for col in 0..9usize {
                    let col_cells: Vec<usize> = (0..9)
                        .filter(|&r| {
                            matches!(grid.get(r, col), CellKind::Empty)
                                && (state.notes_mask(r, col) & (1 << digit)) != 0
                        })
                        .collect();
                    if col_cells.len() != 2 { continue; }
                    let (cr1, cr2) = (col_cells[0], col_cells[1]);

                    // The four cells: (row, rc1), (row, rc2) from row; (cr1, col), (cr2, col) from col
                    // We need exactly one "intersection" — a cell that is in both the row and column
                    // groups AND shares a box with a cell from the other group.
                    // Specifically: one row cell and one col cell are in the same box.
                    // The remaining two cells are the "tips".
                    let row_pair = [(row, rc1), (row, rc2)];
                    let col_pair = [(cr1, col), (cr2, col)];

                    // Try each combination to find intersection
                    for &(r_int, c_int) in &row_pair {
                        for &(r_col_int, c_col_int) in &col_pair {
                            // Skip if same cell
                            if (r_int, c_int) == (r_col_int, c_col_int) { continue; }
                            // Check if they share a box
                            if r_int / 3 != r_col_int / 3 || c_int / 3 != c_col_int / 3 {
                                continue;
                            }
                            // The two tips are the non-intersection cells
                            let tip1 = row_pair.iter()
                                .find(|&&rc| rc != (r_int, c_int))
                                .copied()
                                .unwrap();
                            let tip2 = col_pair.iter()
                                .find(|&&rc| rc != (r_col_int, c_col_int))
                                .copied()
                                .unwrap();
                            // Eliminate digit from cells seeing both tips
                            let cause = vec![(r_int, c_int), (r_col_int, c_col_int), tip1, tip2];
                            let elim: Vec<(usize, usize)> = (0..9)
                                .flat_map(|r| (0..9).map(move |c| (r, c)))
                                .filter(|&(r, c)| {
                                    (r, c) != tip1 && (r, c) != tip2
                                        && sees(r, c, tip1.0, tip1.1)
                                        && sees(r, c, tip2.0, tip2.1)
                                        && matches!(grid.get(r, c), CellKind::Empty)
                                        && (state.notes_mask(r, c) & (1 << digit)) != 0
                                })
                                .collect();
                            if elim.is_empty() { continue; }
                            return Some(Hint {
                                cause_cells:    cause,
                                elim_cells:     elim.clone(),
                                target_cell:    elim[0],
                                elim_digit:     Some(digit),
                                target_digit:   None,
                                name_en:        self.name_en(),
                                name_de:        self.name_de(),
                                explanation_en: format!(
                                    "2-String Kite on {}. Remove {} from notes in highlighted cells.",
                                    digit, digit
                                ),
                                explanation_de: format!(
                                    "2-String Kite auf {}. {} aus den markierten Zellen streichen.",
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

// ── YWing ─────────────────────────────────────────────────────────────────────

impl Strategy for YWing {
    fn name_en(&self) -> &'static str { "Y-Wing" }
    fn name_de(&self) -> &'static str { "Y-Wing" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        // Collect bivalue cells
        let bivalue: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter_map(|(r, c)| {
                if !matches!(grid.get(r, c), CellKind::Empty) { return None; }
                let m = state.notes_mask(r, c);
                if m.count_ones() == 2 { Some((r, c, m)) } else { None }
            })
            .collect();

        for &(pr, pc, pm) in &bivalue {
            // Pivot has {a, b}
            let a = pm.trailing_zeros() as u8;
            let b = (pm >> (a as u32 + 1)).trailing_zeros() as u8 + a + 1;

            // Look for Wing1 with {a, c} seeing pivot
            for &(w1r, w1c, w1m) in &bivalue {
                if (w1r, w1c) == (pr, pc) { continue; }
                if !sees(pr, pc, w1r, w1c) { continue; }
                if (w1m & (1 << a)) == 0 { continue; } // must have a
                if (w1m & (1 << b)) != 0 { continue; } // must not have b (that's pivot's b)
                // Wing1 = {a, c}
                let c_digit = (w1m & !(1u16 << a)).trailing_zeros() as u8;

                // Look for Wing2 with {b, c} seeing pivot (but different from Wing1)
                for &(w2r, w2c, w2m) in &bivalue {
                    if (w2r, w2c) == (pr, pc) || (w2r, w2c) == (w1r, w1c) { continue; }
                    if !sees(pr, pc, w2r, w2c) { continue; }
                    if (w2m & (1 << b)) == 0 { continue; }  // must have b
                    if (w2m & (1 << c_digit)) == 0 { continue; } // must have c
                    if w2m.count_ones() != 2 { continue; }

                    // Eliminate c from cells seeing both Wing1 and Wing2
                    let elim: Vec<(usize, usize)> = (0..9)
                        .flat_map(|r| (0..9).map(move |c| (r, c)))
                        .filter(|&(r, c)| {
                            (r, c) != (w1r, w1c) && (r, c) != (w2r, w2c)
                                && sees(r, c, w1r, w1c)
                                && sees(r, c, w2r, w2c)
                                && matches!(grid.get(r, c), CellKind::Empty)
                                && (state.notes_mask(r, c) & (1 << c_digit)) != 0
                        })
                        .collect();
                    if elim.is_empty() { continue; }
                    return Some(Hint {
                        cause_cells:    vec![(pr, pc), (w1r, w1c), (w2r, w2c)],
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(c_digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!(
                            "Y-Wing: pivot has {}/{}, wings share {}. Remove {} from notes in highlighted cells.",
                            a, b, c_digit, c_digit
                        ),
                        explanation_de: format!(
                            "Y-Wing: Pivot hat {}/{}, Wings teilen {}. {} aus den markierten Zellen streichen.",
                            a, b, c_digit, c_digit
                        ),
                    });
                }
            }
        }
        None
    }
}

// ── XYZWing ───────────────────────────────────────────────────────────────────

impl Strategy for XYZWing {
    fn name_en(&self) -> &'static str { "XYZ-Wing" }
    fn name_de(&self) -> &'static str { "XYZ-Wing" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        // Bivalue cells
        let bivalue: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter_map(|(r, c)| {
                if !matches!(grid.get(r, c), CellKind::Empty) { return None; }
                let m = state.notes_mask(r, c);
                if m.count_ones() == 2 { Some((r, c, m)) } else { None }
            })
            .collect();

        // Trivalue cells (pivots)
        let trivalue: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter_map(|(r, c)| {
                if !matches!(grid.get(r, c), CellKind::Empty) { return None; }
                let m = state.notes_mask(r, c);
                if m.count_ones() == 3 { Some((r, c, m)) } else { None }
            })
            .collect();

        for &(pr, pc, pm) in &trivalue {
            // Extract the three digits {a, b, c} from pivot
            let digits: Vec<u8> = (1u8..=9).filter(|&d| pm & (1 << d) != 0).collect();
            if digits.len() != 3 { continue; }
            let (da, db, dc) = (digits[0], digits[1], digits[2]);

            // Try each digit as the "pinch" digit (the one to eliminate, must be in all three)
            for &c_digit in &[da, db, dc] {
                // Wing masks must be 2-candidate subsets of pivot that include c_digit
                // Wing1: {x, c} where x is one of the other two pivot digits
                // Wing2: {y, c} where y is the remaining pivot digit
                // Both wings see pivot
                let wing_masks: [(u8, u16); 2] = {
                    let others: Vec<u8> = [da, db, dc].iter()
                        .filter(|&&d| d != c_digit)
                        .copied()
                        .collect();
                    [
                        (others[0], (1u16 << others[0]) | (1u16 << c_digit)),
                        (others[1], (1u16 << others[1]) | (1u16 << c_digit)),
                    ]
                };

                // Find wing1 and wing2 among bivalue cells that see pivot
                let wings_for: [Vec<(usize, usize)>; 2] = [
                    bivalue.iter()
                        .filter(|&&(r, c, m)| {
                            (r, c) != (pr, pc) && sees(r, c, pr, pc) && m == wing_masks[0].1
                        })
                        .map(|&(r, c, _)| (r, c))
                        .collect(),
                    bivalue.iter()
                        .filter(|&&(r, c, m)| {
                            (r, c) != (pr, pc) && sees(r, c, pr, pc) && m == wing_masks[1].1
                        })
                        .map(|&(r, c, _)| (r, c))
                        .collect(),
                ];

                for &(w1r, w1c) in &wings_for[0] {
                    for &(w2r, w2c) in &wings_for[1] {
                        if (w1r, w1c) == (w2r, w2c) { continue; }
                        // Eliminate c_digit from cells seeing ALL THREE: pivot, wing1, wing2
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
                        if elim.is_empty() { continue; }
                        return Some(Hint {
                            cause_cells:    vec![(pr, pc), (w1r, w1c), (w2r, w2c)],
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(c_digit),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "XYZ-Wing: pivot has 3 candidates, wings share {}. Remove {} from notes in highlighted cells.",
                                c_digit, c_digit
                            ),
                            explanation_de: format!(
                                "XYZ-Wing: Pivot hat 3 Kandidaten, Wings teilen {}. {} aus den markierten Zellen streichen.",
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

// ── WWing ─────────────────────────────────────────────────────────────────────

impl Strategy for WWing {
    fn name_en(&self) -> &'static str { "W-Wing" }
    fn name_de(&self) -> &'static str { "W-Wing" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        // Collect bivalue cells grouped by mask
        let bivalue: Vec<(usize, usize, u16)> = (0..9)
            .flat_map(|r| (0..9).map(move |c| (r, c)))
            .filter_map(|(r, c)| {
                if !matches!(grid.get(r, c), CellKind::Empty) { return None; }
                let m = state.notes_mask(r, c);
                if m.count_ones() == 2 { Some((r, c, m)) } else { None }
            })
            .collect();

        for pair_mask in (1u16..512).filter(|m| m.count_ones() == 2) {
            // Extract {a, b} from pair_mask
            let a = pair_mask.trailing_zeros() as u8;
            let b = (pair_mask >> (a as u32 + 1)).trailing_zeros() as u8 + a + 1;

            // Collect all bivalue cells with this mask
            let pairs: Vec<(usize, usize)> = bivalue.iter()
                .filter(|&&(_, _, m)| m == pair_mask)
                .map(|&(r, c, _)| (r, c))
                .collect();

            if pairs.len() < 2 { continue; }

            // For each unit: find units where digit `a` appears in exactly 2 empty cells
            for unit in all_units() {
                let unit_a_cells: Vec<(usize, usize)> = unit.iter()
                    .filter(|&&(r, c)| {
                        matches!(grid.get(r, c), CellKind::Empty)
                            && (state.notes_mask(r, c) & (1 << a)) != 0
                    })
                    .copied()
                    .collect();
                if unit_a_cells.len() != 2 { continue; }
                let (e1, e2) = (unit_a_cells[0], unit_a_cells[1]);

                // Find bivalue P1 that sees e1 (P1 != e1)
                for &p1 in pairs.iter() {
                    if p1 == e1 { continue; }
                    if !sees(p1.0, p1.1, e1.0, e1.1) { continue; }
                    // Find bivalue P2 that sees e2 (P2 != e2, P2 != P1, P2 doesn't see P1)
                    for &p2 in pairs.iter() {
                        if p2 == p1 || p2 == e2 { continue; }
                        if !sees(p2.0, p2.1, e2.0, e2.1) { continue; }
                        if sees(p1.0, p1.1, p2.0, p2.1) { continue; } // would be Y-Wing
                        // Eliminate b from cells seeing both P1 and P2
                        let elim: Vec<(usize, usize)> = (0..9)
                            .flat_map(|r| (0..9).map(move |c| (r, c)))
                            .filter(|&(r, c)| {
                                (r, c) != p1 && (r, c) != p2
                                    && sees(r, c, p1.0, p1.1)
                                    && sees(r, c, p2.0, p2.1)
                                    && matches!(grid.get(r, c), CellKind::Empty)
                                    && (state.notes_mask(r, c) & (1 << b)) != 0
                            })
                            .collect();
                        if elim.is_empty() { continue; }
                        let cause = vec![p1, e1, e2, p2];
                        return Some(Hint {
                            cause_cells:    cause,
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

// ── UniqueRectangle ───────────────────────────────────────────────────────────

impl Strategy for UniqueRectangle {
    fn name_en(&self) -> &'static str { "Unique Rectangle" }
    fn name_de(&self) -> &'static str { "Unique Rectangle" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        // UR Type 1: 3 cells of the rectangle have only {a,b} as notes,
        // the 4th cell has {a,b} plus extras → eliminate a and b from the 4th cell.
        // The rectangle spans 2 rows, 2 columns, 2 different boxes.
        for r1 in 0..9 {
            for r2 in (r1 + 1)..9 {
                // Rows must be in different bands
                if r1 / 3 == r2 / 3 { continue; }
                for c1 in 0..9 {
                    for c2 in (c1 + 1)..9 {
                        // Columns must be in different stacks
                        if c1 / 3 == c2 / 3 { continue; }
                        let corners = [(r1, c1), (r1, c2), (r2, c1), (r2, c2)];
                        // All corners must be empty
                        if corners.iter().any(|&(r, c)| {
                            !matches!(grid.get(r, c), CellKind::Empty)
                        }) {
                            continue;
                        }
                        let masks: [u16; 4] = corners.map(|(r, c)| state.notes_mask(r, c));
                        // Collect the pair mask that appears in at least 3 corners with count == 2
                        let pair_mask_opt = masks.iter()
                            .find(|&&m| m.count_ones() == 2)
                            .copied();
                        let pm = match pair_mask_opt { Some(p) => p, None => continue };
                        // At least 3 corners must have pm as a subset
                        let has_pm: Vec<usize> = masks.iter()
                            .enumerate()
                            .filter(|(_, &m)| (m & pm) == pm)
                            .map(|(i, _)| i)
                            .collect();
                        if has_pm.len() < 3 { continue; }
                        // Exactly 3 corners have exactly pm; the 4th has pm + extras
                        let floor: Vec<usize> = has_pm.iter()
                            .filter(|&&i| masks[i] == pm)
                            .copied()
                            .collect();
                        let roof: Vec<usize> = has_pm.iter()
                            .filter(|&&i| masks[i] != pm)
                            .copied()
                            .collect();
                        if floor.len() != 3 || roof.len() != 1 { continue; }
                        let ri = roof[0];
                        let (rr, rc) = corners[ri];
                        let d1 = pm.trailing_zeros() as u8;
                        let d2 = (pm >> (d1 as u32 + 1)).trailing_zeros() as u8 + d1 + 1;
                        return Some(Hint {
                            cause_cells:    floor.iter().map(|&i| corners[i]).collect(),
                            elim_cells:     vec![(rr, rc)],
                            target_cell:    (rr, rc),
                            elim_digit:     Some(d1),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!(
                                "Unique Rectangle: {}/{} would cause multiple solutions. Remove {}/{} from notes in highlighted cell.",
                                d1, d2, d1, d2
                            ),
                            explanation_de: format!(
                                "Unique Rectangle: {}/{} w\u{fc}rde mehrere L\u{f6}sungen erzeugen. {}/{} aus der markierten Zelle streichen.",
                                d1, d2, d1, d2
                            ),
                        });
                    }
                }
            }
        }
        None
    }
}

// ── BugPlusOne ────────────────────────────────────────────────────────────────

impl Strategy for BugPlusOne {
    fn name_en(&self) -> &'static str { "BUG+1" }
    fn name_de(&self) -> &'static str { "BUG+1" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        // Collect empty cells and their note counts
        let mut trivalue_cell: Option<(usize, usize)> = None;
        for r in 0..9 {
            for c in 0..9 {
                if !matches!(grid.get(r, c), CellKind::Empty) { continue; }
                let n = state.notes_mask(r, c).count_ones();
                if n == 0 { return None; } // notes not filled
                if n == 3 {
                    if trivalue_cell.is_some() { return None; } // more than one → not BUG+1
                    trivalue_cell = Some((r, c));
                } else if n != 2 {
                    return None; // not bivalue
                }
            }
        }
        let (br, bc) = trivalue_cell?;
        let mask = state.notes_mask(br, bc);
        let digits: Vec<u8> = (1u8..=9).filter(|&d| mask & (1 << d) != 0).collect();
        if digits.len() != 3 { return None; }

        // Find the digit that appears an odd number of times in row, col, and box
        // (among all notes except the BUG+1 cell itself)
        for &d in &digits {
            let row_count = (0..9)
                .filter(|&c| c != bc
                    && matches!(grid.get(br, c), CellKind::Empty)
                    && (state.notes_mask(br, c) & (1 << d)) != 0)
                .count();
            let col_count = (0..9)
                .filter(|&r| r != br
                    && matches!(grid.get(r, bc), CellKind::Empty)
                    && (state.notes_mask(r, bc) & (1 << d)) != 0)
                .count();
            let box_br = (br / 3) * 3;
            let box_bc = (bc / 3) * 3;
            let box_count = (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (box_br + dr, box_bc + dc)))
                .filter(|&(r, c)| (r, c) != (br, bc)
                    && matches!(grid.get(r, c), CellKind::Empty)
                    && (state.notes_mask(r, c) & (1 << d)) != 0)
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
                        "BUG+1: placing {} here restores bivalue balance everywhere.",
                        d
                    ),
                    explanation_de: format!(
                        "BUG+1: {} hier setzen stellt \u{fc}berall Bivalue-Balance wieder her.",
                        d
                    ),
                });
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
    fn notes_validator_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(NotesValidator.find(&state, &sol).is_none());
    }

    #[test]
    fn naked_triples_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(NakedTriples.find(&state, &sol).is_none());
    }

    #[test]
    fn hidden_triples_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(HiddenTriples.find(&state, &sol).is_none());
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
    fn jellyfish_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(Jellyfish.find(&state, &sol).is_none());
    }

    #[test]
    fn skyscraper_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(Skyscraper.find(&state, &sol).is_none());
    }

    #[test]
    fn two_string_kite_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(TwoStringKite.find(&state, &sol).is_none());
    }

    #[test]
    fn y_wing_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(YWing.find(&state, &sol).is_none());
    }

    #[test]
    fn xyz_wing_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(XYZWing.find(&state, &sol).is_none());
    }

    #[test]
    fn w_wing_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(WWing.find(&state, &sol).is_none());
    }

    #[test]
    fn unique_rectangle_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(UniqueRectangle.find(&state, &sol).is_none());
    }

    #[test]
    fn bug_plus_one_returns_none_without_notes() {
        let state = state_from(PUZZLE);
        let sol = Grid::from_str(SOL).unwrap();
        assert!(BugPlusOne.find(&state, &sol).is_none());
    }
}
