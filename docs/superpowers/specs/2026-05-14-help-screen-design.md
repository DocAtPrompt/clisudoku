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
of `map_key_to_action`, alongside `b` (BossKey).

Help can only be opened from `AppScreen::Start` and `AppScreen::Game`. Opening
from any other screen (DifficultySelect, PatternSelect, etc.) is a no-op.

When closing, the app returns to whichever screen it came from:
- If `app.game_state.is_some()` → return to `AppScreen::Game`
- Otherwise → return to `AppScreen::Start { selected: 0 }`

No extra field needed on App — the return destination is inferred at close time.

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

Handled globally at the top of `App::handle_action`, before per-screen dispatch:

```rust
if let AppAction::ToggleHelp = action {
    if matches!(self.screen, AppScreen::Help { .. }) {
        self.screen = if self.game_state.is_some() {
            AppScreen::Game
        } else {
            AppScreen::Start { selected: 0 }
        };
    } else if matches!(self.screen, AppScreen::Start { .. } | AppScreen::Game) {
        self.screen = AppScreen::Help { section: 0 };
    }
    return;
}
```

`AppAction::Back` (Esc) from Help screen is handled in `handle_help_action` —
same logic: return to Game or Start.

`AppAction::MoveLeft` / `MoveRight` in `handle_help_action` cycle sections
(wrapping: 0 → 2 → 1 → 0).

---

## Screen Layout

Full-screen render, using the current ColorScheme. Width adapts to terminal size;
minimum width 60 columns.

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

Active section tab is highlighted (ui_cursor_bg / ui_cursor_fg). Inactive tabs
are dim (ui_text_dim). The bottom bar uses ui_text_dim.

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

Each color entry shows a filled block (█) in the actual color, followed by a
label. Colors are read from `app.colors` (current ColorScheme) so they are
accurate regardless of active theme.

```
╠══════════════════════════════════════════════════════╣
║  Controls   Rules   [ Colors ]                       ║
╠══════════════════════════════════════════════════════╣
║                                                      ║
║  █  Given digit          █  Your entry              ║
║  █  Error                █  Active cell (cursor)     ║
║  █  Row/col highlight    █  Box highlight            ║
║  █  Scan match           █  Mouse hover              ║
║  █  Hint: cause          █  Hint: elimination        ║
║  █  Hint: target                                     ║
║                                                      ║
╠══════════════════════════════════════════════════════╣
║  ◄ ►  switch section                 ?  close        ║
╚══════════════════════════════════════════════════════╝
```

Color mapping:
| Label | ColorScheme field |
|---|---|
| Given digit | `digit_given` fg on `cell_normal_bg` |
| Your entry | `digit_user` fg on `cell_normal_bg` |
| Error | `digit_error` fg on `cell_normal_bg` |
| Active cell | `ui_cursor_fg` fg on `ui_cursor_bg` |
| Row/col highlight | `digit_user` fg on `cell_active_cross_bg` |
| Box highlight | `digit_user` fg on `cell_active_box_bg` |
| Scan match | `digit_scan` fg on `cell_normal_bg` |
| Mouse hover | `digit_user` fg on `hover_bg` |
| Hint: cause | `grid_cell` fg on `cell_normal_bg`, border `hint_cause_border` |
| Hint: elimination | `grid_cell` fg on `cell_normal_bg`, border `hint_elim_border` |
| Hint: target | `digit_user` fg on `hint_target_bg` |

---

## Panel Hint

In `src/tui/render/status_bar.rs`, the controls list gains one new entry at the
end:

```
?  help
```

Uses the new `strings.ctrl_help` i18n string (see below).

---

## i18n — New Strings

19 new fields added to the `Strings` struct in `src/i18n/mod.rs`.
All 13 languages are provided.

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

> **Note:** The Controls section reuses existing `ctrl_*` strings already present
> in all 13 languages. No duplication needed there.

### All 13 Languages — New String Values

All values below follow the same pattern as existing translations in `mod.rs`.

#### FR (Français)
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

#### ES (Español)
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

#### IT (Italiano)
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
| `help_rules_body` | `Ogni riga, colonna e riquadro 3×3 deve\ncontenere i cifre 1–9 esattamente una volta.` |
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

#### PT (Português)
| Field | Value |
|---|---|
| `ctrl_help` | `  ?      ajuda` |
| `help_title` | `AJUDA` |
| `help_section_controls` | `Controles` |
| `help_section_rules` | `Regras` |
| `help_section_colors` | `Cores` |
| `help_close_hint` | `◄ ►  trocar seção   ?  fechar` |
| `help_group_navigation` | `Navegação` |
| `help_group_quick_nav` | `Navegação rápida` |
| `help_quick_nav_body` | `Pressione Enter para escolher uma caixa,\ndepois 1–9 para a célula.` |
| `help_group_input` | `Entrada` |
| `help_group_functions` | `Funções` |
| `help_group_rules` | `Regras do Sudoku` |
| `help_rules_body` | `Cada linha, coluna e caixa 3×3 deve\nconter os dígitos 1–9 exatamente uma vez.` |
| `help_group_notes` | `Notas` |
| `help_notes_body` | `Modo notas (0): marcar candidatos.\nAs notas são apagadas automaticamente.` |
| `help_group_hints` | `Dicas` |
| `help_hints_body` | `Pressione h para uma dica. As células\ndestacadas mostram o próximo passo.` |
| `help_color_given` | `Dígito dado` |
| `help_color_user` | `Sua entrada` |
| `help_color_error` | `Erro` |
| `help_color_cursor` | `Célula ativa` |
| `help_color_cross` | `Linha/coluna` |
| `help_color_box` | `Caixa` |
| `help_color_scan` | `Correspondência` |
| `help_color_hover` | `Hover mouse` |
| `help_color_hint_cause` | `Dica: causa` |
| `help_color_hint_elim` | `Dica: elim.` |
| `help_color_hint_target` | `Dica: alvo` |

#### NL (Nederlands)
| Field | Value |
|---|---|
| `ctrl_help` | `  ?      help` |
| `help_title` | `HELP` |
| `help_section_controls` | `Bediening` |
| `help_section_rules` | `Regels` |
| `help_section_colors` | `Kleuren` |
| `help_close_hint` | `◄ ►  sectie wisselen   ?  sluiten` |
| `help_group_navigation` | `Navigatie` |
| `help_group_quick_nav` | `Snelle navigatie` |
| `help_quick_nav_body` | `Druk Enter om een vak te kiezen,\ndan 1–9 voor de cel.` |
| `help_group_input` | `Invoer` |
| `help_group_functions` | `Functies` |
| `help_group_rules` | `Sudokuregels` |
| `help_rules_body` | `Elke rij, kolom en 3×3-vak moet\nde cijfers 1–9 precies één keer bevatten.` |
| `help_group_notes` | `Notities` |
| `help_notes_body` | `Notitiemodus (0): kandidaten markeren.\nNotities worden automatisch gewist.` |
| `help_group_hints` | `Hints` |
| `help_hints_body` | `Druk h voor een hint. Gemarkeerde cellen\ntonen de volgende logische stap.` |
| `help_color_given` | `Gegeven cijfer` |
| `help_color_user` | `Jouw invoer` |
| `help_color_error` | `Fout` |
| `help_color_cursor` | `Actieve cel` |
| `help_color_cross` | `Rij/kolom` |
| `help_color_box` | `Vak` |
| `help_color_scan` | `Scan-match` |
| `help_color_hover` | `Muishover` |
| `help_color_hint_cause` | `Hint: oorzaak` |
| `help_color_hint_elim` | `Hint: elim.` |
| `help_color_hint_target` | `Hint: doel` |

#### PL (Polski)
| Field | Value |
|---|---|
| `ctrl_help` | `  ?      pomoc` |
| `help_title` | `POMOC` |
| `help_section_controls` | `Sterowanie` |
| `help_section_rules` | `Zasady` |
| `help_section_colors` | `Kolory` |
| `help_close_hint` | `◄ ►  zmień sekcję   ?  zamknij` |
| `help_group_navigation` | `Nawigacja` |
| `help_group_quick_nav` | `Szybka nawigacja` |
| `help_quick_nav_body` | `Naciśnij Enter, aby wybrać pole,\npotem 1–9 dla komórki.` |
| `help_group_input` | `Wprowadzanie` |
| `help_group_functions` | `Funkcje` |
| `help_group_rules` | `Zasady Sudoku` |
| `help_rules_body` | `Każdy wiersz, kolumna i kwadrat 3×3 musi\nzawierać cyfry 1–9 dokładnie jeden raz.` |
| `help_group_notes` | `Notatki` |
| `help_notes_body` | `Tryb notatek (0): zaznaczaj kandydatów.\nNotatki są usuwane automatycznie.` |
| `help_group_hints` | `Podpowiedzi` |
| `help_hints_body` | `Naciśnij h, aby uzyskać podpowiedź.\nPodświetlone komórki pokazują kolejny krok.` |
| `help_color_given` | `Cyfra podana` |
| `help_color_user` | `Twój wpis` |
| `help_color_error` | `Błąd` |
| `help_color_cursor` | `Aktywna kom.` |
| `help_color_cross` | `Wiersz/kolumna` |
| `help_color_box` | `Kwadrat` |
| `help_color_scan` | `Dopasowanie` |
| `help_color_hover` | `Hover myszy` |
| `help_color_hint_cause` | `Podp.: przyczyna` |
| `help_color_hint_elim` | `Podp.: elim.` |
| `help_color_hint_target` | `Podp.: cel` |

#### CS (Čeština)
| Field | Value |
|---|---|
| `ctrl_help` | `  ?      nápověda` |
| `help_title` | `NÁPOVĚDA` |
| `help_section_controls` | `Ovládání` |
| `help_section_rules` | `Pravidla` |
| `help_section_colors` | `Barvy` |
| `help_close_hint` | `◄ ►  přepnout sekci   ?  zavřít` |
| `help_group_navigation` | `Navigace` |
| `help_group_quick_nav` | `Rychlá navigace` |
| `help_quick_nav_body` | `Stiskni Enter pro výběr čtverce,\npak 1–9 pro buňku.` |
| `help_group_input` | `Vstup` |
| `help_group_functions` | `Funkce` |
| `help_group_rules` | `Pravidla Sudoku` |
| `help_rules_body` | `Každý řádek, sloupec a čtverec 3×3 musí\nobsahovat číslice 1–9 právě jednou.` |
| `help_group_notes` | `Poznámky` |
| `help_notes_body` | `Režim poznámek (0): označit kandidáty.\nPoznámky se automaticky mažou.` |
| `help_group_hints` | `Nápovědy` |
| `help_hints_body` | `Stiskni h pro nápovědu. Zvýrazněné\nbuňky ukazují další logický krok.` |
| `help_color_given` | `Zadaná číslice` |
| `help_color_user` | `Tvůj vstup` |
| `help_color_error` | `Chyba` |
| `help_color_cursor` | `Aktivní buňka` |
| `help_color_cross` | `Řádek/sloupec` |
| `help_color_box` | `Čtverec` |
| `help_color_scan` | `Shoda` |
| `help_color_hover` | `Hover myši` |
| `help_color_hint_cause` | `Náp.: příčina` |
| `help_color_hint_elim` | `Náp.: elim.` |
| `help_color_hint_target` | `Náp.: cíl` |

#### RU (Русский)
| Field | Value |
|---|---|
| `ctrl_help` | `  ?      справка` |
| `help_title` | `СПРАВКА` |
| `help_section_controls` | `Управление` |
| `help_section_rules` | `Правила` |
| `help_section_colors` | `Цвета` |
| `help_close_hint` | `◄ ►  сменить раздел   ?  закрыть` |
| `help_group_navigation` | `Навигация` |
| `help_group_quick_nav` | `Быстрая навигация` |
| `help_quick_nav_body` | `Нажмите Enter для выбора блока,\nзатем 1–9 для ячейки.` |
| `help_group_input` | `Ввод` |
| `help_group_functions` | `Функции` |
| `help_group_rules` | `Правила судоку` |
| `help_rules_body` | `Каждая строка, столбец и блок 3×3 должны\nсодержать цифры 1–9 ровно по одному разу.` |
| `help_group_notes` | `Заметки` |
| `help_notes_body` | `Режим заметок (0): отмечать кандидатов.\nЗаметки удаляются автоматически.` |
| `help_group_hints` | `Подсказки` |
| `help_hints_body` | `Нажмите h для подсказки. Выделенные\nячейки показывают следующий шаг.` |
| `help_color_given` | `Данная цифра` |
| `help_color_user` | `Ваш ввод` |
| `help_color_error` | `Ошибка` |
| `help_color_cursor` | `Активная яч.` |
| `help_color_cross` | `Строка/столбец` |
| `help_color_box` | `Блок` |
| `help_color_scan` | `Совпадение` |
| `help_color_hover` | `Наведение` |
| `help_color_hint_cause` | `Подск.: причина` |
| `help_color_hint_elim` | `Подск.: элим.` |
| `help_color_hint_target` | `Подск.: цель` |

#### JA (日本語)
| Field | Value |
|---|---|
| `ctrl_help` | `  ?      ヘルプ` |
| `help_title` | `ヘルプ` |
| `help_section_controls` | `操作` |
| `help_section_rules` | `ルール` |
| `help_section_colors` | `色` |
| `help_close_hint` | `◄ ►  セクション切替   ?  閉じる` |
| `help_group_navigation` | `ナビゲーション` |
| `help_group_quick_nav` | `クイック操作` |
| `help_quick_nav_body` | `Enterでブロックを選択し、\n1–9でそのマスを選択。` |
| `help_group_input` | `入力` |
| `help_group_functions` | `機能` |
| `help_group_rules` | `数独のルール` |
| `help_rules_body` | `各行・列・3×3ブロックに\n1〜9の数字を一度ずつ入れる。` |
| `help_group_notes` | `メモ` |
| `help_notes_body` | `メモモード(0): 候補数字をメモ。\nメモは自動で削除されます。` |
| `help_group_hints` | `ヒント` |
| `help_hints_body` | `hキーでヒントを表示。\n強調されたマスが次のステップ。` |
| `help_color_given` | `与えられた数字` |
| `help_color_user` | `入力した数字` |
| `help_color_error` | `エラー` |
| `help_color_cursor` | `選択中のマス` |
| `help_color_cross` | `行・列` |
| `help_color_box` | `ブロック` |
| `help_color_scan` | `スキャン一致` |
| `help_color_hover` | `マウスホバー` |
| `help_color_hint_cause` | `ヒント: 原因` |
| `help_color_hint_elim` | `ヒント: 除外` |
| `help_color_hint_target` | `ヒント: 対象` |

#### ZH (中文)
| Field | Value |
|---|---|
| `ctrl_help` | `  ?      帮助` |
| `help_title` | `帮助` |
| `help_section_controls` | `操作` |
| `help_section_rules` | `规则` |
| `help_section_colors` | `颜色` |
| `help_close_hint` | `◄ ►  切换章节   ?  关闭` |
| `help_group_navigation` | `导航` |
| `help_group_quick_nav` | `快速导航` |
| `help_quick_nav_body` | `按 Enter 选择一个宫格，\n然后按 1–9 选择其中的格子。` |
| `help_group_input` | `输入` |
| `help_group_functions` | `功能` |
| `help_group_rules` | `数独规则` |
| `help_rules_body` | `每行、每列和每个 3×3 宫格\n必须各包含数字 1–9 一次。` |
| `help_group_notes` | `笔记` |
| `help_notes_body` | `笔记模式 (0): 标记候选数字。\n笔记在填入数字后自动清除。` |
| `help_group_hints` | `提示` |
| `help_hints_body` | `按 h 获取提示。高亮的格子\n显示下一个逻辑步骤。` |
| `help_color_given` | `已给数字` |
| `help_color_user` | `您的输入` |
| `help_color_error` | `错误` |
| `help_color_cursor` | `当前格子` |
| `help_color_cross` | `行/列` |
| `help_color_box` | `宫格` |
| `help_color_scan` | `扫描匹配` |
| `help_color_hover` | `鼠标悬停` |
| `help_color_hint_cause` | `提示: 原因` |
| `help_color_hint_elim` | `提示: 排除` |
| `help_color_hint_target` | `提示: 目标` |

#### KO (한국어)
| Field | Value |
|---|---|
| `ctrl_help` | `  ?      도움말` |
| `help_title` | `도움말` |
| `help_section_controls` | `조작` |
| `help_section_rules` | `규칙` |
| `help_section_colors` | `색상` |
| `help_close_hint` | `◄ ►  섹션 전환   ?  닫기` |
| `help_group_navigation` | `이동` |
| `help_group_quick_nav` | `빠른 이동` |
| `help_quick_nav_body` | `Enter로 상자를 선택하고,\n1–9로 그 안의 칸을 선택합니다.` |
| `help_group_input` | `입력` |
| `help_group_functions` | `기능` |
| `help_group_rules` | `스도쿠 규칙` |
| `help_rules_body` | `각 행, 열, 3×3 상자에\n1–9가 정확히 한 번씩 들어가야 합니다.` |
| `help_group_notes` | `메모` |
| `help_notes_body` | `메모 모드 (0): 후보 숫자 표시.\n메모는 숫자 입력 시 자동 삭제됩니다.` |
| `help_group_hints` | `힌트` |
| `help_hints_body` | `h를 눌러 힌트를 받으세요.\n강조된 칸이 다음 논리 단계를 보여줍니다.` |
| `help_color_given` | `주어진 숫자` |
| `help_color_user` | `내 입력` |
| `help_color_error` | `오류` |
| `help_color_cursor` | `활성 칸` |
| `help_color_cross` | `행/열` |
| `help_color_box` | `상자` |
| `help_color_scan` | `스캔 일치` |
| `help_color_hover` | `마우스 호버` |
| `help_color_hint_cause` | `힌트: 원인` |
| `help_color_hint_elim` | `힌트: 제거` |
| `help_color_hint_target` | `힌트: 목표` |

---

## Files Affected

| File | Change |
|------|--------|
| `src/tui/input.rs` | Add `AppAction::ToggleHelp`; add `'?' => AppAction::ToggleHelp` in non-remappable match |
| `src/tui/mod.rs` | Add `AppScreen::Help { section: usize }`; add `ToggleHelp` global handler; add `handle_help_action` |
| `src/tui/render/mod.rs` | Add `AppScreen::Help` dispatch to `render_help` |
| `src/tui/render/help.rs` | New: full help screen renderer, 3 sections |
| `src/tui/render/status_bar.rs` | Add `ctrl_help` entry to controls list |
| `src/i18n/mod.rs` | Add 27 new string fields to `Strings` struct; provide values for all 13 languages |

---

## Error Handling

No error paths — the help screen is read-only and stateless. Terminal too small
(<60 cols or <20 rows): fall through to the existing "terminal too small" screen
(already handled globally).
