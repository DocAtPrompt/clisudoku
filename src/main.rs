use clisudoku::{
    puzzle::{Grid, GameState},
    timer::SystemClock,
    tui::App,
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut app = App::new(Box::new(SystemClock));

    // Optional: pre-load a puzzle from CLI args
    if let Some(pos) = args.iter().position(|a| a == "-s") {
        if let Some(puzzle_str) = args.get(pos + 1) {
            match Grid::from_str(puzzle_str) {
                Ok(grid) => {
                    app.game_state = Some(GameState::new(grid));
                    app.screen = clisudoku::tui::AppScreen::Game;
                }
                Err(e) => {
                    eprintln!("Invalid puzzle string: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else if let Some(pos) = args.iter().position(|a| a == "-f") {
        if let Some(path) = args.get(pos + 1) {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let trimmed = content.trim();
                    match Grid::from_str(trimmed) {
                        Ok(grid) => {
                            app.game_state = Some(GameState::new(grid));
                            app.screen = clisudoku::tui::AppScreen::Game;
                        }
                        Err(e) => {
                            eprintln!("Invalid puzzle in file: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
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
