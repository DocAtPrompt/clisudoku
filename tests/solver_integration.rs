use clisudoku::{puzzle::Grid, solver::Solver};

const EASY: &str = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
const EASY_SOL: &str = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

#[test]
fn solver_solves_easy_puzzle_correctly() {
    let grid = Grid::from_str(EASY).unwrap();
    let result = Solver::new().solve(grid);
    assert!(result.grid.is_solved());
    assert_eq!(result.grid.to_str(), EASY_SOL);
}

#[test]
fn solver_handles_already_solved() {
    let grid = Grid::from_str(EASY_SOL).unwrap();
    let result = Solver::new().solve(grid);
    assert!(result.grid.is_solved());
    assert_eq!(result.grid.to_str(), EASY_SOL);
}
