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
    /// Row has crystallised — skip writing; game screen beneath shows through.
    Settled,
    /// Not yet reached — cover with background colour.
    Blank,
    /// Falling rain character. Level: 0 = head (White), 1 = near trail (Green), 2 = far (DarkGreen).
    Rain(char, u8),
}

// ── Individual raindrop ───────────────────────────────────────────────────────

struct RainDrop {
    head:       i16,   // current head row; starts at -(trail_len), falls to target
    target:     i16,   // grid row where this drop crystallises
    trail_len:  u8,
    speed:      u8,    // advance 1 row every `speed` ticks
    frame_tick: u8,
}

impl RainDrop {
    fn new(target: i16, seed: &mut u64) -> Self {
        let speed     = if rng_next(seed) < 0.45 { 1u8 } else { 2u8 };
        let trail_len = 5 + (rng_next(seed) * 9.0) as u8; // 5 ..= 13
        Self { head: -(trail_len as i16), target, trail_len, speed, frame_tick: 0 }
    }

    fn advance(&mut self) {
        if self.head >= self.target { return; }
        self.frame_tick += 1;
        if self.frame_tick >= self.speed {
            self.frame_tick = 0;
            self.head += 1;
        }
    }

    fn done(&self) -> bool { self.head >= self.target }

    /// Brightness level at `row`: 0=head(White), 1=near trail(Green), 2=far(DarkGreen), None=not here.
    fn level_at(&self, row: i16) -> Option<u8> {
        if row < 0 || row >= RAIN_ROWS as i16 { return None; }
        let head = self.head;
        if row == head {
            Some(0)
        } else if row < head && row >= head - self.trail_len as i16 {
            let dist = (head - row) as usize;
            Some(if dist <= self.trail_len as usize / 2 { 1 } else { 2 })
        } else {
            None
        }
    }

    /// Progress through the full fall (0.0 = just spawned, 1.0 = reached target).
    fn progress(&self) -> f32 {
        let total = self.target + self.trail_len as i16;
        if total <= 0 { return 1.0; }
        ((self.head + self.trail_len as i16).max(0) as f32) / total as f32
    }
}

// ── Per-column state ──────────────────────────────────────────────────────────

struct RainColumn {
    start_delay:  u32,
    drop_a:       Option<RainDrop>, // drop currently in flight
    drop_b:       Option<RainDrop>, // pre-spawned successor
    settled_rows: usize,            // rows crystallised from bottom (0 → RAIN_ROWS)
    chars:        Vec<char>,
    col_seed:     u64,
    shimmer:      usize,            // tick counter for char variety
}

impl RainColumn {
    /// Next row (from bottom) that needs to be settled.
    fn next_target(&self) -> i16 {
        RAIN_ROWS as i16 - 1 - self.settled_rows as i16
    }

    fn ensure_drop_a(&mut self) {
        if self.drop_a.is_none() && self.next_target() >= 0 {
            let t = self.next_target();
            self.drop_a = Some(RainDrop::new(t, &mut self.col_seed));
        }
    }

    /// Spawn drop_b once drop_a is more than halfway to its target.
    fn maybe_spawn_drop_b(&mut self) {
        if self.drop_b.is_some() { return; }
        let next_b = self.next_target() - 1;
        if next_b < 0 { return; }
        if self.drop_a.as_ref().map(|a| a.progress() >= 0.5).unwrap_or(false) {
            self.drop_b = Some(RainDrop::new(next_b, &mut self.col_seed));
        }
    }

    fn advance(&mut self) {
        self.shimmer = self.shimmer.wrapping_add(1);
        self.ensure_drop_a();
        if let Some(d) = &mut self.drop_a { d.advance(); }
        if let Some(d) = &mut self.drop_b { d.advance(); }
        self.maybe_spawn_drop_b();

        // Settle drop_a when it has reached its target row.
        if self.drop_a.as_ref().map(|d| d.done()).unwrap_or(false) {
            self.settled_rows += 1;
            self.drop_a = self.drop_b.take();
        }
    }

    fn char_at(&self, row: usize) -> char {
        self.chars[(row.wrapping_add(self.shimmer)) % self.chars.len()]
    }
}

// ── MatrixRainAnim ────────────────────────────────────────────────────────────

pub struct MatrixRainAnim {
    columns: Vec<RainColumn>,
    tick:    u32,
}

impl MatrixRainAnim {
    pub fn new(seed: u64) -> Self {
        let mut s = seed;
        let columns = (0..RAIN_COLS).map(|_| {
            let start_delay = (rng_next(&mut s) * 15.0) as u32;
            let col_seed    = s ^ (rng_next(&mut s) * u64::MAX as f32) as u64;
            let chars = (0..96).map(|_| {
                let idx = (rng_next(&mut s) * MATRIX_CHARS.len() as f32) as usize;
                MATRIX_CHARS[idx % MATRIX_CHARS.len()] as char
            }).collect();
            RainColumn {
                start_delay, drop_a: None, drop_b: None,
                settled_rows: 0, chars, col_seed, shimmer: 0,
            }
        }).collect();
        Self { columns, tick: 0 }
    }

    /// All columns fully crystallised — the whole grid is visible again.
    pub fn done(&self) -> bool {
        self.columns.iter().all(|c| {
            if self.tick <= c.start_delay { return false; }
            c.settled_rows >= RAIN_ROWS
        })
    }

    pub fn advance(&mut self) {
        self.tick += 1;
        for col in &mut self.columns {
            if self.tick <= col.start_delay   { continue; }
            if col.settled_rows >= RAIN_ROWS  { continue; }
            col.advance();
        }
    }

    /// What to render at grid position `(col, row)`.
    ///
    /// Each column has at most two drops in flight: `drop_a` (current) and
    /// `drop_b` (pre-spawned successor). When `drop_a` crystallises its row,
    /// `drop_b` becomes the new `drop_a` and a fresh `drop_b` is queued.
    /// This gives the classic Matrix look: clear head + fading trail, nothing
    /// below the head, and no visual gap between consecutive drops.
    pub fn cell_at(&self, col: usize, row: usize) -> RainCell {
        let c = &self.columns[col];
        if self.tick <= c.start_delay { return RainCell::Blank; }

        // Crystallised zone grows upward from the bottom.
        let settle_row = RAIN_ROWS.saturating_sub(c.settled_rows);
        if row >= settle_row { return RainCell::Settled; }

        // Take the brightest level offered by either drop.
        let row_i = row as i16;
        let la = c.drop_a.as_ref().and_then(|d| d.level_at(row_i));
        let lb = c.drop_b.as_ref().and_then(|d| d.level_at(row_i));
        let best = match (la, lb) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None)    => Some(a),
            (None,    Some(b)) => Some(b),
            (None,    None)    => None,
        };

        match best {
            None        => RainCell::Blank,
            Some(level) => RainCell::Rain(c.char_at(row), level),
        }
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
