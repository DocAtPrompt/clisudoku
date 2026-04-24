// src/tui/anim.rs
//
// Animation state for visual effects:
//   - CompletionSweep: background colour pulse when a row/col/box is completed
//   - Firework: ASCII firework overlay on puzzle completion

use crossterm::style::Color;

// ── Sweep ─────────────────────────────────────────────────────────────────────

/// Number of frames for the completion sweep (each frame ≈ 80 ms).
pub const SWEEP_FRAMES: u8 = 10;

/// Background colours cycled through during the sweep.
/// Index = frame number, wraps if needed.
pub const SWEEP_COLORS: &[Color] = &[
    Color::Rgb { r: 0,   g: 80,  b: 0   },  // dark green
    Color::Rgb { r: 0,   g: 140, b: 0   },
    Color::Rgb { r: 0,   g: 180, b: 40  },
    Color::Rgb { r: 40,  g: 220, b: 80  },
    Color::Rgb { r: 80,  g: 255, b: 120 },  // bright green peak
    Color::Rgb { r: 40,  g: 220, b: 80  },
    Color::Rgb { r: 0,   g: 180, b: 40  },
    Color::Rgb { r: 0,   g: 140, b: 0   },
    Color::Rgb { r: 0,   g: 80,  b: 0   },
    Color::Black,
];

/// A single group sweep animation.
#[derive(Debug, Clone)]
pub struct SweepAnim {
    /// The cells that should be highlighted (row, col).
    pub cells: Vec<(usize, usize)>,
    pub frame: u8,
}

impl SweepAnim {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        Self { cells, frame: 0 }
    }

    pub fn advance(&mut self) { self.frame = self.frame.saturating_add(1); }
    pub fn done(&self)        -> bool { self.frame >= SWEEP_FRAMES }

    pub fn current_bg(&self) -> Color {
        SWEEP_COLORS[self.frame.min(SWEEP_FRAMES - 1) as usize]
    }

    /// Return the override bg for this cell if it's part of this sweep.
    pub fn bg_for(&self, row: usize, col: usize) -> Option<Color> {
        if self.cells.contains(&(row, col)) {
            Some(self.current_bg())
        } else {
            None
        }
    }
}

// ── Firework ──────────────────────────────────────────────────────────────────

pub const FIREWORK_FRAMES: u8 = 28;

/// A single burst site in the firework.
#[derive(Debug, Clone, Copy)]
pub struct BurstSite {
    pub row: u16,
    pub col: u16,
    pub start_frame: u8,
    pub color: Color,
}

/// Fixed burst sites positioned over the 73×37 grid (col_off=2, row_off=1).
pub const BURST_SITES: &[BurstSite] = &[
    BurstSite { row: 8,  col: 15, start_frame: 0,  color: Color::Yellow    },
    BurstSite { row: 5,  col: 50, start_frame: 5,  color: Color::Cyan      },
    BurstSite { row: 15, col: 32, start_frame: 3,  color: Color::Magenta   },
    BurstSite { row: 10, col: 62, start_frame: 8,  color: Color::Green     },
    BurstSite { row: 20, col: 20, start_frame: 6,  color: Color::Red       },
    BurstSite { row: 18, col: 55, start_frame: 10, color: Color::White     },
];

/// Per-burst character patterns indexed by local frame (frame - start_frame).
/// Offsets are (dr, dc, char) relative to the burst centre.
pub fn burst_chars(local_frame: u8) -> &'static [(i16, i16, char)] {
    match local_frame {
        0 => &[(2, 0, '|'), (1, 0, '|')],
        1 => &[(1, 0, '|'), (0, 0, '*')],
        2 => &[(0, 0, '*')],
        3 => &[
            (-1, -1,'\\'), (-1,0,'|'), (-1,1,'/'),
            ( 0, -1,'-' ), ( 0,0,'*'), ( 0,1,'-'),
            ( 1, -1,'/' ), ( 1,0,'|'), ( 1,1,'\\'),
        ],
        4 => &[
            (-2,-2,'*'), (-2,0,'*'), (-2,2,'*'),
            ( 0,-3,'*'),             ( 0,3,'*'),
            ( 2,-2,'*'), ( 2,0,'*'), ( 2,2,'*'),
            (-1,-1,'·'), (-1,1,'·'), ( 1,-1,'·'), ( 1,1,'·'),
        ],
        5 => &[
            (-3,-3,'✦'), (-3,0,'✦'), (-3,3,'✦'),
            ( 0,-4,'✦'),              ( 0,4,'✦'),
            ( 3,-3,'✦'), ( 3,0,'✦'), ( 3,3,'✦'),
            (-2,-1,'·'), (-2,1,'·'), ( 2,-1,'·'), ( 2,1,'·'),
        ],
        6 => &[
            (-3,-3,'·'), (-3,0,'·'), (-3,3,'·'),
            ( 0,-4,'·'),             ( 0,4,'·'),
            ( 3,-3,'·'), ( 3,0,'·'), ( 3,3,'·'),
        ],
        7 => &[
            (-3,-3,'·'), (-3,3,'·'),
            ( 3,-3,'·'), ( 3,3,'·'),
        ],
        _ => &[],
    }
}

#[derive(Debug, Clone)]
pub struct FireworkAnim {
    pub frame: u8,
}

impl FireworkAnim {
    pub fn new() -> Self { Self { frame: 0 } }
    pub fn advance(&mut self) { self.frame = self.frame.saturating_add(1); }
    pub fn done(&self)        -> bool { self.frame >= FIREWORK_FRAMES }
}

// ── Combined state ────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct AnimState {
    pub sweeps:   Vec<SweepAnim>,
    pub firework: Option<FireworkAnim>,
}

impl AnimState {
    pub fn is_active(&self) -> bool {
        !self.sweeps.is_empty() || self.firework.is_some()
    }

    /// Advance all active animations by one frame; discard finished ones.
    pub fn advance(&mut self) {
        self.sweeps.retain_mut(|s| { s.advance(); !s.done() });
        if let Some(fw) = &mut self.firework {
            fw.advance();
            if fw.done() { self.firework = None; }
        }
    }

    /// Return the sweep override background for a cell, if any sweep covers it.
    /// Multiple sweeps may overlap; last writer wins (newest sweep).
    pub fn sweep_bg(&self, row: usize, col: usize) -> Option<Color> {
        self.sweeps.iter().rev().find_map(|s| s.bg_for(row, col))
    }
}
