use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellKind {
    Empty,
    Given(u8),
    Filled(u8),
}

impl CellKind {
    pub fn value(self) -> Option<u8> {
        match self {
            CellKind::Empty => None,
            CellKind::Given(v) | CellKind::Filled(v) => Some(v),
        }
    }
    pub fn is_empty(self) -> bool { matches!(self, CellKind::Empty) }
    pub fn is_given(self) -> bool { matches!(self, CellKind::Given(_)) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    #[serde(with = "serde_arrays")]
    cells: [CellKind; 81],
}

impl Grid {
    pub fn empty() -> Self {
        Self { cells: [CellKind::Empty; 81] }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        let digits: Vec<u8> = s
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .map(|c| if c == '.' { 0 } else { c as u8 - b'0' })
            .collect();
        if digits.len() != 81 {
            return Err(format!("expected 81 cells, got {}", digits.len()));
        }
        let mut cells = [CellKind::Empty; 81];
        for (i, &v) in digits.iter().enumerate() {
            cells[i] = if v == 0 { CellKind::Empty } else { CellKind::Given(v) };
        }
        Ok(Self { cells })
    }

    pub fn to_str(&self) -> String {
        self.cells
            .iter()
            .map(|c| match c.value() {
                None => '0',
                Some(v) => (b'0' + v) as char,
            })
            .collect()
    }

    #[inline]
    fn idx(row: usize, col: usize) -> usize { row * 9 + col }

    pub fn get(&self, row: usize, col: usize) -> CellKind {
        self.cells[Self::idx(row, col)]
    }

    pub fn set_given(&mut self, row: usize, col: usize, v: u8) {
        self.cells[Self::idx(row, col)] = CellKind::Given(v);
    }

    pub fn set_filled(&mut self, row: usize, col: usize, v: u8) {
        self.cells[Self::idx(row, col)] = CellKind::Filled(v);
    }

    pub fn clear(&mut self, row: usize, col: usize) {
        self.cells[Self::idx(row, col)] = CellKind::Empty;
    }

    pub fn row(&self, r: usize) -> [CellKind; 9] {
        std::array::from_fn(|c| self.get(r, c))
    }

    pub fn col(&self, c: usize) -> [CellKind; 9] {
        std::array::from_fn(|r| self.get(r, c))
    }

    /// box_idx 0-8, row-major (0=top-left)
    pub fn box_cells(&self, box_idx: usize) -> [CellKind; 9] {
        let (br, bc) = Self::box_start(box_idx);
        std::array::from_fn(|i| self.get(br + i / 3, bc + i % 3))
    }

    pub fn box_idx(row: usize, col: usize) -> usize { (row / 3) * 3 + col / 3 }

    pub fn box_start(box_idx: usize) -> (usize, usize) {
        ((box_idx / 3) * 3, (box_idx % 3) * 3)
    }

    pub fn is_solved(&self) -> bool {
        let valid = |cells: [CellKind; 9]| -> bool {
            let vals: Vec<u8> = cells.iter().filter_map(|c| c.value()).collect();
            if vals.len() != 9 { return false; }
            let mut seen = 0u16;
            for v in vals {
                let bit = 1u16 << v;
                if seen & bit != 0 { return false; }
                seen |= bit;
            }
            true
        };
        (0..9).all(|i| valid(self.row(i)) && valid(self.col(i)) && valid(self.box_cells(i)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EASY: &str = "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
    const EASY_SOL: &str = "534678912672195348198342567859761423426853791713924856961537284287419635345286179";

    #[test]
    fn from_str_round_trip() {
        let grid = Grid::from_str(EASY).unwrap();
        assert_eq!(grid.to_str(), EASY);
    }

    #[test]
    fn get_set_clear() {
        let mut grid = Grid::empty();
        assert_eq!(grid.get(0, 0), CellKind::Empty);
        grid.set_filled(0, 0, 7);
        assert_eq!(grid.get(0, 0), CellKind::Filled(7));
        grid.clear(0, 0);
        assert_eq!(grid.get(0, 0), CellKind::Empty);
    }

    #[test]
    fn row_helper() {
        let grid = Grid::from_str(EASY).unwrap();
        let row = grid.row(0);
        assert_eq!(row[0], CellKind::Given(5));
        assert_eq!(row[1], CellKind::Given(3));
        assert_eq!(row[2], CellKind::Empty);
        assert_eq!(row[3], CellKind::Empty);
        assert_eq!(row[4], CellKind::Given(7));
    }

    #[test]
    fn col_helper() {
        let grid = Grid::from_str(EASY).unwrap();
        let col = grid.col(0);
        assert_eq!(col[0], CellKind::Given(5));
        assert_eq!(col[1], CellKind::Given(6));
        assert_eq!(col[2], CellKind::Empty);
    }

    #[test]
    fn box_cells_top_left() {
        let grid = Grid::from_str(EASY).unwrap();
        // Box 0: rows 0-2, cols 0-2 → 5,3,0, 6,0,0, 0,9,8
        let b = grid.box_cells(0);
        assert_eq!(b[0], CellKind::Given(5));
        assert_eq!(b[1], CellKind::Given(3));
        assert_eq!(b[2], CellKind::Empty);
        assert_eq!(b[6], CellKind::Empty);
        assert_eq!(b[7], CellKind::Given(9));
        assert_eq!(b[8], CellKind::Given(8));
    }

    #[test]
    fn box_idx_helper() {
        assert_eq!(Grid::box_idx(0, 0), 0);
        assert_eq!(Grid::box_idx(0, 3), 1);
        assert_eq!(Grid::box_idx(3, 0), 3);
        assert_eq!(Grid::box_idx(8, 8), 8);
    }

    #[test]
    fn is_solved_partial() {
        let grid = Grid::from_str(EASY).unwrap();
        assert!(!grid.is_solved());
    }

    #[test]
    fn is_solved_complete() {
        let grid = Grid::from_str(EASY_SOL).unwrap();
        assert!(grid.is_solved());
    }

    #[test]
    fn from_str_rejects_bad_length() {
        assert!(Grid::from_str("1234").is_err());
    }
}
