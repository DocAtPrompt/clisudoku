use clisudoku::{
    i18n::Language,
    puzzle::{Grid, GameState},
    solver::backtracking::solve_backtracking,
    timer::SystemClock,
    tui::App,
    tui::colors::Theme,
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    let mut app = App::new(Box::new(SystemClock));

    // -t / --theme <NAME>  — override the default dark theme.
    if let Some(pos) = args.iter().position(|a| a == "-t" || a == "--theme") {
        match args.get(pos + 1) {
            Some(name) => match Theme::from_code(name) {
                Some(theme) => {
                    app.theme = theme;
                    app.colors = clisudoku::tui::colors::ColorScheme::for_theme(theme);
                }
                None => {
                    eprintln!(
                        "Unknown theme '{}'. Valid themes: dark light high-contrast",
                        name
                    );
                    std::process::exit(1);
                }
            },
            None => {
                eprintln!("Option -t/--theme requires a theme name.");
                std::process::exit(1);
            }
        }
    }

    // -l / --language <CODE>  — override the auto-detected language.
    if let Some(pos) = args.iter().position(|a| a == "-l" || a == "--language") {
        match args.get(pos + 1) {
            Some(code) => match Language::from_code(code) {
                Some(lang) => app.language = lang,
                None => {
                    eprintln!(
                        "Unknown language code '{}'. \
                         Valid codes: en de es fr it sl eo sw af id py tp leet",
                        code
                    );
                    std::process::exit(1);
                }
            },
            None => {
                eprintln!("Option -l/--language requires a language code.");
                std::process::exit(1);
            }
        }
    }

    // Optional: pre-load a puzzle from CLI args
    if let Some(pos) = args.iter().position(|a| a == "-s") {
        if let Some(puzzle_str) = args.get(pos + 1) {
            load_puzzle(&mut app, puzzle_str);
        }
    } else if let Some(pos) = args.iter().position(|a| a == "-f") {
        if let Some(path) = args.get(pos + 1) {
            match std::fs::read_to_string(path) {
                Ok(content) => load_puzzle(&mut app, content.trim()),
                Err(e) => {
                    eprintln!("Cannot read file {}: {}", path, e);
                    std::process::exit(1);
                }
            }
        }
    }

    if let Err(e) = app.run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn print_help() {
    println!("{}", clisudoku::tui::render::start_screen::TITLE);
    println!("  Terminal Sudoku\n");
    println!("\
USAGE:
    clisudoku [OPTIONS]

OPTIONS:
    -s <PUZZLE>        Load a puzzle from an 81-character string.
                       Digits 1-9 are given cells, 0 or . are empty cells.
                       Strings shorter than 81 characters are padded with
                       empty cells. The puzzle is validated for solvability
                       (minimum 17 given cells required).

    -f <FILE>          Load a puzzle from a text file (same format as -s).

    -t, --theme <NAME>     Set the color theme. Available themes:

                             dark            Dark (default)
                             light           Light
                             high-contrast   High Contrast (colorblind-safe)

    -l, --language <CODE>
                       Set the interface language, overriding the system
                       locale. Available codes:

                         en    English          de    Deutsch
                         es    Español          fr    Français
                         it    Italiano         sl    Slovenščina
                         eo    Esperanto        sw    Kiswahili
                         af    Afrikaans        id    Bahasa Indonesia
                         py    Zhōngwén (Pīnyīn)
                         tp    Toki Pona        leet  L33tsp34k

    -h, --help         Show this help and exit.

EXAMPLE:
    clisudoku -s 530070000600195000098000060800060003400803001700020006060000280000419005000080079
    clisudoku -f my_puzzle.txt
    clisudoku -l de -s 530070000600195000098000060800060003400803001700020006060000280000419005000080079
");
}

/// Parse and validate a puzzle string, then either start the game or
/// show the start screen with an explanatory notice.
fn load_puzzle(app: &mut App, s: &str) {
    let grid = match Grid::from_str(s) {
        Ok(g) => g,
        Err(e) => {
            // Unrecoverable parse error (e.g. > 81 cells) — show start screen with notice.
            let msg = app.language.strings().puzzle_invalid.replacen("{}", &e.to_string(), 1);
            app.set_start_notice(msg);
            return;
        }
    };

    // Minimum-givens check (theoretical lower bound for a uniquely-solvable puzzle).
    let given_count = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter(|&(r, c)| grid.get(r, c).is_given())
        .count();
    if given_count < 17 {
        let msg = app.language.strings().puzzle_few_givens.replacen("{}", &given_count.to_string(), 1);
        app.set_start_notice(msg);
        return;
    }

    // Solvability check via backtracking.
    let solved = solve_backtracking(grid.clone());
    if solved.is_none() {
        app.set_start_notice(app.language.strings().puzzle_no_solution.into());
        return;
    }

    // All checks passed — start directly in Game screen.
    app.game_state = Some(GameState::new(grid));
    app.screen = clisudoku::tui::AppScreen::Game;
}
