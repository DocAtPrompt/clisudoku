use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    SetDigit { row: usize, col: usize, digit: u8 },
    ClearCell { row: usize, col: usize },
    ToggleNote { row: usize, col: usize, digit: u8 },
}
