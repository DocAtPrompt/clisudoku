# Config File + CLI Args — Design Spec

## Goal

Add a TOML config file (`~/.config/clisudoku/config.toml`) and complete the CLI
args using `clap`, so users can customize appearance, individual colors, and key
bindings without a settings UI.

---

## Config Loading Order

Later sources override earlier ones:

1. Built-in defaults (`App::new()`)
2. `~/.config/clisudoku/config.toml` (auto-loaded if present; missing = silent)
3. `--config <path>` CLI flag (explicit override path)
4. Remaining clap args (`--theme`, `--language`, `--difficulty`, `--digit-style`)

---

## Config File Format

```toml
[appearance]
theme = "dark"           # dark | light | high-contrast
language = "de"          # en | de | fr | it | es | pt | nl | pl | cs | ru | ja | zh | ko
digit_style = "retro"    # retro | awkward-retro
difficulty = "medium"    # easy | medium | hard | extreme | expert

[colors]
# Optional per-field overrides of all 25 ColorScheme fields.
# Valid values: Black, DarkGrey, Red, DarkRed, Green, DarkGreen, Yellow,
#               DarkYellow, Blue, DarkBlue, Magenta, DarkMagenta, Cyan,
#               DarkCyan, White, Grey, Reset
# cell_normal_bg = "Black"
# digit_given = "Yellow"
# hint_cause_border = "Green"

[keys]
# Optional remapping of game actions.
# Value: single character ("h") or named special key ("Space").
# Named special keys: Space, Minus, Zero
# hint         = "h"
# pause        = "Space"
# scan         = "s"
# errors       = "e"
# note_mode    = "Zero"
# clear        = "Minus"
# undo         = "u"
# redo         = "r"
# mouse_toggle = "m"
```

---

## Remappable Actions

| Config key    | Default | AppAction        |
|---------------|---------|------------------|
| `hint`        | `h`     | RequestHint      |
| `pause`       | `Space` | Pause            |
| `scan`        | `s`     | ToggleScan       |
| `errors`      | `e`     | ToggleErrors     |
| `note_mode`   | `Zero`  | ToggleMode       |
| `clear`       | `Minus` | ClearCell        |
| `undo`        | `u`     | Undo             |
| `redo`        | `r`     | Redo             |
| `mouse_toggle`| `m`     | ToggleMouseMode  |

**Not remappable:** arrow keys, digits 1–9, Enter, Esc, y/n (confirm),
Ctrl+Z/Y (undo/redo combos), b (boss key).

Conflicts are the user's responsibility. If `hint = "b"` is set, pressing `b`
fires RequestHint (KeyMap is checked first); the boss key becomes unreachable.

---

## New CLI Args (clap)

All existing args are preserved unchanged:

| Existing             | Meaning                               |
|----------------------|---------------------------------------|
| `-s <PUZZLE>`        | Load puzzle from 81-char string       |
| `-f <FILE>`          | Load puzzle from file                 |
| `--pattern <81chars>`| Designer pattern                      |
| `-t, --theme <NAME>` | Color theme                           |
| `-l, --language <CODE>` | Interface language               |
| `-h, --help`         | Help                                  |

New args:

| New arg                    | Meaning                                        |
|----------------------------|------------------------------------------------|
| `--config <PATH>`          | Use this file instead of default config path   |
| `--difficulty <LEVEL>`     | easy \| medium \| hard \| extreme \| expert    |
| `--digit-style <STYLE>`    | retro \| awkward-retro                         |

---

## Error Handling

- Missing config file → silent (not an error)
- Unknown TOML keys → ignored (forward compatibility)
- Invalid color name → `eprintln!` + `exit(1)`
- Invalid key notation → `eprintln!` + `exit(1)`
- Invalid enum value (theme, language, etc.) → `eprintln!` + `exit(1)`

---

## Files Affected

| File | Change |
|------|--------|
| `Cargo.toml` | Add `clap` (derive), `toml` |
| `src/config.rs` | New: Config struct, load(), apply_to() |
| `src/tui/input.rs` | Add KeyMap struct, update map_key_to_action() |
| `src/tui/mod.rs` | Add `key_map: KeyMap` + `default_difficulty_index: usize` to App |
| `src/main.rs` | Replace manual parsing with clap, wire config loading |

---

## Architecture Notes

`src/config.rs` owns all config logic: parsing, validation, and application to
`App`. It has no dependency on `tui` internals beyond the public `App` struct.

`KeyMap` lives in `src/tui/input.rs` (alongside `AppAction`) because it
directly shapes key-to-action translation. `map_key_to_action` gains a third
parameter `km: &KeyMap`; the KeyMap is checked first (for non-ctrl char keys),
then the original match handles non-remappable keys.

`default_difficulty_index: usize` is added to App (0 = Easy). When
`DifficultySelect` is opened, it uses this as the initial `selected` value
instead of hardcoded 0.
