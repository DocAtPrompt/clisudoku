// tests/hint_consistency.rs
//
// End-to-end consistency checks for the hint system: at every reachable game
// state, any hint returned by find_hint must agree with the known solution —
//   * a placement hint (target_digit) must name the solution digit, and
//   * an elimination hint (elim_digit) must never eliminate the solution
//     digit of any highlighted elimination cell.

use clisudoku::hint;
use clisudoku::puzzle::{CellKind, GameEvent, GameState, Grid};

const PUZZLE: &str =
    "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
const SOLUTION: &str =
    "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

/// Grid candidates for (r, c): digits not placed in the same row, col, or box.
fn candidates(grid: &Grid, r: usize, c: usize) -> u16 {
    if !matches!(grid.get(r, c), CellKind::Empty) {
        return 0;
    }
    let mut used = 0u16;
    for i in 0..9 {
        if let Some(d) = grid.get(r, i).value() {
            used |= 1 << d;
        }
        if let Some(d) = grid.get(i, c).value() {
            used |= 1 << d;
        }
    }
    let (br, bc) = ((r / 3) * 3, (c / 3) * 3);
    for dr in 0..3 {
        for dc in 0..3 {
            if let Some(d) = grid.get(br + dr, bc + dc).value() {
                used |= 1 << d;
            }
        }
    }
    0b1111111110 & !used
}

/// Bring every empty cell's notes to exactly `desired(r, c)` via ToggleNote.
fn sync_notes(state: &mut GameState, desired: impl Fn(&Grid, usize, usize) -> u16) {
    for r in 0..9 {
        for c in 0..9 {
            let want = desired(state.grid(), r, c);
            let have = state.notes_mask(r, c);
            let diff = want ^ have;
            for d in 1u8..=9 {
                if diff & (1 << d) != 0 {
                    state.apply(GameEvent::ToggleNote { row: r, col: c, digit: d });
                }
            }
        }
    }
}

/// Assert that `hint` is consistent with `sol`, then advance the game by one
/// correct move. Returns the strategy name for diagnostics.
fn check_hint_and_advance(state: &mut GameState, sol: &Grid) {
    let h = hint::find_hint(state, sol)
        .expect("a hint strategy should fire on every unsolved easy-puzzle state");

    if let Some(d) = h.target_digit {
        let (r, c) = h.target_cell;
        assert_eq!(
            sol.get(r, c).value(),
            Some(d),
            "{}: placement hint suggests {} at ({},{}), but the solution digit differs",
            h.name_en, d, r, c
        );
    }
    if let Some(d) = h.elim_digit {
        for &(r, c) in &h.elim_cells {
            assert_ne!(
                sol.get(r, c).value(),
                Some(d),
                "{}: elimination hint removes {} at ({},{}), but that IS the solution digit",
                h.name_en, d, r, c
            );
        }
    }

    // Advance: fill the first empty cell with its solution digit.
    'advance: for r in 0..9 {
        for c in 0..9 {
            if matches!(state.grid().get(r, c), CellKind::Empty) {
                let d = sol.get(r, c).value().unwrap();
                state.apply(GameEvent::SetDigit { row: r, col: c, digit: d });
                break 'advance;
            }
        }
    }
}

#[test]
fn hints_agree_with_solution_on_full_notes() {
    let mut state = GameState::new(Grid::from_str(PUZZLE).unwrap());
    let sol = Grid::from_str(SOLUTION).unwrap();

    let mut guard = 0;
    while !state.grid().is_solved() {
        guard += 1;
        assert!(guard <= 81, "walk did not terminate");
        sync_notes(&mut state, candidates);
        check_hint_and_advance(&mut state, &sol);
    }
}

#[test]
fn hints_agree_with_solution_on_narrowed_notes() {
    // Simulates a player who followed elimination hints: notes are the grid
    // candidates MINUS one non-solution candidate per cell (where possible).
    // The validator must not object, and every hint must stay solution-correct.
    let mut state = GameState::new(Grid::from_str(PUZZLE).unwrap());
    let sol = Grid::from_str(SOLUTION).unwrap();

    let mut guard = 0;
    while !state.grid().is_solved() {
        guard += 1;
        assert!(guard <= 81, "walk did not terminate");
        let narrowed = |grid: &Grid, r: usize, c: usize| -> u16 {
            let mut mask = candidates(grid, r, c);
            if mask.count_ones() >= 3 {
                let sol_d = sol.get(r, c).value().unwrap();
                // Drop the highest non-solution candidate.
                for d in (1u8..=9).rev() {
                    if d != sol_d && mask & (1 << d) != 0 {
                        mask &= !(1 << d);
                        break;
                    }
                }
            }
            mask
        };
        sync_notes(&mut state, narrowed);
        check_hint_and_advance(&mut state, &sol);
    }
}
