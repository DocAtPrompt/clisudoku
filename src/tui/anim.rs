// src/tui/anim.rs
//
// Animation state for visual effects:
//   - SweepAnim:    sequential cell highlight when a row/col/box is completed
//   - FireworkAnim: particle-based firework + blinking congrats overlay on completion

use crossterm::style::Color;

// ── Sweep ─────────────────────────────────────────────────────────────────────

/// Number of 80 ms ticks each cell stays inverted during a completion sweep.
const TICKS_PER_CELL: u8 = 1;

/// A completion sweep: cells light up one by one in the given order.
/// The active cell is rendered inverted (White bg, Black fg).
#[derive(Debug, Clone)]
pub struct SweepAnim {
    /// Ordered cells to highlight (row, col).
    pub cells: Vec<(usize, usize)>,
    tick: u8,
}

impl SweepAnim {
    pub fn new(cells: Vec<(usize, usize)>) -> Self {
        Self { cells, tick: 0 }
    }

    pub fn advance(&mut self) { self.tick = self.tick.saturating_add(1); }

    pub fn done(&self) -> bool {
        self.tick as usize >= self.cells.len() * TICKS_PER_CELL as usize
    }

    /// The cell that is currently highlighted, if any.
    fn active_cell(&self) -> Option<(usize, usize)> {
        let idx = self.tick as usize / TICKS_PER_CELL as usize;
        self.cells.get(idx).copied()
    }
}

// ── Firework / Completion animation ──────────────────────────────────────────

/// Total animation duration: 88 × 80 ms ≈ 7 s.
/// Slightly longer than before because slower gravity means particles linger.
pub const TOTAL_FIREWORK_TICKS: u32 = 88;
/// First phase: blinking "Congrats!" text (25 × 80 ms ≈ 2 s).
pub const CONGRATS_TICKS: u32 = 25;

const GRAVITY: f32 = 0.05;
/// Outer ring of particles per explosion.
const EXPLOSION_OUTER: usize = 96;
/// Middle ring — slightly faster and shorter-lived than outer.
const EXPLOSION_MIDDLE: usize = 48;
/// Inner fast-burst ring per explosion.
const EXPLOSION_INNER: usize = 32;
const EXPLOSION_SPEED: f32 = 2.2;
/// Vertical squish to compensate for terminal character aspect ratio (~2.5:1 h:w).
/// Multiply vy by this so explosions appear circular rather than oval.
const ASPECT_SQUISH: f32 = 0.40;

/// A spark flying from an explosion.
#[derive(Debug, Clone)]
pub struct Particle {
    pub x:        f32,
    pub y:        f32,
    pub vx:       f32,
    pub vy:       f32,
    pub life:     u8,
    pub max_life: u8,
    pub color:    Color,
}

impl Particle {
    /// ASCII character representing this particle's direction and age.
    ///
    /// Visual lifecycle: `*` burst → `+` spreading → `-`/`|`/`\`/`/` directional
    ///                   → `o` slowing → `·` dying
    pub fn glyph(&self) -> char {
        let ratio = self.life as f32 / self.max_life as f32;
        if ratio < 0.12 { return '\u{00b7}'; } // · — nearly dead
        // Speed in corrected space (terminal chars ~2× taller than wide).
        let speed_sq = self.vx * self.vx + (self.vy * 2.0) * (self.vy * 2.0);
        if speed_sq > 12.0 { return '*'; } // fast burst right after explosion
        if speed_sq > 4.5  { return '+'; } // mid-speed spreading phase
        if speed_sq < 0.8  { return 'o'; } // nearly stopped — trailing off
        let ax = self.vx.abs();
        let ay = self.vy.abs() * 2.0;
        if ax > ay * 1.8      { '-' }
        else if ay > ax * 1.8 { '|' }
        else if self.vx * self.vy > 0.0 { '\\' }
        else { '/' }
    }
}

/// A rocket climbing toward its burst point.
#[derive(Debug, Clone)]
pub struct Rocket {
    pub x:     f32,
    pub y:     f32,
    pub vy:    f32,    // negative = moving upward
    pub color: Color,
}

/// Rocket launch schedule: (tick, absolute-terminal-col, color).
/// Grid spans terminal cols 2..75, rows 1..38.
/// Rockets launch from tick 2 so they burst during the "Congrats!" phase.
const ROCKET_SCHEDULE: &[(u32, f32, Color)] = &[
    ( 2,  18.0, Color::Yellow),
    ( 6,  57.0, Color::Cyan),
    (10,  36.0, Color::Magenta),
    (14,  23.0, Color::Green),
    (18,  63.0, Color::Red),
    (22,  45.0, Color::White),
    (27,  30.0, Color::Yellow),
];

/// All colours particles may take on — picked randomly per particle.
const FIREWORK_PALETTE: &[Color] = &[
    Color::Yellow,
    Color::Cyan,
    Color::Magenta,
    Color::Green,
    Color::Red,
    Color::White,
    Color::DarkYellow,
    Color::DarkCyan,
];

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Minimal LCG — avoids pulling in the `rand` crate.
/// Returns a value in [0, 1).
/// NOTE: we shift by 32 (not 33) and cast to u32 first so we get a full 32-bit
/// sample.  Shifting by 33 only yielded 31 bits, producing values in [0, 0.5) —
/// which caused all explosion angles to land in [0, π) and particles to never
/// fly upward.
fn rng_next(seed: &mut u64) -> f32 {
    *seed = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    ((*seed >> 32) as u32 as f32) / (u32::MAX as f32)
}

/// Pick a random colour from the firework palette.
fn random_color(seed: &mut u64) -> Color {
    let idx = (rng_next(seed) * FIREWORK_PALETTE.len() as f32) as usize;
    FIREWORK_PALETTE[idx % FIREWORK_PALETTE.len()]
}

/// Spawn three rings of fully mixed-colour particles from an explosion at `(x, y)`.
///
/// - Outer ring (`EXPLOSION_OUTER`):  normal speed, long lifetime — the main shell.
/// - Middle ring (`EXPLOSION_MIDDLE`): slightly faster, medium lifetime — fills gaps.
/// - Inner ring (`EXPLOSION_INNER`):  fast, short-lived — the initial `*` flash.
fn explode(x: f32, y: f32, _color: Color, seed: &mut u64) -> Vec<Particle> {
    let mut particles = Vec::with_capacity(EXPLOSION_OUTER + EXPLOSION_MIDDLE + EXPLOSION_INNER);

    // ── Outer ring ────────────────────────────────────────────────────────────
    for _ in 0..EXPLOSION_OUTER {
        let angle = rng_next(seed) * std::f32::consts::TAU;
        let speed = EXPLOSION_SPEED * (0.5 + rng_next(seed) * 0.9);
        let max_life = 18 + (rng_next(seed) * 12.0) as u8;
        particles.push(Particle {
            x, y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed * ASPECT_SQUISH,
            life: max_life, max_life,
            color: random_color(seed),
        });
    }

    // ── Middle ring ───────────────────────────────────────────────────────────
    for _ in 0..EXPLOSION_MIDDLE {
        let angle = rng_next(seed) * std::f32::consts::TAU;
        let speed = EXPLOSION_SPEED * (1.0 + rng_next(seed) * 0.7);
        let max_life = 12 + (rng_next(seed) * 8.0) as u8;
        particles.push(Particle {
            x, y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed * ASPECT_SQUISH,
            life: max_life, max_life,
            color: random_color(seed),
        });
    }

    // ── Inner fast burst ──────────────────────────────────────────────────────
    for _ in 0..EXPLOSION_INNER {
        let angle = rng_next(seed) * std::f32::consts::TAU;
        let speed = EXPLOSION_SPEED * (1.6 + rng_next(seed) * 0.6);
        let max_life = 6 + (rng_next(seed) * 5.0) as u8;
        particles.push(Particle {
            x, y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed * ASPECT_SQUISH,
            life: max_life, max_life,
            color: random_color(seed),
        });
    }

    particles
}

// ── FireworkAnim ──────────────────────────────────────────────────────────────

/// Complete firework + congrats animation state.
#[derive(Debug, Clone)]
pub struct FireworkAnim {
    pub tick:      u32,
    pub particles: Vec<Particle>,
    pub rockets:   Vec<Rocket>,
    seed:          u64,
}

impl FireworkAnim {
    pub fn new() -> Self {
        Self {
            tick:      0,
            particles: Vec::new(),
            rockets:   Vec::new(),
            seed:      0xdead_beef_cafe_babe,
        }
    }

    pub fn done(&self) -> bool {
        self.tick >= TOTAL_FIREWORK_TICKS
            && self.particles.is_empty()
            && self.rockets.is_empty()
    }

    pub fn advance(&mut self) {
        self.tick += 1;

        // Launch rockets according to schedule.
        for &(launch_tick, x, color) in ROCKET_SCHEDULE {
            if self.tick == launch_tick {
                let vy = -(1.2 + rng_next(&mut self.seed) * 0.6);
                self.rockets.push(Rocket { x, y: 35.0, vy, color });
            }
        }

        // Update rockets; collect those that have reached their peak.
        let mut exploding: Vec<(f32, f32, Color)> = Vec::new();
        let mut surviving = Vec::new();
        for mut r in self.rockets.drain(..) {
            r.vy += GRAVITY;
            r.y  += r.vy;
            if r.vy >= 0.0 || r.y < 2.0 {
                exploding.push((r.x, r.y, r.color));
            } else {
                surviving.push(r);
            }
        }
        self.rockets = surviving;

        for (x, y, color) in exploding {
            let mut burst = explode(x, y, color, &mut self.seed);
            self.particles.append(&mut burst);
        }

        // Update particles: apply gravity, move, age.
        for p in &mut self.particles {
            p.vy += GRAVITY;
            p.x  += p.vx;
            p.y  += p.vy;
            p.life = p.life.saturating_sub(1);
        }
        self.particles.retain(|p| p.life > 0);
    }
}

// ── Matrix Rain ───────────────────────────────────────────────────────────────

pub const RAIN_COLS: usize = 73;
pub const RAIN_ROWS: usize = 37;

const MATRIX_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@#$&*?!<>|+-=";

/// What the renderer should draw at a given (col, row) position.
pub enum RainCell {
    /// Head has settled here — skip writing; the game screen beneath shows through.
    Settled,
    /// Not yet reached or column not started — cover with background colour.
    Blank,
    /// Falling rain character. Level: 0 = head (White), 1 = near trail (Green), 2 = far (DarkGreen).
    Rain(char, u8),
}

pub struct RainColumn {
    pub start_delay: u32,
    /// Advance head by 1 every `speed` ticks.
    pub speed:       u8,
    frame_tick:      u8,
    pub head:        i16,
    pub trail_len:   u8,
    pub chars:       Vec<char>,
}

pub struct MatrixRainAnim {
    pub columns: Vec<RainColumn>,
    tick:        u32,
}

impl MatrixRainAnim {
    pub fn new(seed: u64) -> Self {
        let mut s = seed;
        let columns = (0..RAIN_COLS).map(|_| {
            let start_delay = (rng_next(&mut s) * 18.0) as u32;
            let speed       = if rng_next(&mut s) < 0.45 { 1u8 } else { 2u8 };
            let trail_len   = 5 + (rng_next(&mut s) * 9.0) as u8; // 5 ..= 13
            let char_count  = (RAIN_ROWS + trail_len as usize + 4).max(60);
            let chars = (0..char_count).map(|_| {
                let idx = (rng_next(&mut s) * MATRIX_CHARS.len() as f32) as usize;
                MATRIX_CHARS[idx % MATRIX_CHARS.len()] as char
            }).collect();
            RainColumn { start_delay, speed, frame_tick: 0, head: -(trail_len as i16), trail_len, chars }
        }).collect();
        Self { columns, tick: 0 }
    }

    /// All columns have fully settled (entire grid visible again).
    pub fn done(&self) -> bool {
        self.columns.iter().all(|c| {
            if self.tick <= c.start_delay { return false; }
            Self::settled_from_bottom(c) >= RAIN_ROWS
        })
    }

    pub fn advance(&mut self) {
        self.tick += 1;
        for col in &mut self.columns {
            if self.tick <= col.start_delay { continue; }
            // Keep advancing until all rows of this column have settled.
            if Self::settled_from_bottom(col) < RAIN_ROWS {
                col.frame_tick += 1;
                if col.frame_tick >= col.speed {
                    col.frame_tick = 0;
                    col.head += 1;
                }
            }
        }
    }

    /// How many rows (counting from the bottom) have crystallised in this column.
    /// Grows from 0 once the head exits the grid, reaching RAIN_ROWS when done.
    fn settled_from_bottom(c: &RainColumn) -> usize {
        (c.head - (RAIN_ROWS as i16 - 1)).max(0) as usize
    }

    /// What to render at grid position `(col, row)`.
    ///
    /// - `Settled`  → don't write; the game screen beneath shows through
    /// - `Blank`    → overdraw with background colour
    /// - `Rain`     → coloured falling character
    pub fn cell_at(&self, col: usize, row: usize) -> RainCell {
        let c = &self.columns[col];

        // Column not yet started → blank the whole column (hides game content).
        if self.tick <= c.start_delay {
            return RainCell::Blank;
        }

        // Settled zone grows upward from the bottom row.
        // When settled_from_bottom = k, the bottom k rows show the real grid.
        let sfb       = Self::settled_from_bottom(c);
        let settle_row = RAIN_ROWS.saturating_sub(sfb); // 0-based: rows [settle_row..] are settled
        if row >= settle_row {
            return RainCell::Settled;
        }

        // Rain: head and trailing glow (only within the visible grid area).
        let row_i = row as i16;
        let head  = c.head;
        if row_i >= 0 && row_i < RAIN_ROWS as i16 {
            if row_i == head {
                let idx = row % c.chars.len();
                return RainCell::Rain(c.chars[idx], 0);
            }
            if row_i < head && row_i >= head - c.trail_len as i16 {
                let dist  = (head - row_i) as usize;
                let level = if dist <= c.trail_len as usize / 2 { 1u8 } else { 2u8 };
                let idx   = row % c.chars.len();
                return RainCell::Rain(c.chars[idx], level);
            }
        }

        RainCell::Blank
    }
}

// ── Combined state ────────────────────────────────────────────────────────────

/// Blink rhythm for error cells: 4 ticks visible (320 ms), 4 ticks hidden (320 ms).
const ERROR_BLINK_TICKS: u32 = 4;
/// Blink rhythm for hint target cell: 4 ticks yellow, 4 ticks cursor colour.
const HINT_BLINK_TICKS: u32 = 4;

pub struct AnimState {
    pub sweeps:   Vec<SweepAnim>,
    pub firework: Option<FireworkAnim>,
    /// Konami Code Matrix rain animation.
    pub matrix_rain: Option<MatrixRainAnim>,
    /// When true the error-cell software blink is running (error_mode is ON).
    pub error_blink: bool,
    /// Tick counter driving the error blink; incremented every advance().
    pub error_blink_tick: u32,
    /// When true the hint target cell blinks yellow↔cursor-colour.
    pub hint_blink:       bool,
    /// Separate tick counter for hint blink, independent of error_blink_tick.
    pub hint_blink_tick:  u32,
}

impl Default for AnimState {
    fn default() -> Self {
        Self {
            sweeps:           Vec::new(),
            firework:         None,
            matrix_rain:      None,
            error_blink:      false,
            error_blink_tick: 0,
            hint_blink:       false,
            hint_blink_tick:  0,
        }
    }
}

impl AnimState {
    pub fn is_active(&self) -> bool {
        !self.sweeps.is_empty()
            || self.firework.is_some()
            || self.matrix_rain.is_some()
            || self.error_blink
            || self.hint_blink
    }

    /// Advance all active animations by one tick; discard finished ones.
    pub fn advance(&mut self) {
        self.sweeps.retain_mut(|s| { s.advance(); !s.done() });
        if let Some(fw) = &mut self.firework {
            fw.advance();
            if fw.done() { self.firework = None; }
        }
        if let Some(rain) = &mut self.matrix_rain {
            rain.advance();
        }
        if self.error_blink {
            self.error_blink_tick = self.error_blink_tick.wrapping_add(1);
        }
        if self.hint_blink {
            self.hint_blink_tick = self.hint_blink_tick.wrapping_add(1);
        }
    }

    /// Returns true when error cells should be rendered visible (red),
    /// false when they should be blanked out (the "off" phase of the blink).
    pub fn error_cell_visible(&self) -> bool {
        (self.error_blink_tick / ERROR_BLINK_TICKS) % 2 == 0
    }

    /// Restart the error blink from the beginning of the visible phase.
    /// Call whenever a new error cell appears so it is always shown immediately.
    pub fn restart_error_blink(&mut self) {
        self.error_blink_tick = 0;
    }

    /// Returns true when the hint target cell should show yellow (hint colour),
    /// false when it should show the cursor colour.
    pub fn hint_cell_yellow_phase(&self) -> bool {
        (self.hint_blink_tick / HINT_BLINK_TICKS) % 2 == 0
    }

    /// If `(row, col)` is the currently active cell of any sweep, return the
    /// inversion colours `(fg=Black, bg=White)` to apply.
    pub fn sweep_highlight(&self, row: usize, col: usize) -> Option<(Color, Color)> {
        for sweep in self.sweeps.iter().rev() {
            if sweep.active_cell() == Some((row, col)) {
                return Some((Color::Black, Color::White));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hint_blink_makes_anim_active() {
        let mut a = AnimState::default();
        assert!(!a.is_active());
        a.hint_blink = true;
        assert!(a.is_active());
    }

    #[test]
    fn hint_cell_phase_alternates() {
        let mut a = AnimState::default();
        a.hint_blink = true;
        // Phase at tick 0 = yellow (true)
        assert!(a.hint_cell_yellow_phase());
        // Advance HINT_BLINK_TICKS times → phase flips to false
        for _ in 0..4 { a.advance(); }
        assert!(!a.hint_cell_yellow_phase());
        // Advance again → flips back
        for _ in 0..4 { a.advance(); }
        assert!(a.hint_cell_yellow_phase());
    }

    #[test]
    fn hint_blink_tick_independent_from_error_blink_tick() {
        let mut a = AnimState::default();
        a.error_blink = true;
        a.hint_blink = true;
        for _ in 0..3 { a.advance(); }
        // Both ticks incremented but they are separate fields
        assert_eq!(a.error_blink_tick, 3);
        assert_eq!(a.hint_blink_tick, 3);
        // Resetting error blink does not affect hint
        a.restart_error_blink();
        assert_eq!(a.error_blink_tick, 0);
        assert_eq!(a.hint_blink_tick, 3);
    }
}
