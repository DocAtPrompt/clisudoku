# Help Screen — Design Spec

## Goal

Add a full-screen Help overlay accessible via `?` from the Start screen and the
Game screen. The screen is organised in three sections (Controls, Rules, Colors)
navigable with ◄/►. All text is fully i18n-translated across all 13 languages.

---

## Trigger & Navigation

| Key | Effect |
|-----|--------|
| `?` | Open Help (from Start or Game); close Help (from Help) |
| `Esc` | Close Help (same as `?`) |
| `◄` / `►` | Previous / next section (wraps around) |

`?` is **not remappable** — it is handled in the non-remappable `match c` branch
of `map_key_to_action`, alongside `b` (BossKey):

```rust
'?' => AppAction::ToggleHelp,
```

This arm must be inserted before the catch-all `_ => AppAction::None`.

### Priority in the event loop

`ToggleHelp` must be intercepted in `App::run()` **before** the hint/overlay
dismissal block, so that pressing `?` while a hint is active opens Help rather
than dismissing the hint. Concretely, in `run()`:

```rust
// Before the hint-dismiss / overlay-dismiss block:
if let AppAction::ToggleHelp = action {
    return self.toggle_help();  // or inline the logic
}
// … existing hint/overlay dismissal …
self.handle_action(action);
```

`toggle_help()` is a private helper on `App` (see Architecture below).

Help can only be opened from `AppScreen::Start` and `AppScreen::Game`. Opening
from any other screen (DifficultySelect, PatternSelect, etc.) is a no-op.

---

## AppScreen Variant

```rust
AppScreen::Help { section: usize }
```

`section`: 0 = Controls, 1 = Rules, 2 = Colors.

---

## AppAction

```rust
AppAction::ToggleHelp
```

### toggle_help() helper

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
}
```

When closing, the return destination is inferred from `game_state.is_some()` —
no extra field needed on App.

### handle_help_action

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

`handle_help_action` is added to the `match &self.screen` dispatch in
`handle_action`, alongside all other per-screen handlers.

---

## Screen Layout

Full-screen render using the current ColorScheme. The screen adapts to terminal
width; the global minimum-size guard (already present) ensures the terminal is
large enough before Help is ever rendered.

```
╔══════════════════════════════════════════════════════╗
║                        HELP                          ║
╠══════════════════════════════════════════════════════╣
║  [ Controls ]   Rules   Colors                       ║
╠══════════════════════════════════════════════════════╣
║                                                      ║
║  ── Navigation ──────────────────────────────────── ║
║  ↑↓←→    move cursor                                ║
║  Enter    select box (numpad mode)                   ║
║  1–9      select cell within box                     ║
║                                                      ║
║  ── Quick Navigation ────────────────────────────── ║
║  Press Enter to select one of 9 boxes,              ║
║  then 1–9 to place the cursor in that cell.         ║
║                                                      ║
║  ── Input ────────────────────────────────────────  ║
║  1–9      enter digit                                ║
║  0        toggle note mode                           ║
║  -        clear cell                                 ║
║  u / r    undo / redo                                ║
║                                                      ║
║  ── Functions ────────────────────────────────────  ║
║  h        hint             s    scan mode            ║
║  e        show errors      Space pause               ║
║  m        mouse toggle     b    boss key             ║
║  Esc      quit to start    ?    this screen          ║
║                                                      ║
╠══════════════════════════════════════════════════════╣
║  ◄ ►  switch section                 ?  close        ║
╚══════════════════════════════════════════════════════╝
```

Active section tab is highlighted (`ui_cursor_bg` / `ui_cursor_fg`). Inactive
tabs are dim (`ui_text_dim`). The bottom bar uses `ui_text_dim`.

The Controls section reuses existing `ctrl_*` strings (already translated for
all 13 languages) for the key descriptions.

### Section: Rules

```
╠══════════════════════════════════════════════════════╣
║  Controls   [ Rules ]   Colors                       ║
╠══════════════════════════════════════════════════════╣
║                                                      ║
║  ── Sudoku Rules ─────────────────────────────────  ║
║  Each row, column and 3×3 box must contain          ║
║  digits 1–9 exactly once.                           ║
║                                                      ║
║  ── Notes ────────────────────────────────────────  ║
║  Note mode (0): mark candidate digits in a cell.    ║
║  Notes are cleared automatically when a digit       ║
║  is entered in the same row, column, or box.        ║
║                                                      ║
║  ── Hints ────────────────────────────────────────  ║
║  Press h to request a hint. The hint highlights     ║
║  the cells involved in the next logical step.       ║
║                                                      ║
╠══════════════════════════════════════════════════════╣
║  ◄ ►  switch section                 ?  close        ║
╚══════════════════════════════════════════════════════╝
```

### Section: Colors

Each entry shows a filled block `█` in the foreground color on the relevant
background, followed by a label. For hint cause and hint elimination — whose
ColorScheme fields are border colors, not backgrounds — use `▐` (right half
block) in the border color on `cell_normal_bg`, so the swatch still conveys the
actual color without requiring a full cell border.

Colors are read from `app.colors` (current ColorScheme) so they remain accurate
regardless of active theme.

```
╠══════════════════════════════════════════════════════╣
║  Controls   Rules   [ Colors ]                       ║
╠══════════════════════════════════════════════════════╣
║                                                      ║
║  █  Given digit          █  Your entry              ║
║  █  Error                █  Active cell (cursor)     ║
║  █  Row/col highlight    █  Box highlight            ║
║  █  Scan match           █  Mouse hover              ║
║  ▐  Hint: cause          ▐  Hint: elimination        ║
║  █  Hint: target                                     ║
║                                                      ║
╠══════════════════════════════════════════════════════╣
║  ◄ ►  switch section                 ?  close        ║
╚══════════════════════════════════════════════════════╝
```

Color mapping:

| Label | Swatch char | Foreground | Background |
|---|---|---|---|
| Given digit | `█` | `digit_given` | `cell_normal_bg` |
| Your entry | `█` | `digit_user` | `cell_normal_bg` |
| Error | `█` | `digit_error` | `cell_normal_bg` |
| Active cell | `█` | `ui_cursor_fg` | `ui_cursor_bg` |
| Row/col highlight | `█` | `digit_user` | `cell_active_cross_bg` |
| Box highlight | `█` | `digit_user` | `cell_active_box_bg` |
| Scan match | `█` | `digit_scan` | `cell_normal_bg` |
| Mouse hover | `█` | `digit_user` | `hover_bg` |
| Hint: cause | `▐` | `hint_cause_border` | `cell_normal_bg` |
| Hint: elimination | `▐` | `hint_elim_border` | `cell_normal_bg` |
| Hint: target | `█` | `digit_user` | `hint_target_bg` |

---

## Panel Hint

In `src/tui/render/status_bar.rs`, the controls list gains one new entry at the
end using the new `strings.ctrl_help` i18n string.

---

## i18n — New Strings (28 fields)

28 new fields added to the `Strings` struct in `src/i18n/mod.rs`.
All 13 project languages provided: EN, DE, ES, IT, FR, SL, EO, TP, LEET, SW, AF, PY, ID.

### Master table (EN / DE)

| Field | EN | DE |
|---|---|---|
| `ctrl_help` | `  ?      help` | `  ?      Hilfe` |
| `help_title` | `HELP` | `HILFE` |
| `help_section_controls` | `Controls` | `Steuerung` |
| `help_section_rules` | `Rules` | `Regeln` |
| `help_section_colors` | `Colors` | `Farben` |
| `help_close_hint` | `◄ ►  switch section   ?  close` | `◄ ►  Abschnitt wechseln   ?  schließen` |
| `help_group_navigation` | `Navigation` | `Navigation` |
| `help_group_quick_nav` | `Quick Navigation` | `Schnellsteuerung` |
| `help_quick_nav_body` | `Press Enter to select one of 9 boxes,\nthen 1–9 to place the cursor in that cell.` | `Enter wählt eine der 9 Boxen,\ndann 1–9 für die Zelle darin.` |
| `help_group_input` | `Input` | `Eingabe` |
| `help_group_functions` | `Functions` | `Funktionen` |
| `help_group_rules` | `Sudoku Rules` | `Spielregeln` |
| `help_rules_body` | `Each row, column and 3×3 box must contain\ndigits 1–9 exactly once.` | `Jede Zeile, Spalte und 3×3-Box muss\ndie Ziffern 1–9 genau einmal enthalten.` |
| `help_group_notes` | `Notes` | `Notizen` |
| `help_notes_body` | `Note mode (0): mark candidate digits.\nNotes clear when a digit is placed nearby.` | `Notizmodus (0): Kandidaten markieren.\nNotizen werden beim Eintragen gecleart.` |
| `help_group_hints` | `Hints` | `Hinweise` |
| `help_hints_body` | `Press h to request a hint. Highlighted\ncells show the next logical solving step.` | `h für einen Hinweis. Markierte Zellen\nzeigen den nächsten logischen Schritt.` |
| `help_color_given` | `Given digit` | `Vorgegebene Ziffer` |
| `help_color_user` | `Your entry` | `Eigene Eingabe` |
| `help_color_error` | `Error` | `Fehler` |
| `help_color_cursor` | `Active cell` | `Aktive Zelle` |
| `help_color_cross` | `Row/col highlight` | `Zeile/Spalte` |
| `help_color_box` | `Box highlight` | `Box-Highlight` |
| `help_color_scan` | `Scan match` | `Scan-Treffer` |
| `help_color_hover` | `Mouse hover` | `Maus-Hover` |
| `help_color_hint_cause` | `Hint: cause` | `Hinweis: Ursache` |
| `help_color_hint_elim` | `Hint: elimination` | `Hinweis: Elimination` |
| `help_color_hint_target` | `Hint: target` | `Hinweis: Ziel` |

### ES (Español)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      ayuda` |
| `help_title` | `AYUDA` |
| `help_section_controls` | `Controles` |
| `help_section_rules` | `Reglas` |
| `help_section_colors` | `Colores` |
| `help_close_hint` | `◄ ►  cambiar sección   ?  cerrar` |
| `help_group_navigation` | `Navegación` |
| `help_group_quick_nav` | `Navegación rápida` |
| `help_quick_nav_body` | `Pulsa Intro para elegir una caja,\nluego 1–9 para la celda.` |
| `help_group_input` | `Entrada` |
| `help_group_functions` | `Funciones` |
| `help_group_rules` | `Reglas del Sudoku` |
| `help_rules_body` | `Cada fila, columna y caja 3×3 debe\ncontener los dígitos 1–9 exactamente una vez.` |
| `help_group_notes` | `Notas` |
| `help_notes_body` | `Modo notas (0): marcar candidatos.\nLas notas se borran automáticamente.` |
| `help_group_hints` | `Pistas` |
| `help_hints_body` | `Pulsa h para una pista. Las celdas\nresaltadas muestran el siguiente paso.` |
| `help_color_given` | `Dígito dado` |
| `help_color_user` | `Tu entrada` |
| `help_color_error` | `Error` |
| `help_color_cursor` | `Celda activa` |
| `help_color_cross` | `Fila/columna` |
| `help_color_box` | `Caja` |
| `help_color_scan` | `Coincidencia` |
| `help_color_hover` | `Hover ratón` |
| `help_color_hint_cause` | `Pista: causa` |
| `help_color_hint_elim` | `Pista: elim.` |
| `help_color_hint_target` | `Pista: objetivo` |

### IT (Italiano)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      aiuto` |
| `help_title` | `AIUTO` |
| `help_section_controls` | `Controlli` |
| `help_section_rules` | `Regole` |
| `help_section_colors` | `Colori` |
| `help_close_hint` | `◄ ►  cambia sezione   ?  chiudi` |
| `help_group_navigation` | `Navigazione` |
| `help_group_quick_nav` | `Navigazione rapida` |
| `help_quick_nav_body` | `Premi Invio per scegliere un riquadro,\npoi 1–9 per la cella.` |
| `help_group_input` | `Inserimento` |
| `help_group_functions` | `Funzioni` |
| `help_group_rules` | `Regole del Sudoku` |
| `help_rules_body` | `Ogni riga, colonna e riquadro 3×3 deve\ncontenere le cifre 1–9 esattamente una volta.` |
| `help_group_notes` | `Note` |
| `help_notes_body` | `Modalità note (0): segnare i candidati.\nLe note vengono cancellate automaticamente.` |
| `help_group_hints` | `Suggerimenti` |
| `help_hints_body` | `Premi h per un suggerimento. Le celle\nevidenziate mostrano il passo successivo.` |
| `help_color_given` | `Cifra data` |
| `help_color_user` | `Tua voce` |
| `help_color_error` | `Errore` |
| `help_color_cursor` | `Cella attiva` |
| `help_color_cross` | `Riga/colonna` |
| `help_color_box` | `Riquadro` |
| `help_color_scan` | `Corrispondenza` |
| `help_color_hover` | `Hover mouse` |
| `help_color_hint_cause` | `Sugger.: causa` |
| `help_color_hint_elim` | `Sugger.: elim.` |
| `help_color_hint_target` | `Sugger.: target` |

### FR (Français)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      aide` |
| `help_title` | `AIDE` |
| `help_section_controls` | `Contrôles` |
| `help_section_rules` | `Règles` |
| `help_section_colors` | `Couleurs` |
| `help_close_hint` | `◄ ►  changer section   ?  fermer` |
| `help_group_navigation` | `Navigation` |
| `help_group_quick_nav` | `Navigation rapide` |
| `help_quick_nav_body` | `Appuyez Entrée pour choisir une boîte,\npuis 1–9 pour la cellule.` |
| `help_group_input` | `Saisie` |
| `help_group_functions` | `Fonctions` |
| `help_group_rules` | `Règles du Sudoku` |
| `help_rules_body` | `Chaque ligne, colonne et boîte 3×3 doit\ncontenir les chiffres 1–9 une seule fois.` |
| `help_group_notes` | `Notes` |
| `help_notes_body` | `Mode notes (0): marquer les candidats.\nLes notes sont effacées automatiquement.` |
| `help_group_hints` | `Indices` |
| `help_hints_body` | `Appuyez h pour un indice. Les cellules\nmises en évidence montrent l'étape suivante.` |
| `help_color_given` | `Chiffre donné` |
| `help_color_user` | `Votre saisie` |
| `help_color_error` | `Erreur` |
| `help_color_cursor` | `Cellule active` |
| `help_color_cross` | `Ligne/colonne` |
| `help_color_box` | `Boîte` |
| `help_color_scan` | `Scan` |
| `help_color_hover` | `Survol souris` |
| `help_color_hint_cause` | `Indice: cause` |
| `help_color_hint_elim` | `Indice: élim.` |
| `help_color_hint_target` | `Indice: cible` |

### SL (Slovenščina)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      pomoč` |
| `help_title` | `POMOČ` |
| `help_section_controls` | `Upravljanje` |
| `help_section_rules` | `Pravila` |
| `help_section_colors` | `Barve` |
| `help_close_hint` | `◄ ►  menjaj razdelek   ?  zapri` |
| `help_group_navigation` | `Pomikanje` |
| `help_group_quick_nav` | `Hitro pomikanje` |
| `help_quick_nav_body` | `Pritisni Enter za izbiro bloka,\npotem 1–9 za celico.` |
| `help_group_input` | `Vnos` |
| `help_group_functions` | `Funkcije` |
| `help_group_rules` | `Pravila Sudokuja` |
| `help_rules_body` | `Vsaka vrstica, stolpec in blok 3×3 mora\nvsebovati cifre 1–9 natanko enkrat.` |
| `help_group_notes` | `Opombe` |
| `help_notes_body` | `Način opomb (0): označi kandidate.\nOpombe se samodejno izbrišejo.` |
| `help_group_hints` | `Namigi` |
| `help_hints_body` | `Pritisni h za namig. Označene celice\npokažejo naslednji logični korak.` |
| `help_color_given` | `Dana cifra` |
| `help_color_user` | `Tvoj vnos` |
| `help_color_error` | `Napaka` |
| `help_color_cursor` | `Aktivna celica` |
| `help_color_cross` | `Vrstica/stolpec` |
| `help_color_box` | `Blok` |
| `help_color_scan` | `Ujemanje` |
| `help_color_hover` | `Lebdenje miške` |
| `help_color_hint_cause` | `Namig: vzrok` |
| `help_color_hint_elim` | `Namig: elim.` |
| `help_color_hint_target` | `Namig: cilj` |

### EO (Esperanto)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      helpo` |
| `help_title` | `HELPO` |
| `help_section_controls` | `Kontroloj` |
| `help_section_rules` | `Reguloj` |
| `help_section_colors` | `Koloroj` |
| `help_close_hint` | `◄ ►  ŝanĝi sekcion   ?  fermi` |
| `help_group_navigation` | `Navigado` |
| `help_group_quick_nav` | `Rapida navigado` |
| `help_quick_nav_body` | `Premu Enter por elekti blokon,\nposte 1–9 por la ĉelo.` |
| `help_group_input` | `Enigo` |
| `help_group_functions` | `Funkcioj` |
| `help_group_rules` | `Reguloj de Sudoku` |
| `help_rules_body` | `Ĉiu vico, kolumno kaj bloko 3×3 devas\nenhavi ciferojn 1–9 ĝuste unufoje.` |
| `help_group_notes` | `Notoj` |
| `help_notes_body` | `Nota reĝimo (0): marki kandidatojn.\nNotoj foriĝas aŭtomate.` |
| `help_group_hints` | `Sugestoj` |
| `help_hints_body` | `Premu h por sugeston. Elstarigitaj\nĉeloj montras la sekvan paŝon.` |
| `help_color_given` | `Donita cifero` |
| `help_color_user` | `Via enigo` |
| `help_color_error` | `Eraro` |
| `help_color_cursor` | `Aktiva ĉelo` |
| `help_color_cross` | `Vico/kolumno` |
| `help_color_box` | `Bloko` |
| `help_color_scan` | `Kongruo` |
| `help_color_hover` | `Musa ŝvebo` |
| `help_color_hint_cause` | `Sugesto: kaŭzo` |
| `help_color_hint_elim` | `Sugesto: elim.` |
| `help_color_hint_target` | `Sugesto: celo` |

### TP (Toki Pona)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      sona` |
| `help_title` | `SONA` |
| `help_section_controls` | `nasin luka` |
| `help_section_rules` | `lawa` |
| `help_section_colors` | `kule` |
| `help_close_hint` | `◄ ►  ante   ?  pini` |
| `help_group_navigation` | `tawa` |
| `help_group_quick_nav` | `tawa pona` |
| `help_quick_nav_body` | `o kepeken Enter tawa poki,\no kepeken 1–9 tawa seli.` |
| `help_group_input` | `pana` |
| `help_group_functions` | `pali` |
| `help_group_rules` | `lawa pi nanpa` |
| `help_rules_body` | `poki ale li wile jo e nanpa 1–9\nwan taso.` |
| `help_group_notes` | `sitelen` |
| `help_notes_body` | `nasin sitelen (0): o sitelen e nasin.\nsitelen li weka lon tenpo kama.` |
| `help_group_hints` | `sona` |
| `help_hints_body` | `o kepeken h tawa sona.\nseli mute li jo e pali kama.` |
| `help_color_given` | `nanpa lon` |
| `help_color_user` | `nanpa sina` |
| `help_color_error` | `pakala` |
| `help_color_cursor` | `seli ni` |
| `help_color_cross` | `poka/anpa` |
| `help_color_box` | `poki` |
| `help_color_scan` | `sama` |
| `help_color_hover` | `noka soweli` |
| `help_color_hint_cause` | `sona: tan` |
| `help_color_hint_elim` | `sona: weka` |
| `help_color_hint_target` | `sona: pini` |

### LEET (L33tsp34k)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      h3lp` |
| `help_title` | `H3LP` |
| `help_section_controls` | `C0NTR0LZ` |
| `help_section_rules` | `RUL3Z` |
| `help_section_colors` | `C0L0RZ` |
| `help_close_hint` | `◄ ►  sw1tch s3ct10n   ?  cl0s3` |
| `help_group_navigation` | `N4V1G4T10N` |
| `help_group_quick_nav` | `QU1CK N4V` |
| `help_quick_nav_body` | `pr3ss 3nt3r t0 ch00s3 4 b0x,\nth3n 1–9 f0r th3 c3ll.` |
| `help_group_input` | `1NPUT` |
| `help_group_functions` | `FUNCT10NZ` |
| `help_group_rules` | `SUDOKU RUL3Z` |
| `help_rules_body` | `3v3ry r0w, c0lumn 4nd 3x3 b0x\nmu5t c0nt41n 1–9 3x4ctly 0nc3.` |
| `help_group_notes` | `N0T3Z` |
| `help_notes_body` | `n0t3 m0d3 (0): m4rk c4nd1d4t3z.\nn0t3z 4r3 cl34r3d 4ut0m4t1c4lly.` |
| `help_group_hints` | `H1NTZ` |
| `help_hints_body` | `pr3ss h f0r 4 h1nt. h1ghl1ght3d\nc3llz sh0w th3 n3xt st3p.` |
| `help_color_given` | `g1v3n d1g1t` |
| `help_color_user` | `ur 1nput` |
| `help_color_error` | `3rr0r` |
| `help_color_cursor` | `4ct1v3 c3ll` |
| `help_color_cross` | `r0w/c0l` |
| `help_color_box` | `b0x` |
| `help_color_scan` | `sc4n m4tch` |
| `help_color_hover` | `m0us3 h0v3r` |
| `help_color_hint_cause` | `h1nt: c4us3` |
| `help_color_hint_elim` | `h1nt: 3l1m.` |
| `help_color_hint_target` | `h1nt: t4rg3t` |

### SW (Kiswahili)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      msaada` |
| `help_title` | `MSAADA` |
| `help_section_controls` | `Vidhibiti` |
| `help_section_rules` | `Kanuni` |
| `help_section_colors` | `Rangi` |
| `help_close_hint` | `◄ ►  badilisha sehemu   ?  funga` |
| `help_group_navigation` | `Uabiri` |
| `help_group_quick_nav` | `Uabiri wa haraka` |
| `help_quick_nav_body` | `Bonyeza Enter kuchagua sanduku,\nkisha 1–9 kwa seli.` |
| `help_group_input` | `Ingizo` |
| `help_group_functions` | `Vitendo` |
| `help_group_rules` | `Kanuni za Sudoku` |
| `help_rules_body` | `Kila safu, safu wima na sanduku 3×3 lazima\liche na nambari 1–9 mara moja tu.` |
| `help_group_notes` | `Maelezo` |
| `help_notes_body` | `Hali ya maelezo (0): weka alama kandideti.\nMaelezo husafishwa kiotomatiki.` |
| `help_group_hints` | `Vidokezo` |
| `help_hints_body` | `Bonyeza h kupata kidokezo. Seli\nzilizoangaziwa zinaonyesha hatua inayofuata.` |
| `help_color_given` | `Nambari iliyopewa` |
| `help_color_user` | `Ingizo lako` |
| `help_color_error` | `Kosa` |
| `help_color_cursor` | `Seli inayofanya kazi` |
| `help_color_cross` | `Safu/safu wima` |
| `help_color_box` | `Sanduku` |
| `help_color_scan` | `Mechi ya skani` |
| `help_color_hover` | `Kuangalia panya` |
| `help_color_hint_cause` | `Kidokezo: sababu` |
| `help_color_hint_elim` | `Kidokezo: elim.` |
| `help_color_hint_target` | `Kidokezo: lengo` |

### AF (Afrikaans)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      hulp` |
| `help_title` | `HULP` |
| `help_section_controls` | `Kontroles` |
| `help_section_rules` | `Reëls` |
| `help_section_colors` | `Kleure` |
| `help_close_hint` | `◄ ►  wissel afdeling   ?  sluit` |
| `help_group_navigation` | `Navigasie` |
| `help_group_quick_nav` | `Vinnige navigasie` |
| `help_quick_nav_body` | `Druk Enter om 'n blok te kies,\ndan 1–9 vir die sel.` |
| `help_group_input` | `Invoer` |
| `help_group_functions` | `Funksies` |
| `help_group_rules` | `Sudoku-reëls` |
| `help_rules_body` | `Elke ry, kolom en 3×3-blok moet\nsyfers 1–9 presies een keer bevat.` |
| `help_group_notes` | `Notas` |
| `help_notes_body` | `Nota-modus (0): merk kandidate.\nNotas word outomaties gevee.` |
| `help_group_hints` | `Leidrade` |
| `help_hints_body` | `Druk h vir 'n leidraad. Gemerkte\nselle wys die volgende stap.` |
| `help_color_given` | `Gegewe syfer` |
| `help_color_user` | `Jou invoer` |
| `help_color_error` | `Fout` |
| `help_color_cursor` | `Aktiewe sel` |
| `help_color_cross` | `Ry/kolom` |
| `help_color_box` | `Blok` |
| `help_color_scan` | `Skandeermatch` |
| `help_color_hover` | `Muis-sweef` |
| `help_color_hint_cause` | `Leidraad: oorsaak` |
| `help_color_hint_elim` | `Leidraad: elim.` |
| `help_color_hint_target` | `Leidraad: doel` |

### PY (Zhōngwén Pīnyīn)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      bāngzhù` |
| `help_title` | `BĀNGZHÙ` |
| `help_section_controls` | `Cāozuò` |
| `help_section_rules` | `Guīzé` |
| `help_section_colors` | `Yánsè` |
| `help_close_hint` | `◄ ►  qiēhuàn zhāngjié   ?  guānbì` |
| `help_group_navigation` | `Dǎoháng` |
| `help_group_quick_nav` | `Kuàisù dǎoháng` |
| `help_quick_nav_body` | `Àn Enter xuǎnzé yīgè gōnggé,\nránhòu àn 1–9 xuǎnzé gézǐ.` |
| `help_group_input` | `Shūrù` |
| `help_group_functions` | `Gōngnéng` |
| `help_group_rules` | `Shùdú guīzé` |
| `help_rules_body` | `Měi háng, liè hé 3×3 gōnggé\nbìxū gè hán shùzì 1–9 yī cì.` |
| `help_group_notes` | `Bìjì` |
| `help_notes_body` | `Bìjì móshì (0): biāojì hòuxuǎn shùzì.\nBìjì zài tián rù shùzì hòu zìdòng qīngchú.` |
| `help_group_hints` | `Tíshì` |
| `help_hints_body` | `Àn h huòqǔ tíshì. Gāoliàng de gézǐ\nxiǎnshì xià yī bù luójí bùzhòu.` |
| `help_color_given` | `Yǐ gěi shùzì` |
| `help_color_user` | `Nǐ de shūrù` |
| `help_color_error` | `Cuòwù` |
| `help_color_cursor` | `Dāngqián gézǐ` |
| `help_color_cross` | `Háng/liè` |
| `help_color_box` | `Gōnggé` |
| `help_color_scan` | `Sǎomiáo pǐpèi` |
| `help_color_hover` | `Shǔbiāo xuánfú` |
| `help_color_hint_cause` | `Tíshì: yuányīn` |
| `help_color_hint_elim` | `Tíshì: páichú` |
| `help_color_hint_target` | `Tíshì: mùbiāo` |

### ID (Bahasa Indonesia)

| Field | Value |
|---|---|
| `ctrl_help` | `  ?      bantuan` |
| `help_title` | `BANTUAN` |
| `help_section_controls` | `Kontrol` |
| `help_section_rules` | `Aturan` |
| `help_section_colors` | `Warna` |
| `help_close_hint` | `◄ ►  ganti bagian   ?  tutup` |
| `help_group_navigation` | `Navigasi` |
| `help_group_quick_nav` | `Navigasi cepat` |
| `help_quick_nav_body` | `Tekan Enter untuk memilih kotak,\nlalu 1–9 untuk sel di dalamnya.` |
| `help_group_input` | `Input` |
| `help_group_functions` | `Fungsi` |
| `help_group_rules` | `Aturan Sudoku` |
| `help_rules_body` | `Setiap baris, kolom, dan kotak 3×3 harus\nmengandung angka 1–9 tepat satu kali.` |
| `help_group_notes` | `Catatan` |
| `help_notes_body` | `Mode catatan (0): tandai kandidat angka.\nCatatan dihapus otomatis saat angka dimasukkan.` |
| `help_group_hints` | `Petunjuk` |
| `help_hints_body` | `Tekan h untuk petunjuk. Sel yang\ndisorot menunjukkan langkah berikutnya.` |
| `help_color_given` | `Angka yang diberikan` |
| `help_color_user` | `Masukan Anda` |
| `help_color_error` | `Kesalahan` |
| `help_color_cursor` | `Sel aktif` |
| `help_color_cross` | `Baris/kolom` |
| `help_color_box` | `Kotak` |
| `help_color_scan` | `Kecocokan pindai` |
| `help_color_hover` | `Hover mouse` |
| `help_color_hint_cause` | `Petunjuk: penyebab` |
| `help_color_hint_elim` | `Petunjuk: elim.` |
| `help_color_hint_target` | `Petunjuk: target` |

---

## Files Affected

| File | Change |
|------|--------|
| `src/tui/input.rs` | Add `AppAction::ToggleHelp`; add `'?' => AppAction::ToggleHelp` in non-remappable match (before `_ => AppAction::None`) |
| `src/tui/mod.rs` | Add `AppScreen::Help { section: usize }` to enum; add `toggle_help()` private helper; intercept `ToggleHelp` in `run()` before hint/overlay dismissal; add `AppScreen::Help { section }` arm to `handle_action` dispatch calling `handle_help_action`; add `AppScreen::Help { section }` arm to `render_current()` match |
| `src/tui/render/mod.rs` | Add `AppScreen::Help { section }` dispatch calling `render::help::render_help(...)` directly (no separate `Screen` enum variant needed — follow the direct-call pattern) |
| `src/tui/render/help.rs` | New file: `render_help(out, section, colors, strings)` — renders all three sections |
| `src/tui/render/status_bar.rs` | Add `ctrl_help` entry to controls list |
| `src/i18n/mod.rs` | Add 28 new `&'static str` fields to `Strings` struct; provide values for all 13 language constants (EN, DE, ES, IT, FR, SL, EO, TP, LEET, SW, AF, PY, ID) |

---

## Error Handling

No error paths — the help screen is read-only and stateless. Terminal too small:
the existing global minimum-size guard runs before any screen renders; the Help
screen is never rendered below the global minimum.
