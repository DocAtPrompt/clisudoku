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

// ── Combined state ────────────────────────────────────────────────────────────

/// Blink rhythm for error cells: 4 ticks visible (320 ms), 4 ticks hidden (320 ms).
const ERROR_BLINK_TICKS: u32 = 4;

pub struct AnimState {
    pub sweeps:   Vec<SweepAnim>,
    pub firework: Option<FireworkAnim>,
    /// When true the error-cell software blink is running (error_mode is ON).
    pub error_blink: bool,
    /// Tick counter driving the error blink; incremented every advance().
    pub error_blink_tick: u32,
}

impl Default for AnimState {
    fn default() -> Self {
        Self {
            sweeps:           Vec::new(),
            firework:         None,
            error_blink:      false,
            error_blink_tick: 0,
        }
    }
}

impl AnimState {
    pub fn is_active(&self) -> bool {
        !self.sweeps.is_empty() || self.firework.is_some() || self.error_blink
    }

    /// Advance all active animations by one tick; discard finished ones.
    pub fn advance(&mut self) {
        self.sweeps.retain_mut(|s| { s.advance(); !s.done() });
        if let Some(fw) = &mut self.firework {
            fw.advance();
            if fw.done() { self.firework = None; }
        }
        if self.error_blink {
            self.error_blink_tick = self.error_blink_tick.wrapping_add(1);
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
