use std::collections::HashMap;
use std::path::{Path, PathBuf};
use crossterm::style::Color;
use serde::Deserialize;
use crate::i18n::Language;
use crate::tui::App;
use crate::tui::colors::{ColorScheme, Theme};
use crate::tui::input::KeyMap;

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
        other         => Err(format!(
            "Unknown color '{}'. Valid: Black, DarkGrey, Red, DarkRed, Green, DarkGreen, \
             Yellow, DarkYellow, Blue, DarkBlue, Magenta, DarkMagenta, Cyan, DarkCyan, \
             White, Grey, Reset",
            other
        )),
    }
}

/// Parse a key notation string into a char.
/// Accepts a single character or named specials: Space, Minus, Zero.
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

fn dirs_or_home() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg));
    }
    std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".config"))
}

fn default_config_path() -> Option<PathBuf> {
    dirs_or_home().map(|d| d.join("clisudoku").join("config.toml"))
}

/// Load config from `explicit_path` or the default path.
/// Returns Config::default() if the file doesn't exist (not an error).
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

impl Config {
    /// Apply config values to `app`; returns Err on the first invalid value.
    pub fn apply_to(&self, app: &mut App) -> Result<(), String> {
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
                "retro"         => app.set_digit_style_retro(),
                "awkward-retro" => app.set_digit_style_awkward(),
                other => return Err(format!(
                    "Unknown digit_style '{}'. Valid: retro, awkward-retro", other
                )),
            }
        }
        if let Some(ref d) = self.appearance.difficulty {
            app.default_difficulty_index = parse_difficulty_index(d)?;
        }

        for (key, val) in &self.colors {
            let color = parse_color(val)?;
            apply_color_field(&mut app.colors, key, color)
                .ok_or_else(|| format!("Unknown color key '{}' in [colors]", key))?;
        }

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
        other => Err(format!(
            "Unknown difficulty '{}'. Valid: easy, medium, hard, extreme, expert", other
        )),
    }
}

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

    #[test]
    fn load_returns_default_when_file_missing() {
        let tmp = std::path::Path::new("/tmp/clisudoku_nonexistent_test_config.toml");
        let cfg = load(Some(tmp)).unwrap();
        assert!(cfg.appearance.theme.is_none());
    }

    #[test]
    fn apply_appearance_theme_overrides_app() {
        use crate::tui::App;
        use crate::tui::colors::Theme;
        use crate::timer::FakeClock;

        let mut cfg = Config::default();
        cfg.appearance.theme = Some("light".to_string());

        let mut app = App::new(Box::new(FakeClock { ms: 0 }));
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

        let mut app = App::new(Box::new(FakeClock { ms: 0 }));
        cfg.apply_to(&mut app).unwrap();

        assert_eq!(app.colors.digit_given, Color::Cyan);
    }

    #[test]
    fn apply_invalid_color_returns_err() {
        use crate::tui::App;
        use crate::timer::FakeClock;

        let mut cfg = Config::default();
        cfg.colors.insert("digit_given".to_string(), "Turquoise".to_string());

        let mut app = App::new(Box::new(FakeClock { ms: 0 }));
        assert!(cfg.apply_to(&mut app).is_err());
    }

    #[test]
    fn apply_key_override_changes_keymap() {
        use crate::tui::App;
        use crate::timer::FakeClock;

        let mut cfg = Config::default();
        cfg.keys.insert("hint".to_string(), "x".to_string());

        let mut app = App::new(Box::new(FakeClock { ms: 0 }));
        cfg.apply_to(&mut app).unwrap();

        assert_eq!(app.key_map.hint, 'x');
    }

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
}
