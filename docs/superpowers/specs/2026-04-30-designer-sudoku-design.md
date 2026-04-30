# Designer Sudoku — Design Spec

## Goal

Add a "Designer Sudoku" mode where given-cells form a recognisable visual pattern (smiley, heart, WiFi symbol, …). The feature integrates with the existing difficulty flow, uses a background generator thread for responsive UI during pattern-constrained generation, and records played designer games in the database under a new `designs` category.

## Architecture

**New module:** `src/pattern/mod.rs`
- `Pattern` struct: name (EN + DE), 81-bit cell mask (`[bool; 81]`), cell count, category tag
- 27 built-in patterns, sorted by cell count descending at compile time
- CLI parsing helper: parse 81-char string of `.`/`0` (empty) and `1`/`*` (pattern cell)

**Modified:** `src/generator/mod.rs`
- `generate_with_pattern(&self, pattern: &Pattern, difficulty: Option<Difficulty>) -> GeneratorResult`
- Fills a complete grid, removes non-pattern cells unconditionally, then iteratively removes pattern cells while uniqueness is maintained
- If the resulting puzzle has fewer than 17 givens: Ansatz C — add back the minimum extra cells outside the pattern needed for uniqueness (logged internally, not visible to user)
- Returns `GeneratorResult { grid, difficulty, extra_cells_used: bool }`

**New:** `src/tui/generating.rs`
- Generates on a background thread (`std::thread::spawn`)
- Main thread polls via `mpsc::channel` with 50 ms tick
- States: `Generating { verb_idx, countdown_secs }` → `NewSeed { countdown_secs }` → `Done(Grid, Difficulty)` → `Cancelled`
- Per-seed timeout: **3 seconds**; on timeout → new seed, display "using new seed…"
- Cancellable at any time with `Esc` → returns to previous screen

**Modified:** `src/tui/mod.rs`
- New `AppScreen::PatternSelect { selected: usize }` state
- New `AppScreen::Generating { ... }` state
- `DifficultySelect` gets a fourth option "Designer ▶"

**Modified:** `src/tui/render/` — two new render functions:
- `pattern_select::render_pattern_select` — miniature + title + count
- `generating::render_generating` — grid-area message overlay

**Modified:** `src/db/` — new `GameCategory::Design` variant; schema migration adds `category TEXT DEFAULT 'classic'` to `games` table.

**Modified:** `src/main.rs` — `--pattern <81chars>` CLI flag

---

## Screen Flow

```
Start Screen
    └─ New Game
           └─ Difficulty Select  [Easy / Medium / Hard / Designer ▶]
                  └─ Designer ▶
                         └─ Pattern Select Screen
                                └─ Enter
                                       └─ Generating Screen
                                              ├─ Done  → Game
                                              └─ Esc   → Pattern Select
CLI: --pattern "1..1..1..."
       └─ Generating Screen (same as above)
              ├─ Done  → Game
              └─ Esc   → Difficulty Select
```

---

## Pattern Select Screen Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                        Designer Sudoku                              │
│                                                                     │
│                           Smiley 😊                                 │
│                                                                     │
│                     · · · █ █ █ · · ·                               │
│                     · · █ · · · █ · ·                               │
│                     · · █ █ · █ █ · ·                               │
│                     · █ · · · · · █ ·                               │
│                     █ · · · · · · · █                               │
│                     █ · █ · · · █ · █                               │
│                     · █ · · · · · █ ·                               │
│                     · · · █ █ █ · · ·                               │
│                     · · · · · · · · ·                               │
│                                                                     │
│                          31 / 81                                    │
│                                                                     │
│                       ◄   7 / 27   ►                                │
│                                                                     │
│                    Enter: select   Esc: back                        │
└─────────────────────────────────────────────────────────────────────┘
```

- Each cell: `█` (U+2588) for pattern cell, `·` for empty, space-separated
- Pattern name above miniature (EN/DE via i18n)
- Position indicator `n / 27` below miniature
- Left/Right arrows navigate; wraps around (zyklisch)
- Patterns sorted by cell count **descending** (most givens first)
- No difficulty shown — determined after generation

---

## Generating Screen Layout

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                     │
│                                                                     │
│                                                                     │
│                                                                     │
│                                                                     │
│                   baking sudoku...          3                       │
│                                                                     │
│                                                                     │
│                                                                     │
│                                                                     │
│                    Esc: cancel                                      │
└─────────────────────────────────────────────────────────────────────┘
```

- Message shown **centered in the grid area** (cols 2–74, rows 1–37)
- Format: `"<verb> sudoku..."` with countdown seconds on the right
- On new seed: `"using new seed…"` for ~1 s, then resumes with next verb
- Verb list (37 entries, randomised order per session):
  `generating` `frying` `baking` `roasting` `shoveling` `tinkering`
  `brewing` `distilling` `cooking` `boiling` `simmering` `grilling`
  `toasting` `smoking` `seasoning` `marinating` `kneading` `blending`
  `mixing` `stirring` `whipping` `grinding` `fermenting` `percolating`
  `crafting` `forging` `spinning` `shuffling` `sculpting` `chiseling`
  `polishing` `weaving` `knitting` `mining` `hatching` `conjuring`
  `assembling` `composing`
- Verbs cycle through the shuffled list; each verb shown for one 3 s seed attempt
- No maximum retry count — user waits as long as needed or cancels

---

## Hint Pre-Check (applies to all game modes)

Before showing any hint (`h`), the app checks in order:

1. **Incorrect filled digits** — any `Filled(d)` cell where `d ≠ solution[r][c]`
   → Show warning panel text (EN: *"Fix your errors first."* / DE: *"Zuerst Fehler korrigieren."*)
   → `hint_count += 1`, no hint shown

2. **Incorrect notes** — any note candidate `d` in cell `(r,c)` that is already present in the same row, column, or box
   → Show warning panel text (EN: *"Your notes contain errors."* / DE: *"Notizen enthalten Fehler."*)
   → `hint_count += 1`, no hint shown

3. **All correct** → proceed with existing hint logic

In both warning cases the panel replaces the controls section identically to a normal hint — dismissed with any key.

---

## Hint Count Display

`hint_count` (already tracked in `GameStats`) is shown in the panel alongside the timer and mode indicators.

Format: `h: 3` or `hints: 3` — fits in the existing panel line budget.

---

## Database

New `GameCategory` enum:
```rust
pub enum GameCategory { Classic, Design }
```

Schema migration (applied on first run if column absent):
```sql
ALTER TABLE games ADD COLUMN category TEXT NOT NULL DEFAULT 'classic';
```

Designer games are stored with `category = 'design'` and additionally `pattern_name TEXT` (nullable for classic games).

---

## CLI Flag

```
--pattern <81chars>
```
- 81 characters: `.` or `0` = empty cell, `1` or `*` = pattern cell
- Exactly 81 characters required; invalid input → error message + exit
- Goes directly to `AppScreen::Generating`; `Esc` during generation returns to `DifficultySelect`

---

## 27 Built-in Patterns

Sorted by cell count descending (name / count):

| # | Name | Cells |
|---|------|-------|
| 1 | Holy Crap | 46 |
| 2 | Checker | 41 |
| 3 | Heart | 40 |
| 4 | Wind Up | 40 |
| 5 | Shit Happens | 39 |
| 6 | Bug or Feature? | 39 |
| 7 | Ripples | 37 |
| 8 | Fire Fighter | 36 |
| 9 | Mamihlapinatapai | 36 |
| 10 | Rudolph | 41 |
| 11 | Border | 32 |
| 12 | Smiley | 31 |
| 13 | Badley | 31 |
| 14 | Bigger Fish to Fry | 31 |
| 15 | Lost My Cherries | 30 |
| 16 | Anchor | 30 |
| 17 | Five to Twelve | 29 |
| 18 | Joshua | 29 |
| 19 | An Apple a Day | 28 |
| 20 | Diamond | 28 |
| 21 | Asterisk | 33 |
| 22 | Rainy Day | 32 |
| 23 | Per Aspera ad Astra | 27 |
| 24 | Wave | 27 |
| 25 | Minion | 27 |
| 26 | 42 | 26 |
| 27 | Home Office | 26 |
| 28 | We Are Connected Wirelessly | 22 |

### Pattern Masks (row by row, 1=pattern cell, 0=empty)

```
diamond
010010010
100101001
001000100
010010010
100101001
010010010
001000100
100101001
010010010

smiley
001111100
010000010
001100110  (eyes at col 2-3 and 5-6: 0,0,1,1,0,1,1,0,0)
010000010
100000001
101000101
010000010
000111000
000000000

badley
001111100
010000010
001100110
010000010
100000001
000111001  (frown: top of inverted U)
101000101
010000010
001111100

heart
011000110
111101111
110111011
110010011
110000011
011000110
001101100
000111000
000010000

wave
001001001
010010010
100100100
010010010
001001001
010010010
100100100
010010010
001001001

asterisk
100010001
010010010
001010100
000111000
111111111
000111000
001010100
010010010
100010001

checker
101010101
010101010
101010101
010101010
101010101
010101010
101010101
010101010
101010101

border
111111111
100000001
100000001
100000001
100000001
100000001
100000001
100000001
111111111

ripples
000111000
011000110
010111010
101000101
101010101
101000101
010111010
011000110
000111000

anchor
000111000
000111000
000010000
001111100
000010000
010010010
111010111
010010010
001101100

42
000000000
101000110
101001001
101000001
101000010
111100100
001001000
001001111
000000000

home_office
000010000
000111000
001101100
110000011
010000010
010000010
010001010
010101010
010100010

shit_happens
000001000
010000000
000110010
001110000
001001100
011111110
011111110
011100011
111111111

rainy_day
000111000
011111110
010010010
111111111
101010101
000010000
000010000
000010000
000110000

an_apple_a_day
000001000
000010000
011111110
110000011
100100001
101000001
100000001
010000010
001111100

joshua
000010000
000111000
000101000
010101000
011101010
000101110
000101110
000101000
001111100

lost_my_cherries
000000111
000001100
000011100
000110100
000100100
011100110
100101001
100101001
011000110

mamihlapinatapai
011000110
000000000
011000110
100101001
100101001
110101101
110101101
100101001
011000110

holy_crap
001101100
110000011
111101111
100000001
101111101
111010111
001111100
011111110
010000010

per_aspera_ad_astra
000010000
000111000
000101000
000111000
000101000
000111000
001111100
001010100
011010110

rudolph
100001000
111111000
001100000
011100000
001111110
001111111
001111111
001010101
001010101

bug_or_feature
000101000
110111011
011111110
000101000
011101110
000101000
011111110
110111011
000010000

five_to_twelve
000111000
011010110
010110010
100010001
100010001
100000001
010000010
011000110
000111000

minion
000111000
001000100
010010010
010101010
010010010
010000010
010111010
001000100
000111000

wind_up
000010000
111101111
000010000
001111000
011011001
010011111
011111100
001001000
011111110

fire_fighter
000111000
001101100
011000110
010010010
111111111
010000010
010101010
010000010
001111100

bigger_fish_to_fry
000100000
011111001
110001011
101001110
100000010
100000110
010001011
001110001
000001000

we_are_connected_wirelessly
001111100
010000010
100000001
001111100
010000010
000000000
000111000
001000100
000010000
```

---

## Out of Scope

- Tier 2 hint strategies (Naked Triples, X-Wing, Swordfish) — separate feature
- Extreme difficulty level — separate feature
- Pattern editor in-game — user can create patterns via `--pattern` CLI flag
- Animated generation (showing cells appear one by one)
- Pattern sharing / export
