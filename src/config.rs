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
}
