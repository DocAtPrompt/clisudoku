use clisudoku::{
    generator::{Difficulty, PuzzleGenerator},
    solver::Solver,
};

#[test]
fn generated_easy_is_valid_and_solvable() {
    let grid = PuzzleGenerator::new(7).generate(Difficulty::Easy, false);
    let result = Solver::new().solve(grid.clone());
    assert!(result.grid.is_solved(), "easy puzzle not solvable");
    let given_count = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter(|&(r, c)| grid.get(r, c).is_given())
        .count();
    assert!(given_count >= 17, "only {} givens", given_count);
}

#[test]
fn generated_medium_is_valid_and_solvable() {
    let grid = PuzzleGenerator::new(13).generate(Difficulty::Medium, false);
    let result = Solver::new().solve(grid);
    assert!(result.grid.is_solved(), "medium puzzle not solvable");
}
