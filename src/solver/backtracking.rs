use crate::puzzle::Grid;

pub fn solve_backtracking(mut grid: Grid) -> Option<Grid> {
    // Pre-validate: if given cells already conflict, bail immediately
    if !is_grid_consistent(&grid) {
        return None;
    }
    solve_inner(&mut grid)?;
    Some(grid)
}

fn solve_inner(grid: &mut Grid) -> Option<()> {
    let empty = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .find(|&(r, c)| grid.get(r, c).is_empty());

    let (row, col) = match empty {
        None => return if grid.is_solved() { Some(()) } else { None },
        Some(pos) => pos,
    };

    for digit in 1u8..=9 {
        if is_valid_placement(grid, row, col, digit) {
            grid.set_filled(row, col, digit);
            if solve_inner(grid).is_some() {
                return Some(());
            }
            grid.clear(row, col);
        }
    }
    None
}

/// Check that no two filled/given cells in the same house share a digit.
fn is_grid_consistent(grid: &Grid) -> bool {
    let valid_house = |cells: [crate::puzzle::CellKind; 9]| -> bool {
        let mut seen = 0u16;
        for cell in cells {
            if let Some(v) = cell.value() {
                let bit = 1u16 << v;
                if seen & bit != 0 { return false; }
                seen |= bit;
            }
        }
        true
    };
    (0..9).all(|i| valid_house(grid.row(i)) && valid_house(grid.col(i)) && valid_house(grid.box_cells(i)))
}

fn is_valid_placement(grid: &Grid, row: usize, col: usize, digit: u8) -> bool {
    for c in 0..9 {
        if grid.get(row, c).value() == Some(digit) { return false; }
    }
    for r in 0..9 {
        if grid.get(r, col).value() == Some(digit) { return false; }
    }
    let (br, bc) = Grid::box_start(Grid::box_idx(row, col));
    for dr in 0..3 {
        for dc in 0..3 {
            if grid.get(br + dr, bc + dc).value() == Some(digit) { return false; }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::puzzle::Grid;

    #[test]
    fn solves_easy_puzzle() {
        let grid = Grid::from_str(
            "530070000600195000098000060800060003400803001700020006060000280000419005000080079"
        ).unwrap();
        let result = solve_backtracking(grid).unwrap();
        assert!(result.is_solved());
        assert_eq!(result.to_str(), "534678912672195348198342567859761423426853791713924856961537284287419635345286179");
    }

    #[test]
    fn solves_hard_puzzle() {
        // Note: the puzzle from the spec had an invalid box conflict; using a known-valid hard puzzle
        let grid = Grid::from_str(
            "000000000000003085001020000000507000004000100090000000500000073002010000000040009"
        ).unwrap();
        let result = solve_backtracking(grid);
        assert!(result.is_some(), "backtracking failed on hard puzzle");
        assert!(result.unwrap().is_solved());
    }

    #[test]
    fn returns_none_on_invalid() {
        // Two 5s in same row → unsolvable
        let mut grid = Grid::empty();
        grid.set_given(0, 0, 5);
        grid.set_given(0, 1, 5);
        let result = solve_backtracking(grid);
        assert!(result.is_none());
    }
}
