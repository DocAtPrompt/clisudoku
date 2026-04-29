// src/hint/strategies/tier1.rs
use crate::hint::{Hint, Strategy};
use crate::puzzle::{CellKind, Grid};
use crate::puzzle::game_state::GameState;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Compute the set of valid candidates for cell (r,c) as a bitmask.
/// Bit d is set if digit d (1-9) is not present in the same row, col, or box.
/// Returns 0 if the cell is not empty.
fn candidates(grid: &Grid, r: usize, c: usize) -> u16 {
    if !matches!(grid.get(r, c), CellKind::Empty) { return 0; }
    let mut used = 0u16;
    for cc in 0..9 { if let Some(d) = grid.get(r, cc).value() { used |= 1 << d; } }
    for rr in 0..9 { if let Some(d) = grid.get(rr, c).value() { used |= 1 << d; } }
    let br = (r / 3) * 3;
    let bc = (c / 3) * 3;
    for dr in 0..3 { for dc in 0..3 {
        if let Some(d) = grid.get(br + dr, bc + dc).value() { used |= 1 << d; }
    }}
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
        units.push((0..3).flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc))).collect()); // box
    }
    units
}

// ── Strategy structs ──────────────────────────────────────────────────────────

pub struct FullHouse;
pub struct NakedSingle;
pub struct HiddenSingle;
pub struct NotesHint;
pub struct NakedPairs;
pub struct HiddenPairs;
pub struct PointingPairs;
pub struct BoxLineReduction;

// ── Implementations ───────────────────────────────────────────────────────────

impl Strategy for FullHouse {
    fn name_en(&self) -> &'static str { "Full House" }
    fn name_de(&self) -> &'static str { "Full House" }

    fn find(&self, state: &GameState, solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            let empty: Vec<(usize, usize)> = unit.iter()
                .filter(|&&(r, c)| matches!(grid.get(r, c), CellKind::Empty))
                .copied().collect();
            if empty.len() == 1 {
                let (r, c) = empty[0];
                let d = solution.get(r, c).value()?;
                return Some(Hint {
                    cause_cells:    vec![],
                    elim_cells:     vec![],
                    target_cell:    (r, c),
                    elim_digit:     None,
                    target_digit:   Some(d),
                    name_en:        self.name_en(),
                    name_de:        self.name_de(),
                    explanation_en: "Only one empty cell remains in this unit.".to_string(),
                    explanation_de: "Nur eine leere Zelle bleibt in dieser Einheit.".to_string(),
                });
            }
        }
        None
    }
}

impl Strategy for NakedSingle {
    fn name_en(&self) -> &'static str { "Naked Single" }
    fn name_de(&self) -> &'static str { "Naked Single" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for r in 0..9 {
            for c in 0..9 {
                let cands = candidates(grid, r, c);
                if cands != 0 && cands.count_ones() == 1 {
                    let d = cands.trailing_zeros() as u8;
                    return Some(Hint {
                        cause_cells:    vec![],
                        elim_cells:     vec![],
                        target_cell:    (r, c),
                        elim_digit:     None,
                        target_digit:   Some(d),
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!("Only {} fits in this cell.", d),
                        explanation_de: format!("Nur {} passt in diese Zelle.", d),
                    });
                }
            }
        }
        None
    }
}

impl Strategy for HiddenSingle {
    fn name_en(&self) -> &'static str { "Hidden Single" }
    fn name_de(&self) -> &'static str { "Hidden Single" }

    fn find(&self, state: &GameState, _solution: &Grid) -> Option<Hint> {
        let grid = state.grid();
        for unit in all_units() {
            for digit in 1u8..=9 {
                let positions: Vec<(usize, usize)> = unit.iter()
                    .filter(|&&(r, c)| {
                        matches!(grid.get(r, c), CellKind::Empty)
                            && (candidates(grid, r, c) & (1 << digit)) != 0
                    })
                    .copied().collect();
                if positions.len() == 1 {
                    let (r, c) = positions[0];
                    let cause: Vec<(usize, usize)> = unit.iter()
                        .filter(|&&(rr, cc)| (rr, cc) != (r, c) && grid.get(rr, cc).value().is_some())
                        .copied().collect();
                    return Some(Hint {
                        cause_cells:    cause,
                        elim_cells:     vec![],
                        target_cell:    (r, c),
                        elim_digit:     None,
                        target_digit:   Some(digit),
                        name_en:        self.name_en(),
                        name_de:        self.name_de(),
                        explanation_en: format!("{} can only go here in this unit.", digit),
                        explanation_de: format!("{} kann nur hier in dieser Einheit stehen.", digit),
                    });
                }
            }
        }
        None
    }
}

impl Strategy for NotesHint {
    fn name_en(&self) -> &'static str { "Add Notes" }
    fn name_de(&self) -> &'static str { "Notizen erg\u{e4}nzen" }
    fn find(&self, _state: &GameState, _solution: &Grid) -> Option<Hint> { None }
}
impl Strategy for NakedPairs {
    fn name_en(&self) -> &'static str { "Naked Pairs" }
    fn name_de(&self) -> &'static str { "Naked Pairs" }
    fn find(&self, _state: &GameState, _solution: &Grid) -> Option<Hint> { None }
}
impl Strategy for HiddenPairs {
    fn name_en(&self) -> &'static str { "Hidden Pairs" }
    fn name_de(&self) -> &'static str { "Hidden Pairs" }
    fn find(&self, _state: &GameState, _solution: &Grid) -> Option<Hint> { None }
}
impl Strategy for PointingPairs {
    fn name_en(&self) -> &'static str { "Pointing Pairs" }
    fn name_de(&self) -> &'static str { "Pointing Pairs" }
    fn find(&self, _state: &GameState, _solution: &Grid) -> Option<Hint> { None }
}
impl Strategy for BoxLineReduction {
    fn name_en(&self) -> &'static str { "Box-Line Reduction" }
    fn name_de(&self) -> &'static str { "Box-Line Reduction" }
    fn find(&self, _state: &GameState, _solution: &Grid) -> Option<Hint> { None }
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
        let hint = FullHouse.find(&state, &sol).expect("should find full house");
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
        let hint = NakedSingle.find(&state, &sol).expect("should find naked single");
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
}
