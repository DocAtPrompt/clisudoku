# Config File + CLI Args Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add TOML config file support and complete CLI args via clap, allowing
appearance, color, and key-binding customization without a settings UI.

**Architecture:** New `src/config.rs` owns parsing/validation/application.
`KeyMap` is added to `src/tui/input.rs`. `main.rs` is migrated from manual
arg parsing to clap; config is loaded first, then CLI args override.

**Tech Stack:** Rust, `clap` 4 (derive), `toml` 0.8, `serde` (already present)

**Spec:** `docs/superpowers/specs/2026-05-14-config-cli-design.md`

---

### Task 1: Add dependencies to Cargo.toml

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Add clap and toml**

```toml
clap = { version = "4", features = ["derive"] }
toml = "0.8"
```

- [ ] **Step 2: Verify build compiles**

```bash
cargo build
```
Expected: compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: add clap and toml dependencies"
```

---

### Task 2: KeyMap struct in src/tui/input.rs

**Files:**
- Modify: `src/tui/input.rs`

- [ ] **Step 1: Write the failing test**

Add at the bottom of `src/tui/input.rs`:

```rust
#[cfg(test)]
mod keymap_tests {
    use super::*;

    #[test]
    fn keymap_default_hint_is_h() {
        assert_eq!(KeyMap::default().hint, 'h');
    }

    #[test]
    fn keymap_default_pause_is_space() {
        assert_eq!(KeyMap::default().pause, ' ');
    }

    #[test]
    fn remapped_hint_fires_correct_action() {
        let mut km = KeyMap::default();
        km.hint = 'x';
        let nav = NavState::default();
        let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(map_key_to_action(key, &nav, &km), AppAction::RequestHint);
    }

    #[test]
    fn remapped_hint_old_key_no_longer_fires() {
        let mut km = KeyMap::default();
        km.hint = 'x';
        let nav = NavState::default();
        let key = KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        // 'h' is no longer hint; falls through to None
        assert_eq!(map_key_to_action(key, &nav, &km), AppAction::None);
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test keymap_tests 2>&1 | head -20
```
Expected: compile error — `KeyMap` not found.

- [ ] **Step 3: Add KeyMap struct before `map_key_to_action`**

```rust
/// Remappable single-key bindings. All fields store the character that
/// triggers the action (case-insensitive for letter keys).
#[derive(Debug, Clone)]
pub struct KeyMap {
    pub hint: char,
    pub pause: char,
    pub scan: char,
    pub errors: char,
    pub note_mode: char,   // default '0'
    pub clear: char,       // default '-'
    pub undo: char,
    pub redo: char,
    pub mouse_toggle: char,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            hint: 'h',
            pause: ' ',
            scan: 's',
            errors: 'e',
            note_mode: '0',
            clear: '-',
            undo: 'u',
            redo: 'r',
            mouse_toggle: 'm',
        }
    }
}
```

- [ ] **Step 4: Update `map_key_to_action` signature and body**

Change signature to:
```rust
pub fn map_key_to_action(key: KeyEvent, nav: &NavState, km: &KeyMap) -> AppAction {
```

**First**, remove the standalone top-level arm `KeyCode::Char('0') => AppAction::ToggleMode`
(currently line 95 in `input.rs`) — `'0'` will now be handled via `km.note_mode`
inside the `Char(c) if !ctrl` arm below.

Then remove the old individual `Char` arms for `'m'/'M'`, `'s'/'S'`, `'e'/'E'`,
`'h'/'H'`, `'-'`, `'u'`, `'r'`, `Ctrl+Z`, `Ctrl+Y` (undo/redo as `'z'/'y'`
without ctrl), and `' '` (pause). Replace with:

```rust
// Ctrl combos — not remappable
KeyCode::Char('z') if ctrl => AppAction::Undo,
KeyCode::Char('y') if ctrl => AppAction::Redo,

KeyCode::Char(c) if !ctrl => {
    let lc = c.to_ascii_lowercase();
    // Remappable actions checked first (case-insensitive for letters,
    // exact match for symbols like ' ', '-', '0').
    if lc == km.hint.to_ascii_lowercase()         { return AppAction::RequestHint; }
    if c == km.pause                              { return AppAction::Pause; }
    if lc == km.scan.to_ascii_lowercase()         { return AppAction::ToggleScan; }
    if lc == km.errors.to_ascii_lowercase()       { return AppAction::ToggleErrors; }
    if c == km.note_mode                          { return AppAction::ToggleMode; }
    if c == km.clear                              { return AppAction::ClearCell; }
    if lc == km.undo.to_ascii_lowercase()         { return AppAction::Undo; }
    if lc == km.redo.to_ascii_lowercase()         { return AppAction::Redo; }
    if lc == km.mouse_toggle.to_ascii_lowercase() { return AppAction::ToggleMouseMode; }

    // Non-remappable char actions
    match c {
        'b' | 'B' => AppAction::BossKey,
        'y' | 'Y' => AppAction::ConfirmYes,
        'n' | 'N' => AppAction::ConfirmNo,
        c if c.is_ascii_digit() && c != '0' => {
            let idx = (c as u8 - b'1') as usize;
            match nav.mode {
                NavMode::Input => AppAction::Digit(c as u8 - b'0'),
                NavMode::Navigation => {
                    if nav.box_idx.is_none() {
                        AppAction::NumpadBox(idx)
                    } else {
                        AppAction::NumpadCell(idx)
                    }
                }
            }
        }
        _ => AppAction::None,
    }
}
```

`NavMode` has exactly two variants: `Input` and `Navigation`. `nav.box_idx: Option<usize>`
indicates whether a box has been selected. Do not use `Grid`, `BoxSelected`, or
`BoxSelecting` — those do not exist.

- [ ] **Step 5: Fix the one call site in src/tui/mod.rs**

In `src/tui/mod.rs` line ~1335:
```rust
// Before:
let action = map_key_to_action(key, &self.nav_state);
// After:
let action = map_key_to_action(key, &self.nav_state, &self.key_map);
```

Also update any test helpers in `input.rs` that call `map_key_to_action` with
only 2 args — add `&KeyMap::default()` as third argument.

- [ ] **Step 6: Add `key_map: KeyMap` to App struct**

In `src/tui/mod.rs`, add to `App` struct:
```rust
pub key_map: KeyMap,
```

Add to `App::new()` initializer:
```rust
key_map: KeyMap::default(),
```

Add the import at the top of mod.rs:
```rust
use crate::tui::input::{map_key_to_action, AppAction, KeyMap, NavMode, NavState};
```

- [ ] **Step 7: Run tests**

```bash
cargo test --lib 2>&1 | grep -E "FAILED|test result"
```
Expected: all pass.

- [ ] **Step 8: Commit**

```bash
git add src/tui/input.rs src/tui/mod.rs
git commit -m "feat: add KeyMap struct and wire into map_key_to_action"
```

---

### Task 3: Add default_difficulty_index to App

**Files:**
- Modify: `src/tui/mod.rs`

- [ ] **Step 1: Write the failing test**

In `src/tui/mod.rs` tests:
```rust
#[test]
fn default_difficulty_index_is_zero() {
    let app = App::new(Box::new(crate::timer::FakeClock::new(0)));
    assert_eq!(app.default_difficulty_index, 0);
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test default_difficulty_index_is_zero 2>&1 | head -10
```
Expected: compile error — field not found.

- [ ] **Step 3: Add field and wire it**

In `App` struct:
```rust
/// Index into the difficulty list used as initial selection when opening
/// the DifficultySelect screen. 0=Easy, 1=Medium, 2=Hard, 3=Extreme, 4=Expert.
pub default_difficulty_index: usize,
```

In `App::new()`:
```rust
default_difficulty_index: 0,
```

In `handle_start_action` (line ~279 in mod.rs), change **only** the site where
the user opens the difficulty screen from the start menu (Enter on item 0):
```rust
// Before:
self.screen = AppScreen::DifficultySelect { selected: 0, sym_focused: false };
// After:
self.screen = AppScreen::DifficultySelect {
    selected: self.default_difficulty_index,
    sym_focused: false,
};
```

All other `DifficultySelect` instantiation sites in `mod.rs` (lines ~309, 315,
325, 333, 339 — arrow navigation within the screen; lines ~465, 489, 495, 500
— back-navigation from generating with specific difficulty indices) must remain
unchanged. They either preserve the current selection or specify a fixed index
for a concrete difficulty (e.g. Expert = 4). Only the **New Game entry point**
(line ~279) uses `default_difficulty_index`.

- [ ] **Step 4: Run tests**

```bash
cargo test --lib 2>&1 | grep -E "FAILED|test result"
```
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat: add default_difficulty_index to App"
```

---

### Task 4: Create src/config.rs — struct and color/key parsing

**Files:**
- Create: `src/config.rs`
- Modify: `src/lib.rs` (add `pub mod config;`)

- [ ] **Step 1: Write failing tests**

Create `src/config.rs` with only the tests for now:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_color_black() {
        assert!(matches!(parse_color("Black").unwrap(), crossterm::style::Color::Black));
    }

    #[test]
    fn parse_color_dark_grey() {
        assert!(matches!(parse_color("DarkGrey").unwrap(), crossterm::style::Color::DarkGrey));
    }

    #[test]
    fn parse_color_invalid() {
        assert!(parse_color("Turquoise").is_err());
    }

    #[test]
    fn parse_key_letter() {
        assert_eq!(parse_key("h").unwrap(), 'h');
    }

    #[test]
    fn parse_key_space() {
        assert_eq!(parse_key("Space").unwrap(), ' ');
    }

    #[test]
    fn parse_key_minus() {
        assert_eq!(parse_key("Minus").unwrap(), '-');
    }

    #[test]
    fn parse_key_zero() {
        assert_eq!(parse_key("Zero").unwrap(), '0');
    }

    #[test]
    fn parse_key_invalid() {
        assert!(parse_key("F12").is_err());
    }

    #[test]
    fn config_default_deserializes_from_empty_toml() {
        let cfg: Config = toml::from_str("").unwrap();
        assert!(cfg.appearance.theme.is_none());
        assert!(cfg.colors.is_empty());
        assert!(cfg.keys.is_empty());
    }
}
```

- [ ] **Step 2: Run to verify it fails**

```bash
cargo test config:: 2>&1 | head -20
```
Expected: compile error — module not found.

- [ ] **Step 3: Add `pub mod config;` to src/lib.rs**

- [ ] **Step 4: Implement src/config.rs**

```rust
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crossterm::style::Color;
use serde::Deserialize;

// ── Serde structs ─────────────────────────────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub appearance: AppearanceConfig,
    #[serde(default)]
    pub colors: HashMap<String, String>,
    #[serde(default)]
    pub keys: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct AppearanceConfig {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub digit_style: Option<String>,
    pub difficulty: Option<String>,
}

// ── Parsing helpers ───────────────────────────────────────────────────────────

pub fn parse_color(s: &str) -> Result<Color, String> {
    match s {
        "Black"       => Ok(Color::Black),
        "DarkGrey"    => Ok(Color::DarkGrey),
        "Red"         => Ok(Color::Red),
        "DarkRed"     => Ok(Color::DarkRed),
        "Green"       => Ok(Color::Green),
        "DarkGreen"   => Ok(Color::DarkGreen),
        "Yellow"      => Ok(Color::Yellow),
        "DarkYellow"  => Ok(Color::DarkYellow),
        "Blue"        => Ok(Color::Blue),
        "DarkBlue"    => Ok(Color::DarkBlue),
        "Magenta"     => Ok(Color::Magenta),
        "DarkMagenta" => Ok(Color::DarkMagenta),
        "Cyan"        => Ok(Color::Cyan),
        "DarkCyan"    => Ok(Color::DarkCyan),
        "White"       => Ok(Color::White),
        "Grey"        => Ok(Color::Grey),
        "Reset"       => Ok(Color::Reset),
        other         => Err(format!("Unknown color '{}'. Valid colors: Black, DarkGrey, Red, DarkRed, Green, DarkGreen, Yellow, DarkYellow, Blue, DarkBlue, Magenta, DarkMagenta, Cyan, DarkCyan, White, Grey, Reset", other)),
    }
}

/// Parse a key notation string into a char.
/// Accepts: single character ("h", "s") or named special keys ("Space", "Minus", "Zero").
pub fn parse_key(s: &str) -> Result<char, String> {
    match s {
        "Space" => Ok(' '),
        "Minus" => Ok('-'),
        "Zero"  => Ok('0'),
        other if other.chars().count() == 1 => Ok(other.chars().next().unwrap()),
        other => Err(format!(
            "Invalid key '{}'. Use a single character or: Space, Minus, Zero",
            other
        )),
    }
}

// ── Config loading ────────────────────────────────────────────────────────────

/// Default config file path: ~/.config/clisudoku/config.toml
fn default_config_path() -> Option<PathBuf> {
    dirs_or_home().map(|d| d.join("clisudoku").join("config.toml"))
}

fn dirs_or_home() -> Option<PathBuf> {
    // Use $XDG_CONFIG_HOME if set, otherwise ~/.config
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg));
    }
    std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config"))
}

/// Load a Config from `explicit_path` if given, otherwise from the default path.
/// Returns `Config::default()` if the file doesn't exist.
/// Returns `Err` if the file exists but cannot be parsed.
pub fn load(explicit_path: Option<&Path>) -> Result<Config, String> {
    let path = match explicit_path {
        Some(p) => p.to_path_buf(),
        None => match default_config_path() {
            Some(p) => p,
            None => return Ok(Config::default()),
        },
    };

    if !path.exists() {
        return Ok(Config::default());
    }

    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read config file {}: {}", path.display(), e))?;

    toml::from_str(&text)
        .map_err(|e| format!("Config file {} parse error: {}", path.display(), e))
}
```

- [ ] **Step 5: Run tests**

```bash
cargo test config:: 2>&1 | grep -E "FAILED|test result|ok$"
```
Expected: all tests in `config::tests` pass.

- [ ] **Step 6: Commit**

```bash
git add src/config.rs src/lib.rs
git commit -m "feat: add config module with TOML parsing and color/key helpers"
```

---

### Task 5: Config::apply_to() — wire config into App

**Files:**
- Modify: `src/config.rs`

- [ ] **Step 1: Write the failing test**

In `src/config.rs` tests:

```rust
#[test]
fn apply_appearance_theme_overrides_app() {
    use crate::tui::App;
    use crate::tui::colors::Theme;
    use crate::timer::FakeClock;

    let mut cfg = Config::default();
    cfg.appearance.theme = Some("light".to_string());

    let mut app = App::new(Box::new(FakeClock::new(0)));
    cfg.apply_to(&mut app).unwrap();

    assert_eq!(app.theme, Theme::Light);
}

#[test]
fn apply_color_override_changes_color_field() {
    use crate::tui::App;
    use crate::timer::FakeClock;
    use crossterm::style::Color;

    let mut cfg = Config::default();
    cfg.colors.insert("digit_given".to_string(), "Cyan".to_string());

    let mut app = App::new(Box::new(FakeClock::new(0)));
    cfg.apply_to(&mut app).unwrap();

    assert_eq!(app.colors.digit_given, Color::Cyan);
}

#[test]
fn apply_invalid_color_returns_err() {
    use crate::tui::App;
    use crate::timer::FakeClock;

    let mut cfg = Config::default();
    cfg.colors.insert("digit_given".to_string(), "Turquoise".to_string());

    let mut app = App::new(Box::new(FakeClock::new(0)));
    assert!(cfg.apply_to(&mut app).is_err());
}

#[test]
fn apply_key_override_changes_keymap() {
    use crate::tui::App;
    use crate::timer::FakeClock;

    let mut cfg = Config::default();
    cfg.keys.insert("hint".to_string(), "x".to_string());

    let mut app = App::new(Box::new(FakeClock::new(0)));
    cfg.apply_to(&mut app).unwrap();

    assert_eq!(app.key_map.hint, 'x');
}
```

- [ ] **Step 2: Run to verify they fail**

```bash
cargo test apply_ 2>&1 | head -20
```
Expected: compile errors — `apply_to` not defined.

- [ ] **Step 3: Implement `Config::apply_to()`**

Add to `src/config.rs`, after the `load()` function:

```rust
use crate::i18n::Language;
use crate::tui::App;
use crate::tui::colors::{ColorScheme, Theme};
use crate::tui::input::KeyMap;
// Do NOT import digit_style types here — use the App helper methods below.

impl Config {
    /// Apply this config to `app`. Returns Err with a human-readable message
    /// on the first invalid value found.
    pub fn apply_to(&self, app: &mut App) -> Result<(), String> {
        // ── Appearance ────────────────────────────────────────────────────────
        if let Some(ref t) = self.appearance.theme {
            app.theme = Theme::from_code(t)
                .ok_or_else(|| format!("Unknown theme '{}'. Valid: dark, light, high-contrast", t))?;
            app.colors = ColorScheme::for_theme(app.theme);
        }
        if let Some(ref l) = self.appearance.language {
            app.language = Language::from_code(l)
                .ok_or_else(|| format!("Unknown language code '{}'", l))?;
        }
        if let Some(ref ds) = self.appearance.digit_style {
            match ds.as_str() {
                "retro" => {
                    app.set_digit_style_retro();
                }
                "awkward-retro" => {
                    app.set_digit_style_awkward();
                }
                other => return Err(format!("Unknown digit_style '{}'. Valid: retro, awkward-retro", other)),
            }
        }
        if let Some(ref d) = self.appearance.difficulty {
            app.default_difficulty_index = parse_difficulty_index(d)?;
        }

        // ── Colors ────────────────────────────────────────────────────────────
        for (key, val) in &self.colors {
            let color = parse_color(val)?;
            apply_color_field(&mut app.colors, key, color)
                .ok_or_else(|| format!("Unknown color key '{}' in [colors]", key))?;
        }

        // ── Keys ──────────────────────────────────────────────────────────────
        for (action, key_str) in &self.keys {
            let ch = parse_key(key_str)?;
            apply_key_binding(&mut app.key_map, action, ch)
                .ok_or_else(|| format!("Unknown key action '{}' in [keys]", action))?;
        }

        Ok(())
    }
}

fn parse_difficulty_index(s: &str) -> Result<usize, String> {
    match s {
        "easy"    => Ok(0),
        "medium"  => Ok(1),
        "hard"    => Ok(2),
        "extreme" => Ok(3),
        "expert"  => Ok(4),
        other => Err(format!("Unknown difficulty '{}'. Valid: easy, medium, hard, extreme, expert", other)),
    }
}

/// Apply a color to the named field of `cs`. Returns Some(()) on success, None if
/// the field name is unknown.
fn apply_color_field(cs: &mut ColorScheme, key: &str, color: Color) -> Option<()> {
    match key {
        "ui_background"        => cs.ui_background = color,
        "grid_border"          => cs.grid_border = color,
        "grid_box"             => cs.grid_box = color,
        "grid_cell"            => cs.grid_cell = color,
        "cell_normal_bg"       => cs.cell_normal_bg = color,
        "cell_active_bg"       => cs.cell_active_bg = color,
        "cell_active_box_bg"   => cs.cell_active_box_bg = color,
        "cell_active_cross_bg" => cs.cell_active_cross_bg = color,
        "digit_given"          => cs.digit_given = color,
        "digit_user"           => cs.digit_user = color,
        "digit_error"          => cs.digit_error = color,
        "digit_highlight"      => cs.digit_highlight = color,
        "note_normal"          => cs.note_normal = color,
        "note_highlight"       => cs.note_highlight = color,
        "digit_scan"           => cs.digit_scan = color,
        "ui_text"              => cs.ui_text = color,
        "ui_text_dim"          => cs.ui_text_dim = color,
        "ui_cursor_bg"         => cs.ui_cursor_bg = color,
        "ui_cursor_fg"         => cs.ui_cursor_fg = color,
        "hint_cause_border"    => cs.hint_cause_border = color,
        "hint_elim_border"     => cs.hint_elim_border = color,
        "hint_target_bg"       => cs.hint_target_bg = color,
        "hover_bg"             => cs.hover_bg = color,
        _ => return None,
    }
    Some(())
}

fn apply_key_binding(km: &mut KeyMap, action: &str, ch: char) -> Option<()> {
    match action {
        "hint"         => km.hint = ch,
        "pause"        => km.pause = ch,
        "scan"         => km.scan = ch,
        "errors"       => km.errors = ch,
        "note_mode"    => km.note_mode = ch,
        "clear"        => km.clear = ch,
        "undo"         => km.undo = ch,
        "redo"         => km.redo = ch,
        "mouse_toggle" => km.mouse_toggle = ch,
        _ => return None,
    }
    Some(())
}
```

**Note:** Add these two public helper methods to `App` in `src/tui/mod.rs`
(they already import `RetroStyle` and `AwkwardRetroStyle` at the top of that file):

```rust
pub fn set_digit_style_retro(&mut self) {
    self.style = Box::new(RetroStyle);
    self.awkward_style = false;
}

pub fn set_digit_style_awkward(&mut self) {
    self.style = Box::new(AwkwardRetroStyle);
    self.awkward_style = true;
}
```

`config.rs` calls `app.set_digit_style_retro()` / `app.set_digit_style_awkward()`
and does NOT import the concrete style types itself.

**Note:** The `ColorScheme` has fields like `hint_cause_border` and
`hint_elim_border` but check the exact field names in `src/tui/colors.rs`
lines 80-87 — the `apply_color_field` match must use the exact Rust field
names (with `_fg`/`_bg`/`_border` suffixes as defined in the struct).

- [ ] **Step 4: Run tests**

```bash
cargo test --lib 2>&1 | grep -E "FAILED|test result"
```
Expected: all pass.

- [ ] **Step 5: Commit**

```bash
git add src/config.rs src/tui/mod.rs
git commit -m "feat: implement Config::apply_to() for appearance, colors, and keys"
```

---

### Task 6: Migrate main.rs to clap + wire config loading

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write the test**

In `src/main.rs` tests (replace/extend existing):

```rust
#[test]
fn cli_help_flag_exits_cleanly() {
    // Clap handles --help by printing and exiting; just verify it compiles
    // and the Cli struct parses --help without panic in unit context.
    // Integration-level: run binary with --help and check exit 0.
    // Here we verify the Cli struct itself is well-formed.
    use clap::CommandFactory;
    Cli::command().debug_assert();
}
```

- [ ] **Step 2: Rewrite main.rs**

Replace the entire file with:

```rust
use clap::Parser;
use clisudoku::{
    config,
    i18n::Language,
    puzzle::{GameState, Grid},
    solver::backtracking::solve_backtracking,
    timer::SystemClock,
    tui::colors::{ColorScheme, Theme},
    tui::App,
};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "clisudoku", about = "Terminal Sudoku", long_about = None)]
struct Cli {
    /// Load a puzzle from an 81-character string (digits 1-9 = given, 0/. = empty).
    #[arg(short = 's', value_name = "PUZZLE")]
    puzzle_str: Option<String>,

    /// Load a puzzle from a text file (same format as -s).
    #[arg(short = 'f', value_name = "FILE")]
    puzzle_file: Option<PathBuf>,

    /// Generate a puzzle from a custom cell pattern (81 chars: 1/* = given, ./0 = empty).
    #[arg(long, value_name = "81CHARS")]
    pattern: Option<String>,

    /// Color theme. Valid: dark (default), light, high-contrast
    #[arg(short = 't', long, value_name = "NAME")]
    theme: Option<String>,

    /// Interface language code. Valid: en de fr it es pt nl pl cs ru ja zh ko
    #[arg(short = 'l', long, value_name = "CODE")]
    language: Option<String>,

    /// Default starting difficulty. Valid: easy medium hard extreme expert
    #[arg(long, value_name = "LEVEL")]
    difficulty: Option<String>,

    /// Digit rendering style. Valid: retro (default), awkward-retro
    #[arg(long = "digit-style", value_name = "STYLE")]
    digit_style: Option<String>,

    /// Path to config file (default: ~/.config/clisudoku/config.toml)
    #[arg(long, value_name = "PATH")]
    config: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let mut app = App::new(Box::new(SystemClock));

    // 1. Load and apply config file (CLI --config overrides default path).
    let cfg = match config::load(cli.config.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Config error: {}", e);
            std::process::exit(1);
        }
    };
    if let Err(e) = cfg.apply_to(&mut app) {
        eprintln!("Config error: {}", e);
        std::process::exit(1);
    }

    // 2. CLI args override config (each arg is independent).
    if let Some(ref name) = cli.theme {
        match Theme::from_code(name) {
            Some(theme) => {
                app.theme = theme;
                app.colors = ColorScheme::for_theme(theme);
            }
            None => {
                eprintln!("Unknown theme '{}'. Valid: dark, light, high-contrast", name);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref code) = cli.language {
        match Language::from_code(code) {
            Some(lang) => app.language = lang,
            None => {
                eprintln!("Unknown language code '{}'", code);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref d) = cli.difficulty {
        match d.as_str() {
            "easy"    => app.default_difficulty_index = 0,
            "medium"  => app.default_difficulty_index = 1,
            "hard"    => app.default_difficulty_index = 2,
            "extreme" => app.default_difficulty_index = 3,
            "expert"  => app.default_difficulty_index = 4,
            other => {
                eprintln!("Unknown difficulty '{}'. Valid: easy, medium, hard, extreme, expert", other);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref ds) = cli.digit_style {
        match ds.as_str() {
            "retro"         => app.set_digit_style_retro(),
            "awkward-retro" => app.set_digit_style_awkward(),
            other => {
                eprintln!("Unknown digit-style '{}'. Valid: retro, awkward-retro", other);
                std::process::exit(1);
            }
        }
    }

    // 3. Load puzzle (mutually exclusive: -s wins over -f; --pattern is independent).
    if let Some(ref s) = cli.puzzle_str {
        load_puzzle(&mut app, s);
    } else if let Some(ref path) = cli.puzzle_file {
        match std::fs::read_to_string(path) {
            Ok(content) => load_puzzle(&mut app, content.trim()),
            Err(e) => {
                eprintln!("Cannot read file {}: {}", path.display(), e);
                std::process::exit(1);
            }
        }
    }

    if let Some(ref s) = cli.pattern {
        match clisudoku::pattern::Pattern::from_cli_str(s) {
            Ok(pattern) => app.start_generating(pattern, true),
            Err(e) => {
                eprintln!("Invalid pattern string: {}", e);
                std::process::exit(1);
            }
        }
    }

    if let Err(e) = app.run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn load_puzzle(app: &mut App, s: &str) {
    let grid = match Grid::from_str(s) {
        Ok(g) => g,
        Err(e) => {
            let msg = app
                .language
                .strings()
                .puzzle_invalid
                .replacen("{}", &e.to_string(), 1);
            app.set_start_notice(msg);
            return;
        }
    };

    let given_count = (0..9)
        .flat_map(|r| (0..9).map(move |c| (r, c)))
        .filter(|&(r, c)| grid.get(r, c).is_given())
        .count();
    if given_count < 17 {
        let msg = app
            .language
            .strings()
            .puzzle_few_givens
            .replacen("{}", &given_count.to_string(), 1);
        app.set_start_notice(msg);
        return;
    }

    let solved = solve_backtracking(grid.clone());
    if solved.is_none() {
        app.set_start_notice(app.language.strings().puzzle_no_solution.into());
        return;
    }

    app.game_state = Some(GameState::new(grid));
    app.screen = clisudoku::tui::AppScreen::Game;
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_struct_is_valid() {
        super::Cli::command().debug_assert();
    }

    #[test]
    fn parse_pattern_str_valid() {
        let p = clisudoku::pattern::Pattern::from_cli_str(&"1".repeat(81)).unwrap();
        assert_eq!(p.cell_count, 81);
    }

    #[test]
    fn parse_pattern_str_invalid_length() {
        assert!(clisudoku::pattern::Pattern::from_cli_str("1111").is_err());
    }
}
```

- [ ] **Step 3: Run tests**

```bash
cargo test --lib 2>&1 | grep -E "FAILED|test result"
cargo test 2>&1 | grep -E "FAILED|test result"
```
Expected: all pass (300+ tests).

- [ ] **Step 4: Smoke-test the binary**

```bash
cargo run -- --help
cargo run -- --theme light --language en
cargo run -- --config /dev/null   # empty config → silent, defaults used
```

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: migrate to clap, add --config, --difficulty, --digit-style"
```

---

### Task 7: Verify full ColorScheme field coverage

**Files:**
- Modify: `src/config.rs` (fix any missed color fields)

- [ ] **Step 1: Cross-check apply_color_field against ColorScheme**

Count fields in `src/tui/colors.rs` struct (around line 45–90) and verify
`apply_color_field` in `src/config.rs` covers all of them. The struct has
fields with `_fg` and `_bg` suffixes — these must match exactly.

```bash
grep "pub " src/tui/colors.rs | grep -v "fn\|enum\|struct\|const" | wc -l
```

Compare against the number of arms in `apply_color_field`. Fix any gaps.

- [ ] **Step 2: Write a coverage test**

In `src/config.rs` tests:

```rust
#[test]
fn all_color_fields_are_reachable() {
    use crate::tui::colors::ColorScheme;
    use crossterm::style::Color;

    let field_names = [
        "ui_background", "grid_border", "grid_box", "grid_cell",
        "cell_normal_bg", "cell_active_bg", "cell_active_box_bg", "cell_active_cross_bg",
        "digit_given", "digit_user", "digit_error", "digit_highlight",
        "note_normal", "note_highlight", "digit_scan",
        "ui_text", "ui_text_dim", "ui_cursor_bg", "ui_cursor_fg",
        "hint_cause_border", "hint_elim_border", "hint_target_bg",
        "hover_bg",
    ];
    let mut cs = ColorScheme::default();
    for name in &field_names {
        assert!(
            apply_color_field(&mut cs, name, Color::Black).is_some(),
            "apply_color_field: unknown field '{}'",
            name
        );
    }
}
```

- [ ] **Step 3: Run**

```bash
cargo test all_color_fields_are_reachable
```
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/config.rs
git commit -m "test: verify all ColorScheme fields are covered in apply_color_field"
```

---

### Task 8: Final verification

- [ ] **Step 1: Full test suite**

```bash
cargo test --lib 2>&1 | grep -E "FAILED|test result"
```
Expected: 0 failures.

- [ ] **Step 2: Build release binary**

```bash
cargo build --release 2>&1 | tail -5
```
Expected: exit 0.

- [ ] **Step 3: End-to-end smoke test**

```bash
# Write a minimal config
mkdir -p /tmp/clisudoku_test
cat > /tmp/clisudoku_test/config.toml << 'EOF'
[appearance]
theme = "light"
language = "en"
difficulty = "hard"
digit_style = "awkward-retro"

[colors]
digit_given = "Cyan"

[keys]
hint = "x"
EOF

cargo run -- --config /tmp/clisudoku_test/config.toml --help
```
Expected: help text printed, exit 0.

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete TOML config file and CLI args with clap"
```
