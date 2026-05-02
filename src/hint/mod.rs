// src/hint/mod.rs
pub mod strategies;

use crate::puzzle::{CellKind, Grid};
use crate::puzzle::game_state::GameState;

// ── Hint ──────────────────────────────────────────────────────────────────────

/// A single hint produced by a strategy, carrying all the information needed
/// for rendering (which cells to highlight) and display (explanation text).
#[derive(Debug, Clone)]
pub struct Hint {
    /// Cells explaining WHY the hint works — green/cyan border.
    pub cause_cells:     Vec<(usize, usize)>,
    /// Cells where a candidate can be eliminated — red/magenta border.
    pub elim_cells:      Vec<(usize, usize)>,
    /// The cell where the player should act — blinking yellow background.
    pub target_cell:     (usize, usize),
    /// Digit being eliminated (used in explanation text placeholders).
    pub elim_digit:      Option<u8>,
    /// Digit that goes into the target cell.
    pub target_digit:    Option<u8>,
    /// Pre-formatted English explanation.
    pub explanation_en:  String,
    /// Pre-formatted German explanation.
    pub explanation_de:  String,
    /// Strategy name in English (for panel header).
    pub name_en:         &'static str,
    /// Strategy name in German.
    pub name_de:         &'static str,
}

// ── Strategy trait ────────────────────────────────────────────────────────────

pub trait Strategy: Send + Sync {
    fn name_en(&self) -> &'static str;
    fn name_de(&self) -> &'static str;
    fn find(&self, state: &GameState, solution: &Grid) -> Option<Hint>;
}

// ── Registry entry point ──────────────────────────────────────────────────────

/// Try all registered strategies in order; return the first hint found,
/// or `None` if no strategy applies (Reveal is handled by the caller).
pub fn find_hint(state: &GameState, solution: &Grid) -> Option<Hint> {
    use strategies::tier1::*;
    use strategies::tier2::*;
    let strategies: &[&dyn Strategy] = &[
        // Tier 1 — basic logic
        &FullHouse,
        &NakedSingle,
        &HiddenSingle,
        &NotesHint,
        &NotesValidator,
        &NakedPairs,
        &HiddenPairs,
        &PointingPairs,
        &BoxLineReduction,
        // Tier 2 — advanced eliminations
        &NakedTriples,
        &HiddenTriples,
        &NakedQuads,
        &HiddenQuads,
        &XWing,
        &Swordfish,
        &Jellyfish,
        &Skyscraper,
        &TwoStringKite,
        &YWing,
        &XYZWing,
        &WWing,
        &UniqueRectangle,
        &BugPlusOne,
    ];
    for s in strategies {
        if let Some(h) = s.find(state, solution) {
            return Some(h);
        }
    }
    None
}

// ── Reveal helpers ─────────────────────────────────────────────────────────────

/// Find the most constrained empty cell (fewest notes-mask bits set).
/// Tiebroken by reading order (row-major). Returns None if no empty cells.
pub fn most_constrained_cell(state: &GameState) -> Option<(usize, usize)> {
    let mut best: Option<(usize, usize)> = None;
    let mut best_count = u32::MAX;
    for r in 0..9 {
        for c in 0..9 {
            if matches!(state.grid().get(r, c), CellKind::Empty) {
                let bits = state.notes_mask(r, c).count_ones();
                if bits < best_count {
                    best_count = bits;
                    best = Some((r, c));
                }
            }
        }
    }
    best
}

/// Returns true if every empty cell has at least one note (notes mask != 0).
pub fn all_empty_have_notes(state: &GameState) -> bool {
    for r in 0..9 {
        for c in 0..9 {
            if matches!(state.grid().get(r, c), CellKind::Empty)
                && state.notes_mask(r, c) == 0
            {
                return false;
            }
        }
    }
    true
}
