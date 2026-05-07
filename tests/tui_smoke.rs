// tests/tui_smoke.rs
use clisudoku::i18n::EN;
use clisudoku::puzzle::{GameState, Grid};
use clisudoku::tui::anim::AnimState;
use clisudoku::tui::colors::ColorScheme;
use clisudoku::tui::digit_style::{AwkwardRetroStyle, RetroStyle};
use clisudoku::tui::input::NavState;
use clisudoku::tui::render::{render_frame, Screen};

/// Verifies that render_frame produces non-empty output without panicking.
/// Writes to Vec<u8> to avoid requiring a real terminal.
#[test]
fn render_game_screen_does_not_panic() {
    let grid = Grid::from_str(
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
    )
    .unwrap();
    let state = GameState::new(grid);
    let mut buf = Vec::new();

    render_frame(
        &mut buf,
        &Screen::Game {
            state: &state,
            cursor: (4, 4),
            note_mode: false,
            scan_mode: false,
            error_mode: false,
            solution: None,
            errors_shown: 0,
            elapsed_ms: 125_000,
            paused: false,
            nav: &NavState::default(),
            anim: &AnimState::default(),
            scan_digit: None,
            hint: None,
            hint_warning: None,
            hint_count: 0,
            matrix_mode: false,
            mouse_mode: false,
            hover_cell: None,
        },
        &ColorScheme::default(),
        &RetroStyle,
        &AwkwardRetroStyle,
        &EN,
    )
    .unwrap();

    assert!(!buf.is_empty());
    let s = String::from_utf8_lossy(&buf);
    // Must contain border characters
    assert!(s.contains('╔'));
    assert!(s.contains('╝'));
    // Must contain the timer
    assert!(s.contains("02:05"));
}

#[test]
fn render_start_screen_does_not_panic() {
    let mut buf = Vec::new();
    render_frame(
        &mut buf,
        &clisudoku::tui::render::Screen::Start { selected: 0 },
        &ColorScheme::default(),
        &RetroStyle,
        &AwkwardRetroStyle,
        &EN,
    )
    .unwrap();
    assert!(!buf.is_empty());
}
