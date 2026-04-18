use serde::{Deserialize, Serialize};
use crate::puzzle::{event::GameEvent, grid::{CellKind, Grid}};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    event: GameEvent,
    prev_cell: CellKind,
    prev_notes: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    grid: Grid,
    #[serde(with = "serde_arrays")]
    notes: [u16; 81],
    undo_stack: Vec<HistoryEntry>,
    redo_stack: Vec<HistoryEntry>,
    pub elapsed_ms: u64,
}

impl GameState {
    pub fn new(grid: Grid) -> Self {
        Self {
            grid,
            notes: [0u16; 81],
            undo_stack: vec![],
            redo_stack: vec![],
            elapsed_ms: 0,
        }
    }

    pub fn grid(&self) -> &Grid { &self.grid }

    /// Returns the 16-bit note bitmask for the cell at `(row, col)`.
    /// Bit `d` (1-indexed) is set when digit `d` is marked as a candidate.
    pub fn notes_mask(&self, row: usize, col: usize) -> u16 {
        self.notes[Self::idx(row, col)]
    }

    fn idx(row: usize, col: usize) -> usize { row * 9 + col }

    pub fn has_note(&self, row: usize, col: usize, digit: u8) -> bool {
        self.notes[Self::idx(row, col)] & (1 << digit) != 0
    }

    pub fn apply(&mut self, event: GameEvent) {
        let (row, col) = match &event {
            GameEvent::SetDigit { row, col, .. } => (*row, *col),
            GameEvent::ClearCell { row, col } => (*row, *col),
            GameEvent::ToggleNote { row, col, .. } => (*row, *col),
        };
        if self.grid.get(row, col).is_given() { return; }
        let prev_cell = self.grid.get(row, col);
        let prev_notes = self.notes[Self::idx(row, col)];
        self.redo_stack.clear();
        match &event {
            GameEvent::SetDigit { row, col, digit } => {
                self.grid.set_filled(*row, *col, *digit);
            }
            GameEvent::ClearCell { row, col } => {
                self.grid.clear(*row, *col);
                self.notes[Self::idx(*row, *col)] = 0;
            }
            GameEvent::ToggleNote { row, col, digit } => {
                self.notes[Self::idx(*row, *col)] ^= 1 << digit;
            }
        }
        self.undo_stack.push(HistoryEntry { event, prev_cell, prev_notes });
    }

    pub fn undo(&mut self) {
        if let Some(entry) = self.undo_stack.pop() {
            let (row, col) = match &entry.event {
                GameEvent::SetDigit { row, col, .. } => (*row, *col),
                GameEvent::ClearCell { row, col } => (*row, *col),
                GameEvent::ToggleNote { row, col, .. } => (*row, *col),
            };
            let idx = Self::idx(row, col);
            match entry.prev_cell {
                CellKind::Empty => self.grid.clear(row, col),
                CellKind::Filled(v) => self.grid.set_filled(row, col, v),
                CellKind::Given(_) => {}
            }
            self.notes[idx] = entry.prev_notes;
            self.redo_stack.push(entry);
        }
    }

    pub fn redo(&mut self) {
        if let Some(entry) = self.redo_stack.pop() {
            let (row, col) = match &entry.event {
                GameEvent::SetDigit { row, col, .. } => (*row, *col),
                GameEvent::ClearCell { row, col } => (*row, *col),
                GameEvent::ToggleNote { row, col, .. } => (*row, *col),
            };
            let prev_cell = self.grid.get(row, col);
            let prev_notes = self.notes[Self::idx(row, col)];
            match &entry.event {
                GameEvent::SetDigit { row, col, digit } => self.grid.set_filled(*row, *col, *digit),
                GameEvent::ClearCell { row, col } => {
                    self.grid.clear(*row, *col);
                    self.notes[Self::idx(*row, *col)] = 0;
                }
                GameEvent::ToggleNote { row, col, digit } => {
                    self.notes[Self::idx(*row, *col)] ^= 1 << digit;
                }
            }
            self.undo_stack.push(HistoryEntry { event: entry.event, prev_cell, prev_notes });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::{event::GameEvent, grid::{CellKind, Grid}};

    const EASY: &str = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    fn easy_state() -> GameState {
        GameState::new(Grid::from_str(EASY).unwrap())
    }

    #[test]
    fn set_digit_on_empty_cell() {
        let mut s = easy_state();
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 4 });
        assert_eq!(s.grid().get(0, 2), CellKind::Filled(4));
    }

    #[test]
    fn set_digit_ignores_given() {
        let mut s = easy_state();
        s.apply(GameEvent::SetDigit { row: 0, col: 0, digit: 9 }); // Given(5)
        assert_eq!(s.grid().get(0, 0), CellKind::Given(5));
    }

    #[test]
    fn undo_set_digit() {
        let mut s = easy_state();
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 4 });
        s.undo();
        assert_eq!(s.grid().get(0, 2), CellKind::Empty);
    }

    #[test]
    fn redo_after_undo() {
        let mut s = easy_state();
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 4 });
        s.undo();
        s.redo();
        assert_eq!(s.grid().get(0, 2), CellKind::Filled(4));
    }

    #[test]
    fn new_action_clears_redo() {
        let mut s = easy_state();
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 4 });
        s.undo();
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 6 });
        s.redo(); // nothing to redo
        assert_eq!(s.grid().get(0, 2), CellKind::Filled(6));
    }

    #[test]
    fn toggle_note_on_off() {
        let mut s = easy_state();
        s.apply(GameEvent::ToggleNote { row: 0, col: 2, digit: 4 });
        assert!(s.has_note(0, 2, 4));
        s.apply(GameEvent::ToggleNote { row: 0, col: 2, digit: 4 });
        assert!(!s.has_note(0, 2, 4));
    }

    #[test]
    fn undo_toggle_note() {
        let mut s = easy_state();
        s.apply(GameEvent::ToggleNote { row: 0, col: 2, digit: 4 });
        s.undo();
        assert!(!s.has_note(0, 2, 4));
    }

    #[test]
    fn clear_cell_removes_digit_and_notes() {
        let mut s = easy_state();
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 4 });
        s.apply(GameEvent::ToggleNote { row: 0, col: 2, digit: 5 });
        s.apply(GameEvent::ClearCell { row: 0, col: 2 });
        assert_eq!(s.grid().get(0, 2), CellKind::Empty);
        assert!(!s.has_note(0, 2, 5));
    }

    #[test]
    fn undo_clear_restores_digit_and_notes() {
        let mut s = easy_state();
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 4 });
        s.apply(GameEvent::ToggleNote { row: 0, col: 2, digit: 5 });
        s.apply(GameEvent::ClearCell { row: 0, col: 2 });
        s.undo();
        assert_eq!(s.grid().get(0, 2), CellKind::Filled(4));
        assert!(s.has_note(0, 2, 5));
    }

    #[test]
    fn serialization_round_trip() {
        let mut s = easy_state();
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 4 });
        s.elapsed_ms = 12345;
        let json = serde_json::to_string(&s).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();
        assert_eq!(s.grid().to_str(), restored.grid().to_str());
        assert_eq!(restored.elapsed_ms, 12345);
    }
}
