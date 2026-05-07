pub mod anim;
pub mod colors;
pub mod digit_style;
pub mod generating;
pub mod input;
pub mod render;
pub mod seq_detect;
pub mod terminal;

use crate::generator::{Difficulty, PuzzleGenerator};
use crate::i18n::Language;
use crate::puzzle::{CellKind, GameState, Grid};
use crate::solver::backtracking::solve_backtracking;
use crate::solver::candidates::CandidateGrid;
use crate::timer::Clock;
use crate::tui::anim::{AnimState, FireworkAnim, SweepAnim};
use crate::tui::colors::{ColorScheme, Theme};
use crate::tui::digit_style::{AwkwardRetroStyle, DigitStyle, RetroStyle};
use crate::tui::input::{map_key_to_action, AppAction, NavMode, NavState};
use crate::tui::render::start_screen::START_ITEM_COUNT;
use crate::tui::render::{box_cells, box_cells_serpentine, col_cells, row_cells};
use crate::tui::render::{render_frame, render_info_overlay, Screen};
use crate::tui::seq_detect::{EasterEgg, SeqDetector};
use crate::tui::terminal::Terminal;
use crossterm::event::{self, Event};
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};

/// Minimum terminal dimensions required to render the full game layout.
/// Grid (73 wide × 37 tall) at col 2 + panel (38 wide) at col 77
/// → panel occupies cols 77–114, plus 2-col right margin → 117 cols.
/// Panel bottom border at row 37 + 2 margin rows → 39 rows.
const MIN_COLS: u16 = 117;
const MIN_ROWS: u16 = 39;
use std::io::{self, BufWriter, Write};
use std::time::Duration;

pub enum AppScreen {
    Start { selected: usize },
    DifficultySelect { selected: usize, sym_focused: bool },
    LanguageSelect { selected: usize },
    ThemeSelect { selected: usize },
    Game,
    PatternSelect { selected: usize },
    Generating(crate::tui::generating::GeneratingState),
}

/// Pending confirmation action.
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    QuitGame,
}

/// Category of a completed game, for future database integration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum GameCategory {
    #[default]
    Classic,
    Design,
    BareMinimum,
}

/// Per-game statistics tracked for database / post-game summary.
#[derive(Debug, Clone, Default)]
pub struct GameStats {
    /// Number of wrong solution digits entered while error display was active.
    pub errors_shown: u32,
    /// Whether the `iddqd` god-mode cheat was used.
    pub cheat_god_mode: bool,
    /// Whether the `idkfa` fill-notes cheat was used.
    pub cheat_fill_notes: bool,
    /// Whether scan mode was activated at least once during this game.
    pub scan_used: bool,
    /// Number of hints requested during this game.
    pub hint_count: u32,
    /// Category for DB storage.
    pub category: GameCategory,
    /// Pattern name for designer games; None for classic games.
    pub pattern_name: Option<String>,
}

pub struct App {
    pub screen: AppScreen,
    pub language: Language,
    /// Active color theme. Always starts as Dark regardless of terminal settings.
    pub theme: Theme,
    /// Whether newly generated puzzles should have 180° rotational symmetry.
    pub symmetry: bool,
    pub game_state: Option<GameState>,
    pub cursor: (usize, usize),
    pub nav_state: NavState,
    pub note_mode: bool,
    /// Passive digit scan: highlight all cells with the same digit as the cursor.
    pub scan_mode: bool,
    /// Show wrong solution digits in red.
    pub error_mode: bool,
    /// Pre-computed solution for the current puzzle (used for error detection).
    pub solution: Option<Grid>,
    /// Per-game statistics accumulated during play.
    pub stats: GameStats,
    /// Cells permanently shown in red because a wrong digit was revealed while error_mode was on.
    pub revealed_errors: std::collections::HashSet<(usize, usize)>,
    pub paused: bool,
    /// Boss Key active — game hidden behind a fake terminal.
    pub boss_mode: bool,
    /// Matrix Mode active — all digits rendered in Matrix green.
    pub matrix_mode: bool,
    pub confirm_pending: Option<ConfirmAction>,
    pub should_quit: bool,
    /// Set whenever the screen variant changes so the next render clears first.
    pub needs_clear: bool,
    clock: Box<dyn Clock>,
    game_start_ms: u64,
    /// Elapsed ms frozen at the moment the game was paused or boss key was pressed.
    paused_elapsed_ms: u64,
    pub colors: ColorScheme,
    /// Style for pre-filled (given) cells — `RetroStyle` half-block glyphs.
    given_style: Box<dyn DigitStyle>,
    /// Style for player-entered (filled) cells — `AwkwardRetroStyle` full-block glyphs.
    filled_style: Box<dyn DigitStyle>,
    /// Typed sequence detector for easter eggs.
    seq: SeqDetector,
    /// Active animations (sweep + firework).
    pub anim: AnimState,
    /// Info overlay: (message, subtitle, auto_dismiss_after_3s, shown_at).
    /// Puzzle-error overlays set auto_dismiss=false so the player must press a key.
    pub info_overlay: Option<(String, Option<String>, bool, std::time::Instant)>,
    /// Currently displayed hint, if any. Cleared on any keypress.
    pub active_hint: Option<crate::hint::Hint>,
    /// Warning text shown in the hint panel when the pre-check fails.
    /// `(name, explanation)` in the current language.
    pub hint_warning: Option<(&'static str, &'static str)>,
    /// When true, drain all buffered input events at the top of the next run() loop iteration.
    /// Set after start_game() so key presses made during puzzle generation are discarded.
    drain_input: bool,
    /// Whether mouse capture mode is active.
    pub mouse_mode: bool,
    /// Grid cell currently under the mouse cursor; `None` when not hovering a cell.
    pub hover_cell: Option<(usize, usize)>,
}

impl App {
    pub fn new(clock: Box<dyn Clock>) -> Self {
        Self {
            screen: AppScreen::Start { selected: 0 },
            language: Language::detect(),
            theme: Theme::Dark,
            symmetry: true,
            game_state: None,
            cursor: (0, 0),
            nav_state: NavState::default(),
            note_mode: false,
            scan_mode: false,
            error_mode: false,
            solution: None,
            stats: GameStats::default(),
            revealed_errors: std::collections::HashSet::new(),
            paused: false,
            boss_mode: false,
            matrix_mode: false,
            confirm_pending: None,
            should_quit: false,
            needs_clear: false,
            game_start_ms: 0,
            paused_elapsed_ms: 0,
            colors: ColorScheme::default(),
            given_style: Box::new(RetroStyle),
            filled_style: Box::new(AwkwardRetroStyle),
            clock,
            seq: SeqDetector::default(),
            anim: AnimState::default(),
            info_overlay: None,
            active_hint: None,
            hint_warning: None,
            drain_input: false,
            mouse_mode: false,
            hover_cell: None,
        }
    }

    /// Start a new game at the given difficulty.
    fn start_game(&mut self, difficulty: Difficulty) {
        if difficulty == Difficulty::Expert {
            // Expert requires advanced techniques — show Generating screen.
            let state = crate::tui::generating::GeneratingState::new_expert(self.symmetry);
            self.screen = AppScreen::Generating(state);
            self.needs_clear = true;
            self.drain_input = true;
            return;
        }
        if difficulty == Difficulty::BareMinimum {
            // BareMinimum runs multiple long passes — show Generating screen.
            let state = crate::tui::generating::GeneratingState::new_bare_minimum();
            self.screen = AppScreen::Generating(state);
            self.needs_clear = true;
            self.drain_input = true;
            return;
        }
        let seed = self.clock.now_ms();
        let puzzle = PuzzleGenerator::new(seed).generate(difficulty, self.symmetry);
        self.enter_game(puzzle);
    }

    /// Elapsed game time in milliseconds, frozen while paused or in boss mode.
    fn elapsed_ms(&self) -> u64 {
        if self.paused || self.boss_mode || self.game_start_ms == 0 {
            self.paused_elapsed_ms
        } else {
            self.clock.now_ms().saturating_sub(self.game_start_ms)
        }
    }

    /// Handle a single `AppAction`, updating all state.
    pub fn handle_action(&mut self, action: AppAction) {
        // Any action dismisses a hint warning panel.
        if self.hint_warning.is_some() {
            self.hint_warning = None;
            self.needs_clear = true;
        }

        // Confirm dialog takes priority
        if self.confirm_pending.is_some() {
            match action {
                AppAction::ConfirmYes => {
                    match self.confirm_pending.take() {
                        Some(ConfirmAction::QuitGame) => {
                            self.screen = AppScreen::Start { selected: 0 };
                        }
                        None => {}
                    }
                    self.needs_clear = true;
                }
                AppAction::ConfirmNo | AppAction::Back => {
                    self.confirm_pending = None;
                    self.needs_clear = true;
                }
                _ => {}
            }
            return;
        }

        match &self.screen {
            AppScreen::Start { selected } => self.handle_start_action(action, *selected),
            AppScreen::DifficultySelect {
                selected,
                sym_focused,
            } => self.handle_difficulty_action(action, *selected, *sym_focused),
            AppScreen::LanguageSelect { selected } => {
                self.handle_language_action(action, *selected)
            }
            AppScreen::ThemeSelect { selected } => self.handle_theme_action(action, *selected),
            AppScreen::Game => self.handle_game_action(action),
            AppScreen::PatternSelect { selected } => {
                let s = *selected;
                self.handle_pattern_action(action, s);
            }
            AppScreen::Generating(_) => self.handle_generating_action(action),
        }
    }

    fn handle_start_action(&mut self, action: AppAction, selected: usize) {
        match action {
            AppAction::MoveUp => {
                self.screen = AppScreen::Start {
                    selected: selected.saturating_sub(1),
                };
            }
            AppAction::MoveDown => {
                self.screen = AppScreen::Start {
                    selected: (selected + 1).min(START_ITEM_COUNT - 1),
                };
            }
            AppAction::Enter => match selected {
                0 => {
                    self.screen = AppScreen::DifficultySelect {
                        selected: 0,
                        sym_focused: false,
                    };
                    self.needs_clear = true;
                }
                1 => {
                    self.screen = AppScreen::LanguageSelect {
                        selected: self.language.as_index(),
                    };
                    self.needs_clear = true;
                }
                2 => {
                    self.screen = AppScreen::ThemeSelect {
                        selected: self.theme.as_index(),
                    };
                    self.needs_clear = true;
                }
                _ => self.should_quit = true,
            },
            AppAction::Back => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_difficulty_action(&mut self, action: AppAction, selected: usize, sym_focused: bool) {
        const DIFFICULTY_COUNT: usize = 7;
        match action {
            // ── Navigation between columns ───────────────────────────────────
            AppAction::MoveRight if !sym_focused => {
                self.screen = AppScreen::DifficultySelect {
                    selected,
                    sym_focused: true,
                };
            }
            AppAction::MoveLeft if sym_focused => {
                self.screen = AppScreen::DifficultySelect {
                    selected,
                    sym_focused: false,
                };
            }

            // ── Symmetry column: toggle with Enter or Space (Pause), then
            //    jump back to difficulty column so the user can confirm quickly.
            AppAction::Enter | AppAction::Pause if sym_focused => {
                self.symmetry = !self.symmetry;
                self.screen = AppScreen::DifficultySelect {
                    selected,
                    sym_focused: false,
                };
            }

            // ── Difficulty column: move selection ────────────────────────────
            AppAction::MoveUp if !sym_focused => {
                self.screen = AppScreen::DifficultySelect {
                    selected: selected.saturating_sub(1),
                    sym_focused: false,
                };
            }
            AppAction::MoveDown if !sym_focused => {
                self.screen = AppScreen::DifficultySelect {
                    selected: (selected + 1).min(DIFFICULTY_COUNT - 1),
                    sym_focused: false,
                };
            }

            // ── Confirm: start game ──────────────────────────────────────────
            AppAction::Enter if !sym_focused => match selected {
                0 => {
                    self.start_game(Difficulty::Easy);
                    self.needs_clear = true;
                }
                1 => {
                    self.start_game(Difficulty::Medium);
                    self.needs_clear = true;
                }
                2 => {
                    self.start_game(Difficulty::Hard);
                    self.needs_clear = true;
                }
                3 => {
                    self.start_game(Difficulty::Extreme);
                    self.needs_clear = true;
                }
                4 => {
                    self.start_game(Difficulty::Expert);
                    self.needs_clear = true;
                }
                5 => {
                    self.start_game(Difficulty::BareMinimum);
                    self.needs_clear = true;
                }
                6 => {
                    self.screen = AppScreen::PatternSelect { selected: 0 };
                    self.needs_clear = true;
                }
                _ => {}
            },

            // ── Back always goes to Start ────────────────────────────────────
            AppAction::Back => {
                self.screen = AppScreen::Start { selected: 0 };
                self.needs_clear = true;
            }
            _ => {}
        }
    }

    fn handle_language_action(&mut self, action: AppAction, selected: usize) {
        use crate::i18n::LANGUAGE_COUNT;
        match action {
            AppAction::MoveUp => {
                self.screen = AppScreen::LanguageSelect {
                    selected: selected.saturating_sub(1),
                };
            }
            AppAction::MoveDown => {
                self.screen = AppScreen::LanguageSelect {
                    selected: (selected + 1).min(LANGUAGE_COUNT - 1),
                };
            }
            AppAction::Enter => {
                self.language = Language::from_index(selected);
                self.screen = AppScreen::Start { selected: 0 };
                self.needs_clear = true;
            }
            AppAction::Back => {
                self.screen = AppScreen::Start { selected: 0 };
                self.needs_clear = true;
            }
            _ => {}
        }
    }

    fn handle_theme_action(&mut self, action: AppAction, selected: usize) {
        use crate::tui::colors::THEME_COUNT;
        match action {
            // Navigation applies the theme immediately for live preview.
            AppAction::MoveUp => {
                let s = selected.saturating_sub(1);
                self.colors = ColorScheme::for_theme(Theme::from_index(s));
                self.screen = AppScreen::ThemeSelect { selected: s };
                self.needs_clear = true;
            }
            AppAction::MoveDown => {
                let s = (selected + 1).min(THEME_COUNT - 1);
                self.colors = ColorScheme::for_theme(Theme::from_index(s));
                self.screen = AppScreen::ThemeSelect { selected: s };
                self.needs_clear = true;
            }
            // Enter confirms and saves.
            AppAction::Enter => {
                self.theme = Theme::from_index(selected);
                self.screen = AppScreen::Start { selected: 0 };
                self.needs_clear = true;
            }
            // Back restores the previously saved theme.
            AppAction::Back => {
                self.colors = ColorScheme::for_theme(self.theme);
                self.screen = AppScreen::Start { selected: 0 };
                self.needs_clear = true;
            }
            _ => {}
        }
    }

    fn handle_pattern_action(&mut self, action: AppAction, selected: usize) {
        const COUNT: usize = crate::pattern::PATTERNS.len();
        match action {
            AppAction::MoveRight => {
                self.screen = AppScreen::PatternSelect {
                    selected: (selected + 1) % COUNT,
                };
                self.needs_clear = true;
            }
            AppAction::MoveLeft => {
                self.screen = AppScreen::PatternSelect {
                    selected: selected.checked_sub(1).unwrap_or(COUNT - 1),
                };
                self.needs_clear = true;
            }
            AppAction::Enter => {
                let pattern = crate::pattern::PATTERNS[selected].clone();
                self.start_generating(pattern, false);
            }
            AppAction::Back => {
                self.screen = AppScreen::DifficultySelect {
                    selected: 6,
                    sym_focused: false,
                };
                self.needs_clear = true;
            }
            _ => {}
        }
    }

    fn handle_generating_action(&mut self, action: AppAction) {
        if matches!(action, AppAction::Back) {
            let (bare_minimum, expert, from_cli, pat_selected) =
                if let AppScreen::Generating(ref s) = self.screen {
                    let idx = crate::pattern::PATTERNS
                        .iter()
                        .position(|p| p.name_en == s.pattern.name_en)
                        .unwrap_or(0);
                    (s.bare_minimum, s.expert, s.from_cli, idx)
                } else {
                    (false, false, false, 0)
                };
            self.screen = if expert {
                // Expert: go back to DifficultySelect at index 4.
                AppScreen::DifficultySelect {
                    selected: 4,
                    sym_focused: false,
                }
            } else if bare_minimum {
                // BareMinimum: go back to DifficultySelect at index 5.
                AppScreen::DifficultySelect {
                    selected: 5,
                    sym_focused: false,
                }
            } else if from_cli {
                AppScreen::DifficultySelect {
                    selected: 6,
                    sym_focused: false,
                }
            } else {
                AppScreen::PatternSelect {
                    selected: pat_selected,
                }
            };
            self.needs_clear = true;
        }
    }

    pub fn start_generating(&mut self, pattern: crate::pattern::Pattern, from_cli: bool) {
        let state = crate::tui::generating::GeneratingState::new(pattern, from_cli);
        self.screen = AppScreen::Generating(state);
        self.needs_clear = true;
    }

    fn enter_game(&mut self, puzzle: Grid) {
        self.solution = solve_backtracking(puzzle.clone());
        self.game_state = Some(GameState::new(puzzle));
        self.stats = GameStats::default();
        self.cursor = (0, 0);
        self.nav_state = NavState::default();
        self.note_mode = false;
        self.scan_mode = false;
        self.error_mode = false;
        self.anim.error_blink = false;
        self.revealed_errors.clear();
        self.paused = false;
        self.active_hint = None;
        self.hint_warning = None;
        self.anim.hint_blink = false;
        self.anim.hint_blink_tick = 0;
        self.game_start_ms = self.clock.now_ms();
        self.drain_input = true;
        self.confirm_pending = None;
        self.boss_mode = false;
        // Always disable mouse capture when starting a new game.
        if self.mouse_mode {
            let _ = crate::tui::terminal::disable_mouse_capture();
        }
        self.mouse_mode = false;
        self.hover_cell = None;
        self.info_overlay = None;
        self.screen = AppScreen::Game;
        self.needs_clear = true;
    }

    fn handle_game_action(&mut self, action: AppAction) {
        // ── Boss mode: only BossKey (toggle back) and Esc (quit) are accepted ──
        if self.boss_mode {
            match action {
                AppAction::BossKey => {
                    // Resume: shift game_start_ms so timer continues from frozen value
                    self.game_start_ms = self.clock.now_ms().saturating_sub(self.paused_elapsed_ms);
                    self.boss_mode = false;
                    self.needs_clear = true;
                }
                AppAction::Back => {
                    // Esc in boss mode: silent immediate quit (later: save first)
                    self.should_quit = true;
                }
                _ => {}
            }
            return;
        }

        if self.paused {
            match action {
                AppAction::Pause => {
                    // Resume: shift game_start_ms forward so elapsed continues from frozen value
                    self.game_start_ms = self.clock.now_ms().saturating_sub(self.paused_elapsed_ms);
                    self.paused = false;
                }
                AppAction::Back => {
                    self.paused = false;
                    self.confirm_pending = Some(ConfirmAction::QuitGame);
                    self.needs_clear = true;
                }
                _ => {}
            }
            return;
        }

        match action {
            AppAction::Back => {
                self.confirm_pending = Some(ConfirmAction::QuitGame);
                self.needs_clear = true;
            }
            AppAction::Pause => {
                self.paused_elapsed_ms = self.elapsed_ms();
                self.paused = true;
            }
            AppAction::BossKey => {
                self.paused_elapsed_ms = self.elapsed_ms();
                self.boss_mode = true;
                self.needs_clear = true;
            }
            AppAction::MoveUp => self.move_cursor(-1, 0),
            AppAction::MoveDown => self.move_cursor(1, 0),
            AppAction::MoveLeft => self.move_cursor(0, -1),
            AppAction::MoveRight => self.move_cursor(0, 1),
            AppAction::NumpadBox(idx) => {
                self.nav_state.box_idx = Some(idx);
                self.nav_state.mode = NavMode::Navigation;
            }
            AppAction::NumpadCell(cell_idx) => {
                if let Some(box_idx) = self.nav_state.box_idx.take() {
                    let (row, col) = numpad_to_cell(box_idx, cell_idx);
                    self.cursor = (row, col);
                    self.nav_state.mode = NavMode::Input;
                }
            }
            AppAction::Enter => {
                // Toggle between modes; clear any partial box selection on exit.
                self.nav_state.mode = match self.nav_state.mode {
                    NavMode::Input => NavMode::Navigation,
                    NavMode::Navigation => {
                        self.nav_state.box_idx = None;
                        NavMode::Input
                    }
                };
            }
            AppAction::ToggleMode => {
                self.note_mode = !self.note_mode;
            }
            AppAction::ToggleScan => {
                self.scan_mode = !self.scan_mode;
                if self.scan_mode {
                    self.stats.scan_used = true;
                }
            }
            AppAction::RequestHint => {
                self.handle_hint_request();
            }
            AppAction::ToggleErrors => {
                self.error_mode = !self.error_mode;
                self.anim.error_blink = self.error_mode;
                self.anim.error_blink_tick = 0; // start in "visible" phase immediately
                if self.error_mode {
                    // Switching ON: count all currently wrong filled cells not yet counted.
                    if let (Some(state), Some(sol)) = (&self.game_state, &self.solution) {
                        for r in 0..9 {
                            for c in 0..9 {
                                if let CellKind::Filled(d) = state.grid().get(r, c) {
                                    let wrong = sol
                                        .get(r, c)
                                        .value()
                                        .map(|correct| correct != d)
                                        .unwrap_or(false);
                                    if wrong && !self.revealed_errors.contains(&(r, c)) {
                                        self.stats.errors_shown += 1;
                                        self.revealed_errors.insert((r, c));
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Switching OFF: revert all cells to normal colour.
                    self.revealed_errors.clear();
                }
            }
            AppAction::Digit(d) => {
                let (row, col) = self.cursor;
                if let Some(state) = &mut self.game_state {
                    use crate::puzzle::GameEvent;
                    let event = if self.note_mode {
                        GameEvent::ToggleNote { row, col, digit: d }
                    } else {
                        // Count errors when error display is active and digit is wrong.
                        if self.error_mode && !self.note_mode {
                            if let Some(sol) = &self.solution {
                                if !matches!(state.grid().get(row, col), CellKind::Given(_))
                                    && !self.revealed_errors.contains(&(row, col))
                                    && sol.get(row, col).value() != Some(d)
                                {
                                    self.stats.errors_shown += 1;
                                    self.revealed_errors.insert((row, col));
                                    // Always start blink from visible phase so the
                                    // new error digit appears immediately.
                                    self.anim.restart_error_blink();
                                }
                            }
                        }
                        GameEvent::SetDigit { row, col, digit: d }
                    };
                    state.apply(event);
                }
                if !self.note_mode {
                    self.check_completion(row, col);
                }
            }
            AppAction::ClearCell => {
                let (row, col) = self.cursor;
                if let Some(state) = &mut self.game_state {
                    use crate::puzzle::GameEvent;
                    state.apply(GameEvent::ClearCell { row, col });
                }
            }
            AppAction::Undo => {
                if let Some(state) = &mut self.game_state {
                    state.undo();
                }
            }
            AppAction::Redo => {
                if let Some(state) = &mut self.game_state {
                    state.redo();
                }
            }
            AppAction::ToggleMouseMode => {
                // Flip state only after the IO call succeeds, so terminal state
                // and self.mouse_mode never diverge on error.
                let target = !self.mouse_mode;
                let ok = if target {
                    crate::tui::terminal::enable_mouse_capture()
                } else {
                    crate::tui::terminal::disable_mouse_capture()
                };
                if ok.is_ok() {
                    self.mouse_mode = target;
                    self.hover_cell = None;
                }
            }
            AppAction::MouseHover(r, c) => {
                self.hover_cell = Some((r, c));
            }
            AppAction::MouseSelectCell(r, c) => {
                self.cursor = (r, c);
                self.nav_state.mode = crate::tui::input::NavMode::Input;
                self.nav_state.box_idx = None;
            }
            AppAction::MouseButton(btn) => {
                use crate::tui::input::MousePanelButton;
                let action = match btn {
                    MousePanelButton::NotesSolToggle => AppAction::ToggleMode,
                    MousePanelButton::Undo           => AppAction::Undo,
                    MousePanelButton::Redo           => AppAction::Redo,
                    MousePanelButton::Clear          => AppAction::ClearCell,
                    MousePanelButton::Digit(d)       => AppAction::Digit(d),
                };
                self.handle_game_action(action);
            }
            _ => {}
        }
    }

    fn handle_hint_request(&mut self) {
        use crate::hint;

        // If hint already active, close it and search fresh.
        self.active_hint = None;
        self.anim.hint_blink = false;

        let strings = self.language.strings();

        // ── Pre-check 1: incorrect filled digits ────────────────────────────────
        let has_errors = {
            let state = match &self.game_state {
                Some(s) => s,
                None => return,
            };
            let solution = match &self.solution {
                Some(sol) => sol,
                None => return,
            };
            let grid = state.grid();
            let mut found = false;
            'outer1: for r in 0..9 {
                for c in 0..9 {
                    if let crate::puzzle::CellKind::Filled(d) = grid.get(r, c) {
                        if solution.get(r, c).value() != Some(d) {
                            found = true;
                            break 'outer1;
                        }
                    }
                }
            }
            found
        };
        if has_errors {
            self.stats.hint_count += 1;
            self.hint_warning = Some((strings.hint_has_errors, strings.hint_has_errors));
            return;
        }

        // ── Pre-check 2: incorrect notes ────────────────────────────────────────
        let has_wrong_notes = {
            let state = match &self.game_state {
                Some(s) => s,
                None => return,
            };
            let grid = state.grid();
            let mut found = false;
            'outer2: for r in 0..9 {
                for c in 0..9 {
                    if !matches!(grid.get(r, c), crate::puzzle::CellKind::Empty) {
                        continue;
                    }
                    let notes = state.notes_mask(r, c);
                    for d in 1u8..=9 {
                        if notes & (1 << d) == 0 {
                            continue;
                        }
                        // A note is wrong if d already appears in the same row, col, or box
                        // (i.e., d conflicts with an already-placed value).
                        let mut conflict = false;
                        for cc in 0..9 {
                            if grid.get(r, cc).value() == Some(d) {
                                conflict = true;
                                break;
                            }
                        }
                        if !conflict {
                            for rr in 0..9 {
                                if grid.get(rr, c).value() == Some(d) {
                                    conflict = true;
                                    break;
                                }
                            }
                        }
                        if !conflict {
                            let (br, bc) = (r / 3 * 3, c / 3 * 3);
                            'box_check: for dr in 0..3 {
                                for dc in 0..3 {
                                    if grid.get(br + dr, bc + dc).value() == Some(d) {
                                        conflict = true;
                                        break 'box_check;
                                    }
                                }
                            }
                        }
                        if conflict {
                            found = true;
                            break 'outer2;
                        }
                    }
                }
            }
            found
        };
        if has_wrong_notes {
            self.stats.hint_count += 1;
            self.hint_warning = Some((strings.hint_has_wrong_notes, strings.hint_has_wrong_notes));
            return;
        }

        // ── All clear: proceed with hint ────────────────────────────────────────
        let (state, solution) = match (&self.game_state, &self.solution) {
            (Some(s), Some(sol)) => (s, sol),
            _ => return,
        };

        // Puzzle already solved — no hint needed.
        if state.grid().is_solved() {
            return;
        }

        // NotesHint is part of the registry, so find_hint() already handles the
        // "missing notes" case. If find_hint returns None, no strategy fired at all
        // (including NotesHint), which means every empty cell has at least one note
        // but no logical move is deducible → fall through to Reveal.
        let h = match hint::find_hint(state, solution) {
            Some(h) => h,
            None => {
                let sol_clone = solution.clone();
                self.perform_reveal(sol_clone);
                return;
            }
        };

        self.stats.hint_count += 1;
        self.anim.hint_blink = true;
        self.anim.hint_blink_tick = 0;
        self.active_hint = Some(h);
    }

    fn perform_reveal(&mut self, solution: Grid) {
        use crate::hint;
        use crate::puzzle::GameEvent;

        let state = match &self.game_state {
            Some(s) => s,
            None => return,
        };
        let (row, col) = match hint::most_constrained_cell(state) {
            Some(c) => c,
            None => return,
        };
        let digit = match solution.get(row, col).value() {
            Some(d) => d,
            None => return,
        };

        self.stats.hint_count += 1;

        if let Some(state) = &mut self.game_state {
            state.apply(GameEvent::SetDigit { row, col, digit });
        }
        self.check_completion(row, col);
    }

    fn move_cursor(&mut self, dr: i8, dc: i8) {
        let (r, c) = self.cursor;
        let new_r = ((r as i8 + dr).rem_euclid(9)) as usize;
        let new_c = ((c as i8 + dc).rem_euclid(9)) as usize;
        self.cursor = (new_r, new_c);
        self.nav_state.mode = NavMode::Input;
        self.nav_state.box_idx = None;
    }

    // ── Easter eggs ───────────────────────────────────────────────────────────

    fn handle_easter_egg(&mut self, egg: EasterEgg) {
        match egg {
            EasterEgg::GodMode => {
                self.stats.cheat_god_mode = true;
                self.easter_god_mode();
            }
            EasterEgg::FillNotes => {
                self.stats.cheat_fill_notes = true;
                self.easter_fill_notes();
            }
            EasterEgg::Xyzzy => self.set_overlay("Nothing happens."),
            EasterEgg::Sudo => {
                self.set_overlay("user is not in the sudoers file. This incident will be reported.")
            }
            EasterEgg::Help => self.set_overlay("This is not a text adventure."),
            EasterEgg::FortyTwo => self.set_overlay("42 — Life, the Universe, and Everything."),
            EasterEgg::KonamiCode => {
                let seed = self.clock.now_ms();
                self.anim.matrix_rain = Some(crate::tui::anim::MatrixRainAnim::new(seed));
                self.matrix_mode = true; // grid renders in Matrix green from frame 1
                self.needs_clear = true;
            }
            EasterEgg::MatrixMode => {
                self.matrix_mode = !self.matrix_mode;
                let msg = if self.matrix_mode {
                    "Wake up, Neo... The Matrix has you."
                } else {
                    "You took the blue pill."
                };
                self.set_overlay(msg);
                self.needs_clear = true;
            }
        }
    }

    fn set_overlay(&mut self, msg: &str) {
        self.info_overlay = Some((msg.into(), None, true, std::time::Instant::now()));
    }

    /// Show a persistent notice on the start screen (e.g. invalid CLI puzzle).
    /// The overlay must be dismissed manually; the screen stays at Start.
    /// No subtitle needed — the standard dismiss footer already says "press any key".
    pub fn set_start_notice(&mut self, msg: String) {
        self.info_overlay = Some((msg, None, false, std::time::Instant::now()));
    }

    /// Show the "puzzle contains errors" overlay — must be dismissed manually, no auto-timeout.
    fn set_puzzle_error_overlay(&mut self) {
        let strings = self.language.strings();
        self.info_overlay = Some((
            strings.puzzle_has_errors.into(),
            Some(strings.puzzle_errors_hint.into()),
            false,
            std::time::Instant::now(),
        ));
    }

    /// `iddqd` — fill every non-given cell with the correct solution value.
    fn easter_god_mode(&mut self) {
        let state = match &mut self.game_state {
            Some(s) => s,
            None => return,
        };
        // Build a givens-only grid and solve it.
        use crate::puzzle::Grid;
        let mut given_grid = Grid::empty();
        for r in 0..9 {
            for c in 0..9 {
                if let CellKind::Given(v) = state.grid().get(r, c) {
                    given_grid.set_given(r, c, v);
                }
            }
        }
        if let Some(solution) = solve_backtracking(given_grid) {
            use crate::puzzle::GameEvent;
            for r in 0..9 {
                for c in 0..9 {
                    if !matches!(state.grid().get(r, c), CellKind::Given(_)) {
                        if let Some(v) = solution.get(r, c).value() {
                            state.apply(GameEvent::SetDigit {
                                row: r,
                                col: c,
                                digit: v,
                            });
                        }
                    }
                }
            }
        }
    }

    /// `idkfa` — set a single correct note in every empty cell.
    fn easter_fill_notes(&mut self) {
        let state = match &mut self.game_state {
            Some(s) => s,
            None => return,
        };
        // Compute all valid candidates for every empty cell using constraint propagation.
        let candidates = CandidateGrid::from_grid(state.grid());
        use crate::puzzle::GameEvent;
        for r in 0..9 {
            for c in 0..9 {
                if matches!(state.grid().get(r, c), CellKind::Empty) {
                    let mask = candidates.mask(r, c);
                    for digit in 1u8..=9 {
                        if mask & (1 << digit) != 0 {
                            // Only toggle on if not already set.
                            if state.notes_mask(r, c) & (1 << digit) == 0 {
                                state.apply(GameEvent::ToggleNote {
                                    row: r,
                                    col: c,
                                    digit,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // ── Completion detection ──────────────────────────────────────────────────

    /// Call after every SetDigit to detect newly completed groups and trigger sweeps.
    fn check_completion(&mut self, changed_row: usize, changed_col: usize) {
        let state = match &self.game_state {
            Some(s) => s,
            None => return,
        };
        let grid = state.grid();

        let group_complete = |cells: &Vec<(usize, usize)>| -> bool {
            let mut seen = [false; 10];
            for &(r, c) in cells {
                match grid.get(r, c).value() {
                    Some(v) if v >= 1 && v <= 9 => {
                        if seen[v as usize] {
                            return false;
                        }
                        seen[v as usize] = true;
                    }
                    _ => return false,
                }
            }
            seen[1..=9].iter().all(|&b| b)
        };

        let box_idx = (changed_row / 3) * 3 + changed_col / 3;
        // Completion check uses reading-order cells; sweep uses direction-specific ordering.
        let groups = [
            (row_cells(changed_row), row_cells(changed_row)),
            (col_cells(changed_col), col_cells(changed_col)),
            (box_cells(box_idx), box_cells_serpentine(box_idx)),
        ];
        for (check_cells, sweep_cells) in &groups {
            if group_complete(check_cells) {
                self.anim.sweeps.push(SweepAnim::new(sweep_cells.clone()));
            }
        }

        // Full puzzle solved → firework
        if grid.is_solved() {
            self.anim.firework = Some(FireworkAnim::new());
        } else {
            // All cells filled but solution wrong → hint overlay (shown at most once).
            let all_filled =
                (0..9).all(|r| (0..9).all(|c| !matches!(grid.get(r, c), CellKind::Empty)));
            if all_filled && self.info_overlay.is_none() && !self.error_mode {
                self.set_puzzle_error_overlay();
            }
        }
    }

    /// Main event loop. Renders, reads input, dispatches until quit.
    /// Block until the terminal is at least MIN_COLS × MIN_ROWS.
    /// Renders an informational message and waits for `Event::Resize`.
    /// Returns immediately if the terminal is already large enough.
    fn wait_for_adequate_size(&self, out: &mut impl Write) -> io::Result<()> {
        loop {
            let (cols, rows) = crossterm::terminal::size()?;
            if cols >= MIN_COLS && rows >= MIN_ROWS {
                return Ok(());
            }

            // Clear and render the "too small" notice centred in the current window.
            queue!(out, SetBackgroundColor(Color::Black), Clear(ClearType::All))?;

            let strings = self.language.strings();
            let line1 = strings
                .resize_too_small
                .replacen("{}", &cols.to_string(), 1)
                .replacen("{}", &rows.to_string(), 1);
            let line2 = strings
                .resize_required
                .replacen("{}", &MIN_COLS.to_string(), 1)
                .replacen("{}", &MIN_ROWS.to_string(), 1);
            let line3 = strings.resize_hint;

            for (i, line) in [line1.as_str(), line2.as_str(), "", line3]
                .iter()
                .enumerate()
            {
                let col = cols.saturating_sub(line.chars().count() as u16) / 2;
                let row = rows.saturating_sub(4) / 2 + i as u16;
                queue!(
                    out,
                    MoveTo(col, row),
                    SetForegroundColor(if i == 3 {
                        Color::DarkGrey
                    } else {
                        Color::White
                    }),
                    Print(line)
                )?;
            }
            queue!(out, ResetColor)?;
            out.flush()?;

            // Wait for the next event — only Resize matters here.
            match event::read()? {
                Event::Key(key)
                    if key.kind == crossterm::event::KeyEventKind::Press
                        && (key.code == crossterm::event::KeyCode::Esc
                            || key.code == crossterm::event::KeyCode::Char('q')) =>
                {
                    // Allow quitting even from the resize-wait screen.
                    // Propagate by returning an io::Error so run() exits cleanly.
                    return Err(io::Error::new(io::ErrorKind::Interrupted, "quit"));
                }
                _ => {} // any other event (Resize, mouse, …) → loop and re-check
            }
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        let _terminal = Terminal::setup()?;
        let mut out = BufWriter::new(std::io::stdout());

        // Block until the terminal is large enough to render the full layout.
        // Returns Interrupted if the user presses Esc/q while waiting — treat as clean exit.
        match self.wait_for_adequate_size(&mut out) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::Interrupted => return Ok(()),
            Err(e) => return Err(e),
        }

        // Fill the entire screen with the background colour once at startup.
        // Subsequent frames overwrite content in place (no Clear per frame)
        // so there is no flicker, but unused terminal space stays black.
        queue!(
            out,
            SetBackgroundColor(self.colors.ui_background),
            Clear(ClearType::All)
        )?;
        out.flush()?;

        // Dirty-render tracking: only write to the terminal when state changed.
        // This prevents flicker in mouse mode caused by continuous re-rendering
        // even when the display content is identical.
        let mut needs_render = true; // always render the first frame
        let mut last_elapsed_s: u64 = u64::MAX; // detect timer-second ticks

        loop {
            // Drain any buffered input events that accumulated during a slow operation
            // (e.g. puzzle generation). This prevents stray key presses from being
            // processed as game actions on the very first frame after start_game().
            if self.drain_input {
                self.drain_input = false;
                while event::poll(Duration::from_millis(0))? {
                    let _ = event::read()?;
                }
                needs_render = true;
            }

            // Timer-second tick: re-render once per second so the clock stays current.
            let current_elapsed_s = self.elapsed_ms() / 1000;
            if current_elapsed_s != last_elapsed_s {
                last_elapsed_s = current_elapsed_s;
                needs_render = true;
            }

            // Render only when something changed (dirty-flag approach).
            // Always render for Generating screens and active animations.
            if needs_render
                || self.needs_clear
                || self.anim.is_active()
                || matches!(self.screen, AppScreen::Generating(_))
            {
                self.render_current(&mut out)?;
                out.flush()?;
                needs_render = false;
            }

            // ── Poll background generator ────────────────────────────────────
            // First: tick and check timeouts (only for Designer/pattern mode).
            if let AppScreen::Generating(ref mut gs) = self.screen {
                if !gs.bare_minimum && !gs.expert {
                    gs.tick_new_seed_expiry();
                    if !gs.show_new_seed && gs.started_at.elapsed().as_secs() >= 3 {
                        gs.try_new_seed();
                    }
                }
            }
            // Then: drain all pending messages without holding the mutable borrow long.
            let gen_result = if let AppScreen::Generating(ref mut gs) = self.screen {
                match gs.rx.try_recv() {
                    Ok(msg) => Some(msg),
                    Err(_) => None,
                }
            } else {
                None
            };
            // Handle incoming messages.
            match gen_result {
                Some(crate::tui::generating::GenMsg::BareMinimumProgress {
                    done,
                    total,
                    best_count,
                }) => {
                    if let AppScreen::Generating(ref mut gs) = self.screen {
                        gs.bm_done = done;
                        gs.bm_total = total;
                        gs.bm_best_count = best_count;
                        gs.verb_pos = done; // cycle verb with each attempt
                    }
                    needs_render = true;
                }
                Some(crate::tui::generating::GenMsg::Done(grid, difficulty)) => {
                    let (is_bare_minimum, is_expert, pattern_name) =
                        if let AppScreen::Generating(ref gs) = self.screen {
                            (gs.bare_minimum, gs.expert, gs.pattern.name_en.to_string())
                        } else {
                            (false, false, String::new())
                        };
                    self.enter_game(grid);
                    if is_bare_minimum || difficulty == Difficulty::BareMinimum {
                        self.stats.category = GameCategory::BareMinimum;
                    } else if is_expert || difficulty == Difficulty::Expert {
                        self.stats.category = GameCategory::Classic;
                    } else {
                        self.stats.category = GameCategory::Design;
                        self.stats.pattern_name = Some(pattern_name);
                    }
                    needs_render = true;
                }
                None => {}
            }

            // Shorten poll timeout when an animation is running or generating so frames advance.
            // Mouse mode uses the same 80 ms interval for event responsiveness, but no longer
            // forces a re-render every tick — only actual state changes trigger a redraw.
            let poll_ms = if self.anim.matrix_rain.is_some() {
                50
            } else if matches!(self.screen, AppScreen::Generating(_)) {
                50
            } else if self.anim.is_active() || self.mouse_mode {
                80
            } else {
                500
            };

            // Snapshot hover position before polling so we can detect changes.
            let prev_hover = self.hover_cell;

            if event::poll(Duration::from_millis(poll_ms))? {
                match event::read()? {
                    Event::Key(key)
                        if key.kind == crossterm::event::KeyEventKind::Press
                            || key.kind == crossterm::event::KeyEventKind::Repeat =>
                    {
                        // Active hint: any key dismisses it (key is consumed, not forwarded).
                        if self.active_hint.is_some() {
                            self.active_hint = None;
                            self.anim.hint_blink = false;
                            self.needs_clear = true;
                        // Hint warning: any key dismisses it (key is consumed, not forwarded).
                        } else if self.hint_warning.is_some() {
                            self.hint_warning = None;
                            self.needs_clear = true;
                        // Info-overlay: any key dismisses it early.
                        } else if self.info_overlay.is_some() {
                            self.info_overlay = None;
                            self.needs_clear = true;
                        } else {
                            // Feed raw char/direction to sequence detector (easter eggs).
                            let egg = match key.code {
                                crossterm::event::KeyCode::Char(c) => self.seq.push(c),
                                crossterm::event::KeyCode::Up => self
                                    .seq
                                    .push_direction(crate::tui::seq_detect::Direction::Up),
                                crossterm::event::KeyCode::Down => self
                                    .seq
                                    .push_direction(crate::tui::seq_detect::Direction::Down),
                                crossterm::event::KeyCode::Left => self
                                    .seq
                                    .push_direction(crate::tui::seq_detect::Direction::Left),
                                crossterm::event::KeyCode::Right => self
                                    .seq
                                    .push_direction(crate::tui::seq_detect::Direction::Right),
                                _ => None,
                            };
                            if let Some(egg) = egg {
                                self.handle_easter_egg(egg);
                            }
                            let action = map_key_to_action(key, &self.nav_state);
                            self.handle_action(action);
                        }
                        needs_render = true;
                    }
                    Event::Mouse(mouse_event)
                        if matches!(self.screen, AppScreen::Game) && self.mouse_mode =>
                    {
                        use crate::tui::input::map_mouse_to_action;
                        let action = map_mouse_to_action(mouse_event, true);
                        match action {
                            AppAction::MouseHover(r, c) => {
                                // Pure hover: update position, no hint/warning dismissal.
                                // needs_render is set below based on whether hover actually changed.
                                self.hover_cell = Some((r, c));
                            }
                            AppAction::MouseSelectCell(..) | AppAction::MouseButton(_) => {
                                // Clicks behave like key presses for hint/overlay dismissal.
                                if self.active_hint.is_some() {
                                    self.active_hint = None;
                                    self.anim.hint_blink = false;
                                    self.needs_clear = true;
                                } else if self.hint_warning.is_some() {
                                    self.hint_warning = None;
                                    self.needs_clear = true;
                                } else if self.info_overlay.is_some() {
                                    self.info_overlay = None;
                                    self.needs_clear = true;
                                } else {
                                    self.handle_action(action);
                                }
                                needs_render = true;
                            }
                            _ => {
                                // Move/drag outside the grid: clear the hover highlight
                                // so it doesn't linger on the last in-grid position.
                                use crossterm::event::MouseEventKind;
                                if matches!(
                                    mouse_event.kind,
                                    MouseEventKind::Moved | MouseEventKind::Drag(_)
                                ) {
                                    self.hover_cell = None;
                                }
                            }
                        }
                    }
                    Event::Resize(cols, rows) => {
                        if cols < MIN_COLS || rows < MIN_ROWS {
                            // Terminal shrank below minimum — pause and wait.
                            match self.wait_for_adequate_size(&mut out) {
                                Ok(()) => {}
                                Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                                    self.should_quit = true;
                                }
                                Err(e) => return Err(e),
                            }
                        }
                        // Always redraw after any resize.
                        self.needs_clear = true;
                    }
                    _ => {}
                }
            }

            // Hover position changed → re-render so the highlight moves.
            if self.hover_cell != prev_hover {
                needs_render = true;
            }

            // Advance animations every poll cycle (≈80 ms when active).
            // Matrix rain advances twice per frame so it completes in roughly half the time
            // without touching any visual parameters (trail length, colours, density).
            let anim_was_active = self.anim.is_active();
            self.anim.advance();
            if self.anim.matrix_rain.is_some() {
                self.anim.advance();
            }
            if anim_was_active || self.anim.is_active() {
                needs_render = true;
            }

            // Matrix rain finished → show the Neo message (matrix_mode already active).
            if matches!(&self.anim.matrix_rain, Some(r) if r.done()) {
                self.anim.matrix_rain = None;
                self.set_overlay("Wake up, Neo... The Matrix has you.");
                self.needs_clear = true;
            }

            // Auto-dismiss info overlay after 3 seconds (only when auto_dismiss=true).
            if let Some((_, _, auto_dismiss, shown_at)) = &self.info_overlay {
                if *auto_dismiss && shown_at.elapsed() >= Duration::from_secs(3) {
                    self.info_overlay = None;
                    self.needs_clear = true;
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn render_current(&mut self, out: &mut impl Write) -> io::Result<()> {
        if self.needs_clear {
            queue!(
                out,
                SetBackgroundColor(self.colors.ui_background),
                Clear(ClearType::All)
            )?;
            self.needs_clear = false;
        }

        // Boss Key: replace entire screen with fake terminal, skip normal rendering.
        if self.boss_mode {
            return crate::tui::render::boss::render_boss(out);
        }

        let strings = self.language.strings();

        match &self.screen {
            AppScreen::Start { selected } => render_frame(
                out,
                &Screen::Start {
                    selected: *selected,
                },
                &self.colors,
                self.given_style.as_ref(),
                self.filled_style.as_ref(),
                strings,
            ),
            AppScreen::DifficultySelect {
                selected,
                sym_focused,
            } => render_frame(
                out,
                &Screen::DifficultySelect {
                    selected: *selected,
                    sym_focused: *sym_focused,
                    symmetry: self.symmetry,
                },
                &self.colors,
                self.given_style.as_ref(),
                self.filled_style.as_ref(),
                strings,
            ),
            AppScreen::LanguageSelect { selected } => render_frame(
                out,
                &Screen::LanguageSelect {
                    selected: *selected,
                },
                &self.colors,
                self.given_style.as_ref(),
                self.filled_style.as_ref(),
                strings,
            ),
            AppScreen::ThemeSelect { selected } => render_frame(
                out,
                &Screen::ThemeSelect {
                    selected: *selected,
                },
                &self.colors,
                self.given_style.as_ref(),
                self.filled_style.as_ref(),
                strings,
            ),
            AppScreen::PatternSelect { selected } => render_frame(
                out,
                &Screen::PatternSelect {
                    selected: *selected,
                },
                &self.colors,
                self.given_style.as_ref(),
                self.filled_style.as_ref(),
                strings,
            ),
            AppScreen::Generating(ref gs) => {
                let bare_minimum = if gs.bare_minimum {
                    Some((gs.bm_done, gs.bm_total, gs.bm_best_count))
                } else {
                    None
                };
                let screen = Screen::Generating {
                    verb: gs.current_verb(),
                    countdown: gs.countdown_secs(),
                    show_new_seed: gs.show_new_seed,
                    bare_minimum,
                };
                render_frame(
                    out,
                    &screen,
                    &self.colors,
                    self.given_style.as_ref(),
                    self.filled_style.as_ref(),
                    strings,
                )
            }
            AppScreen::Game => {
                if let Some(state) = &self.game_state {
                    let scan_digit = if self.scan_mode {
                        let (r, c) = self.cursor;
                        match state.grid().get(r, c) {
                            CellKind::Given(d) | CellKind::Filled(d) => Some(d),
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let solution_ref = self.solution.as_ref();
                    let game_screen = || Screen::Game {
                        state,
                        cursor: self.cursor,
                        note_mode: self.note_mode,
                        scan_mode: self.scan_mode,
                        error_mode: self.error_mode,
                        solution: solution_ref,
                        errors_shown: self.stats.errors_shown,
                        elapsed_ms: self.elapsed_ms(),
                        paused: self.paused,
                        nav: &self.nav_state,
                        anim: &self.anim,
                        scan_digit,
                        hint: self.active_hint.as_ref(),
                        hint_warning: self.hint_warning,
                        hint_count: self.stats.hint_count,
                        matrix_mode: self.matrix_mode,
                        mouse_mode: self.mouse_mode,
                        hover_cell: self.hover_cell,
                    };
                    let screen = match &self.confirm_pending {
                        Some(ConfirmAction::QuitGame) => Screen::Confirm {
                            underneath: Box::new(game_screen()),
                            title: strings.confirm_quit_title.into(),
                            options: strings.confirm_quit_options.into(),
                        },
                        None => game_screen(),
                    };
                    render_frame(
                        out,
                        &screen,
                        &self.colors,
                        self.given_style.as_ref(),
                        self.filled_style.as_ref(),
                        strings,
                    )?;
                    Ok(())
                } else {
                    Ok(())
                }
            }
        }?;

        // Matrix rain overlay — drawn over the grid area when active.
        if matches!(self.screen, AppScreen::Game) {
            if let Some(rain) = &self.anim.matrix_rain {
                crate::tui::render::matrix_rain::render_matrix_rain(
                    out,
                    (1, 2),
                    rain,
                    self.colors.ui_background,
                )?;
            }
        }

        // Info overlay is drawn on top of every screen (start, game, difficulty, …).
        if let Some((msg, subtitle, _, _)) = &self.info_overlay {
            let msg = msg.clone();
            let sub = subtitle.as_deref();
            render_info_overlay(out, (15, 10), &msg, sub, strings.dismiss, &self.colors)?;
        }
        Ok(())
    }
}

/// Convert numpad box index and within-box cell index to grid (row, col).
///
/// Numpad layout (0-indexed from key '1'=0 to '9'=8):
///   6 7 8    (keys 7 8 9 — top row)
///   3 4 5    (keys 4 5 6 — middle row)
///   0 1 2    (keys 1 2 3 — bottom row)
fn numpad_to_cell(box_idx: usize, cell_idx: usize) -> (usize, usize) {
    // Box: row of boxes = 2 - box_idx/3, col of boxes = box_idx%3
    let box_row = 2 - box_idx / 3;
    let box_col = box_idx % 3;
    // Cell within box: same layout
    let cell_row = 2 - cell_idx / 3;
    let cell_col = cell_idx % 3;
    (box_row * 3 + cell_row, box_col * 3 + cell_col)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timer::FakeClock;

    fn make_app() -> App {
        App::new(Box::new(FakeClock { ms: 1000 }))
    }

    #[test]
    fn game_stats_has_category_fields() {
        let stats = GameStats::default();
        assert!(matches!(stats.category, GameCategory::Classic));
        assert!(stats.pattern_name.is_none());
    }

    #[test]
    fn initial_screen_is_start() {
        let app = make_app();
        assert!(matches!(app.screen, AppScreen::Start { .. }));
    }

    #[test]
    fn selecting_new_game_goes_to_difficulty() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        assert!(matches!(app.screen, AppScreen::DifficultySelect { .. }));
    }

    #[test]
    fn selecting_difficulty_starts_game() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        assert!(matches!(app.screen, AppScreen::Game));
        assert!(app.game_state.is_some());
    }

    #[test]
    fn escape_from_game_shows_quit_confirm() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        assert!(matches!(app.screen, AppScreen::Game));
        // Esc opens confirm dialog, does not immediately leave
        app.handle_action(AppAction::Back);
        assert!(matches!(app.screen, AppScreen::Game));
        assert!(matches!(app.confirm_pending, Some(ConfirmAction::QuitGame)));
    }

    #[test]
    fn confirm_yes_quits_to_start() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Back); // open confirm
        app.handle_action(AppAction::ConfirmYes); // confirm → Start
        assert!(matches!(app.screen, AppScreen::Start { .. }));
        assert!(app.confirm_pending.is_none());
    }

    #[test]
    fn confirm_no_returns_to_game() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Back); // open confirm
        app.handle_action(AppAction::ConfirmNo); // dismiss → stay in game
        assert!(matches!(app.screen, AppScreen::Game));
        assert!(app.confirm_pending.is_none());
    }

    #[test]
    fn arrow_keys_move_cursor_with_wrap() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::MoveRight);
        assert_eq!(app.cursor, (0, 1));
        app.handle_action(AppAction::MoveLeft);
        assert_eq!(app.cursor, (0, 0));
        // Wrap: left from col 0 -> col 8
        app.handle_action(AppAction::MoveLeft);
        assert_eq!(app.cursor, (0, 8));
    }

    #[test]
    fn pause_toggles_paused_state() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        assert!(!app.paused);
        app.handle_action(AppAction::Pause);
        assert!(app.paused);
        app.handle_action(AppAction::Pause);
        assert!(!app.paused);
    }

    #[test]
    fn clear_cell_clears_immediately_without_confirm() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        // Set digit first so there's something to clear
        app.handle_action(AppAction::Digit(5));
        app.handle_action(AppAction::ClearCell);
        // No confirm dialog — cleared immediately
        assert!(app.confirm_pending.is_none());
        assert!(matches!(app.screen, AppScreen::Game));
    }

    #[test]
    fn requesting_hint_on_game_screen_sets_active_hint() {
        // Nearly-solved puzzle: last digit missing (the '0' at position 71)
        let puzzle =
            "534678912672195348198342567859761423426853791713924856961537284287419630345286179";
        let grid = crate::puzzle::Grid::from_str(puzzle).unwrap();
        let solution = crate::solver::backtracking::solve_backtracking(grid.clone());
        let mut app = App::new(Box::new(FakeClock { ms: 1000 }));
        app.game_state = Some(crate::puzzle::GameState::new(grid));
        app.solution = solution;
        app.screen = AppScreen::Game;
        app.handle_action(AppAction::RequestHint);
        // Either a hint was found, or reveal was performed (hint_count > 0 either way)
        assert!(
            app.stats.hint_count > 0 || app.active_hint.is_some(),
            "RequestHint should produce some response"
        );
    }

    #[test]
    fn numpad_navigation_selects_cell() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        // Numpad '1' -> box_idx 0 (bottom-left box)
        app.handle_action(AppAction::NumpadBox(0));
        assert_eq!(app.nav_state.box_idx, Some(0));
        // Numpad '9' -> cell_idx 8 (top-right cell in box)
        app.handle_action(AppAction::NumpadCell(8));
        let (r, c) = app.cursor;
        assert!(r < 9 && c < 9, "cursor out of bounds: ({}, {})", r, c);
    }

    #[test]
    fn hint_request_with_wrong_digit_sets_warning_not_hint() {
        use crate::puzzle::CellKind;
        use crate::timer::SystemClock;

        let mut app = App::new(Box::new(SystemClock));
        app.start_game(crate::generator::Difficulty::Easy);

        // Find an empty cell and fill it with the WRONG digit
        let (wrong_r, wrong_c, wrong_digit) = {
            let state = app.game_state.as_ref().unwrap();
            let sol = app.solution.as_ref().unwrap();
            let mut found = None;
            'outer: for r in 0..9 {
                for c in 0..9 {
                    if matches!(state.grid().get(r, c), CellKind::Empty) {
                        let correct = sol.get(r, c).value().unwrap();
                        let wrong = if correct == 9 { 1 } else { correct + 1 };
                        found = Some((r, c, wrong));
                        break 'outer;
                    }
                }
            }
            found.expect("no empty cell found")
        };

        app.game_state
            .as_mut()
            .unwrap()
            .apply(crate::puzzle::event::GameEvent::SetDigit {
                row: wrong_r,
                col: wrong_c,
                digit: wrong_digit,
            });

        let hint_count_before = app.stats.hint_count;
        app.handle_action(crate::tui::input::AppAction::RequestHint);

        assert!(app.hint_warning.is_some(), "hint_warning should be set");
        assert!(app.active_hint.is_none(), "active_hint should NOT be set");
        assert_eq!(
            app.stats.hint_count,
            hint_count_before + 1,
            "hint_count should increment"
        );
    }

    #[test]
    fn hint_warning_dismissed_by_any_key() {
        use crate::timer::SystemClock;
        let mut app = App::new(Box::new(SystemClock));
        app.hint_warning = Some(("Warning", "Test warning"));
        app.handle_action(crate::tui::input::AppAction::MoveRight);
        assert!(app.hint_warning.is_none());
    }

    #[test]
    fn m_key_toggles_mouse_mode() {
        let mut app = make_app();
        // Navigate to game
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        assert!(matches!(app.screen, AppScreen::Game));
        assert!(!app.mouse_mode);
        // Toggle on
        app.handle_action(AppAction::ToggleMouseMode);
        assert!(app.mouse_mode);
        assert!(app.hover_cell.is_none());
        // Toggle off
        app.handle_action(AppAction::ToggleMouseMode);
        assert!(!app.mouse_mode);
        assert!(app.hover_cell.is_none());
    }

    #[test]
    fn mouse_hover_updates_hover_cell() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::ToggleMouseMode);
        app.handle_action(AppAction::MouseHover(3, 5));
        assert_eq!(app.hover_cell, Some((3, 5)));
        // Hover elsewhere
        app.handle_action(AppAction::MouseHover(0, 0));
        assert_eq!(app.hover_cell, Some((0, 0)));
    }

    #[test]
    fn mouse_select_moves_cursor() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::ToggleMouseMode);
        app.handle_action(AppAction::MouseSelectCell(4, 7));
        assert_eq!(app.cursor, (4, 7));
    }

    #[test]
    fn enter_game_resets_mouse_mode() {
        use crate::timer::FakeClock;
        let mut app = App::new(Box::new(FakeClock { ms: 1000 }));
        // Manually force mouse_mode true without IO (simulates prior activation)
        app.mouse_mode = true;
        app.hover_cell = Some((2, 3));
        // Start a game — should reset mouse state
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        assert!(!app.mouse_mode);
        assert!(app.hover_cell.is_none());
    }

    #[test]
    fn selecting_designer_from_difficulty_goes_to_pattern_select() {
        use crate::timer::SystemClock;
        let mut app = App::new(Box::new(SystemClock));
        app.screen = AppScreen::DifficultySelect {
            selected: 0,
            sym_focused: false,
        };
        for _ in 0..6 {
            app.handle_action(AppAction::MoveDown);
        }
        app.handle_action(AppAction::Enter);
        assert!(matches!(app.screen, AppScreen::PatternSelect { .. }));
    }

    #[test]
    fn pattern_select_wraps_around() {
        use crate::timer::SystemClock;
        let mut app = App::new(Box::new(SystemClock));
        app.screen = AppScreen::PatternSelect { selected: 0 };
        app.handle_action(AppAction::MoveLeft);
        assert!(matches!(
            app.screen,
            AppScreen::PatternSelect { selected: 29 }
        ));
    }

    #[test]
    fn pattern_select_back_goes_to_difficulty() {
        use crate::timer::SystemClock;
        let mut app = App::new(Box::new(SystemClock));
        app.screen = AppScreen::PatternSelect { selected: 0 };
        app.handle_action(AppAction::Back);
        assert!(matches!(app.screen, AppScreen::DifficultySelect { .. }));
    }

    #[test]
    fn expert_difficulty_enters_generating_screen() {
        // Expert (index 4) must route through Generating, not start a Game immediately.
        let mut app = make_app();
        app.screen = AppScreen::DifficultySelect { selected: 4, sym_focused: false };
        app.handle_action(AppAction::Enter);
        assert!(matches!(app.screen, AppScreen::Generating(_)),
            "Expert should open Generating screen, got {:?}", std::mem::discriminant(&app.screen));
    }

    #[test]
    fn back_from_expert_generating_returns_to_index_4() {
        // Back on the Generating screen while generating an Expert puzzle must return
        // to DifficultySelect at index 4 (Expert's position).
        let mut app = make_app();
        app.screen = AppScreen::Generating(
            crate::tui::generating::GeneratingState::new_expert(false)
        );
        app.handle_action(AppAction::Back);
        assert!(matches!(
            app.screen,
            AppScreen::DifficultySelect { selected: 4, sym_focused: false }
        ));
    }
}
