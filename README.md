# clisudoku

A Sudoku game for the terminal — keyboard-driven, colourful, no dependencies beyond a modern terminal.

<!-- Replace with your screenshot: -->
<!-- ![clisudoku screenshot](docs/screenshot.png) -->

## Features

- **6 difficulty levels** — Easy · Medium · Hard · Extreme · Expert · Just 17
- **Hint system** — step-by-step hints with cause / elimination / target highlighting
- **Notes mode** — pencil marks per cell, auto-cleared on digit entry
- **Undo / redo** — full move history
- **Passive scan** — highlights matching digits across the grid
- **Mouse support** — click to select, hover highlights the cell
- **3 colour themes** — Dark (default) · Light · High Contrast (colourblind-safe)
- **13 interface languages** — EN · DE · ES · IT · FR · SL · EO · TP · Leet · SW · AF · PY · ID
- **2 digit styles** — Retro · Awkward-Retro
- **Boss key** — instant blank screen (press `B`)
- **Configurable keybindings** — via `~/.config/clisudoku/config.toml`
- **Load a custom puzzle** — paste an 81-char string or point to a file
- **Custom cell patterns** — generate puzzles from a 81-char pattern mask

## Requirements

- Rust 1.70+ (for building)
- A terminal with 16-colour ANSI support (xterm-256color or similar)
- Minimum terminal size: 117 × 39 characters

## Installation

```bash
git clone https://github.com/<your-username>/clisudoku.git
cd clisudoku
cargo build --release
# Binary at: target/release/clisudoku
```

Or install directly:

```bash
cargo install --path .
```

## Usage

```
clisudoku [OPTIONS]

Options:
  -s <PUZZLE>       Load puzzle from 81-char string (1-9 = given, 0/. = empty)
  -f <FILE>         Load puzzle from text file (same format as -s)
  --pattern <81C>   Generate puzzle from a custom cell-pattern mask
  -t <NAME>         Colour theme: dark (default) | light | high-contrast
  -l <CODE>         Language: en de es it fr sl eo tp leet sw af py id
  --difficulty <L>  Starting difficulty: easy medium hard extreme expert
  --digit-style <S> Digit style: retro (default) | awkward-retro
  --config <PATH>   Config file path (default: ~/.config/clisudoku/config.toml)
```

### Examples

```bash
# Start with a specific puzzle
clisudoku -s 530070000600195000098000060800060003400803001700020006060000280000419005000080079

# Light theme, German interface
clisudoku -t light -l de

# High-contrast, start on Hard
clisudoku -t high-contrast --difficulty hard
```

## Controls

| Key | Action |
|-----|--------|
| `↑ ↓ ← →` | Move cursor |
| `1`–`9` | Enter digit |
| `0` / `-` | Clear cell / toggle notes mode |
| `Z` / `Y` | Undo / Redo |
| `H` | Request hint |
| `S` | Toggle passive scan |
| `E` | Toggle error highlighting |
| `Space` | Pause |
| `M` | Toggle mouse mode |
| `B` | Boss key (blank screen) |
| `?` | Help screen |
| `Q` / `Esc` | Quit / Back |

The numpad can be used for 3×3 box navigation: press a numpad key to select a box, then another to pick the cell within it.

Press `?` in-game for the full controls, rules, and colour reference.

## Configuration

Create `~/.config/clisudoku/config.toml`:

```toml
[appearance]
theme = "dark"          # dark | light | high-contrast
language = "en"         # en de es it fr sl eo tp leet sw af py id
digit_style = "retro"   # retro | awkward-retro

[keys]
hint = "h"
pause = " "
```

## Difficulty levels

| Level | Techniques required |
|-------|-------------------|
| Easy | Naked / hidden singles |
| Medium | Naked pairs, box-line reduction |
| Hard | X-Wing |
| Extreme | Swordfish, Jellyfish |
| Expert | XY-Wing, XYZ-Wing, chains, unique rectangles, … |
| Just 17 | Exactly 17 clues — the mathematical minimum for a unique solution |

## License

MIT
