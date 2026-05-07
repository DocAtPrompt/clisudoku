// src/tui/generating.rs

use crate::generator::{Difficulty, PuzzleGenerator};
use crate::pattern::Pattern;
use crate::puzzle::Grid;
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

/// Message sent from the generator thread to the main thread.
pub enum GenMsg {
    Done(Grid, Difficulty),
    /// Intermediate progress from the BareMinimum multi-attempt generator.
    BareMinimumProgress {
        done: usize,
        total: usize,
        best_count: usize,
    },
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

/// Spawn a background thread that runs `attempts` BareMinimum generations with
/// different seeds and returns the puzzle with the fewest given cells.
///
/// Sends one `BareMinimumProgress` message after each completed attempt so the
/// UI can show live progress, followed by a single `Done` with the best grid.
pub fn spawn_bare_minimum(seed: u64, attempts: usize) -> mpsc::Receiver<GenMsg> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut best_grid: Option<Grid> = None;
        let mut best_count = usize::MAX;
        for i in 0..attempts {
            // Derive a distinct seed per attempt using a LCG step.
            let s = seed.wrapping_add((i as u64).wrapping_mul(0x9e3779b97f4a7c15));
            let grid = PuzzleGenerator::new(s).generate(Difficulty::BareMinimum, false);
            let count = (0..9)
                .flat_map(|r| (0..9).map(move |c| (r, c)))
                .filter(|&(r, c)| grid.get(r, c).is_given())
                .count();
            if count < best_count {
                best_count = count;
                best_grid = Some(grid);
            }
            let _ = tx.send(GenMsg::BareMinimumProgress {
                done: i + 1,
                total: attempts,
                best_count,
            });
        }
        if let Some(grid) = best_grid {
            let _ = tx.send(GenMsg::Done(grid, Difficulty::BareMinimum));
        }
    });
    rx
}

/// Spawn a background thread that generates one Expert puzzle.
pub fn spawn_expert(seed: u64, symmetry: bool) -> mpsc::Receiver<GenMsg> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let grid = PuzzleGenerator::new(seed)
            .generate(Difficulty::Expert, symmetry);
        let _ = tx.send(GenMsg::Done(grid, Difficulty::Expert));
    });
    rx
}

/// Number of BareMinimum generation attempts per request.
pub const BARE_MINIMUM_ATTEMPTS: usize = 5;

/// Cyclic verb list for the generating message "baking sudoku..."
pub const VERBS: &[&str] = &[
    "generating",
    "frying",
    "baking",
    "roasting",
    "shoveling",
    "tinkering",
    "brewing",
    "distilling",
    "cooking",
    "boiling",
    "simmering",
    "grilling",
    "toasting",
    "smoking",
    "seasoning",
    "marinating",
    "kneading",
    "blending",
    "mixing",
    "stirring",
    "whipping",
    "grinding",
    "fermenting",
    "percolating",
    "crafting",
    "forging",
    "spinning",
    "shuffling",
    "sculpting",
    "chiseling",
    "polishing",
    "weaving",
    "knitting",
    "mining",
    "hatching",
    "conjuring",
    "assembling",
    "composing",
];

/// All state needed by AppScreen::Generating.
pub struct GeneratingState {
    pub pattern: Pattern,
    pub rx: mpsc::Receiver<GenMsg>,
    pub seed: u64,
    pub started_at: Instant,
    /// Shuffled indices into VERBS; cycles when exhausted.
    pub verb_order: Vec<usize>,
    /// Current position in verb_order.
    pub verb_pos: usize,
    /// True for ~1 second after a timeout triggers a new seed.
    pub show_new_seed: bool,
    /// When show_new_seed was set (to expire it after 1 s).
    pub new_seed_at: Option<Instant>,
    /// True when entered from --pattern CLI flag (Esc → DifficultySelect).
    /// False when entered from PatternSelect screen (Esc → PatternSelect).
    pub from_cli: bool,
    /// True when generating a BareMinimum puzzle (multi-attempt mode).
    pub bare_minimum: bool,
    /// True when generating an Expert puzzle (single-attempt mode).
    pub expert: bool,
    /// Completed attempt count for BareMinimum progress display.
    pub bm_done: usize,
    /// Total attempt count for BareMinimum progress display.
    pub bm_total: usize,
    /// Best (fewest) given count seen so far across BareMinimum attempts.
    pub bm_best_count: usize,
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
            bare_minimum: false,
            expert: false,
            bm_done: 0,
            bm_total: 0,
            bm_best_count: 0,
        }
    }

    /// Create a state for BareMinimum multi-attempt generation.
    /// No pattern is involved; back navigation always returns to DifficultySelect.
    pub fn new_bare_minimum() -> Self {
        let seed = random_seed();
        let rx = spawn_bare_minimum(seed, BARE_MINIMUM_ATTEMPTS);
        let n = VERBS.len();
        let mut verb_order: Vec<usize> = (0..n).collect();
        lcg_shuffle(&mut verb_order, seed);
        // Dummy pattern — never used in bare-minimum mode.
        let dummy = Pattern {
            name_en: "",
            mask: [false; 81],
            cell_count: 0,
        };
        GeneratingState {
            pattern: dummy,
            rx,
            seed,
            started_at: Instant::now(),
            verb_order,
            verb_pos: 0,
            show_new_seed: false,
            new_seed_at: None,
            from_cli: false,
            bare_minimum: true,
            expert: false,
            bm_done: 0,
            bm_total: BARE_MINIMUM_ATTEMPTS,
            bm_best_count: 0,
        }
    }

    /// Create a state for Expert single-attempt generation.
    /// No pattern is involved; back navigation returns to DifficultySelect (index 4).
    pub fn new_expert(symmetry: bool) -> Self {
        let seed = random_seed();
        let rx = spawn_expert(seed, symmetry);
        let n = VERBS.len();
        let mut verb_order: Vec<usize> = (0..n).collect();
        lcg_shuffle(&mut verb_order, seed);
        let dummy = Pattern { name_en: "", mask: [false; 81], cell_count: 0 };
        GeneratingState {
            pattern: dummy,
            rx,
            seed,
            started_at: Instant::now(),
            verb_order,
            verb_pos: 0,
            show_new_seed: false,
            new_seed_at: None,
            from_cli: false,
            bare_minimum: false,
            expert: true,
            bm_done: 0,
            bm_total: 0,
            bm_best_count: 0,
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

    /// Clear `show_new_seed` once it has been visible for 1 second.
    /// Call this on every UI tick while in the Generating screen.
    pub fn tick_new_seed_expiry(&mut self) {
        if let Some(at) = self.new_seed_at {
            if at.elapsed().as_secs() >= 1 {
                self.show_new_seed = false;
                self.new_seed_at = None;
            }
        }
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
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
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
                Ok(GenMsg::BareMinimumProgress { .. }) => { /* ignore progress in this test */ }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if std::time::Instant::now() > deadline {
                        panic!("timeout");
                    }
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
