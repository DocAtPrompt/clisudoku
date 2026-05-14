# Help Screen Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a full-screen Help overlay accessible via `?` from the Start and Game screens, with three ◄/►-navigable sections (Controls, Rules, Colors) fully translated into all 13 project languages.

**Architecture:** A new `AppScreen::Help { section: usize }` variant carries section state (0=Controls, 1=Rules, 2=Colors). A private `toggle_help()` helper on `App` opens/closes it; `handle_help_action()` handles ◄/► navigation. Rendering is done in a new `src/tui/render/help.rs` module, called directly from `render_current()` following the same pattern as `render_boss`. The `?` key is non-remappable and is intercepted in `run()` before the hint/overlay-dismissal block.

**Tech Stack:** Rust, crossterm, existing `ColorScheme` / `Strings` / `AppScreen` infrastructure.

---

## File Map

| File | What changes |
|------|--------------|
| `src/i18n/mod.rs` | 28 new `&'static str` fields in `Strings` struct; values for all 13 language constants |
| `src/tui/input.rs` | Add `AppAction::ToggleHelp` variant; add `'?' => AppAction::ToggleHelp` in non-remappable match |
| `src/tui/mod.rs` | Add `AppScreen::Help { section: usize }`; add `toggle_help()`; intercept `ToggleHelp` in `run()` before hint-dismiss; add `handle_help_action()`; wire `AppScreen::Help` arm in `handle_action` and `render_current()` |
| `src/tui/render/mod.rs` | Add `pub mod help;` |
| `src/tui/render/help.rs` | New: `render_help(out, section, colors, strings)` |
| `src/tui/render/status_bar.rs` | Add `ctrl_help` entry to controls list |

---

## Task 1: i18n — Add 28 new string fields to `Strings` struct

**Files:**
- Modify: `src/i18n/mod.rs`

### Step 1.1: Write a failing compile-test (missing fields cause compile errors)

- [ ] In `src/i18n/mod.rs`, add the 28 new fields to the `Strings` struct **after the existing `ctrl_mouse` field** (around line 68), before `difficulty_designer`:

```rust
    /// Help screen: panel control label (≤ 34 chars).
    pub ctrl_help: &'static str,
    /// Help screen title.
    pub help_title: &'static str,
    pub help_section_controls: &'static str,
    pub help_section_rules: &'static str,
    pub help_section_colors: &'static str,
    /// Bottom bar hint: "◄ ►  switch section   ?  close"
    pub help_close_hint: &'static str,
    pub help_group_navigation: &'static str,
    pub help_group_quick_nav: &'static str,
    /// Two-line body; lines separated by `\n`.
    pub help_quick_nav_body: &'static str,
    pub help_group_input: &'static str,
    pub help_group_functions: &'static str,
    pub help_group_rules: &'static str,
    /// Two-line body; lines separated by `\n`.
    pub help_rules_body: &'static str,
    pub help_group_notes: &'static str,
    /// Two-line body; lines separated by `\n`.
    pub help_notes_body: &'static str,
    pub help_group_hints: &'static str,
    /// Two-line body; lines separated by `\n`.
    pub help_hints_body: &'static str,
    pub help_color_given: &'static str,
    pub help_color_user: &'static str,
    pub help_color_error: &'static str,
    pub help_color_cursor: &'static str,
    pub help_color_cross: &'static str,
    pub help_color_box: &'static str,
    pub help_color_scan: &'static str,
    pub help_color_hover: &'static str,
    pub help_color_hint_cause: &'static str,
    pub help_color_hint_elim: &'static str,
    pub help_color_hint_target: &'static str,
```

- [ ] **Run build to see compiler errors** — every language constant is now missing these 28 fields:

```
cargo build 2>&1 | head -40
```

Expected: `error[E0063]: missing fields` for each of EN, DE, ES, IT, FR, SL, EO, TP, LEET, SW, AF, PY, ID.

### Step 1.2: Add the 28 fields to the `EN` constant

- [ ] In `src/i18n/mod.rs`, find `pub static EN: Strings = Strings {` (around line 261) and add the 28 fields **after `ctrl_mouse`**:

```rust
    ctrl_help: "  ?      help",
    help_title: "HELP",
    help_section_controls: "Controls",
    help_section_rules: "Rules",
    help_section_colors: "Colors",
    help_close_hint: "\u{25c4} \u{25ba}  switch section   ?  close",
    help_group_navigation: "Navigation",
    help_group_quick_nav: "Quick Navigation",
    help_quick_nav_body: "Press Enter to select one of 9 boxes,\nthen 1\u{2013}9 to place the cursor in that cell.",
    help_group_input: "Input",
    help_group_functions: "Functions",
    help_group_rules: "Sudoku Rules",
    help_rules_body: "Each row, column and 3\u{d7}3 box must contain\ndigits 1\u{2013}9 exactly once.",
    help_group_notes: "Notes",
    help_notes_body: "Note mode (0): mark candidate digits.\nNotes clear when a digit is placed nearby.",
    help_group_hints: "Hints",
    help_hints_body: "Press h to request a hint. Highlighted\ncells show the next logical solving step.",
    help_color_given: "Given digit",
    help_color_user: "Your entry",
    help_color_error: "Error",
    help_color_cursor: "Active cell",
    help_color_cross: "Row/col highlight",
    help_color_box: "Box highlight",
    help_color_scan: "Scan match",
    help_color_hover: "Mouse hover",
    help_color_hint_cause: "Hint: cause",
    help_color_hint_elim: "Hint: elimination",
    help_color_hint_target: "Hint: target",
```

### Step 1.3: Add the 28 fields to all remaining language constants

> Note: `◄` is `\u{25c4}`, `►` is `\u{25ba}`, `–` is `\u{2013}`, `×` is `\u{d7}`.

- [ ] Add to **DE** (`pub static DE: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      Hilfe",
    help_title: "HILFE",
    help_section_controls: "Steuerung",
    help_section_rules: "Regeln",
    help_section_colors: "Farben",
    help_close_hint: "\u{25c4} \u{25ba}  Abschnitt wechseln   ?  schlie\u{df}en",
    help_group_navigation: "Navigation",
    help_group_quick_nav: "Schnellsteuerung",
    help_quick_nav_body: "Enter w\u{e4}hlt eine der 9 Boxen,\ndann 1\u{2013}9 f\u{fc}r die Zelle darin.",
    help_group_input: "Eingabe",
    help_group_functions: "Funktionen",
    help_group_rules: "Spielregeln",
    help_rules_body: "Jede Zeile, Spalte und 3\u{d7}3-Box muss\ndie Ziffern 1\u{2013}9 genau einmal enthalten.",
    help_group_notes: "Notizen",
    help_notes_body: "Notizmodus (0): Kandidaten markieren.\nNotizen werden beim Eintragen gecleart.",
    help_group_hints: "Hinweise",
    help_hints_body: "h f\u{fc}r einen Hinweis. Markierte Zellen\nzeigen den n\u{e4}chsten logischen Schritt.",
    help_color_given: "Vorgegebene Ziffer",
    help_color_user: "Eigene Eingabe",
    help_color_error: "Fehler",
    help_color_cursor: "Aktive Zelle",
    help_color_cross: "Zeile/Spalte",
    help_color_box: "Box-Highlight",
    help_color_scan: "Scan-Treffer",
    help_color_hover: "Maus-Hover",
    help_color_hint_cause: "Hinweis: Ursache",
    help_color_hint_elim: "Hinweis: Elimination",
    help_color_hint_target: "Hinweis: Ziel",
```

- [ ] Add to **ES** (`pub static ES: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      ayuda",
    help_title: "AYUDA",
    help_section_controls: "Controles",
    help_section_rules: "Reglas",
    help_section_colors: "Colores",
    help_close_hint: "\u{25c4} \u{25ba}  cambiar secci\u{f3}n   ?  cerrar",
    help_group_navigation: "Navegaci\u{f3}n",
    help_group_quick_nav: "Navegaci\u{f3}n r\u{e1}pida",
    help_quick_nav_body: "Pulsa Intro para elegir una caja,\nluego 1\u{2013}9 para la celda.",
    help_group_input: "Entrada",
    help_group_functions: "Funciones",
    help_group_rules: "Reglas del Sudoku",
    help_rules_body: "Cada fila, columna y caja 3\u{d7}3 debe\ncontener los d\u{ed}gitos 1\u{2013}9 exactamente una vez.",
    help_group_notes: "Notas",
    help_notes_body: "Modo notas (0): marcar candidatos.\nLas notas se borran autom\u{e1}ticamente.",
    help_group_hints: "Pistas",
    help_hints_body: "Pulsa h para una pista. Las celdas\nresaltadas muestran el siguiente paso.",
    help_color_given: "D\u{ed}gito dado",
    help_color_user: "Tu entrada",
    help_color_error: "Error",
    help_color_cursor: "Celda activa",
    help_color_cross: "Fila/columna",
    help_color_box: "Caja",
    help_color_scan: "Coincidencia",
    help_color_hover: "Hover rat\u{f3}n",
    help_color_hint_cause: "Pista: causa",
    help_color_hint_elim: "Pista: elim.",
    help_color_hint_target: "Pista: objetivo",
```

- [ ] Add to **IT** (`pub static IT: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      aiuto",
    help_title: "AIUTO",
    help_section_controls: "Controlli",
    help_section_rules: "Regole",
    help_section_colors: "Colori",
    help_close_hint: "\u{25c4} \u{25ba}  cambia sezione   ?  chiudi",
    help_group_navigation: "Navigazione",
    help_group_quick_nav: "Navigazione rapida",
    help_quick_nav_body: "Premi Invio per scegliere un riquadro,\npoi 1\u{2013}9 per la cella.",
    help_group_input: "Inserimento",
    help_group_functions: "Funzioni",
    help_group_rules: "Regole del Sudoku",
    help_rules_body: "Ogni riga, colonna e riquadro 3\u{d7}3 deve\ncontenere le cifre 1\u{2013}9 esattamente una volta.",
    help_group_notes: "Note",
    help_notes_body: "Modalit\u{e0} note (0): segnare i candidati.\nLe note vengono cancellate automaticamente.",
    help_group_hints: "Suggerimenti",
    help_hints_body: "Premi h per un suggerimento. Le celle\nevidenziate mostrano il passo successivo.",
    help_color_given: "Cifra data",
    help_color_user: "Tua voce",
    help_color_error: "Errore",
    help_color_cursor: "Cella attiva",
    help_color_cross: "Riga/colonna",
    help_color_box: "Riquadro",
    help_color_scan: "Corrispondenza",
    help_color_hover: "Hover mouse",
    help_color_hint_cause: "Sugger.: causa",
    help_color_hint_elim: "Sugger.: elim.",
    help_color_hint_target: "Sugger.: target",
```

- [ ] Add to **FR** (`pub static FR: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      aide",
    help_title: "AIDE",
    help_section_controls: "Contr\u{f4}les",
    help_section_rules: "R\u{e8}gles",
    help_section_colors: "Couleurs",
    help_close_hint: "\u{25c4} \u{25ba}  changer section   ?  fermer",
    help_group_navigation: "Navigation",
    help_group_quick_nav: "Navigation rapide",
    help_quick_nav_body: "Appuyez Entr\u{e9}e pour choisir une bo\u{ee}te,\npuis 1\u{2013}9 pour la cellule.",
    help_group_input: "Saisie",
    help_group_functions: "Fonctions",
    help_group_rules: "R\u{e8}gles du Sudoku",
    help_rules_body: "Chaque ligne, colonne et bo\u{ee}te 3\u{d7}3 doit\ncontenir les chiffres 1\u{2013}9 une seule fois.",
    help_group_notes: "Notes",
    help_notes_body: "Mode notes (0): marquer les candidats.\nLes notes sont effac\u{e9}es automatiquement.",
    help_group_hints: "Indices",
    help_hints_body: "Appuyez h pour un indice. Les cellules\nmises en \u{e9}vidence montrent l'\u{e9}tape suivante.",
    help_color_given: "Chiffre donn\u{e9}",
    help_color_user: "Votre saisie",
    help_color_error: "Erreur",
    help_color_cursor: "Cellule active",
    help_color_cross: "Ligne/colonne",
    help_color_box: "Bo\u{ee}te",
    help_color_scan: "Scan",
    help_color_hover: "Survol souris",
    help_color_hint_cause: "Indice: cause",
    help_color_hint_elim: "Indice: \u{e9}lim.",
    help_color_hint_target: "Indice: cible",
```

- [ ] Add to **SL** (`pub static SL: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      pomo\u{10d}",
    help_title: "POMO\u{10c}",
    help_section_controls: "Upravljanje",
    help_section_rules: "Pravila",
    help_section_colors: "Barve",
    help_close_hint: "\u{25c4} \u{25ba}  menjaj razdelek   ?  zapri",
    help_group_navigation: "Pomikanje",
    help_group_quick_nav: "Hitro pomikanje",
    help_quick_nav_body: "Pritisni Enter za izbiro bloka,\npotem 1\u{2013}9 za celico.",
    help_group_input: "Vnos",
    help_group_functions: "Funkcije",
    help_group_rules: "Pravila Sudokuja",
    help_rules_body: "Vsaka vrstica, stolpec in blok 3\u{d7}3 mora\nvsebovati cifre 1\u{2013}9 natanko enkrat.",
    help_group_notes: "Opombe",
    help_notes_body: "Na\u{10d}in opomb (0): ozna\u{10d}i kandidate.\nOpombe se samodejno izbri\u{161}ejo.",
    help_group_hints: "Namigi",
    help_hints_body: "Pritisni h za namig. Ozna\u{10d}ene celice\npoka\u{17e}ejo naslednji logi\u{10d}ni korak.",
    help_color_given: "Dana cifra",
    help_color_user: "Tvoj vnos",
    help_color_error: "Napaka",
    help_color_cursor: "Aktivna celica",
    help_color_cross: "Vrstica/stolpec",
    help_color_box: "Blok",
    help_color_scan: "Ujemanje",
    help_color_hover: "Lebdenje mi\u{161}ke",
    help_color_hint_cause: "Namig: vzrok",
    help_color_hint_elim: "Namig: elim.",
    help_color_hint_target: "Namig: cilj",
```

- [ ] Add to **EO** (`pub static EO: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      helpo",
    help_title: "HELPO",
    help_section_controls: "Kontroloj",
    help_section_rules: "Reguloj",
    help_section_colors: "Koloroj",
    help_close_hint: "\u{25c4} \u{25ba}  \u{15d}an\u{11d}i sekcion   ?  fermi",
    help_group_navigation: "Navigado",
    help_group_quick_nav: "Rapida navigado",
    help_quick_nav_body: "Premu Enter por elekti blokon,\nposte 1\u{2013}9 por la \u{109}elo.",
    help_group_input: "Enigo",
    help_group_functions: "Funkcioj",
    help_group_rules: "Reguloj de Sudoku",
    help_rules_body: "\u{108}iu vico, kolumno kaj bloko 3\u{d7}3 devas\nenhavi ciferojn 1\u{2013}9 \u{11d}uste unufoje.",
    help_group_notes: "Notoj",
    help_notes_body: "Nota re\u{11d}imo (0): marki kandidatojn.\nNotoj fori\u{11d}as a\u{16d}tomate.",
    help_group_hints: "Sugestoj",
    help_hints_body: "Premu h por sugeston. Elstarigitaj\n\u{109}eloj montras la sekvan pa\u{15d}on.",
    help_color_given: "Donita cifero",
    help_color_user: "Via enigo",
    help_color_error: "Eraro",
    help_color_cursor: "Aktiva \u{109}elo",
    help_color_cross: "Vico/kolumno",
    help_color_box: "Bloko",
    help_color_scan: "Kongruo",
    help_color_hover: "Musa \u{15d}vebo",
    help_color_hint_cause: "Sugesto: ka\u{16d}zo",
    help_color_hint_elim: "Sugesto: elim.",
    help_color_hint_target: "Sugesto: celo",
```

- [ ] Add to **TP** (`pub static TP: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      sona",
    help_title: "SONA",
    help_section_controls: "nasin luka",
    help_section_rules: "lawa",
    help_section_colors: "kule",
    help_close_hint: "\u{25c4} \u{25ba}  ante   ?  pini",
    help_group_navigation: "tawa",
    help_group_quick_nav: "tawa pona",
    help_quick_nav_body: "o kepeken Enter tawa poki,\no kepeken 1\u{2013}9 tawa seli.",
    help_group_input: "pana",
    help_group_functions: "pali",
    help_group_rules: "lawa pi nanpa",
    help_rules_body: "poki ale li wile jo e nanpa 1\u{2013}9\nwan taso.",
    help_group_notes: "sitelen",
    help_notes_body: "nasin sitelen (0): o sitelen e nasin.\nsitelen li weka lon tenpo kama.",
    help_group_hints: "sona",
    help_hints_body: "o kepeken h tawa sona.\nseli mute li jo e pali kama.",
    help_color_given: "nanpa lon",
    help_color_user: "nanpa sina",
    help_color_error: "pakala",
    help_color_cursor: "seli ni",
    help_color_cross: "poka/anpa",
    help_color_box: "poki",
    help_color_scan: "sama",
    help_color_hover: "noka soweli",
    help_color_hint_cause: "sona: tan",
    help_color_hint_elim: "sona: weka",
    help_color_hint_target: "sona: pini",
```

- [ ] Add to **LEET** (`pub static LEET: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      h3lp",
    help_title: "H3LP",
    help_section_controls: "C0NTR0LZ",
    help_section_rules: "RUL3Z",
    help_section_colors: "C0L0RZ",
    help_close_hint: "\u{25c4} \u{25ba}  sw1tch s3ct10n   ?  cl0s3",
    help_group_navigation: "N4V1G4T10N",
    help_group_quick_nav: "QU1CK N4V",
    help_quick_nav_body: "pr3ss 3nt3r t0 ch00s3 4 b0x,\nth3n 1\u{2013}9 f0r th3 c3ll.",
    help_group_input: "1NPUT",
    help_group_functions: "FUNCT10NZ",
    help_group_rules: "SUDOKU RUL3Z",
    help_rules_body: "3v3ry r0w, c0lumn 4nd 3\u{d7}3 b0x\nmu5t c0nt41n 1\u{2013}9 3x4ctly 0nc3.",
    help_group_notes: "N0T3Z",
    help_notes_body: "n0t3 m0d3 (0): m4rk c4nd1d4t3z.\nn0t3z 4r3 cl34r3d 4ut0m4t1c4lly.",
    help_group_hints: "H1NTZ",
    help_hints_body: "pr3ss h f0r 4 h1nt. h1ghl1ght3d\nc3llz sh0w th3 n3xt st3p.",
    help_color_given: "g1v3n d1g1t",
    help_color_user: "ur 1nput",
    help_color_error: "3rr0r",
    help_color_cursor: "4ct1v3 c3ll",
    help_color_cross: "r0w/c0l",
    help_color_box: "b0x",
    help_color_scan: "sc4n m4tch",
    help_color_hover: "m0us3 h0v3r",
    help_color_hint_cause: "h1nt: c4us3",
    help_color_hint_elim: "h1nt: 3l1m.",
    help_color_hint_target: "h1nt: t4rg3t",
```

- [ ] Add to **SW** (`pub static SW: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      msaada",
    help_title: "MSAADA",
    help_section_controls: "Vidhibiti",
    help_section_rules: "Kanuni",
    help_section_colors: "Rangi",
    help_close_hint: "\u{25c4} \u{25ba}  badilisha sehemu   ?  funga",
    help_group_navigation: "Uabiri",
    help_group_quick_nav: "Uabiri wa haraka",
    help_quick_nav_body: "Bonyeza Enter kuchagua sanduku,\nkisha 1\u{2013}9 kwa seli.",
    help_group_input: "Ingizo",
    help_group_functions: "Vitendo",
    help_group_rules: "Kanuni za Sudoku",
    help_rules_body: "Kila safu, safu wima na sanduku 3\u{d7}3 lazima\nliche na nambari 1\u{2013}9 mara moja tu.",
    help_group_notes: "Maelezo",
    help_notes_body: "Hali ya maelezo (0): weka alama kandideti.\nMaelezo husafishwa kiotomatiki.",
    help_group_hints: "Vidokezo",
    help_hints_body: "Bonyeza h kupata kidokezo. Seli\nzilizoangaziwa zinaonyesha hatua inayofuata.",
    help_color_given: "Nambari iliyopewa",
    help_color_user: "Ingizo lako",
    help_color_error: "Kosa",
    help_color_cursor: "Seli inayofanya kazi",
    help_color_cross: "Safu/safu wima",
    help_color_box: "Sanduku",
    help_color_scan: "Mechi ya skani",
    help_color_hover: "Kuangalia panya",
    help_color_hint_cause: "Kidokezo: sababu",
    help_color_hint_elim: "Kidokezo: elim.",
    help_color_hint_target: "Kidokezo: lengo",
```

- [ ] Add to **AF** (`pub static AF: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      hulp",
    help_title: "HULP",
    help_section_controls: "Kontroles",
    help_section_rules: "Re\u{eb}ls",
    help_section_colors: "Kleure",
    help_close_hint: "\u{25c4} \u{25ba}  wissel afdeling   ?  sluit",
    help_group_navigation: "Navigasie",
    help_group_quick_nav: "Vinnige navigasie",
    help_quick_nav_body: "Druk Enter om \u{2019}n blok te kies,\ndan 1\u{2013}9 vir die sel.",
    help_group_input: "Invoer",
    help_group_functions: "Funksies",
    help_group_rules: "Sudoku-re\u{eb}ls",
    help_rules_body: "Elke ry, kolom en 3\u{d7}3-blok moet\nsyfers 1\u{2013}9 presies een keer bevat.",
    help_group_notes: "Notas",
    help_notes_body: "Nota-modus (0): merk kandidate.\nNotas word outomaties gevee.",
    help_group_hints: "Leidrade",
    help_hints_body: "Druk h vir \u{2019}n leidraad. Gemerkte\nselle wys die volgende stap.",
    help_color_given: "Gegewe syfer",
    help_color_user: "Jou invoer",
    help_color_error: "Fout",
    help_color_cursor: "Aktiewe sel",
    help_color_cross: "Ry/kolom",
    help_color_box: "Blok",
    help_color_scan: "Skandeermatch",
    help_color_hover: "Muis-sweef",
    help_color_hint_cause: "Leidraad: oorsaak",
    help_color_hint_elim: "Leidraad: elim.",
    help_color_hint_target: "Leidraad: doel",
```

- [ ] Add to **PY** (`pub static PY: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      b\u{101}ngzh\u{f9}",
    help_title: "B\u{100}NGZH\u{d9}",
    help_section_controls: "C\u{101}ozu\u{f2}",
    help_section_rules: "Gu\u{12b}z\u{e9}",
    help_section_colors: "Y\u{e1}ns\u{e8}",
    help_close_hint: "\u{25c4} \u{25ba}  qi\u{113}hu\u{e0}n zh\u{101}ngji\u{e9}   ?  gu\u{101}nb\u{ec}",
    help_group_navigation: "D\u{1ce}oh\u{e1}ng",
    help_group_quick_nav: "Ku\u{e0}is\u{f9} d\u{1ce}oh\u{e1}ng",
    help_quick_nav_body: "\u{c0}n Enter xu\u{e3}nz\u{e9} y\u{12b}g\u{e8} g\u{14d}ngg\u{e9},\nr\u{e1}nh\u{f2}u \u{e0}n 1\u{2013}9 xu\u{e3}nz\u{e9} g\u{e9}z\u{12d}.",
    help_group_input: "Sh\u{16b}r\u{f9}",
    help_group_functions: "G\u{14d}ngn\u{e9}ng",
    help_group_rules: "Sh\u{f9}d\u{fa} gu\u{12b}z\u{e9}",
    help_rules_body: "M\u{11b}i h\u{e1}ng, li\u{e8} h\u{e9} 3\u{d7}3 g\u{14d}ngg\u{e9}\nb\u{ec}x\u{16b} g\u{e8} h\u{e1}n sh\u{f9}z\u{ec} 1\u{2013}9 y\u{12b} c\u{ec}.",
    help_group_notes: "B\u{12d}j\u{ec}",
    help_notes_body: "B\u{12d}j\u{ec} m\u{f3}sh\u{ec} (0): bi\u{101}oj\u{ec} h\u{f2}ux\u{16e}n sh\u{f9}z\u{ec}.\nB\u{12d}j\u{ec} z\u{e0}i ti\u{e1}n r\u{f9} sh\u{f9}z\u{ec} h\u{f2}u z\u{ec}d\u{f2}ng q\u{12b}ngch\u{fa}.",
    help_group_hints: "T\u{ed}sh\u{ec}",
    help_hints_body: "\u{c0}n h hu\u{f2}q\u{16d} t\u{ed}sh\u{ec}. G\u{101}oli\u{e0}ng de g\u{e9}z\u{12d}\nxi\u{1ce}nsh\u{ec} xi\u{e0} y\u{12b} b\u{f9} lu\u{f3}j\u{ed} b\u{f9}zh\u{f2}u.",
    help_color_given: "Y\u{1d0} g\u{11b}i sh\u{f9}z\u{ec}",
    help_color_user: "N\u{1d0} de sh\u{16b}r\u{f9}",
    help_color_error: "Cu\u{f2}w\u{f9}",
    help_color_cursor: "D\u{101}ngqi\u{e1}n g\u{e9}z\u{12d}",
    help_color_cross: "H\u{e1}ng/li\u{e8}",
    help_color_box: "G\u{14d}ngg\u{e9}",
    help_color_scan: "S\u{1ce}omi\u{e1}o p\u{12d}p\u{e8}i",
    help_color_hover: "Sh\u{16d}bi\u{101}o xu\u{e1}nf\u{fa}",
    help_color_hint_cause: "T\u{ed}sh\u{ec}: yu\u{e1}ny\u{12b}n",
    help_color_hint_elim: "T\u{ed}sh\u{ec}: p\u{e1}ich\u{fa}",
    help_color_hint_target: "T\u{ed}sh\u{ec}: m\u{f9}bi\u{101}o",
```

- [ ] Add to **ID** (`pub static ID: Strings`), after `ctrl_mouse`:

```rust
    ctrl_help: "  ?      bantuan",
    help_title: "BANTUAN",
    help_section_controls: "Kontrol",
    help_section_rules: "Aturan",
    help_section_colors: "Warna",
    help_close_hint: "\u{25c4} \u{25ba}  ganti bagian   ?  tutup",
    help_group_navigation: "Navigasi",
    help_group_quick_nav: "Navigasi cepat",
    help_quick_nav_body: "Tekan Enter untuk memilih kotak,\nlalu 1\u{2013}9 untuk sel di dalamnya.",
    help_group_input: "Input",
    help_group_functions: "Fungsi",
    help_group_rules: "Aturan Sudoku",
    help_rules_body: "Setiap baris, kolom, dan kotak 3\u{d7}3 harus\nmengandung angka 1\u{2013}9 tepat satu kali.",
    help_group_notes: "Catatan",
    help_notes_body: "Mode catatan (0): tandai kandidat angka.\nCatatan dihapus otomatis saat angka dimasukkan.",
    help_group_hints: "Petunjuk",
    help_hints_body: "Tekan h untuk petunjuk. Sel yang\ndisorot menunjukkan langkah berikutnya.",
    help_color_given: "Angka yang diberikan",
    help_color_user: "Masukan Anda",
    help_color_error: "Kesalahan",
    help_color_cursor: "Sel aktif",
    help_color_cross: "Baris/kolom",
    help_color_box: "Kotak",
    help_color_scan: "Kecocokan pindai",
    help_color_hover: "Hover mouse",
    help_color_hint_cause: "Petunjuk: penyebab",
    help_color_hint_elim: "Petunjuk: elim.",
    help_color_hint_target: "Petunjuk: target",
```

### Step 1.4: Verify build compiles cleanly

- [ ] Run:

```
cargo build 2>&1 | grep -E "^error" | head -20
```

Expected: no errors. Fix any missing-field errors before proceeding.

### Step 1.5: Run test suite to confirm no regressions

- [ ] Run:

```
cargo test 2>&1 | tail -5
```

Expected: all tests pass, 0 failures.

### Step 1.6: Commit

- [ ] Commit:

```bash
git add src/i18n/mod.rs
git commit -m "feat(i18n): add 28 help-screen string fields for all 13 languages"
```

---

## Task 2: AppAction::ToggleHelp in `src/tui/input.rs`

**Files:**
- Modify: `src/tui/input.rs`

### Step 2.1: Add `AppAction::ToggleHelp` variant

- [ ] In `src/tui/input.rs`, find the `AppAction` enum (line ~30). Add `ToggleHelp` before `None`:

```rust
    /// `?` key: open/close help screen.
    ToggleHelp,
    None,
```

### Step 2.2: Add `'?' => AppAction::ToggleHelp` in non-remappable match

- [ ] In `map_key_to_action`, find the inner `match c {` block (line ~135). Add before `_ => AppAction::None`:

```rust
                '?' => AppAction::ToggleHelp,
                _ => AppAction::None,
```

### Step 2.3: Build to check for exhaustiveness warnings

- [ ] Run:

```
cargo build 2>&1 | grep -E "warning.*ToggleHelp|error" | head -20
```

Expect: warnings that `ToggleHelp` is unmatched in `handle_action` match arms — this is expected and will be fixed in Task 3. No hard errors.

### Step 2.4: Commit

- [ ] Commit:

```bash
git add src/tui/input.rs
git commit -m "feat(input): add AppAction::ToggleHelp for ? key"
```

---

## Task 3: AppScreen::Help, toggle_help(), handle_help_action(), run() intercept — `src/tui/mod.rs`

**Files:**
- Modify: `src/tui/mod.rs`

### Step 3.1: Add `AppScreen::Help { section: usize }` to the enum

- [ ] In `src/tui/mod.rs`, find the `AppScreen` enum (line ~42). Add after `Generating`:

```rust
    Help { section: usize },
```

### Step 3.2: Add `toggle_help()` private helper

- [ ] Find a convenient `impl App` block (near other small helpers — e.g., after `toggle_digit_style`). Add:

```rust
fn toggle_help(&mut self) {
    if matches!(self.screen, AppScreen::Help { .. }) {
        self.screen = if self.game_state.is_some() {
            AppScreen::Game
        } else {
            AppScreen::Start { selected: 0 }
        };
    } else if matches!(self.screen, AppScreen::Start { .. } | AppScreen::Game) {
        self.screen = AppScreen::Help { section: 0 };
    }
    self.needs_clear = true;
}
```

### Step 3.3: Add `handle_help_action()` private method

- [ ] In `src/tui/mod.rs`, add near the other `handle_*_action` methods:

```rust
fn handle_help_action(&mut self, action: AppAction, section: usize) {
    match action {
        AppAction::ToggleHelp | AppAction::Back => self.toggle_help(),
        AppAction::MoveLeft => {
            self.screen = AppScreen::Help {
                section: if section == 0 { 2 } else { section - 1 },
            };
        }
        AppAction::MoveRight => {
            self.screen = AppScreen::Help {
                section: (section + 1) % 3,
            };
        }
        _ => {}
    }
}
```

### Step 3.4: Intercept `ToggleHelp` in `run()` before hint/overlay dismissal

- [ ] In `src/tui/mod.rs`, find `run()`. Locate the key-press branch (around line 1312). The structure is:

```rust
if self.active_hint.is_some() {          // ← line ~1313: dismisses hint
    ...
} else if self.hint_warning.is_some() {  // ← dismisses warning
    ...
} else if self.info_overlay.is_some() {  // ← dismisses overlay
    ...
} else {
    // sequence detector, '#' toggle, map_key_to_action, handle_action
}
```

Insert **before the entire if/else chain** (i.e., before line 1313, before `if self.active_hint.is_some()`):

```rust
// `?` opens/closes help regardless of hint/overlay state.
if key.code == crossterm::event::KeyCode::Char('?') {
    self.toggle_help();
    needs_render = true;
    continue;
}
// Active hint: any key dismisses it (key is consumed, not forwarded).
if self.active_hint.is_some() {
```

> **Critical:** The intercept must be OUTSIDE all the hint/overlay branches. If placed inside the `else` branch, pressing `?` while a hint is active would dismiss the hint instead of opening Help.

> The `continue` skips the rest of the key-handling block for this event.

### Step 3.5: Add `AppScreen::Help` arm to `handle_action` dispatch

- [ ] In `src/tui/mod.rs`, find the `match &self.screen {` dispatch in `handle_action` (line ~252). Add:

```rust
AppScreen::Help { section } => {
    let s = *section;
    self.handle_help_action(action, s);
}
```

### Step 3.6: Add `AppScreen::Help` arm to `render_current()`

- [ ] In `src/tui/mod.rs`, find `render_current()` (line ~1456). After the `AppScreen::Generating` arm, add:

```rust
AppScreen::Help { section } => {
    crate::tui::render::help::render_help(out, *section, &self.colors, strings)
}
```

### Step 3.7: Build and verify

- [ ] Run:

```
cargo build 2>&1 | grep -E "^error" | head -20
```

Expected: no errors. The compiler will now require `help::render_help` to exist — that is added in Task 4.

### Step 3.8: Commit (after Task 4 compiles)

> Commit after Task 4 is complete and the build is clean (since Task 3 and Task 4 are coupled at the build level — `render_help` must exist for this to compile).

---

## Task 4: New renderer `src/tui/render/help.rs`

**Files:**
- Create: `src/tui/render/help.rs`
- Modify: `src/tui/render/mod.rs`

### Step 4.1: Add `pub mod help;` to `src/tui/render/mod.rs`

- [ ] In `src/tui/render/mod.rs`, add to the `pub mod` list at the top (after `pub mod boss;`):

```rust
pub mod help;
```

### Step 4.2: Create `src/tui/render/help.rs`

- [ ] Create the file with the following content:

```rust
// src/tui/render/help.rs
use crate::i18n::Strings;
use crate::tui::colors::ColorScheme;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::io::{self, Write};

/// Render the full-screen help overlay.
///
/// `section`: 0 = Controls, 1 = Rules, 2 = Colors.
pub fn render_help(
    out: &mut impl Write,
    section: usize,
    colors: &ColorScheme,
    strings: &Strings,
) -> io::Result<()> {
    let (cols, rows) = terminal::size().unwrap_or((117, 39));
    let width = cols.min(117) as usize;

    let bg = colors.ui_background;
    let fg = colors.ui_text;
    let dim = colors.ui_text_dim;
    let tab_bg = colors.ui_cursor_bg;
    let tab_fg = colors.ui_cursor_fg;

    // Helper: print a full-width line at (row, 0).
    // `content` is the inner text; padded / bordered to `width`.
    let inner = width.saturating_sub(2); // space between ║ borders

    // ── Title bar ────────────────────────────────────────────────────────────
    let title = strings.help_title;
    let title_pad = inner.saturating_sub(title.len()) / 2;
    queue!(
        out,
        MoveTo(0, 0),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(format!(
            "\u{2554}{}\u{2557}",
            "\u{2550}".repeat(width.saturating_sub(2))
        )),
        MoveTo(0, 1),
        Print(format!(
            "\u{2551}{}{}{}\u{2551}",
            " ".repeat(title_pad),
            title,
            " ".repeat(inner.saturating_sub(title_pad + title.len()))
        )),
        MoveTo(0, 2),
        Print(format!(
            "\u{2560}{}\u{2563}",
            "\u{2550}".repeat(width.saturating_sub(2))
        )),
    )?;

    // ── Tab bar ───────────────────────────────────────────────────────────────
    let tabs = [
        strings.help_section_controls,
        strings.help_section_rules,
        strings.help_section_colors,
    ];
    queue!(out, MoveTo(0, 3), SetBackgroundColor(bg), Print("\u{2551}"))?;
    for (i, tab) in tabs.iter().enumerate() {
        if i == section {
            queue!(
                out,
                SetBackgroundColor(tab_bg),
                SetForegroundColor(tab_fg),
                Print(format!(" [ {} ] ", tab)),
                SetBackgroundColor(bg),
                SetForegroundColor(dim),
            )?;
        } else {
            queue!(
                out,
                SetBackgroundColor(bg),
                SetForegroundColor(dim),
                Print(format!("  {}  ", tab)),
            )?;
        }
    }
    // Pad remainder of tab bar.
    let tab_used: usize = tabs
        .iter()
        .enumerate()
        .map(|(i, t)| if i == section { t.len() + 7 } else { t.len() + 4 })
        .sum::<usize>()
        + 1; // leading ║
    let tab_pad = width.saturating_sub(tab_used + 1);
    queue!(
        out,
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(" ".repeat(tab_pad)),
        Print("\u{2551}"),
    )?;
    queue!(
        out,
        MoveTo(0, 4),
        SetForegroundColor(fg),
        Print(format!(
            "\u{2560}{}\u{2563}",
            "\u{2550}".repeat(width.saturating_sub(2))
        )),
    )?;

    // ── Section content ───────────────────────────────────────────────────────
    let content_rows: Vec<String> = match section {
        0 => render_controls_lines(strings, colors, inner),
        1 => render_rules_lines(strings, inner),
        _ => render_colors_lines(strings, colors, inner),
    };

    let content_start_row = 5u16;
    // Available rows: rows - 3 (title+divider) - 2 (tab+divider) - 2 (bottom divider+bar) = rows - 7
    let available = (rows as usize).saturating_sub(8);

    for (i, line) in content_rows.iter().take(available).enumerate() {
        queue!(
            out,
            MoveTo(0, content_start_row + i as u16),
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print("\u{2551} "),
            Print(line),
        )?;
        // Pad to width.
        let printed = 2 + display_len(line);
        if printed < width.saturating_sub(1) {
            let pad = width.saturating_sub(1).saturating_sub(printed);
            queue!(out, Print(" ".repeat(pad)))?;
        }
        queue!(out, SetForegroundColor(fg), Print("\u{2551}"))?;
    }
    // Fill remaining rows with blank lines.
    for i in content_rows.len()..available {
        queue!(
            out,
            MoveTo(0, content_start_row + i as u16),
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print(format!(
                "\u{2551}{}\u{2551}",
                " ".repeat(width.saturating_sub(2))
            )),
        )?;
    }

    // ── Bottom bar ────────────────────────────────────────────────────────────
    let bottom_row = content_start_row + available as u16;
    queue!(
        out,
        MoveTo(0, bottom_row),
        SetForegroundColor(fg),
        Print(format!(
            "\u{2560}{}\u{2563}",
            "\u{2550}".repeat(width.saturating_sub(2))
        )),
        MoveTo(0, bottom_row + 1),
        SetForegroundColor(dim),
        Print("\u{2551} "),
        Print(strings.help_close_hint),
    )?;
    let hint_used = 2 + strings.help_close_hint.len();
    let hint_pad = width.saturating_sub(hint_used + 1);
    queue!(
        out,
        Print(" ".repeat(hint_pad)),
        SetForegroundColor(fg),
        Print("\u{2551}"),
        MoveTo(0, bottom_row + 2),
        Print(format!(
            "\u{255a}{}\u{255d}",
            "\u{2550}".repeat(width.saturating_sub(2))
        )),
    )?;

    queue!(out, ResetColor)?;
    Ok(())
}

// ── Section renderers ─────────────────────────────────────────────────────────

fn render_controls_lines(strings: &Strings, _colors: &ColorScheme, _inner: usize) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    v.push(String::new());
    push_group(&mut v, strings.help_group_navigation);
    v.push(format!("  \u{2191}\u{2193}\u{2190}\u{2192}     {}", strings.ctrl_move.trim_start()));
    v.push(format!("  Enter     {}", strings.ctrl_goto.trim_start()));
    v.push(format!("  1\u{2013}9       {}", strings.ctrl_digit.trim_start()));
    v.push(String::new());
    push_group(&mut v, strings.help_group_quick_nav);
    for line in strings.help_quick_nav_body.split('\n') {
        v.push(format!("  {}", line));
    }
    v.push(String::new());
    push_group(&mut v, strings.help_group_input);
    v.push(format!("  1\u{2013}9       {}", strings.ctrl_digit.trim_start()));
    v.push(format!("  0         {}", strings.ctrl_mode.trim_start()));
    v.push(format!("  -         {}", strings.ctrl_clear.trim_start()));
    v.push(format!("  u / r     {} / {}", strings.ctrl_undo.trim_start(), strings.ctrl_redo.trim_start()));
    v.push(String::new());
    push_group(&mut v, strings.help_group_functions);
    v.push(format!("  h  {}   s  {}", strings.ctrl_hint.trim_start(), strings.ctrl_scan.trim_start()));
    v.push(format!("  e  {}   Space  {}", strings.ctrl_errors.trim_start(), strings.ctrl_pause.trim_start()));
    v.push(format!("  m  {}   b  {}", strings.ctrl_mouse.trim_start(), strings.ctrl_boss.trim_start()));
    v.push(format!("  Esc  {}   ?  {}", strings.ctrl_quit.trim_start(), strings.ctrl_help.trim_start()));
    v.push(String::new());
    v
}

fn render_rules_lines(strings: &Strings, _inner: usize) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    v.push(String::new());
    push_group(&mut v, strings.help_group_rules);
    for line in strings.help_rules_body.split('\n') {
        v.push(format!("  {}", line));
    }
    v.push(String::new());
    push_group(&mut v, strings.help_group_notes);
    for line in strings.help_notes_body.split('\n') {
        v.push(format!("  {}", line));
    }
    v.push(String::new());
    push_group(&mut v, strings.help_group_hints);
    for line in strings.help_hints_body.split('\n') {
        v.push(format!("  {}", line));
    }
    v.push(String::new());
    v
}

fn render_colors_lines(strings: &Strings, colors: &ColorScheme, _inner: usize) -> Vec<String> {
    // Returns lines as plain Strings — caller queues them; we embed ANSI via a
    // separate function that writes directly.  Because our line-based helper
    // can't carry color state, we return a sentinel and handle color swatches
    // in a post-pass.  Instead, we use a simpler approach: produce pre-formatted
    // strings with embedded escape sequences via a helper that returns
    // Vec<(swatch_fg, swatch_bg, swatch_char, label)>.
    // The actual rendering is done separately; this function is a no-op stub.
    // See render_colors_section() which is called directly.
    let _ = (strings, colors);
    vec![] // handled by render_colors_section in render_help
}

/// Color swatch entry: (foreground, background, swatch char '█' or '▐', label).
type Swatch = (Color, Color, char, &'static str);

fn color_swatches<'a>(strings: &'a Strings, colors: &ColorScheme) -> Vec<Swatch> {
    vec![
        (colors.digit_given,       colors.cell_normal_bg,      '\u{2588}', strings.help_color_given),
        (colors.digit_user,        colors.cell_normal_bg,      '\u{2588}', strings.help_color_user),
        (colors.digit_error,       colors.cell_normal_bg,      '\u{2588}', strings.help_color_error),
        (colors.ui_cursor_fg,      colors.ui_cursor_bg,        '\u{2588}', strings.help_color_cursor),
        (colors.digit_user,        colors.cell_active_cross_bg,'\u{2588}', strings.help_color_cross),
        (colors.digit_user,        colors.cell_active_box_bg,  '\u{2588}', strings.help_color_box),
        (colors.digit_scan,        colors.cell_normal_bg,      '\u{2588}', strings.help_color_scan),
        (colors.digit_user,        colors.hover_bg,            '\u{2588}', strings.help_color_hover),
        (colors.hint_cause_border, colors.cell_normal_bg,      '\u{2590}', strings.help_color_hint_cause),
        (colors.hint_elim_border,  colors.cell_normal_bg,      '\u{2590}', strings.help_color_hint_elim),
        (colors.digit_user,        colors.hint_target_bg,      '\u{2588}', strings.help_color_hint_target),
    ]
}

fn push_group(v: &mut Vec<String>, label: &str) {
    v.push(format!("  \u{2500}\u{2500} {} {}", label, "\u{2500}".repeat(30_usize.saturating_sub(label.len() + 4))));
}

/// Approximate display width: counts chars (ignores multi-width CJK for now).
fn display_len(s: &str) -> usize {
    s.chars().count()
}
```

Because the Colors section requires interleaved color commands, we need to restructure `render_help` slightly for that section. Replace the `render_colors_lines` stub approach with a direct render path. The complete `render_help` function handles the colors section differently:

- [ ] Update `render_help` to call `render_colors_section` directly for section 2 instead of going through the line-based path. Replace the `content_rows` block with:

```rust
    // ── Section content ───────────────────────────────────────────────────────
    let content_start_row = 5u16;
    let available = (rows as usize).saturating_sub(8);

    if section == 2 {
        render_colors_section(out, strings, colors, bg, fg, dim, width, content_start_row, available)?;
    } else {
        let content_rows: Vec<String> = match section {
            0 => render_controls_lines(strings, colors, inner),
            _ => render_rules_lines(strings, inner),
        };
        for (i, line) in content_rows.iter().take(available).enumerate() {
            queue!(
                out,
                MoveTo(0, content_start_row + i as u16),
                SetBackgroundColor(bg),
                SetForegroundColor(fg),
                Print("\u{2551} "),
                Print(line),
            )?;
            let printed = 2 + display_len(line);
            if printed < width.saturating_sub(1) {
                let pad = width.saturating_sub(1).saturating_sub(printed);
                queue!(out, Print(" ".repeat(pad)))?;
            }
            queue!(out, SetForegroundColor(fg), Print("\u{2551}"))?;
        }
        for i in content_rows.len()..available {
            queue!(
                out,
                MoveTo(0, content_start_row + i as u16),
                SetBackgroundColor(bg),
                SetForegroundColor(fg),
                Print(format!(
                    "\u{2551}{}\u{2551}",
                    " ".repeat(width.saturating_sub(2))
                )),
            )?;
        }
    }
```

- [ ] Add the `render_colors_section` function to `src/tui/render/help.rs`:

```rust
fn render_colors_section(
    out: &mut impl Write,
    strings: &Strings,
    colors: &ColorScheme,
    bg: Color,
    fg: Color,
    _dim: Color,
    width: usize,
    start_row: u16,
    available: usize,
) -> io::Result<()> {
    let swatches = color_swatches(strings, colors);
    // Render pairs side-by-side: (swatch0, label0)   (swatch1, label1)
    let mut row_idx = 0usize;

    // Blank line before first entry.
    queue!(
        out,
        MoveTo(0, start_row + row_idx as u16),
        SetBackgroundColor(bg),
        SetForegroundColor(fg),
        Print(format!("\u{2551}{}\u{2551}", " ".repeat(width.saturating_sub(2)))),
    )?;
    row_idx += 1;

    let mut i = 0;
    while i < swatches.len() && row_idx < available {
        let (fg0, bg0, ch0, label0) = swatches[i];
        queue!(
            out,
            MoveTo(0, start_row + row_idx as u16),
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print("\u{2551}  "),
            SetForegroundColor(fg0),
            SetBackgroundColor(bg0),
            Print(ch0),
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print(format!("  {:<20}", label0)),
        )?;

        if i + 1 < swatches.len() {
            let (fg1, bg1, ch1, label1) = swatches[i + 1];
            queue!(
                out,
                SetForegroundColor(fg1),
                SetBackgroundColor(bg1),
                Print(ch1),
                SetBackgroundColor(bg),
                SetForegroundColor(fg),
                Print(format!("  {}", label1)),
            )?;
            // Pad to width.
            let used = 3 + 1 + 2 + 20 + 1 + 2 + label1.len();
            if used < width.saturating_sub(1) {
                queue!(out, Print(" ".repeat(width.saturating_sub(1).saturating_sub(used))))?;
            }
        } else {
            // Odd last entry — pad rest of line.
            let used = 3 + 1 + 2 + 20;
            queue!(out, Print(" ".repeat(width.saturating_sub(1).saturating_sub(used))))?;
        }
        queue!(out, SetForegroundColor(fg), Print("\u{2551}"))?;
        row_idx += 1;
        i += 2;
    }

    // Fill remaining rows.
    while row_idx < available {
        queue!(
            out,
            MoveTo(0, start_row + row_idx as u16),
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print(format!("\u{2551}{}\u{2551}", " ".repeat(width.saturating_sub(2)))),
        )?;
        row_idx += 1;
    }
    Ok(())
}
```

### Step 4.3: Build — Tasks 3 and 4 combined must compile clean

- [ ] Run:

```
cargo build 2>&1 | grep -E "^error" | head -20
```

Expected: no errors. Fix any type errors or missing imports before committing.

### Step 4.4: Run full test suite

- [ ] Run:

```
cargo test 2>&1 | tail -5
```

Expected: all tests pass.

### Step 4.5: Commit Tasks 3 + 4 together

- [ ] Commit:

```bash
git add src/tui/mod.rs src/tui/render/mod.rs src/tui/render/help.rs
git commit -m "feat(help): add Help screen (AppScreen::Help, toggle_help, render_help)"
```

---

## Task 5: Panel hint — `ctrl_help` in status bar

**Files:**
- Modify: `src/tui/render/status_bar.rs`

### Step 5.1: Add `ctrl_help` to controls list

- [ ] In `src/tui/render/status_bar.rs`, find the controls list (around line 120). After the `ctrl_quit` entry (line ~136), add:

```rust
            (strings.ctrl_help.into(), d, false),
```

The list should now end:

```rust
            (strings.ctrl_boss.into(), d, false),
            (strings.ctrl_quit.into(), d, false),
            (strings.ctrl_help.into(), d, false),
        ]);
```

### Step 5.2: Build

- [ ] Run:

```
cargo build 2>&1 | grep -E "^error" | head -10
```

Expected: clean.

### Step 5.3: Commit

- [ ] Commit:

```bash
git add src/tui/render/status_bar.rs
git commit -m "feat(status_bar): add ctrl_help entry to controls list"
```

---

## Task 6: Final verification

### Step 6.1: Full test suite

- [ ] Run:

```
cargo test 2>&1 | tail -10
```

Expected: all tests pass, 0 failures.

### Step 6.2: Release build

- [ ] Run:

```
cargo build --release 2>&1 | grep -E "^error" | head -10
```

Expected: clean.

### Step 6.3: Smoke test — press `?` in both Start and Game screens

- [ ] Launch the binary: `cargo run --release`
- [ ] From the Start screen, press `?` → Help screen opens on Controls section
- [ ] Press `►` → Rules section, press `►` → Colors section, press `►` → wraps to Controls
- [ ] Press `◄` → Colors, press `?` → returns to Start screen
- [ ] Start a game, press `?` → Help opens, press `Esc` → returns to Game
- [ ] From Game, trigger a hint with `h`, then press `?` → Help opens (hint is NOT just dismissed)
- [ ] Check Colors section: 11 swatches visible with correct colors for current theme

### Step 6.4: Final commit (if needed)

- [ ] If Step 6.3 revealed rendering bugs, fix them and commit with `fix(help): …`.

---

## Key reference

**`src/tui/input.rs`:** `AppAction::ToggleHelp` before `None` (line ~69); `'?' => AppAction::ToggleHelp` before `_ => AppAction::None` (line ~152).

**`src/tui/mod.rs`:** `AppScreen::Help { section: usize }` after `Generating` (line ~49); `toggle_help()` sets `needs_clear = true`; intercept in `run()` before line 1313 hint-dismiss block using `continue`; `handle_help_action` dispatch in `handle_action` match (line ~265); `render_current()` arm (line ~1530).

**`src/tui/render/help.rs`:** `render_help(out, section, colors, strings)` — section 0/1 use line-based rendering, section 2 uses `render_colors_section` with direct crossterm color commands. Colors section renders swatches in pairs; `▐` (`\u{2590}`) for hint border swatches, `█` (`\u{2588}`) for all others.

**ANSI-only rule:** All colors must use named `crossterm::style::Color` variants. `Color::Rgb` and `Color::AnsiValue` are forbidden.
