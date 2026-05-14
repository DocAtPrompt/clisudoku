use clap::Parser;
use clisudoku::{
    config,
    i18n::Language,
    puzzle::{GameState, Grid},
    solver::backtracking::solve_backtracking,
    timer::SystemClock,
    tui::colors::{ColorScheme, Theme},
    tui::App,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "clisudoku", about = "Terminal Sudoku", long_about = None)]
struct Cli {
    /// Load a puzzle from an 81-character string (digits 1-9 = given, 0/. = empty).
    #[arg(short = 's', value_name = "PUZZLE")]
    puzzle_str: Option<String>,

    /// Load a puzzle from a text file (same format as -s).
    #[arg(short = 'f', value_name = "FILE")]
    puzzle_file: Option<PathBuf>,

    /// Generate a puzzle from a custom cell pattern (81 chars: 1/* = given, ./0 = empty).
    #[arg(short = 'p', long, value_name = "81CHARS")]
    pattern: Option<String>,

    /// Color theme. Valid: dark (default), light, high-contrast
    #[arg(short = 't', long, value_name = "NAME")]
    theme: Option<String>,

    /// Interface language code. Valid: en de fr it es pt nl pl cs ru ja zh ko
    #[arg(short = 'l', long, value_name = "CODE")]
    language: Option<String>,

    /// Default starting difficulty. Valid: easy medium hard extreme expert
    #[arg(long, value_name = "LEVEL")]
    difficulty: Option<String>,

    /// Digit rendering style. Valid: retro (default), awkward-retro
    #[arg(long = "digit-style", value_name = "STYLE")]
    digit_style: Option<String>,

    /// Path to config file (default: ~/.config/clisudoku/config.toml)
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new(Box::new(SystemClock));

    // 1. Load and apply config file (CLI --config overrides default path).
    let cfg = match config::load(cli.config.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Config error: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = cfg.apply_to(&mut app) {
        eprintln!("Config error: {}", e);
        std::process::exit(1);
    }

    // 2. CLI args override config (each arg is independent).
    if let Some(ref name) = cli.theme {
        match Theme::from_code(name) {
            Some(theme) => {
                app.theme = theme;
                app.colors = ColorScheme::for_theme(theme);
            }
            None => {
                eprintln!("Unknown theme '{}'. Valid: dark, light, high-contrast", name);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref code) = cli.language {
        match Language::from_code(code) {
            Some(lang) => app.language = lang,
            None => {
                eprintln!("Unknown language code '{}'", code);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref d) = cli.difficulty {
        match d.as_str() {
            "easy" => app.default_difficulty_index = 0,
            "medium" => app.default_difficulty_index = 1,
            "hard" => app.default_difficulty_index = 2,
            "extreme" => app.default_difficulty_index = 3,
            "expert" => app.default_difficulty_index = 4,
            other => {
                eprintln!(
                    "Unknown difficulty '{}'. Valid: easy, medium, hard, extreme, expert",
                    other
                );
                std::process::exit(1);
            }
        }
    }

    if let Some(ref ds) = cli.digit_style {
        match ds.as_str() {
            "retro" => app.set_digit_style_retro(),
            "awkward-retro" => app.set_digit_style_awkward(),
            other => {
                eprintln!(
                    "Unknown digit-style '{}'. Valid: retro, awkward-retro",
                    other
                );
                std::process::exit(1);
            }
        }
    }

    // 3. Load puzzle (-s wins over -f; --pattern is independent).
    if let Some(ref s) = cli.puzzle_str {
        load_puzzle(&mut app, s);
    } else if let Some(ref path) = cli.puzzle_file {
        match std::fs::read_to_string(path) {
            Ok(content) => load_puzzle(&mut app, content.trim()),
            Err(e) => {
                eprintln!("Cannot read file {}: {}", path.display(), e);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref s) = cli.pattern {
        match clisudoku::pattern::Pattern::from_cli_str(s) {
            Ok(pattern) => app.start_generating(pattern, true),
            Err(e) => {
                eprintln!("Invalid pattern string: {}", e);
                std::process::exit(1);
            }
        }
    }

    if let Err(e) = app.run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn load_puzzle(app: &mut App, s: &str) {
    let grid = match Grid::from_str(s) {
        Ok(g) => g,
        Err(e) => {
            let msg = app
                .language
                .strings()
                .puzzle_invalid
                .replacen("{}", &e.to_string(), 1);
            app.set_start_notice(msg);
            return;
        }
    };

    let given_count = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter(|&(r, c)| grid.get(r, c).is_given())
        .count();
    if given_count < 17 {
        let msg = app
            .language
            .strings()
            .puzzle_few_givens
            .replacen("{}", &given_count.to_string(), 1);
        app.set_start_notice(msg);
        return;
    }

    let solved = solve_backtracking(grid.clone());
    if solved.is_none() {
        app.set_start_notice(app.language.strings().puzzle_no_solution.into());
        return;
    }

    app.game_state = Some(GameState::new(grid));
    app.screen = clisudoku::tui::AppScreen::Game;
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_struct_is_valid() {
        super::Cli::command().debug_assert();
    }

    #[test]
    fn parse_pattern_str_valid() {
        let p = clisudoku::pattern::Pattern::from_cli_str(&"1".repeat(81)).unwrap();
        assert_eq!(p.cell_count, 81);
    }

    #[test]
    fn parse_pattern_str_invalid_length() {
        assert!(clisudoku::pattern::Pattern::from_cli_str("1111").is_err());
    }
}
