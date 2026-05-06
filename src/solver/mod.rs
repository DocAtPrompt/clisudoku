pub mod backtracking;
pub mod box_line_reduction;
pub mod candidates;
pub mod expert;
pub mod hidden_pair;
pub mod hidden_single;
pub mod naked_pair;
pub mod naked_single;
pub mod naked_triple;
pub mod pointing_pair;
pub mod swordfish;
pub mod x_wing;

pub use candidates::{CandidateGrid, Elimination, SolveStep, Strategy};

use crate::puzzle::Grid;
use std::collections::HashSet;

pub struct SolveResult {
    pub grid: Grid,
    pub used_strategies: Vec<Strategy>,
    pub steps: Vec<SolveStep>,
}

pub struct Solver {
    pub max_strategy: Option<Strategy>,
    pub use_backtracking: bool,
}

impl Solver {
    pub fn new() -> Self {
        Self {
            max_strategy: None,
            use_backtracking: true,
        }
    }

    pub fn for_difficulty(difficulty: &crate::generator::difficulty::Difficulty) -> Self {
        use crate::generator::difficulty::Difficulty;
        match difficulty {
            Difficulty::Easy => Self {
                max_strategy: Some(Strategy::HiddenSingle),
                use_backtracking: false,
            },
            Difficulty::Medium => Self {
                max_strategy: Some(Strategy::PointingPair),
                use_backtracking: false,
            },
            Difficulty::Hard => Self {
                max_strategy: Some(Strategy::XWing),
                use_backtracking: false,
            },
            Difficulty::Extreme => Self {
                max_strategy: Some(Strategy::Swordfish),
                use_backtracking: false,
            },
            Difficulty::Expert => Self {
                max_strategy: Some(Strategy::Expert),
                use_backtracking: false,
            },
            Difficulty::BareMinimum => Self {
                max_strategy: None,
                use_backtracking: true,
            },
        }
    }

    fn strategy_order() -> &'static [Strategy] {
        &[
            Strategy::NakedSingle,
            Strategy::HiddenSingle,
            Strategy::NakedPair,
            Strategy::PointingPair,
            Strategy::NakedTriple,
            Strategy::HiddenPair,
            Strategy::BoxLineReduction,
            Strategy::XWing,
            Strategy::Swordfish,
            Strategy::Expert,
            Strategy::Backtracking,
        ]
    }

    fn allowed(&self, s: Strategy) -> bool {
        if s == Strategy::Backtracking {
            return self.use_backtracking;
        }
        match self.max_strategy {
            None => true,
            Some(max) => {
                let order = Self::strategy_order();
                let pos_s = order.iter().position(|&x| x == s).unwrap_or(usize::MAX);
                let pos_max = order.iter().position(|&x| x == max).unwrap_or(usize::MAX);
                pos_s <= pos_max
            }
        }
    }

    pub fn solve(&self, mut grid: Grid) -> SolveResult {
        let mut used: HashSet<Strategy> = HashSet::new();
        let mut steps: Vec<SolveStep> = vec![];

        let mut cands = CandidateGrid::from_grid(&grid);

        'outer: loop {
            // Apply ONE naked single then restart
            if self.allowed(Strategy::NakedSingle) {
                if let Some(step) = naked_single::find_naked_singles(&grid, &cands)
                    .into_iter()
                    .next()
                {
                    cands.eliminate_from_peers(step.row, step.col, step.digit);
                    grid.set_filled(step.row, step.col, step.digit);
                    used.insert(Strategy::NakedSingle);
                    steps.push(step);
                    continue 'outer;
                }
            }

            // Apply ONE hidden single then restart
            if self.allowed(Strategy::HiddenSingle) {
                if let Some(step) = hidden_single::find_hidden_singles(&grid, &cands)
                    .into_iter()
                    .next()
                {
                    cands.eliminate_from_peers(step.row, step.col, step.digit);
                    grid.set_filled(step.row, step.col, step.digit);
                    used.insert(Strategy::HiddenSingle);
                    steps.push(step);
                    continue 'outer;
                }
            }

            macro_rules! apply_elims {
                ($find_fn:expr, $strat:expr) => {
                    if self.allowed($strat) {
                        let elims = $find_fn(&cands);
                        if !elims.is_empty() {
                            for e in &elims {
                                cands.remove(e.row, e.col, e.digit);
                            }
                            used.insert($strat);
                            continue 'outer;
                        }
                    }
                };
            }

            apply_elims!(naked_pair::find_naked_pairs, Strategy::NakedPair);
            apply_elims!(pointing_pair::find_pointing_pairs, Strategy::PointingPair);
            apply_elims!(naked_triple::find_naked_triples, Strategy::NakedTriple);
            apply_elims!(hidden_pair::find_hidden_pairs, Strategy::HiddenPair);
            apply_elims!(
                box_line_reduction::find_box_line_reductions,
                Strategy::BoxLineReduction
            );
            apply_elims!(x_wing::find_x_wings, Strategy::XWing);
            apply_elims!(swordfish::find_swordfish, Strategy::Swordfish);

            // Expert block: BUG+1 placement first, then 13 elimination functions.
            if self.allowed(Strategy::Expert) {
                // BUG+1 is a placement, handled like NakedSingle
                if let Some(step) = expert::find_bug_plus_one_step(&cands) {
                    cands.eliminate_from_peers(step.row, step.col, step.digit);
                    grid.set_filled(step.row, step.col, step.digit);
                    used.insert(Strategy::Expert);
                    steps.push(step);
                    continue 'outer;
                }
                // 13 elimination functions
                let expert_fns: &[fn(&CandidateGrid) -> Vec<Elimination>] = &[
                    expert::find_jellyfish,
                    expert::find_naked_quad,
                    expert::find_hidden_triple,
                    expert::find_hidden_quad,
                    expert::find_skyscraper,
                    expert::find_two_string_kite,
                    expert::find_y_wing,
                    expert::find_xyz_wing,
                    expert::find_w_wing,
                    expert::find_unique_rectangle,
                    expert::find_empty_rectangle,
                    expert::find_simple_coloring,
                    expert::find_xy_chain,
                ];
                for f in expert_fns {
                    let elims = f(&cands);
                    if !elims.is_empty() {
                        for elim in &elims {
                            cands.remove(elim.row, elim.col, elim.digit);
                        }
                        used.insert(Strategy::Expert);
                        continue 'outer;
                    }
                }
            }

            // Backtracking fallback
            if self.use_backtracking && !grid.is_solved() {
                if let Some(solved) = backtracking::solve_backtracking(grid.clone()) {
                    used.insert(Strategy::Backtracking);
                    grid = solved;
                }
            }

            break;
        }

        SolveResult {
            grid,
            used_strategies: used.into_iter().collect(),
            steps,
        }
    }
}

impl Default for Solver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Grid;

    const EASY: &str =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const EASY_SOL: &str =
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    const MEDIUM: &str =
        "000000000904607000076804100309701080008000300050308702007502610000403208000000000";
    const MEDIUM_SOL: &str =
        "583219467914637825276854139349721586728965341651348792497582613165493278832176954";

    #[test]
    fn solves_easy_with_logic_only() {
        let grid = Grid::from_str(EASY).unwrap();
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved());
        assert_eq!(result.grid.to_str(), EASY_SOL);
        assert!(!result.used_strategies.contains(&Strategy::Backtracking));
    }

    #[test]
    fn solves_medium_with_elimination_strategies() {
        let grid = Grid::from_str(MEDIUM).unwrap();
        let result = Solver::new().solve(grid);
        assert!(result.grid.is_solved());
        assert_eq!(result.grid.to_str(), MEDIUM_SOL);
        assert!(!result.used_strategies.contains(&Strategy::Backtracking));
    }

    #[test]
    fn restricted_solver_stops_at_max_strategy() {
        let grid = Grid::from_str(EASY).unwrap();
        let mut solver = Solver::new();
        solver.max_strategy = Some(Strategy::NakedSingle);
        solver.use_backtracking = false;
        let result = solver.solve(grid);
        assert!(!result.used_strategies.contains(&Strategy::HiddenSingle));
        assert!(!result.used_strategies.contains(&Strategy::Backtracking));
    }

    #[test]
    fn for_difficulty_expert_has_correct_config() {
        let solver = Solver::for_difficulty(&crate::generator::difficulty::Difficulty::Expert);
        assert_eq!(solver.max_strategy, Some(Strategy::Expert));
        assert!(!solver.use_backtracking);
    }

    #[test]
    fn expert_comes_after_swordfish_in_strategy_order() {
        let order = Solver::strategy_order();
        let swordfish_pos = order.iter().position(|&s| s == Strategy::Swordfish).unwrap();
        let expert_pos = order.iter().position(|&s| s == Strategy::Expert).unwrap();
        assert!(expert_pos > swordfish_pos);
    }

    #[test]
    fn expert_comes_before_backtracking_in_strategy_order() {
        let order = Solver::strategy_order();
        let expert_pos = order.iter().position(|&s| s == Strategy::Expert).unwrap();
        let back_pos = order.iter().position(|&s| s == Strategy::Backtracking).unwrap();
        assert!(expert_pos < back_pos);
    }

    #[test]
    fn expert_solver_allows_expert_strategy() {
        // Verifies that allowed() returns true for Expert (and lower strategies)
        // and false for Backtracking when configured for Expert difficulty.
        let solver = Solver::for_difficulty(&crate::generator::difficulty::Difficulty::Expert);
        assert!(solver.allowed(Strategy::Expert));
        assert!(solver.allowed(Strategy::Swordfish)); // lower strategies still allowed
        assert!(!solver.allowed(Strategy::Backtracking));
    }
}
