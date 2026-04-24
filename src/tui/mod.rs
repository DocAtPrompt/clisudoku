pub mod anim;
pub mod colors;
pub mod digit_style;
pub mod input;
pub mod render;
pub mod seq_detect;
pub mod terminal;

use crate::generator::{Difficulty, PuzzleGenerator};
use crate::puzzle::{CellKind, GameState};
use crate::solver::backtracking::solve_backtracking;
use crate::solver::candidates::CandidateGrid;
use crate::timer::Clock;
use crate::tui::anim::{AnimState, FireworkAnim, SweepAnim};
use crate::tui::colors::ColorScheme;
use crate::tui::digit_style::{DigitStyle, RetroStyle};
use crate::tui::input::{map_key_to_action, AppAction, NavMode, NavState};
use crate::tui::render::{render_frame, render_info_overlay, Screen};
use crate::tui::render::{box_cells, col_cells, row_cells};
use crate::tui::seq_detect::{EasterEgg, SeqDetector};
use crate::tui::terminal::Terminal;
use crossterm::event::{self, Event};
use crossterm::{queue, style::SetBackgroundColor, terminal::{Clear, ClearType}};
use std::io::{self, BufWriter, Write};
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum AppScreen {
    Start { selected: usize },
    DifficultySelect { selected: usize },
    Game,
}

/// Pending confirmation action.
#[derive(Debug, Clone)]
pub enum ConfirmAction {
    ClearCell { row: usize, col: usize },
    QuitGame,
}

pub struct App {
    pub screen: AppScreen,
    pub game_state: Option<GameState>,
    pub cursor: (usize, usize),
    pub nav_state: NavState,
    pub note_mode: bool,
    pub paused: bool,
    /// Boss Key active — game hidden behind a fake terminal.
    pub boss_mode: bool,
    pub confirm_pending: Option<ConfirmAction>,
    pub should_quit: bool,
    /// Set whenever the screen variant changes so the next render clears first.
    pub needs_clear: bool,
    clock: Box<dyn Clock>,
    game_start_ms: u64,
    /// Elapsed ms frozen at the moment the game was paused or boss key was pressed.
    paused_elapsed_ms: u64,
    colors: ColorScheme,
    style: Box<dyn DigitStyle>,
    /// Typed sequence detector for easter eggs.
    seq: SeqDetector,
    /// Active animations (sweep + firework).
    pub anim: AnimState,
    /// One-line info overlay text (easter egg messages); dismissed by any key.
    pub info_overlay: Option<String>,
}

impl App {
    pub fn new(clock: Box<dyn Clock>) -> Self {
        Self {
            screen: AppScreen::Start { selected: 0 },
            game_state: None,
            cursor: (0, 0),
            nav_state: NavState::default(),
            note_mode: false,
            paused: false,
            boss_mode: false,
            confirm_pending: None,
            should_quit: false,
            needs_clear: false,
            game_start_ms: 0,
            paused_elapsed_ms: 0,
            colors: ColorScheme::default(),
            style: Box::new(RetroStyle),
            clock,
            seq: SeqDetector::default(),
            anim: AnimState::default(),
            info_overlay: None,
        }
    }

    /// Start a new game at the given difficulty.
    fn start_game(&mut self, difficulty: Difficulty) {
        let seed = self.clock.now_ms();
        let puzzle = PuzzleGenerator::new(seed).generate(difficulty);
        self.game_state = Some(GameState::new(puzzle));
        self.cursor = (0, 0);
        self.nav_state = NavState::default();
        self.note_mode = false;
        self.paused = false;
        self.game_start_ms = self.clock.now_ms();
        self.screen = AppScreen::Game;
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
        // Confirm dialog takes priority
        if self.confirm_pending.is_some() {
            match action {
                AppAction::ConfirmYes => {
                    match self.confirm_pending.take() {
                        Some(ConfirmAction::ClearCell { row, col }) => {
                            if let Some(state) = &mut self.game_state {
                                use crate::puzzle::GameEvent;
                                state.apply(GameEvent::ClearCell { row, col });
                            }
                        }
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
            AppScreen::DifficultySelect { selected } => self.handle_difficulty_action(action, *selected),
            AppScreen::Game => self.handle_game_action(action),
        }
    }

    fn handle_start_action(&mut self, action: AppAction, selected: usize) {
        use render::start_screen::START_ITEMS;
        match action {
            AppAction::MoveUp => {
                self.screen = AppScreen::Start {
                    selected: selected.saturating_sub(1),
                };
            }
            AppAction::MoveDown => {
                self.screen = AppScreen::Start {
                    selected: (selected + 1).min(START_ITEMS.len() - 1),
                };
            }
            AppAction::Enter => match selected {
                0 => {
                    self.screen = AppScreen::DifficultySelect { selected: 0 };
                    self.needs_clear = true;
                }
                _ => self.should_quit = true,
            },
            AppAction::Back => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_difficulty_action(&mut self, action: AppAction, selected: usize) {
        use render::start_screen::DIFFICULTY_ITEMS;
        match action {
            AppAction::MoveUp => {
                self.screen = AppScreen::DifficultySelect {
                    selected: selected.saturating_sub(1),
                };
            }
            AppAction::MoveDown => {
                self.screen = AppScreen::DifficultySelect {
                    selected: (selected + 1).min(DIFFICULTY_ITEMS.len() - 1),
                };
            }
            AppAction::Enter => {
                let difficulty = match selected {
                    0 => Difficulty::Easy,
                    1 => Difficulty::Medium,
                    _ => Difficulty::Hard,
                };
                self.start_game(difficulty);
                self.needs_clear = true;
            }
            AppAction::Back => {
                self.screen = AppScreen::Start { selected: 0 };
                self.needs_clear = true;
            }
            _ => {}
        }
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
                    NavMode::Input      => NavMode::Navigation,
                    NavMode::Navigation => { self.nav_state.box_idx = None; NavMode::Input }
                };
            }
            AppAction::ToggleMode => {
                self.note_mode = !self.note_mode;
            }
            AppAction::Digit(d) => {
                let (row, col) = self.cursor;
                if let Some(state) = &mut self.game_state {
                    use crate::puzzle::GameEvent;
                    let event = if self.note_mode {
                        GameEvent::ToggleNote { row, col, digit: d }
                    } else {
                        GameEvent::SetDigit { row, col, digit: d }
                    };
                    state.apply(event);
                }
                if !self.note_mode {
                    self.check_completion(row, col);
                }
            }
            AppAction::ClearCell => {
                self.confirm_pending = Some(ConfirmAction::ClearCell {
                    row: self.cursor.0,
                    col: self.cursor.1,
                });
                self.needs_clear = true;
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
            _ => {}
        }
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
            EasterEgg::GodMode => self.easter_god_mode(),
            EasterEgg::FillNotes => self.easter_fill_notes(),
            EasterEgg::Xyzzy    => self.info_overlay = Some("Nothing happens.".into()),
            EasterEgg::Sudo     => self.info_overlay = Some(
                "user is not in the sudoers file. This incident will be reported.".into()
            ),
            EasterEgg::Help     => self.info_overlay = Some(
                "This is not a text adventure.".into()
            ),
            EasterEgg::FortyTwo => self.info_overlay = Some(
                "42 — Life, the Universe, and Everything.".into()
            ),
        }
    }

    /// `iddqd` — fill every non-given cell with the correct solution value.
    fn easter_god_mode(&mut self) {
        let state = match &mut self.game_state { Some(s) => s, None => return };
        // Build a givens-only grid and solve it.
        use crate::puzzle::Grid;
        let mut given_grid = Grid::empty();
        for r in 0..9 { for c in 0..9 {
            if let CellKind::Given(v) = state.grid().get(r, c) {
                given_grid.set_given(r, c, v);
            }
        }}
        if let Some(solution) = solve_backtracking(given_grid) {
            use crate::puzzle::GameEvent;
            for r in 0..9 { for c in 0..9 {
                if !matches!(state.grid().get(r, c), CellKind::Given(_)) {
                    if let Some(v) = solution.get(r, c).value() {
                        state.apply(GameEvent::SetDigit { row: r, col: c, digit: v });
                    }
                }
            }}
        }
    }

    /// `idkfa` — set a single correct note in every empty cell.
    fn easter_fill_notes(&mut self) {
        let state = match &mut self.game_state { Some(s) => s, None => return };
        // Solve to get the correct values, then set one note per empty cell.
        use crate::puzzle::Grid;
        let mut given_grid = Grid::empty();
        for r in 0..9 { for c in 0..9 {
            if let CellKind::Given(v) = state.grid().get(r, c) {
                given_grid.set_given(r, c, v);
            }
        }}
        if let Some(solution) = solve_backtracking(given_grid) {
            use crate::puzzle::GameEvent;
            for r in 0..9 { for c in 0..9 {
                if matches!(state.grid().get(r, c), CellKind::Empty) {
                    if let Some(v) = solution.get(r, c).value() {
                        state.apply(GameEvent::ToggleNote { row: r, col: c, digit: v });
                    }
                }
            }}
        }
    }

    // ── Completion detection ──────────────────────────────────────────────────

    /// Call after every SetDigit to detect newly completed groups and trigger sweeps.
    fn check_completion(&mut self, changed_row: usize, changed_col: usize) {
        let state = match &self.game_state { Some(s) => s, None => return };
        let grid = state.grid();

        let group_complete = |cells: &Vec<(usize, usize)>| -> bool {
            let mut seen = [false; 10];
            for &(r, c) in cells {
                match grid.get(r, c).value() {
                    Some(v) if v >= 1 && v <= 9 => {
                        if seen[v as usize] { return false; }
                        seen[v as usize] = true;
                    }
                    _ => return false,
                }
            }
            seen[1..=9].iter().all(|&b| b)
        };

        let box_idx = (changed_row / 3) * 3 + changed_col / 3;
        let groups: &[(Vec<(usize, usize)>, bool)] = &[
            (row_cells(changed_row), true),
            (col_cells(changed_col), true),
            (box_cells(box_idx),     true),
        ];
        for (cells, _) in groups {
            if group_complete(cells) {
                self.anim.sweeps.push(SweepAnim::new(cells.clone()));
            }
        }

        // Full puzzle solved → firework
        if grid.is_solved() {
            self.anim.firework = Some(FireworkAnim::new());
        }
    }

    /// Main event loop. Renders, reads input, dispatches until quit.
    pub fn run(&mut self) -> io::Result<()> {
        let _terminal = Terminal::setup()?;
        let mut out = BufWriter::new(std::io::stdout());

        // Fill the entire screen with the background colour once at startup.
        // Subsequent frames overwrite content in place (no Clear per frame)
        // so there is no flicker, but unused terminal space stays black.
        queue!(out, SetBackgroundColor(self.colors.ui_background), Clear(ClearType::All))?;
        out.flush()?;

        loop {
            self.render_current(&mut out)?;
            out.flush()?;

            // Shorten poll timeout when an animation is running so frames advance.
            let poll_ms = if self.anim.is_active() { 80 } else { 500 };

            if event::poll(Duration::from_millis(poll_ms))? {
                match event::read()? {
                    Event::Key(key) => {
                        // Info-overlay: any key dismisses it.
                        if self.info_overlay.is_some() {
                            self.info_overlay = None;
                        } else {
                            // Feed raw char to sequence detector (easter eggs).
                            if let crossterm::event::KeyCode::Char(c) = key.code {
                                if let Some(egg) = self.seq.push(c) {
                                    self.handle_easter_egg(egg);
                                }
                            }
                            let action = map_key_to_action(key, &self.nav_state);
                            self.handle_action(action);
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }

            // Advance animations every poll cycle (≈80 ms when active).
            self.anim.advance();

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn render_current(&mut self, out: &mut impl Write) -> io::Result<()> {
        if self.needs_clear {
            queue!(out, SetBackgroundColor(self.colors.ui_background), Clear(ClearType::All))?;
            self.needs_clear = false;
        }

        // Boss Key: replace entire screen with fake terminal, skip normal rendering.
        if self.boss_mode {
            return crate::tui::render::boss::render_boss(out);
        }

        match &self.screen {
            AppScreen::Start { selected } => {
                render_frame(out, &Screen::Start { selected: *selected }, &self.colors, self.style.as_ref())
            }
            AppScreen::DifficultySelect { selected } => {
                render_frame(
                    out,
                    &Screen::DifficultySelect { selected: *selected },
                    &self.colors,
                    self.style.as_ref(),
                )
            }
            AppScreen::Game => {
                if let Some(state) = &self.game_state {
                    let game_screen = || Screen::Game {
                        state,
                        cursor: self.cursor,
                        note_mode: self.note_mode,
                        elapsed_ms: self.elapsed_ms(),
                        paused: self.paused,
                        nav: &self.nav_state,
                        anim: &self.anim,
                    };
                    let screen = match &self.confirm_pending {
                        Some(ConfirmAction::ClearCell { .. }) => Screen::Confirm {
                            underneath: Box::new(game_screen()),
                            title:   "Clear this cell?".into(),
                            options: "[Y]es  [N]o".into(),
                        },
                        Some(ConfirmAction::QuitGame) => Screen::Confirm {
                            underneath: Box::new(game_screen()),
                            title:   "Quit game?".into(),
                            options: "[Y]es  [N]o  [S]ave & quit".into(),
                        },
                        None => game_screen(),
                    };
                    render_frame(out, &screen, &self.colors, self.style.as_ref())?;
                    if let Some(msg) = &self.info_overlay {
                        let msg = msg.clone();
                        render_info_overlay(out, (15, 10), &msg, &self.colors)?;
                    }
                    Ok(())
                } else {
                    Ok(())
                }
            }
        }
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
        app.handle_action(AppAction::Back);       // open confirm
        app.handle_action(AppAction::ConfirmYes); // confirm → Start
        assert!(matches!(app.screen, AppScreen::Start { .. }));
        assert!(app.confirm_pending.is_none());
    }

    #[test]
    fn confirm_no_returns_to_game() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Back);       // open confirm
        app.handle_action(AppAction::ConfirmNo);  // dismiss → stay in game
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
    fn clear_cell_on_game_shows_confirm() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::ClearCell);
        assert!(matches!(app.confirm_pending, Some(_)));
    }

    #[test]
    fn confirm_no_dismisses_dialog() {
        let mut app = make_app();
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::Enter);
        app.handle_action(AppAction::ClearCell);
        app.handle_action(AppAction::ConfirmNo);
        assert!(app.confirm_pending.is_none());
        assert!(matches!(app.screen, AppScreen::Game));
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
}
