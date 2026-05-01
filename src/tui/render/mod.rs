// src/tui/render/mod.rs
pub mod boss;
pub mod cell;
pub mod confirm;
pub mod firework;
pub mod generating;
pub mod grid;
pub mod matrix_rain;
pub mod pattern_select;
pub mod start_screen;
pub mod status_bar;

use crate::i18n::Strings;
use crate::puzzle::{GameState, Grid};
use crate::tui::anim::AnimState;
use crate::tui::colors::ColorScheme;
use crate::tui::digit_style::DigitStyle;
use crate::tui::input::NavState;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io::{self, Write};

/// All possible UI screens.
pub enum Screen<'a> {
    Start { selected: usize },
    DifficultySelect { selected: usize, sym_focused: bool, symmetry: bool },
    LanguageSelect { selected: usize },
    ThemeSelect { selected: usize },
    Game {
        state: &'a GameState,
        cursor: (usize, usize),
        note_mode: bool,
        /// Whether passive digit scan mode is active.
        scan_mode: bool,
        /// Whether wrong solution digits are shown in red.
        error_mode: bool,
        /// Pre-computed solution (always provided when available).
        solution: Option<&'a Grid>,
        /// Error count for panel display.
        errors_shown: u32,
        elapsed_ms: u64,
        paused: bool,
        nav: &'a NavState,
        anim: &'a AnimState,
        /// Digit at the cursor cell; `Some` only when `scan_mode` is true.
        scan_digit: Option<u8>,
        /// Active hint to highlight (cause/elim/target cells).
        hint: Option<&'a crate::hint::Hint>,
        /// Warning text when hint pre-check failed: (name, explanation).
        hint_warning: Option<(&'a str, &'a str)>,
        /// Number of hints requested this game (for panel display).
        hint_count: u32,
        /// Matrix Mode active — digits rendered in Matrix green.
        matrix_mode: bool,
    },
    PatternSelect { selected: usize },
    Generating {
        verb:          &'a str,
        countdown:     u8,
        show_new_seed: bool,
    },
    Confirm {
        /// Screen rendered underneath the overlay.
        underneath: Box<Screen<'a>>,
        /// First line: short description of what is being confirmed.
        title: String,
        /// Second line: the available key options.
        options: String,
    },
}

/// Render the full terminal frame for the given screen.
pub fn render_frame(
    out: &mut impl Write,
    screen: &Screen<'_>,
    colors: &ColorScheme,
    style: &dyn DigitStyle,
    strings: &'static Strings,
) -> io::Result<()> {
    // No full Clear — we overwrite every position explicitly via MoveTo.
    queue!(out, MoveTo(0, 0))?;

    match screen {
        Screen::Start { selected } => {
            start_screen::render_start(out, (2, 4), *selected, strings, colors)?;
        }
        Screen::DifficultySelect { selected, sym_focused, symmetry } => {
            start_screen::render_difficulty(out, (2, 4), *selected, *sym_focused, *symmetry, strings, colors)?;
        }
        Screen::LanguageSelect { selected } => {
            start_screen::render_language(out, (2, 4), *selected, strings, colors)?;
        }
        Screen::ThemeSelect { selected } => {
            start_screen::render_theme(out, (2, 4), *selected, strings, colors)?;
        }
        Screen::PatternSelect { selected } => {
            pattern_select::render_pattern_select(out, *selected, strings, colors)?;
        }
        Screen::Generating { verb, countdown, show_new_seed } => {
            crate::tui::render::generating::render_generating(
                out, verb, *countdown, *show_new_seed, strings, colors,
            )?;
        }
        Screen::Game { state, cursor, note_mode, scan_mode, error_mode, solution, errors_shown, elapsed_ms, paused, nav, anim, scan_digit, hint, hint_warning, hint_count, matrix_mode } => {
            grid::render_grid(out, (1, 2), state, *cursor, *note_mode, *paused, nav, anim, *scan_digit, *error_mode, *solution, *hint, colors, style, *matrix_mode)?;
            // Count filled cells and per-digit placements for the panel display.
            let mut digit_counts = [0u8; 10];
            let mut filled_count = 0u8;
            for r in 0..9 {
                for c in 0..9 {
                    if let Some(v) = state.grid().get(r, c).value() {
                        if v >= 1 && v <= 9 { digit_counts[v as usize] += 1; }
                        filled_count += 1;
                    }
                }
            }
            // Panel to the right of the grid: col 2 + 73 (grid) + 2 (gap) = 77
            // hint_warning takes priority over active hint text
            let hint_text: Option<(&str, &str)> = if let Some((name, expl)) = hint_warning {
                Some((name, expl))
            } else {
                hint.map(|h| {
                    if std::ptr::eq(strings, &crate::i18n::DE) {
                        (h.name_de, h.explanation_de.as_str())
                    } else {
                        (h.name_en, h.explanation_en.as_str())
                    }
                })
            };
            status_bar::render_panel(out, (1, 77), *elapsed_ms, *note_mode, *scan_mode, *error_mode, *errors_shown, filled_count, digit_counts, *scan_digit, colors, strings, *hint_count, hint_text)?;
            if *paused {
                render_paused_overlay(out, strings.resume_hint, colors)?;
            }
            // Firework overlay on top of everything
            if let Some(fw) = &anim.firework {
                firework::render_firework(out, (1, 2), fw, colors)?;
            }
        }
        Screen::Confirm { underneath, title, options } => {
            render_frame(out, underneath, colors, style, strings)?;
            confirm::render_confirm(out, (17, 20), title, options, colors)?;
        }
    }

    queue!(out, ResetColor)
}

/// Render a simple read-only info overlay (easter egg messages, puzzle hints, etc.).
/// Dismissed by any key. Centred at `(row, col)`.
///
/// If `subtitle` is `Some`, an extra content row is rendered between the message and
/// the dismiss line, making the overlay one row taller.
pub fn render_info_overlay(
    out: &mut impl Write,
    (row_off, col_off): (u16, u16),
    message: &str,
    subtitle: Option<&str>,
    dismiss: &str,
    colors: &ColorScheme,
) -> io::Result<()> {
    // Reuse confirm colours: cyan border, dark-grey bg, white text.
    let inner = message.chars().count()
        .max(subtitle.map(|s| s.chars().count()).unwrap_or(0))
        .max(dismiss.chars().count())
        .max(20);
    let border_h = "─".repeat(inner + 4);
    let _ = colors;

    queue!(out,
        MoveTo(col_off, row_off),
        SetForegroundColor(Color::Cyan), SetBackgroundColor(Color::DarkGrey),
        Print(format!("┌{}┐", border_h)),

        MoveTo(col_off, row_off + 1),
        SetForegroundColor(Color::Cyan),  SetBackgroundColor(Color::DarkGrey), Print('│'),
        SetForegroundColor(Color::White), SetBackgroundColor(Color::DarkGrey),
        Print(format!("  {:<inner$}  ", message, inner = inner)),
        SetForegroundColor(Color::Cyan),  SetBackgroundColor(Color::DarkGrey), Print('│'),
    )?;

    let mut next_row = row_off + 2;

    if let Some(sub) = subtitle {
        queue!(out,
            MoveTo(col_off, next_row),
            SetForegroundColor(Color::Cyan),  SetBackgroundColor(Color::DarkGrey), Print('│'),
            SetForegroundColor(Color::White), SetBackgroundColor(Color::DarkGrey),
            Print(format!("  {:<inner$}  ", sub, inner = inner)),
            SetForegroundColor(Color::Cyan),  SetBackgroundColor(Color::DarkGrey), Print('│'),
        )?;
        next_row += 1;
    }

    queue!(out,
        MoveTo(col_off, next_row),
        SetForegroundColor(Color::Cyan),     SetBackgroundColor(Color::DarkGrey), Print('│'),
        SetForegroundColor(Color::DarkGrey), SetBackgroundColor(Color::DarkGrey),
        Print(format!("  {:<inner$}  ", dismiss, inner = inner)),
        SetForegroundColor(Color::Cyan),     SetBackgroundColor(Color::DarkGrey), Print('│'),

        MoveTo(col_off, next_row + 1),
        SetForegroundColor(Color::Cyan), SetBackgroundColor(Color::DarkGrey),
        Print(format!("└{}┘", border_h)),

        ResetColor
    )
}

// ── Paused overlay ───────────────────────────────────────────────────────────

/// Large ASCII-art "paused" rendered centred over the (hidden) grid.
/// Grid occupies col 2..75, rows 1..37 (73 wide × 37 tall).
fn render_paused_overlay(out: &mut impl Write, resume_hint: &str, colors: &ColorScheme) -> io::Result<()> {
    // figlet "paused" (standard font) — kept in English as decorative ASCII art.
    const ART: &[&str] = &[
        "                                 _ ",
        "  _ __   __ _ _   _ ___  ___  __| |",
        " | '_ \\ / _` | | | / __|/ _ \\/ _` |",
        " | |_) | (_| | |_| \\__ \\  __/ (_| |",
        " | .__/ \\__,_|\\__,_|___/\\___|\\ __,_|",
        " |_|                                ",
    ];

    // Grid: col_off=2, width=73; rows 1..=37.
    let grid_col: u16 = 2;
    let grid_width: u16 = 73;
    let grid_row: u16 = 1;
    let grid_height: u16 = 37;

    let art_w = ART.iter().map(|l| l.chars().count()).max().unwrap_or(36) as u16;
    let art_h = ART.len() as u16;
    let total_h = art_h + 2; // art + blank + resume

    let art_col    = grid_col + (grid_width.saturating_sub(art_w)) / 2;
    let art_row    = grid_row + (grid_height.saturating_sub(total_h)) / 2;
    let resume_col = grid_col + (grid_width.saturating_sub(resume_hint.chars().count() as u16)) / 2;
    let resume_row = art_row + art_h + 1;

    let bg = colors.ui_background;

    for (i, line) in ART.iter().enumerate() {
        queue!(out,
            MoveTo(art_col, art_row + i as u16),
            SetForegroundColor(colors.digit_given),
            SetBackgroundColor(bg),
            Print(line)
        )?;
    }

    queue!(out,
        MoveTo(resume_col, resume_row),
        SetForegroundColor(colors.ui_text_dim),
        SetBackgroundColor(bg),
        Print(resume_hint),
        ResetColor
    )
}

// ── Completion helper ─────────────────────────────────────────────────────────

/// Cells of a grid row left→right.
pub fn row_cells(r: usize) -> Vec<(usize, usize)> { (0..9).map(|c| (r, c)).collect() }
/// Cells of a grid column top→bottom.
pub fn col_cells(c: usize) -> Vec<(usize, usize)> { (0..9).map(|r| (r, c)).collect() }
/// Cells of box `b` in reading order (used for completion checks).
pub fn box_cells(b: usize) -> Vec<(usize, usize)> {
    let br = (b / 3) * 3;
    let bc = (b % 3) * 3;
    (0..3).flat_map(|dr| (0..3).map(move |dc| (br + dr, bc + dc))).collect()
}
/// Cells of box `b` in serpentine order for the completion sweep animation:
/// row 0 left→right, row 1 right→left, row 2 left→right.
pub fn box_cells_serpentine(b: usize) -> Vec<(usize, usize)> {
    let br = (b / 3) * 3;
    let bc = (b % 3) * 3;
    let mut cells = Vec::with_capacity(9);
    for dr in 0..3usize {
        if dr % 2 == 0 {
            for dc in 0..3 { cells.push((br + dr, bc + dc)); }
        } else {
            for dc in (0..3).rev() { cells.push((br + dr, bc + dc)); }
        }
    }
    cells
}
