// src/tui/generating.rs

use std::sync::mpsc;
use std::thread;
use std::time::Instant;
use crate::generator::{Difficulty, PuzzleGenerator};
use crate::pattern::Pattern;
use crate::puzzle::Grid;

/// Message sent from the generator thread to the main thread.
pub enum GenMsg {
    Done(Grid, Difficulty),
}

/// Spawn a background thread that generates one puzzle with the given seed.
/// Returns the receiving end of the channel.
pub fn spawn_generation(pattern: Pattern, seed: u64) -> mpsc::Receiver<GenMsg> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = PuzzleGenerator::new(seed).generate_with_pattern(&pattern);
        let _ = tx.send(GenMsg::Done(result.grid, result.difficulty));
    });
    rx
}

/// Cyclic verb list for the generating message "baking sudoku..."
pub const VERBS: &[&str] = &[
    "generating", "frying",    "baking",    "roasting",   "shoveling",  "tinkering",
    "brewing",    "distilling","cooking",   "boiling",    "simmering",  "grilling",
    "toasting",   "smoking",   "seasoning", "marinating", "kneading",   "blending",
    "mixing",     "stirring",  "whipping",  "grinding",   "fermenting", "percolating",
    "crafting",   "forging",   "spinning",  "shuffling",  "sculpting",  "chiseling",
    "polishing",  "weaving",   "knitting",  "mining",     "hatching",   "conjuring",
    "assembling", "composing",
];

/// All state needed by AppScreen::Generating.
pub struct GeneratingState {
    pub pattern:       Pattern,
    pub rx:            mpsc::Receiver<GenMsg>,
    pub seed:          u64,
    pub started_at:    Instant,
    /// Shuffled indices into VERBS; cycles when exhausted.
    pub verb_order:    Vec<usize>,
    /// Current position in verb_order.
    pub verb_pos:      usize,
    /// True for ~1 second after a timeout triggers a new seed.
    pub show_new_seed: bool,
    /// When show_new_seed was set (to expire it after 1 s).
    pub new_seed_at:   Option<Instant>,
    /// True when entered from --pattern CLI flag (Esc → DifficultySelect).
    /// False when entered from PatternSelect screen (Esc → PatternSelect).
    pub from_cli:      bool,
}

impl GeneratingState {
    pub fn new(pattern: Pattern, from_cli: bool) -> Self {
        let seed = random_seed();
        let rx = spawn_generation(pattern.clone(), seed);
        let n = VERBS.len();
        let mut verb_order: Vec<usize> = (0..n).collect();
        lcg_shuffle(&mut verb_order, seed);
        GeneratingState {
            pattern,
            rx,
            seed,
            started_at: Instant::now(),
            verb_order,
            verb_pos: 0,
            show_new_seed: false,
            new_seed_at: None,
            from_cli,
        }
    }

    /// Current verb string to show, e.g. "baking".
    pub fn current_verb(&self) -> &'static str {
        VERBS[self.verb_order[self.verb_pos % self.verb_order.len()]]
    }

    /// Seconds remaining in the current 3-second attempt (0..=3).
    pub fn countdown_secs(&self) -> u8 {
        let elapsed = self.started_at.elapsed().as_secs();
        3u64.saturating_sub(elapsed) as u8
    }

    /// Advance to a new seed attempt after timeout.
    pub fn try_new_seed(&mut self) {
        self.seed = self.seed.wrapping_add(0x9e3779b97f4a7c15);
        self.rx = spawn_generation(self.pattern.clone(), self.seed);
        self.started_at = Instant::now();
        self.verb_pos += 1;
        self.show_new_seed = true;
        self.new_seed_at = Some(Instant::now());
    }
}

/// Generate a seed from the current time (no external deps).
pub fn random_seed() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(42)
}

fn lcg_shuffle(v: &mut Vec<usize>, seed: u64) {
    let mut state = seed ^ 0x12345678;
    for i in (1..v.len()).rev() {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (state as usize) % (i + 1);
        v.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::PATTERNS;

    #[test]
    fn spawn_generation_completes_for_asterisk_pattern() {
        // Asterisk pattern (index 10) has 33 cells — interior-heavy, reliable test case.
        let rx = spawn_generation(PATTERNS[10].clone(), 42);
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            match rx.try_recv() {
                Ok(GenMsg::Done(grid, _)) => {
                    assert!(crate::solver::Solver::new().solve(grid).grid.is_solved());
                    return;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if std::time::Instant::now() > deadline { panic!("timeout"); }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(e) => panic!("{e}"),
            }
        }
    }

    #[test]
    fn generating_state_has_verb() {
        let state = GeneratingState::new(PATTERNS[10].clone(), false);
        assert!(!state.current_verb().is_empty());
    }

    #[test]
    fn countdown_starts_at_3() {
        let state = GeneratingState::new(PATTERNS[10].clone(), false);
        assert_eq!(state.countdown_secs(), 3);
    }
}
