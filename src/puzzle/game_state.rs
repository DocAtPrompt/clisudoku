use serde::{Deserialize, Serialize};
use crate::puzzle::{event::GameEvent, grid::{CellKind, Grid}};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    event: GameEvent,
    prev_cell: CellKind,
    prev_notes: u16,
    /// Notes cleared from peer cells (same row/col/box) as a side-effect of SetDigit.
    /// Each entry is `(cell_index, old_notes_mask)` for cells where the placed digit
    /// was present as a note. Restored verbatim on undo.
    peer_note_changes: Vec<(usize, u16)>,
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

    /// All cell indices in the same row, column, and 3×3 box as `(row, col)`,
    /// excluding `(row, col)` itself.
    fn peer_indices(row: usize, col: usize) -> impl Iterator<Item = usize> {
        let mut seen = [false; 81];
        let mut peers = Vec::with_capacity(20);
        let origin = Self::idx(row, col);
        // Row
        for c in 0..9 {
            let i = row * 9 + c;
            if i != origin && !seen[i] { seen[i] = true; peers.push(i); }
        }
        // Column
        for r in 0..9 {
            let i = r * 9 + col;
            if i != origin && !seen[i] { seen[i] = true; peers.push(i); }
        }
        // Box
        let br = (row / 3) * 3;
        let bc = (col / 3) * 3;
        for dr in 0..3 { for dc in 0..3 {
            let i = (br + dr) * 9 + (bc + dc);
            if i != origin && !seen[i] { seen[i] = true; peers.push(i); }
        }}
        peers.into_iter()
    }

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
        let peer_note_changes = match &event {
            GameEvent::SetDigit { row, col, digit } => {
                self.grid.set_filled(*row, *col, *digit);
                // Clear this digit's note from all peers (same row, col, box).
                let bit = 1u16 << digit;
                let mut changes = Vec::new();
                for idx in Self::peer_indices(*row, *col) {
                    if self.notes[idx] & bit != 0 {
                        changes.push((idx, self.notes[idx]));
                        self.notes[idx] &= !bit;
                    }
                }
                // Also clear all notes on the placed cell itself.
                let cell_idx = Self::idx(*row, *col);
                if self.notes[cell_idx] != 0 {
                    changes.push((cell_idx, self.notes[cell_idx]));
                    self.notes[cell_idx] = 0;
                }
                changes
            }
            GameEvent::ClearCell { row, col } => {
                self.grid.clear(*row, *col);
                self.notes[Self::idx(*row, *col)] = 0;
                vec![]
            }
            GameEvent::ToggleNote { row, col, digit } => {
                self.notes[Self::idx(*row, *col)] ^= 1 << digit;
                vec![]
            }
        };
        self.undo_stack.push(HistoryEntry { event, prev_cell, prev_notes, peer_note_changes });
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
            // Restore peer notes that were cleared as a side-effect of SetDigit.
            for (peer_idx, old_mask) in &entry.peer_note_changes {
                self.notes[*peer_idx] = *old_mask;
            }
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
                GameEvent::SetDigit { row, col, digit } => {
                    self.grid.set_filled(*row, *col, *digit);
                    // Re-apply peer note cleanup stored in the entry.
                    for (peer_idx, _) in &entry.peer_note_changes {
                        self.notes[*peer_idx] &= !(1u16 << digit);
                    }
                }
                GameEvent::ClearCell { row, col } => {
                    self.grid.clear(*row, *col);
                    self.notes[Self::idx(*row, *col)] = 0;
                }
                GameEvent::ToggleNote { row, col, digit } => {
                    self.notes[Self::idx(*row, *col)] ^= 1 << digit;
                }
            }
            self.undo_stack.push(HistoryEntry {
                event: entry.event,
                prev_cell,
                prev_notes,
                peer_note_changes: entry.peer_note_changes,
            });
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
    fn set_digit_clears_peer_notes() {
        let mut s = easy_state();
        // EASY row 0: 530070000 — col 2,3,5,6,7,8 are empty.
        // Place notes for digit 4 in peers of (0,2):
        s.apply(GameEvent::ToggleNote { row: 1, col: 2, digit: 4 }); // same column
        s.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 4 }); // same row, different box
        // Place digit 4 at (0,2).
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 4 });
        // Peer notes for 4 must be cleared.
        assert!(!s.has_note(0, 2, 4), "placed cell notes should be cleared");
        assert!(!s.has_note(1, 2, 4), "same-column peer note should be cleared");
        assert!(!s.has_note(0, 5, 4), "same-row peer note should be cleared");
    }

    #[test]
    fn undo_set_digit_restores_peer_notes() {
        let mut s = easy_state();
        // EASY row 0: 530070000 — (0,5) is empty; row 1: 600195000 — (1,2) is empty.
        s.apply(GameEvent::ToggleNote { row: 1, col: 2, digit: 4 });
        s.apply(GameEvent::ToggleNote { row: 0, col: 5, digit: 4 });
        s.apply(GameEvent::SetDigit { row: 0, col: 2, digit: 4 });
        s.undo();
        // After undo, peer notes are restored.
        assert!(s.has_note(1, 2, 4), "peer note should be restored after undo");
        assert!(s.has_note(0, 5, 4), "peer note should be restored after undo");
        assert_eq!(s.grid().get(0, 2), CellKind::Empty);
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
