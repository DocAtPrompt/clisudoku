// src/hint/strategies/tier1.rs
use crate::hint::{Hint, Strategy};
use crate::puzzle::game_state::GameState;
use crate::puzzle::{CellKind, Grid};

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Compute the set of valid candidates for cell (r,c) as a bitmask.
/// Bit d is set if digit d (1-9) is not present in the same row, col, or box.
/// Returns 0 if the cell is not empty.
fn candidates(grid: &Grid, r: usize, c: usize) -> u16 {
    if !matches!(grid.get(r, c), CellKind::Empty) {
        return 0;
    }
    let mut used = 0u16;
    for cc in 0..9 {
        if let Some(d) = grid.get(r, cc).value() {
            used |= 1 << d;
        }
    }
    for rr in 0..9 {
        if let Some(d) = grid.get(rr, c).value() {
            used |= 1 << d;
        }
    }
    let br = (r / 3) * 3;
    let bc = (c / 3) * 3;
    for dr in 0..3 {
        for dc in 0..3 {
            if let Some(d) = grid.get(br + dr, bc + dc).value() {
                used |= 1 << d;
            }
        }
    }
    let all: u16 = 0b1111111110; // bits 1..=9
    all & !used
}

/// Iterate the 27 units (9 rows, 9 cols, 9 boxes) as lists of (row,col).
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
        ); // box
    }
    units
}

// ── Strategy structs ──────────────────────────────────────────────────────────

pub struct FullHouse;
pub struct NakedSingle;
pub struct HiddenSingle;
pub struct NotesHint;
pub struct NotesValidator;
pub struct NakedPairs;
pub struct HiddenPairs;
pub struct PointingPairs;
pub struct BoxLineReduction;

// ── Implementations ───────────────────────────────────────────────────────────

impl Strategy for FullHouse {
    fn name_en(&self) -> &'static str {
        "Full House"
    }
    fn name_de(&self) -> &'static str {
        "Full House"
    }

    fn find(&self, state: &GameState, solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let empty: Vec<(usize, usize)> = unit
                .iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .copied()
                .collect();
            if empty.len() == 1 {
                let (r, c) = empty[0];
                let d = solution.get(r, c).value()?;
                return Some(Hint {
                    cause_cells: vec![],
                    elim_cells: vec![],
                    target_cell: (r, c),
                    elim_digit: None,
                    target_digit: Some(d),
                    name_en: self.name_en(),
                    name_de: self.name_de(),
                    explanation_en: format!("Last empty cell in this unit — place {} here.", d),
                    explanation_de: format!("Letzte leere Zelle hier — {} eintragen.", d),
                });
            }
        }
        None
    }
}

impl Strategy for NakedSingle {
    fn name_en(&self) -> &'static str {
        "Naked Single"
    }
    fn name_de(&self) -> &'static str {
        "Naked Single"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for r in 0..9 {
            for c in 0..9 {
                let cands = candidates(grid, r, c);
                if cands != 0 && cands.count_ones() == 1 {
                    let d = cands.trailing_zeros() as u8;
                    return Some(Hint {
                        cause_cells: vec![],
                        elim_cells: vec![],
                        target_cell: (r, c),
                        elim_digit: None,
                        target_digit: Some(d),
                        name_en: self.name_en(),
                        name_de: self.name_de(),
                        explanation_en: format!(
                            "All other digits are blocked — {} is the only fit here.",
                            d
                        ),
                        explanation_de: format!(
                            "Alle anderen Ziffern blockiert — nur {} passt hier.",
                            d
                        ),
                    });
                }
            }
        }
        None
    }
}

impl Strategy for HiddenSingle {
    fn name_en(&self) -> &'static str {
        "Hidden Single"
    }
    fn name_de(&self) -> &'static str {
        "Hidden Single"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            for digit in 1u8..=9 {
                let positions: Vec<(usize, usize)> = unit
                    .iter()
                    .filter(|&&(r, c)| {
                        matches!(grid.get(r, c), CellKind::Empty)
                            && (candidates(grid, r, c) & (1 << digit)) != 0
                    })
                    .copied()
                    .collect();
                if positions.len() == 1 {
                    let (r, c) = positions[0];
                    // Collect cells that already contain `digit` and "see" the
                    // other empty cells in this unit — showing WHY only (r, c)
                    // can hold the digit (all other positions are blocked).
                    let mut cause_set: std::collections::HashSet<(usize, usize)> =
                        std::collections::HashSet::new();
                    for &(rr, cc) in &unit {
                        if (rr, cc) == (r, c) {
                            continue;
                        }
                        if !matches!(grid.get(rr, cc), CellKind::Empty) {
                            continue;
                        }
                        // Same row
                        for col in 0..9 {
                            if grid.get(rr, col).value() == Some(digit) {
                                cause_set.insert((rr, col));
                            }
                        }
                        // Same column
                        for row in 0..9 {
                            if grid.get(row, cc).value() == Some(digit) {
                                cause_set.insert((row, cc));
                            }
                        }
                        // Same box
                        let br = (rr / 3) * 3;
                        let bc = (cc / 3) * 3;
                        for dr in 0..3 {
                            for dc in 0..3 {
                                if grid.get(br + dr, bc + dc).value() == Some(digit) {
                                    cause_set.insert((br + dr, bc + dc));
                                }
                            }
                        }
                    }
                    let cause: Vec<(usize, usize)> = cause_set.into_iter().collect();
                    return Some(Hint {
                        cause_cells: cause,
                        elim_cells: vec![],
                        target_cell: (r, c),
                        elim_digit: None,
                        target_digit: Some(digit),
                        name_en: self.name_en(),
                        name_de: self.name_de(),
                        explanation_en: format!(
                            "{} has only one possible cell in this unit — place it here.",
                            digit
                        ),
                        explanation_de: format!(
                            "{} passt in dieser Einheit nur hier — hier eintragen.",
                            digit
                        ),
                    });
                }
            }
        }
        None
    }
}

impl Strategy for NotesHint {
    fn name_en(&self) -> &'static str {
        "Add Notes"
    }
    fn name_de(&self) -> &'static str {
        "Notizen erg\u{e4}nzen"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        let units = all_units();
        let mut best_unit: Option<Vec<(usize, usize)>> = None;
        let mut best_empty = usize::MAX;
        let mut best_is_box = false;

        for (i, unit) in units.iter().enumerate() {
            // In all_units(), for each i in 0..9 we push: row (i*3), col (i*3+1), box (i*3+2)
            // So boxes are at indices where i % 3 == 2
            let is_box = i % 3 == 2;
            // Does this unit have at least one empty cell with zero notes?
            let has_empty_no_notes = unit.iter().any(|&(r, c)| {
                matches!(grid.get(r, c), CellKind::Empty) && state.notes_mask(r, c) == 0
            });
            if !has_empty_no_notes {
                continue;
            }

            let total_empty = unit
                .iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .count();
            if total_empty == 0 {
                continue;
            }

            let better = best_unit.is_none()
                || (is_box && !best_is_box)
                || (is_box == best_is_box && total_empty < best_empty);
            if better {
                best_unit = Some(unit.clone());
                best_empty = total_empty;
                best_is_box = is_box;
            }
        }

        let unit = best_unit?;
        // target_cell = most constrained empty cell in unit (fewest candidates)
        let target = unit
            .iter()
            .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
            .min_by_key(|&&(r, c)| candidates(grid, r, c).count_ones())
            .copied()?;

        Some(Hint {
            cause_cells: unit
                .iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty) && (r, c) != target)
                .copied()
                .collect(),
            elim_cells: vec![],
            target_cell: target,
            elim_digit: None,
            target_digit: None,
            name_en: self.name_en(),
            name_de: self.name_de(),
            explanation_en:
                "Note all possible digits in the empty cells here to find your next move."
                    .to_string(),
            explanation_de:
                "Alle m\u{f6}glichen Ziffern in die leeren Zellen dieser Einheit eintragen."
                    .to_string(),
        })
    }
}

// ── Notes Validator ───────────────────────────────────────────────────────────
//
// Fires when any empty cell with at least one note contains:
//   Pass 1 — a WRONG note   (digit noted but cannot go in this cell)
//   Pass 2 — a MISSING note (valid candidate absent from the notes)
//
// Both checks use actual grid candidates so they are independent of whether the
// player has filled in all their notes correctly elsewhere.
//
// Placed between NotesHint and NakedPairs so that note-dependent strategies
// always operate on verified, complete candidate sets.

impl Strategy for NotesValidator {
    fn name_en(&self) -> &'static str {
        "Fix Notes"
    }
    fn name_de(&self) -> &'static str {
        "Notizen korrigieren"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();

        // Pass 1: wrong notes — digit is noted but doesn't fit (already in row/col/box).
        for r in 0..9 {
            for c in 0..9 {
                if !matches!(grid.get(r, c), CellKind::Empty) {
                    continue;
                }
                let notes = state.notes_mask(r, c);
                if notes == 0 {
                    continue;
                }
                let actual = candidates(grid, r, c);
                let wrong = notes & !actual;
                if wrong == 0 {
                    continue;
                }

                let d = wrong.trailing_zeros() as u8;
                // Cause: peer cells that already contain d, explaining the conflict.
                let mut cause: Vec<(usize, usize)> = vec![];
                for cc in 0..9 {
                    if grid.get(r, cc).value() == Some(d) {
                        cause.push((r, cc));
                    }
                }
                for rr in 0..9 {
                    if grid.get(rr, c).value() == Some(d) {
                        cause.push((rr, c));
                    }
                }
                let (br, bc) = ((r / 3) * 3, (c / 3) * 3);
                for dr in 0..3 {
                    for dc in 0..3 {
                        if grid.get(br + dr, bc + dc).value() == Some(d) {
                            cause.push((br + dr, bc + dc));
                        }
                    }
                }
                cause.sort_unstable();
                cause.dedup();

                return Some(Hint {
                    cause_cells: cause,
                    elim_cells: vec![(r, c)],
                    target_cell: (r, c),
                    elim_digit: Some(d),
                    target_digit: None,
                    name_en: "Wrong Note",
                    name_de: "Falsche Notiz",
                    explanation_en: format!(
                        "{} already appears in this row, col, or box — remove this note.",
                        d
                    ),
                    explanation_de: format!(
                        "{} ist schon in Zeile, Spalte oder Box — diese Notiz streichen.",
                        d
                    ),
                });
            }
        }

        // Pass 2: missing notes — valid candidate absent from an already-started cell.
        for r in 0..9 {
            for c in 0..9 {
                if !matches!(grid.get(r, c), CellKind::Empty) {
                    continue;
                }
                let notes = state.notes_mask(r, c);
                if notes == 0 {
                    continue;
                } // NotesHint handles zero-note cells
                let actual = candidates(grid, r, c);
                let missing = actual & !notes;
                if missing == 0 {
                    continue;
                }

                let d = missing.trailing_zeros() as u8;
                return Some(Hint {
                    cause_cells: vec![],
                    elim_cells: vec![],
                    target_cell: (r, c),
                    elim_digit: None,
                    target_digit: Some(d),
                    name_en: "Missing Note",
                    name_de: "Fehlende Notiz",
                    explanation_en: format!(
                        "{} is a valid candidate here — add it to the notes in this cell.",
                        d
                    ),
                    explanation_de: format!(
                        "{} ist hier m\u{f6}glich — zur Notiz hinzuf\u{fc}gen.",
                        d
                    ),
                });
            }
        }

        None
    }
}

impl Strategy for NakedPairs {
    fn name_en(&self) -> &'static str {
        "Naked Pairs"
    }
    fn name_de(&self) -> &'static str {
        "Naked Pairs"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let empties: Vec<(usize, usize)> = unit
                .iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .copied()
                .collect();
            for i in 0..empties.len() {
                let (r1, c1) = empties[i];
                let m1 = state.notes_mask(r1, c1);
                if m1.count_ones() != 2 {
                    continue;
                }
                for j in (i + 1)..empties.len() {
                    let (r2, c2) = empties[j];
                    if state.notes_mask(r2, c2) != m1 {
                        continue;
                    }
                    // Found a naked pair — extract the two digits from the mask
                    let d1 = m1.trailing_zeros() as u8;
                    let remaining = m1 >> (d1 as u32 + 1);
                    let d2 = d1 + 1 + remaining.trailing_zeros() as u8;
                    let elim: Vec<(usize, usize)> = empties
                        .iter()
                        .filter(|&&(r, c)| {
                            (r, c) != (r1, c1)
                                && (r, c) != (r2, c2)
                                && (state.notes_mask(r, c) & m1) != 0
                        })
                        .copied()
                        .collect();
                    if elim.is_empty() {
                        continue;
                    }
                    let target = elim[0];
                    return Some(Hint {
                        cause_cells:    vec![(r1, c1), (r2, c2)],
                        elim_cells:     elim,
                        target_cell:    target,
                        elim_digit:     Some(d1),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!("These 2 cells can only hold {} or {}. Remove both from notes in highlighted cells.", d1, d2),
                        explanation_de: format!("Diese 2 Zellen k\u{f6}nnen nur {} oder {} enthalten. Beide aus den markierten Zellen streichen.", d1, d2),
                    });
                }
            }
        }
        None
    }
}
impl Strategy for HiddenPairs {
    fn name_en(&self) -> &'static str {
        "Hidden Pairs"
    }
    fn name_de(&self) -> &'static str {
        "Hidden Pairs"
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
                    // Find cells in this unit where d1 or d2 appear in notes
                    let pair_cells: Vec<(usize, usize)> = empties
                        .iter()
                        .filter(|&&(r, c)| {
                            let m = state.notes_mask(r, c);
                            (m & (1 << d1)) != 0 || (m & (1 << d2)) != 0
                        })
                        .copied()
                        .collect();
                    if pair_cells.len() != 2 {
                        continue;
                    }
                    // Both d1 AND d2 must appear in BOTH cells
                    let both_digits = pair_cells.iter().all(|&(r, c)| {
                        let m = state.notes_mask(r, c);
                        (m & (1 << d1)) != 0 && (m & (1 << d2)) != 0
                    });
                    if !both_digits {
                        continue;
                    }
                    // The pair cells must have extra candidates to eliminate
                    let pair_mask = (1u16 << d1) | (1u16 << d2);
                    let elim: Vec<(usize, usize)> = pair_cells
                        .iter()
                        .filter(|&&(r, c)| state.notes_mask(r, c) & !pair_mask != 0)
                        .copied()
                        .collect();
                    if elim.is_empty() {
                        continue;
                    }
                    let target = elim[0];
                    return Some(Hint {
                        cause_cells:    pair_cells,
                        elim_cells:     elim,
                        target_cell:    target,
                        elim_digit:     Some(d1),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!("Only these 2 cells can hold {}/{}. Remove all other notes from them.", d1, d2),
                        explanation_de: format!("Nur diese 2 Zellen k\u{f6}nnen {}/{} enthalten. Alle anderen Notizen daraus streichen.", d1, d2),
                    });
                }
            }
        }
        None
    }
}

impl Strategy for PointingPairs {
    fn name_en(&self) -> &'static str {
        "Pointing Pairs"
    }
    fn name_de(&self) -> &'static str {
        "Pointing Pairs"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for box_idx in 0..9usize {
            let br = (box_idx / 3) * 3;
            let bc = (box_idx % 3) * 3;
            let box_cells: Vec<(usize, usize)> = (0..3)
                .flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc)))
                .collect();
            for digit in 1u8..=9 {
                // Use ACTUAL candidates (grid constraints) — not player notes — to
                // determine where the digit can go within the box.  Using notes here
                // causes false positives when the player hasn't yet noted the digit in
                // every cell where it fits (the strategy would incorrectly conclude the
                // digit is confined to one row/col).
                let cand_cells: Vec<(usize, usize)> = box_cells
                    .iter()
                    .filter(|&&(r, c)| (candidates(grid, r, c) & (1 << digit)) != 0)
                    .copied()
                    .collect();
                if cand_cells.len() < 2 {
                    continue;
                }
                // All in same row?
                let row = cand_cells[0].0;
                if cand_cells.iter().all(|&(r, _)| r == row) {
                    let elim: Vec<(usize, usize)> = (0..9)
                        .filter(|&c| {
                            let cell_box = (row / 3) * 3 + c / 3;
                            cell_box != box_idx
                                && matches!(grid.get(row, c), CellKind::Empty)
                                && (state.notes_mask(row, c) & (1 << digit)) != 0
                        })
                        .map(|c| (row, c))
                        .collect();
                    if !elim.is_empty() {
                        return Some(Hint {
                            cause_cells:    cand_cells,
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(digit),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!("In this box, {} fits only in this row. Remove {} from notes in highlighted cells.", digit, digit),
                            explanation_de: format!("In dieser Box passt {} nur in diese Zeile. {} aus den markierten Zellen streichen.", digit, digit),
                        });
                    }
                }
                // All in same col?
                let col = cand_cells[0].1;
                if cand_cells.iter().all(|&(_, c)| c == col) {
                    let elim: Vec<(usize, usize)> = (0..9)
                        .filter(|&r| {
                            let cell_box = (r / 3) * 3 + col / 3;
                            cell_box != box_idx
                                && matches!(grid.get(r, col), CellKind::Empty)
                                && (state.notes_mask(r, col) & (1 << digit)) != 0
                        })
                        .map(|r| (r, col))
                        .collect();
                    if !elim.is_empty() {
                        return Some(Hint {
                            cause_cells:    cand_cells,
                            elim_cells:     elim.clone(),
                            target_cell:    elim[0],
                            elim_digit:     Some(digit),
                            target_digit:   None,
                            name_en:        self.name_en(),
                            name_de:        self.name_de(),
                            explanation_en: format!("In this box, {} fits only in this column. Remove {} from notes in highlighted cells.", digit, digit),
                            explanation_de: format!("In dieser Box passt {} nur in diese Spalte. {} aus den markierten Zellen streichen.", digit, digit),
                        });
                    }
                }
            }
        }
        None
    }
}

impl Strategy for BoxLineReduction {
    fn name_en(&self) -> &'static str {
        "Box-Line Reduction"
    }
    fn name_de(&self) -> &'static str {
        "Box-Line Reduction"
    }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        // For each row: if all ACTUAL candidates for a digit are in the same box → eliminate from rest of box
        for row in 0..9usize {
            for digit in 1u8..=9 {
                let cand_cols: Vec<usize> = (0..9)
                    .filter(|&c| (candidates(grid, row, c) & (1 << digit)) != 0)
                    .collect();
                if cand_cols.len() < 2 {
                    continue;
                }
                let box_col = cand_cols[0] / 3;
                if !cand_cols.iter().all(|&c| c / 3 == box_col) {
                    continue;
                }
                let br = (row / 3) * 3;
                let bc = box_col * 3;
                let cand_cells: Vec<(usize, usize)> = cand_cols.iter().map(|&c| (row, c)).collect();
                let elim: Vec<(usize, usize)> = (0..3)
                    .flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc)))
                    .filter(|&(r, c)| {
                        r != row
                            && matches!(grid.get(r, c), CellKind::Empty)
                            && (state.notes_mask(r, c) & (1 << digit)) != 0
                    })
                    .collect();
                if !elim.is_empty() {
                    return Some(Hint {
                        cause_cells:    cand_cells,
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!("In this row, {} fits only within one box. Remove {} from other notes in that box.", digit, digit),
                        explanation_de: format!("In dieser Zeile passt {} nur in eine Box. {} aus den anderen Box-Zellen streichen.", digit, digit),
                    });
                }
            }
        }
        // For each col: if all ACTUAL candidates for a digit are in the same box → eliminate from rest of box
        for col in 0..9usize {
            for digit in 1u8..=9 {
                let cand_rows: Vec<usize> = (0..9)
                    .filter(|&r| (candidates(grid, r, col) & (1 << digit)) != 0)
                    .collect();
                if cand_rows.len() < 2 {
                    continue;
                }
                let box_row = cand_rows[0] / 3;
                if !cand_rows.iter().all(|&r| r / 3 == box_row) {
                    continue;
                }
                let br = box_row * 3;
                let bc = (col / 3) * 3;
                let cand_cells: Vec<(usize, usize)> = cand_rows.iter().map(|&r| (r, col)).collect();
                let elim: Vec<(usize, usize)> = (0..3)
                    .flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc)))
                    .filter(|&(r, c)| {
                        c != col
                            && matches!(grid.get(r, c), CellKind::Empty)
                            && (state.notes_mask(r, c) & (1 << digit)) != 0
                    })
                    .collect();
                if !elim.is_empty() {
                    return Some(Hint {
                        cause_cells:    cand_cells,
                        elim_cells:     elim.clone(),
                        target_cell:    elim[0],
                        elim_digit:     Some(digit),
                        target_digit:   None,
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!("In this column, {} fits only within one box. Remove {} from other notes in that box.", digit, digit),
                        explanation_de: format!("In dieser Spalte passt {} nur in eine Box. {} aus den anderen Box-Zellen streichen.", digit, digit),
                    });
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
    use crate::puzzle::{GameState, Grid};

    fn state_from(s: &str) -> GameState {
        let grid = Grid::from_str(s).unwrap();
        GameState::new(grid)
    }

    // Puzzle derived from EASY_SOL with cell (0,8) cleared → Full House in row 0.
    // EASY_SOL = "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    // Cell (0,8) = '2' replaced with '0'.
    const FULL_HOUSE_PUZZLE: &str =
        "534678910672195348198342567859761423426853791713924856961537284287419635345286179";
    const FULL_HOUSE_SOL: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn full_house_finds_last_cell_in_row() {
        let state = state_from(FULL_HOUSE_PUZZLE);
        let sol = Grid::from_str(FULL_HOUSE_SOL).unwrap();
        let hint = FullHouse
            .find(&state, &sol)
            .expect("should find full house");
        assert_eq!(hint.target_cell, (0, 8));
        assert_eq!(hint.target_digit, Some(2));
    }

    // Nearly-solved puzzle: cell (4,5) has only digit 8 as a candidate.
    const NAKED_SINGLE_PUZZLE: &str =
        "123456789456789123789123456214365978365970214897214365531642897642897531978531642";
    const NAKED_SINGLE_SOL: &str =
        "123456789456789123789123456214365978365978214897214365531642897642897531978531642";

    #[test]
    fn naked_single_finds_only_candidate() {
        let state = state_from(NAKED_SINGLE_PUZZLE);
        let sol = Grid::from_str(NAKED_SINGLE_SOL).unwrap();
        let hint = NakedSingle
            .find(&state, &sol)
            .expect("should find naked single");
        assert_eq!(hint.target_digit, Some(8));
        assert!(hint.cause_cells.is_empty());
    }

    // Puzzle with several empty cells in row 0; each has a unique candidate
    // (so they are also naked singles, but hidden single logic must find one).
    const HIDDEN_SINGLE_PUZZLE: &str =
        "023456000456789123789123456214365978365978214897214365531642897642897531978531642";
    const HIDDEN_SINGLE_SOL: &str =
        "123456789456789123789123456214365978365978214897214365531642897642897531978531642";

    #[test]
    fn hidden_single_finds_only_position_in_unit() {
        let state = state_from(HIDDEN_SINGLE_PUZZLE);
        let sol = Grid::from_str(HIDDEN_SINGLE_SOL).unwrap();
        let hint = HiddenSingle.find(&state, &sol);
        assert!(hint.is_some(), "should find a hidden single");
    }

    #[test]
    fn notes_hint_fires_when_empty_cell_has_no_notes() {
        // Any standard puzzle where no notes have been entered yet
        let state = state_from(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        );
        let sol = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        // With no notes set, NotesHint should fire
        let hint = NotesHint.find(&state, &sol);
        assert!(
            hint.is_some(),
            "should find notes hint when notes are missing"
        );
    }

    #[test]
    fn notes_hint_silent_when_all_empty_cells_have_notes() {
        // A nearly-solved puzzle with one empty cell that has a note
        let puzzle =
            "534678912672195348198342567859761423426853791713924856961537284287419635345286170";
        let mut state = state_from(puzzle);
        // Toggle a note for the last empty cell (8,8) — digit 9
        use crate::puzzle::GameEvent;
        state.apply(GameEvent::ToggleNote {
            row: 8,
            col: 8,
            digit: 9,
        });
        let sol = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        // NotesHint should NOT fire — all empty cells have at least one note
        let hint = NotesHint.find(&state, &sol);
        assert!(hint.is_none());
    }

    #[test]
    fn naked_pairs_returns_none_without_notes() {
        // Without notes, NakedPairs cannot fire (it works from notes masks)
        let state = state_from(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        );
        let sol = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        let hint = NakedPairs.find(&state, &sol);
        assert!(hint.is_none());
    }

    #[test]
    fn naked_pairs_finds_pair_and_returns_elimination() {
        use crate::puzzle::GameEvent;
        // Empty board — all cells are CellKind::Empty, no givens.
        let grid = Grid::from_str(&"0".repeat(81)).unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();

        // Row 0: cells (0,5) and (0,6) form a naked pair with notes {1,2}.
        for digit in [1, 2] {
            state.apply(GameEvent::ToggleNote { row: 0, col: 5, digit });
            state.apply(GameEvent::ToggleNote { row: 0, col: 6, digit });
        }
        // Cell (0,7) has notes {1,2,3} — the 1 and 2 should be eliminated.
        for digit in [1, 2, 3] {
            state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit });
        }

        let hint = NakedPairs.find(&state, &sol);
        assert!(hint.is_some(), "NakedPairs should detect the {{1,2}} pair in row 0");
        let h = hint.unwrap();
        assert_eq!(h.name_en, "Naked Pairs");
        assert!(
            h.cause_cells.contains(&(0, 5)) && h.cause_cells.contains(&(0, 6)),
            "cause_cells should be the pair cells; got {:?}", h.cause_cells
        );
        assert!(
            h.elim_cells.contains(&(0, 7)),
            "cell (0,7) should be an elimination target; got {:?}", h.elim_cells
        );
    }

    #[test]
    fn hidden_pairs_returns_none_without_notes() {
        let state = state_from(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        );
        let sol = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        assert!(HiddenPairs.find(&state, &sol).is_none());
    }

    #[test]
    fn hidden_pairs_finds_pair_and_returns_elimination() {
        use crate::puzzle::GameEvent;
        // Row 0 cols 0-6 filled → only cells (0,7) and (0,8) are empty in row 0.
        // Digits 1 and 2 exist only in those two cells → hidden pair.
        // Extra notes (3 and 4 respectively) are the elimination targets.
        let grid = Grid::from_str("345678900000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let mut state = GameState::new(grid);
        let sol = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();

        // (0,7): notes {1,2,3} — 3 is extra and must be eliminated
        for digit in [1, 2, 3] {
            state.apply(GameEvent::ToggleNote { row: 0, col: 7, digit });
        }
        // (0,8): notes {1,2,4} — 4 is extra and must be eliminated
        for digit in [1, 2, 4] {
            state.apply(GameEvent::ToggleNote { row: 0, col: 8, digit });
        }

        let hint = HiddenPairs.find(&state, &sol);
        assert!(hint.is_some(), "HiddenPairs should detect the {{1,2}} hidden pair in row 0");
        let h = hint.unwrap();
        assert_eq!(h.name_en, "Hidden Pairs");
        // Both pair cells must be in elim (they each have extra notes to remove)
        assert!(
            h.elim_cells.contains(&(0, 7)) && h.elim_cells.contains(&(0, 8)),
            "both pair cells should be elimination targets; got {:?}", h.elim_cells
        );
    }

    #[test]
    fn pointing_pairs_returns_none_without_notes() {
        let state = state_from(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        );
        let sol = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        assert!(PointingPairs.find(&state, &sol).is_none());
    }

    #[test]
    fn box_line_reduction_returns_none_without_notes() {
        let state = state_from(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        );
        let sol = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        assert!(BoxLineReduction.find(&state, &sol).is_none());
    }

    // ── NotesValidator tests ──────────────────────────────────────────────────

    // FULL_HOUSE_PUZZLE row 0: 5 3 4 6 7 8 9 1 _  → only digit 2 fits in (0,8).
    // Noting digit 5 there is wrong (5 already in row 0 at col 0).
    #[test]
    fn notes_validator_detects_wrong_note() {
        use crate::puzzle::GameEvent;
        let mut state = state_from(FULL_HOUSE_PUZZLE);
        let sol = Grid::from_str(FULL_HOUSE_SOL).unwrap();
        state.apply(GameEvent::ToggleNote {
            row: 0,
            col: 8,
            digit: 5,
        });
        let hint = NotesValidator.find(&state, &sol);
        assert!(hint.is_some(), "should detect wrong note 5 at (0,8)");
        let h = hint.unwrap();
        assert_eq!(h.target_cell, (0, 8));
        assert_eq!(h.elim_digit, Some(5));
        assert_eq!(h.name_en, "Wrong Note");
        // Cause must include the cell that already contains 5
        assert!(
            !h.cause_cells.is_empty(),
            "cause cells should explain the conflict"
        );
    }

    // Cell (0,8) in FULL_HOUSE_PUZZLE can only hold 2.
    // Note digit 1 (which is also wrong) — then note the correct digit 2.
    // After removing the wrong note, only 2 is noted and it matches actual → no violation.
    #[test]
    fn notes_validator_silent_when_notes_correct_and_complete() {
        use crate::puzzle::GameEvent;
        let mut state = state_from(FULL_HOUSE_PUZZLE);
        let sol = Grid::from_str(FULL_HOUSE_SOL).unwrap();
        // Only the correct digit noted
        state.apply(GameEvent::ToggleNote {
            row: 0,
            col: 8,
            digit: 2,
        });
        assert!(
            NotesValidator.find(&state, &sol).is_none(),
            "no violation when correct digit is noted"
        );
    }

    // Cell (0,2) in the medium puzzle can hold {1,2,4}.
    // Noting only digit 1 leaves 2 and 4 missing.
    #[test]
    fn notes_validator_detects_missing_note() {
        use crate::puzzle::GameEvent;
        const MEDIUM: &str =
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
        const MEDIUM_SOL: &str =
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179";
        let mut state = state_from(MEDIUM);
        let sol = Grid::from_str(MEDIUM_SOL).unwrap();
        // Note only digit 1 in cell (0,2) — valid candidates are {1,2,4}, so 2 and 4 are missing
        state.apply(GameEvent::ToggleNote {
            row: 0,
            col: 2,
            digit: 1,
        });
        let hint = NotesValidator.find(&state, &sol);
        assert!(hint.is_some(), "should detect missing note in (0,2)");
        let h = hint.unwrap();
        assert_eq!(h.target_cell, (0, 2));
        assert_eq!(h.name_en, "Missing Note");
        assert!(h.target_digit.is_some());
    }

    // Without any notes, NotesValidator must not fire (NotesHint handles that case).
    #[test]
    fn notes_validator_returns_none_without_notes() {
        let state = state_from(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        );
        let sol = Grid::from_str(
            "534678912672195348198342567859761423426853791713924856961537284287419635345286179",
        )
        .unwrap();
        assert!(NotesValidator.find(&state, &sol).is_none());
    }
}
