// src/hint/strategies/tier1.rs
use crate::hint::{Hint, Strategy};
use crate::puzzle::Grid;
use crate::puzzle::game_state::GameState;

pub struct FullHouse;
pub struct NakedSingle;
pub struct HiddenSingle;
pub struct NotesHint;
pub struct NakedPairs;
pub struct HiddenPairs;
pub struct PointingPairs;
pub struct BoxLineReduction;

// Implementations added in Tasks 7-9.
impl Strategy for FullHouse {
    fn name_en(&self) -> &'static str { "Full House" }
    fn name_de(&self) -> &'static str { "Full House" }
    fn find(&self, _state: &GameState, _solution: &Grid) -> Option<Hint> { None }
}
impl Strategy for NakedSingle {
    fn name_en(&self) -> &'static str { "Naked Single" }
    fn name_de(&self) -> &'static str { "Naked Single" }
    fn find(&self, _state: &GameState, _solution: &Grid) -> Option<Hint> { None }
}
impl Strategy for HiddenSingle {
    fn name_en(&self) -> &'static str { "Hidden Single" }
    fn name_de(&self) -> &'static str { "Hidden Single" }
    fn find(&self, _state: &GameState, _solution: &Grid) -> Option<Hint> { None }
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
