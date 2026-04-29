// src/i18n/mod.rs
//
// All user-visible strings, organised per language.
// Every language is a `const Strings` value — zero runtime cost, fully embedded
// in the binary.  Dynamic values (counts, times) use `{}` as placeholder; call
// `.replacen("{}", value, 1)` at the use-site.
//
// Panel control strings must fit in 34 chars (the inner width of the status panel).
// Width formula: 2-space indent + key + spacing + description ≤ 34.

// ── String catalogue ──────────────────────────────────────────────────────────

pub struct Strings {
    // ── Start / navigation menus ─────────────────────────────────────────────
    pub menu_new_game:         &'static str,
    pub menu_language:         &'static str,
    pub menu_theme:            &'static str,
    pub menu_quit:             &'static str,
    pub difficulty_title:      &'static str,
    pub difficulty_easy:       &'static str,
    pub difficulty_medium:     &'static str,
    pub difficulty_hard:       &'static str,
    pub symmetry_label:        &'static str,
    pub language_title:        &'static str,
    pub theme_title:           &'static str,

    // ── Status panel — dynamic rows (use {} placeholder for the value) ───────
    /// e.g. "  Time:  {}"  →  "  Time:  01:30"
    pub panel_time:            &'static str,
    /// e.g. "  Mode:  {}"  →  "  Mode:  Notes"
    pub panel_mode:            &'static str,
    /// e.g. "  Show errors: {}"  →  "  Show errors: on"
    pub panel_errors:          &'static str,
    /// e.g. "  Count: {}"  →  "  Count: 3"
    pub panel_count:           &'static str,
    /// e.g. "  Filled: {}/81"  →  "  Filled: 23/81"
    pub panel_filled:          &'static str,

    // ── Status panel — static rows ───────────────────────────────────────────
    pub panel_remaining:       &'static str,
    pub panel_controls:        &'static str,

    // ── Mode / toggle values ─────────────────────────────────────────────────
    pub mode_notes:            &'static str,
    pub mode_solution:         &'static str,
    pub toggle_on:             &'static str,
    pub toggle_off:            &'static str,

    // ── Control hints (≤ 34 chars each, shown in status panel) ──────────────
    pub ctrl_move:             &'static str,
    pub ctrl_goto:             &'static str,
    pub ctrl_digit:            &'static str,
    pub ctrl_mode:             &'static str,
    pub ctrl_scan:             &'static str,
    pub ctrl_errors:           &'static str,
    pub ctrl_undo:             &'static str,
    pub ctrl_redo:             &'static str,
    pub ctrl_clear:            &'static str,
    pub ctrl_pause:            &'static str,
    pub ctrl_boss:             &'static str,
    pub ctrl_quit:             &'static str,
    /// Hint key control label.
    pub ctrl_hint: &'static str,

    // ── Hint strategy names and explanations ─────────────────────────────────
    // Explanations use {row}, {col}, {box}, {digit} as placeholders.
    // Only EN and DE have translated values; all other languages copy EN.
    pub hint_full_house_name:        &'static str,
    pub hint_full_house_explain:     &'static str,
    pub hint_naked_single_name:      &'static str,
    pub hint_naked_single_explain:   &'static str,
    pub hint_hidden_single_name:     &'static str,
    pub hint_hidden_single_explain:  &'static str,
    pub hint_notes_name:             &'static str,
    pub hint_notes_explain:          &'static str,
    pub hint_naked_pairs_name:       &'static str,
    pub hint_naked_pairs_explain:    &'static str,
    pub hint_hidden_pairs_name:      &'static str,
    pub hint_hidden_pairs_explain:   &'static str,
    pub hint_pointing_pairs_name:    &'static str,
    pub hint_pointing_pairs_explain: &'static str,
    pub hint_box_line_name:          &'static str,
    pub hint_box_line_explain:       &'static str,
    pub hint_reveal_name:            &'static str,
    pub hint_reveal_explain:         &'static str,

    // ── Overlay dismiss line (dim text at the bottom of every info overlay) ──
    pub dismiss:               &'static str,

    // ── Resize-wait screen ───────────────────────────────────────────────────
    /// Two `{}` placeholders: cols, rows.
    pub resize_too_small:      &'static str,
    /// Two `{}` placeholders: min_cols, min_rows.
    pub resize_required:       &'static str,
    pub resize_hint:           &'static str,

    // ── CLI / start-screen error notices ────────────────────────────────────
    /// One `{}` placeholder: the parse-error detail (always in English).
    pub puzzle_invalid:        &'static str,
    /// One `{}` placeholder: the given-cell count.
    pub puzzle_few_givens:     &'static str,
    pub puzzle_no_solution:    &'static str,

    // ── In-game overlays ─────────────────────────────────────────────────────
    pub puzzle_has_errors:     &'static str,
    pub puzzle_errors_hint:    &'static str,
    pub confirm_quit_title:    &'static str,
    pub confirm_quit_options:  &'static str,
    pub resume_hint:           &'static str,
}

// ── Language enum ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English    = 0,
    German     = 1,
    Spanish    = 2,
    Italian    = 3,
    French     = 4,
    Slovenian  = 5,
    Esperanto  = 6,
    TokiPona   = 7,
    Leet       = 8,
    Swahili    = 9,
    Afrikaans  = 10,
    Pinyin     = 11,
    Indonesian = 12,
}

/// Display names shown in the language-selection menu (each in its own language).
pub const LANGUAGE_NAMES: &[&str] = &[
    "English",
    "Deutsch",
    "Español",
    "Italiano",
    "Français",
    "Slovenščina",
    "Esperanto",
    "Toki Pona",
    "L33tsp34k",
    "Kiswahili",
    "Afrikaans",
    "Zh\u{14d}ngw\u{e9}n (P\u{12b}ny\u{12b}n)",
    "Bahasa Indonesia",
];

pub const LANGUAGE_COUNT: usize = 13;

impl Language {
    /// Return the `&'static Strings` for this language.
    pub fn strings(self) -> &'static Strings {
        match self {
            Language::English    => &EN,
            Language::German     => &DE,
            Language::Spanish    => &ES,
            Language::Italian    => &IT,
            Language::French     => &FR,
            Language::Slovenian  => &SL,
            Language::Esperanto  => &EO,
            Language::TokiPona   => &TP,
            Language::Leet       => &LEET,
            Language::Swahili    => &SW,
            Language::Afrikaans  => &AF,
            Language::Pinyin     => &PY,
            Language::Indonesian => &ID,
        }
    }

    /// Index into `LANGUAGE_NAMES`.
    pub fn as_index(self) -> usize { self as usize }

    /// Construct from index; out-of-range defaults to English.
    pub fn from_index(i: usize) -> Self {
        match i {
            0  => Language::English,
            1  => Language::German,
            2  => Language::Spanish,
            3  => Language::Italian,
            4  => Language::French,
            5  => Language::Slovenian,
            6  => Language::Esperanto,
            7  => Language::TokiPona,
            8  => Language::Leet,
            9  => Language::Swahili,
            10 => Language::Afrikaans,
            11 => Language::Pinyin,
            12 => Language::Indonesian,
            _  => Language::English,
        }
    }

    /// Parse a short language code string; returns `None` for unknown codes.
    /// Codes match BCP-47 where possible; special cases: `tp`, `leet`/`1337`, `py`.
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "en"           => Some(Language::English),
            "de"           => Some(Language::German),
            "es"           => Some(Language::Spanish),
            "it"           => Some(Language::Italian),
            "fr"           => Some(Language::French),
            "sl"           => Some(Language::Slovenian),
            "eo"           => Some(Language::Esperanto),
            "tp"           => Some(Language::TokiPona),
            "leet" | "1337" => Some(Language::Leet),
            "sw"           => Some(Language::Swahili),
            "af"           => Some(Language::Afrikaans),
            "py"           => Some(Language::Pinyin),
            "id"           => Some(Language::Indonesian),
            _              => None,
        }
    }

    /// Auto-detect from the OS locale; default English.
    /// Toki Pona (tok) and Leet are never auto-detected — they must be chosen.
    pub fn detect() -> Self {
        let locale = sys_locale::get_locale().unwrap_or_default();
        let code = locale
            .split(|c| c == '-' || c == '_')
            .next()
            .unwrap_or("en")
            .to_lowercase();
        match code.as_str() {
            "de" => Language::German,
            "es" => Language::Spanish,
            "it" => Language::Italian,
            "fr" => Language::French,
            "sl" => Language::Slovenian,
            "eo" => Language::Esperanto,
            "sw" => Language::Swahili,
            "af" => Language::Afrikaans,
            "id" => Language::Indonesian,
            // Pinyin is a romanisation choice, not a locale → manual only
            _    => Language::English,
        }
    }
}

// ── English ───────────────────────────────────────────────────────────────────

pub const EN: Strings = Strings {
    menu_new_game:        "New Game",
    menu_language:        "Language",
    menu_theme:           "Theme",
    menu_quit:            "Quit",
    difficulty_title:     "Select difficulty:",
    difficulty_easy:      "Easy",
    difficulty_medium:    "Medium",
    difficulty_hard:      "Hard",
    symmetry_label:       "Symmetry",
    language_title:       "Select language:",
    theme_title:          "Select theme:",
    panel_time:           "  Time:  {}",
    panel_mode:           "  Mode:  {}",
    panel_errors:         "  Show errors: {}",
    panel_count:          "  Count: {}",
    panel_filled:         "  Filled: {}/81",
    panel_remaining:      "  Remaining:",
    panel_controls:       "  Controls",
    mode_notes:           "Notes",
    mode_solution:        "Solution",
    toggle_on:            "on",
    toggle_off:           "off",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   move",
    ctrl_goto:            "  Enter  goto",
    ctrl_digit:           "  1-9    digit",
    ctrl_mode:            "  0      notes\u{21d4}sol",
    ctrl_scan:            "  s      scan",
    ctrl_errors:          "  e      errors",
    ctrl_undo:            "  u/^Z   undo",
    ctrl_redo:            "  r/^Y   redo",
    ctrl_clear:           "  -      clear",
    ctrl_pause:           "  Spc    pause",
    ctrl_boss:            "  b      boss key",
    ctrl_quit:            "  Esc    quit",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} press any key \u{2014}",
    resize_too_small:     "Terminal too small: {}\u{d7}{}",
    resize_required:      "Required: {}\u{d7}{}",
    resize_hint:          "Please resize the window ...",
    puzzle_invalid:       "Invalid puzzle: {}",
    puzzle_few_givens:    "Only {} given cells (min. 17).",
    puzzle_no_solution:   "Puzzle has no solution.",
    puzzle_has_errors:    "The puzzle contains errors.",
    puzzle_errors_hint:   "Press [e] to reveal errors.",
    confirm_quit_title:   "Quit game?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  to resume",
};

// ── German ────────────────────────────────────────────────────────────────────

pub const DE: Strings = Strings {
    menu_new_game:        "Neues Spiel",
    menu_language:        "Sprache",
    menu_theme:           "Theme",
    menu_quit:            "Beenden",
    difficulty_title:     "Schwierigkeit:",
    difficulty_easy:      "Leicht",
    difficulty_medium:    "Mittel",
    difficulty_hard:      "Schwer",
    symmetry_label:       "Symmetrie",
    language_title:       "Sprache w\u{e4}hlen:",
    theme_title:          "Design w\u{e4}hlen:",
    panel_time:           "  Zeit:  {}",
    panel_mode:           "  Modus: {}",
    panel_errors:         "  Fehler anz.: {}",
    panel_count:          "  Anzahl: {}",
    panel_filled:         "  Gel\u{f6}st: {}/81",
    panel_remaining:      "  Verbleibend:",
    panel_controls:       "  Steuerung",
    mode_notes:           "Notizen",
    mode_solution:        "L\u{f6}sung",
    toggle_on:            "an",
    toggle_off:           "aus",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   bewegen",
    ctrl_goto:            "  Enter  anw\u{e4}hlen",
    ctrl_digit:           "  1-9    Ziffer",
    ctrl_mode:            "  0      Notiz\u{21d4}Lsg",
    ctrl_scan:            "  s      Scan",
    ctrl_errors:          "  e      Fehler",
    ctrl_undo:            "  u/^Z   r\u{fc}ckg\u{e4}ng.",
    ctrl_redo:            "  r/^Y   wiederh.",
    ctrl_clear:           "  -      l\u{f6}schen",
    ctrl_pause:           "  Spc    Pause",
    ctrl_boss:            "  b      Tarnmodus",
    ctrl_quit:            "  Esc    Beenden",
    ctrl_hint:                   "  h      Hinweis",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Nur eine leere Zelle bleibt in dieser Einheit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Nur {digit} passt in diese Zelle.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} kann nur hier in dieser Einheit stehen.",
    hint_notes_name:             "Notizen erg\u{e4}nzen",
    hint_notes_explain:          "Trage Notizen in diese Einheit ein.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "Diese zwei Zellen halten {digit}. In anderen eliminieren.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Nur diese Zellen k\u{f6}nnen dieses Paar halten.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in dieser Box zeigt auf diese Zeile/Spalte.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in dieser Zeile/Spalte ist auf diese Box beschr\u{e4}nkt.",
    hint_reveal_name:            "Aufdecken",
    hint_reveal_explain:         "Kein logischer Zug m\u{f6}glich. F\u{fc}lle die engste Zelle.",
    dismiss:              "\u{2014} beliebige Taste \u{2014}",
    resize_too_small:     "Terminal zu klein: {}\u{d7}{}",
    resize_required:      "Ben\u{f6}tigt: {}\u{d7}{}",
    resize_hint:          "Bitte Fenster vergr\u{f6}\u{df}ern ...",
    puzzle_invalid:       "Ung\u{fc}ltiges R\u{e4}tsel: {}",
    puzzle_few_givens:    "Nur {} Startzellen (mind. 17).",
    puzzle_no_solution:   "R\u{e4}tsel hat keine L\u{f6}sung.",
    puzzle_has_errors:    "Das R\u{e4}tsel enth\u{e4}lt Fehler.",
    puzzle_errors_hint:   "[e] dr\u{fc}cken f\u{fc}r Fehler.",
    confirm_quit_title:   "Spiel beenden?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  Fortsetzen",
};

// ── Spanish ───────────────────────────────────────────────────────────────────

pub const ES: Strings = Strings {
    menu_new_game:        "Nueva partida",
    menu_language:        "Idioma",
    menu_theme:           "Tema",
    menu_quit:            "Salir",
    difficulty_title:     "Dificultad:",
    difficulty_easy:      "F\u{e1}cil",
    difficulty_medium:    "Medio",
    difficulty_hard:      "Dif\u{ed}cil",
    symmetry_label:       "Simetr\u{ed}a",
    language_title:       "Selecciona idioma:",
    theme_title:          "Selecciona tema:",
    panel_time:           "  Tiempo: {}",
    panel_mode:           "  Modo:   {}",
    panel_errors:         "  Ver errores: {}",
    panel_count:          "  Total: {}",
    panel_filled:         "  Llenado: {}/81",
    panel_remaining:      "  Restantes:",
    panel_controls:       "  Controles",
    mode_notes:           "Notas",
    mode_solution:        "Soluci\u{f3}n",
    toggle_on:            "s\u{ed}",
    toggle_off:           "no",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   mover",
    ctrl_goto:            "  Enter  ir a",
    ctrl_digit:           "  1-9    d\u{ed}gito",
    ctrl_mode:            "  0      notas\u{21d4}sol",
    ctrl_scan:            "  s      escanear",
    ctrl_errors:          "  e      errores",
    ctrl_undo:            "  u/^Z   deshacer",
    ctrl_redo:            "  r/^Y   rehacer",
    ctrl_clear:           "  -      borrar",
    ctrl_pause:           "  Spc    pausa",
    ctrl_boss:            "  b      pantalla",
    ctrl_quit:            "  Esc    salir",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} cualquier tecla \u{2014}",
    resize_too_small:     "Terminal peque\u{f1}o: {}\u{d7}{}",
    resize_required:      "Necesario: {}\u{d7}{}",
    resize_hint:          "Ampl\u{ed}a la ventana ...",
    puzzle_invalid:       "Puzle inv\u{e1}lido: {}",
    puzzle_few_givens:    "Solo {} celdas dadas (m\u{ed}n. 17).",
    puzzle_no_solution:   "El puzle no tiene soluci\u{f3}n.",
    puzzle_has_errors:    "El puzle contiene errores.",
    puzzle_errors_hint:   "[e] para ver los errores.",
    confirm_quit_title:   "\u{bf}Salir del juego?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  reanudar",
};

// ── Italian ───────────────────────────────────────────────────────────────────

pub const IT: Strings = Strings {
    menu_new_game:        "Nuova partita",
    menu_language:        "Lingua",
    menu_theme:           "Tema",
    menu_quit:            "Esci",
    difficulty_title:     "Difficolt\u{e0}:",
    difficulty_easy:      "Facile",
    difficulty_medium:    "Medio",
    difficulty_hard:      "Difficile",
    symmetry_label:       "Simmetria",
    language_title:       "Seleziona lingua:",
    theme_title:          "Seleziona tema:",
    panel_time:           "  Tempo:  {}",
    panel_mode:           "  Modo:  {}",
    panel_errors:         "  Mostra err.: {}",
    panel_count:          "  Totale: {}",
    panel_filled:         "  Riempito: {}/81",
    panel_remaining:      "  Rimanenti:",
    panel_controls:       "  Controlli",
    mode_notes:           "Note",
    mode_solution:        "Soluzione",
    toggle_on:            "s\u{ec}",
    toggle_off:           "no",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   sposta",
    ctrl_goto:            "  Enter  vai a",
    ctrl_digit:           "  1-9    cifra",
    ctrl_mode:            "  0      note\u{21d4}sol",
    ctrl_scan:            "  s      scansione",
    ctrl_errors:          "  e      errori",
    ctrl_undo:            "  u/^Z   annulla",
    ctrl_redo:            "  r/^Y  ripristina",
    ctrl_clear:           "  -      cancella",
    ctrl_pause:           "  Spc    pausa",
    ctrl_boss:            "  b      schermo",
    ctrl_quit:            "  Esc    esci",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} qualsiasi tasto \u{2014}",
    resize_too_small:     "Terminale piccolo: {}\u{d7}{}",
    resize_required:      "Richiesto: {}\u{d7}{}",
    resize_hint:          "Ridimensiona la finestra ...",
    puzzle_invalid:       "Puzzle non valido: {}",
    puzzle_few_givens:    "Solo {} celle iniziali (min. 17).",
    puzzle_no_solution:   "Il puzzle non ha soluzione.",
    puzzle_has_errors:    "Il puzzle contiene errori.",
    puzzle_errors_hint:   "[e] per mostrare gli errori.",
    confirm_quit_title:   "Vuoi uscire?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  riprendi",
};

// ── French ────────────────────────────────────────────────────────────────────

pub const FR: Strings = Strings {
    menu_new_game:        "Nouvelle partie",
    menu_language:        "Langue",
    menu_theme:           "Thème",
    menu_quit:            "Quitter",
    difficulty_title:     "Difficult\u{e9} :",
    difficulty_easy:      "Facile",
    difficulty_medium:    "Moyen",
    difficulty_hard:      "Difficile",
    symmetry_label:       "Sym\u{e9}trie",
    language_title:       "Choisir la langue :",
    theme_title:          "Choisir le thème :",
    panel_time:           "  Temps :  {}",
    panel_mode:           "  Mode :  {}",
    panel_errors:         "  Voir err. : {}",
    panel_count:          "  Total : {}",
    panel_filled:         "  Rempli : {}/81",
    panel_remaining:      "  Restants :",
    panel_controls:       "  Commandes",
    mode_notes:           "Notes",
    mode_solution:        "Solution",
    toggle_on:            "oui",
    toggle_off:           "non",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   d\u{e9}placer",
    ctrl_goto:            "  Enter  aller",
    ctrl_digit:           "  1-9    chiffre",
    ctrl_mode:            "  0      notes\u{21d4}sol",
    ctrl_scan:            "  s      chercher",
    ctrl_errors:          "  e      erreurs",
    ctrl_undo:            "  u/^Z   annuler",
    ctrl_redo:            "  r/^Y   refaire",
    ctrl_clear:           "  -      effacer",
    ctrl_pause:           "  Spc    pause",
    ctrl_boss:            "  b      patron",
    ctrl_quit:            "  Esc    quitter",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} une touche \u{2014}",
    resize_too_small:     "Terminal trop petit : {}\u{d7}{}",
    resize_required:      "Requis : {}\u{d7}{}",
    resize_hint:          "Agrandissez la fen\u{ea}tre ...",
    puzzle_invalid:       "Puzzle invalide : {}",
    puzzle_few_givens:    "Que {} cases initiales (min. 17).",
    puzzle_no_solution:   "Le puzzle n\u{2019}a pas de solution.",
    puzzle_has_errors:    "Le puzzle contient des erreurs.",
    puzzle_errors_hint:   "[e] pour afficher les erreurs.",
    confirm_quit_title:   "Quitter le jeu ?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  reprendre",
};

// ── Slovenian ─────────────────────────────────────────────────────────────────

pub const SL: Strings = Strings {
    menu_new_game:        "Nova igra",
    menu_language:        "Jezik",
    menu_theme:           "Tema",
    menu_quit:            "Izhod",
    difficulty_title:     "Te\u{17e}avnost:",
    difficulty_easy:      "Lahka",
    difficulty_medium:    "Srednja",
    difficulty_hard:      "Te\u{17e}ka",
    symmetry_label:       "Simetrija",
    language_title:       "Izberi jezik:",
    theme_title:          "Izberi temo:",
    panel_time:           "  \u{10c}as:    {}",
    panel_mode:           "  Na\u{10d}in:  {}",
    panel_errors:         "  Poka\u{17e}i nap.: {}",
    panel_count:          "  \u{160}tevilo: {}",
    panel_filled:         "  Polno: {}/81",
    panel_remaining:      "  Preostalo:",
    panel_controls:       "  Kontrole",
    mode_notes:           "Opombe",
    mode_solution:        "Re\u{161}itev",
    toggle_on:            "da",
    toggle_off:           "ne",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   premik",
    ctrl_goto:            "  Enter  pojdi",
    ctrl_digit:           "  1-9    \u{161}tevilka",
    ctrl_mode:            "  0      op.\u{21d4}re\u{161}.",
    ctrl_scan:            "  s      iskanje",
    ctrl_errors:          "  e      napake",
    ctrl_undo:            "  u/^Z  razveljavi",
    ctrl_redo:            "  r/^Y   ponovi",
    ctrl_clear:           "  -      zbri\u{161}i",
    ctrl_pause:           "  Spc    premor",
    ctrl_boss:            "  b      \u{161}ef",
    ctrl_quit:            "  Esc    izhod",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} katera koli tipka \u{2014}",
    resize_too_small:     "Terminal premajhen: {}\u{d7}{}",
    resize_required:      "Potrebno: {}\u{d7}{}",
    resize_hint:          "Prosim pove\u{10d}aj okno ...",
    puzzle_invalid:       "Neveljavna uganka: {}",
    puzzle_few_givens:    "Samo {} danih celic (min. 17).",
    puzzle_no_solution:   "Uganke ni mogo\u{10d}e re\u{161}iti.",
    puzzle_has_errors:    "Uganka vsebuje napake.",
    puzzle_errors_hint:   "[e] za prikaz napak.",
    confirm_quit_title:   "Zapustiti igro?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  nadaljuj",
};

// ── Esperanto ─────────────────────────────────────────────────────────────────

pub const EO: Strings = Strings {
    menu_new_game:        "Nova ludo",
    menu_language:        "Lingvo",
    menu_theme:           "Temo",
    menu_quit:            "Eliri",
    difficulty_title:     "Malfacileco:",
    difficulty_easy:      "Facila",
    difficulty_medium:    "Meza",
    difficulty_hard:      "Malfacila",
    symmetry_label:       "Simetrio",
    language_title:       "Elektu lingvon:",
    theme_title:          "Elektu temon:",
    panel_time:           "  Tempo:  {}",
    panel_mode:           "  Re\u{11d}imo: {}",
    panel_errors:         "  Montri err.: {}",
    panel_count:          "  Nombro: {}",
    panel_filled:         "  Pleniga: {}/81",
    panel_remaining:      "  Restantaj:",
    panel_controls:       "  Kontroloj",
    mode_notes:           "Notoj",
    mode_solution:        "Solva\u{135}o",
    toggle_on:            "jes",
    toggle_off:           "ne",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   movi",
    ctrl_goto:            "  Enter  iri al",
    ctrl_digit:           "  1-9    cifero",
    ctrl_mode:            "  0      notoj\u{21d4}sol",
    ctrl_scan:            "  s      skani",
    ctrl_errors:          "  e      eraroj",
    ctrl_undo:            "  u/^Z   malfari",
    ctrl_redo:            "  r/^Y   refari",
    ctrl_clear:           "  -      forigi",
    ctrl_pause:           "  Spc    pa\u{16d}zo",
    ctrl_boss:            "  b      ka\u{15d}modo",
    ctrl_quit:            "  Esc    eliri",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} ia klavo \u{2014}",
    resize_too_small:     "Terminalo tro malgranda: {}\u{d7}{}",
    resize_required:      "Bezonata: {}\u{d7}{}",
    resize_hint:          "Bonvolu pligrandigi fenestron ...",
    puzzle_invalid:       "Nevalida enigmo: {}",
    puzzle_few_givens:    "Nur {} donitaj \u{109}eloj (min. 17).",
    puzzle_no_solution:   "La enigmo ne havas solvon.",
    puzzle_has_errors:    "La enigmo enhavas erarojn.",
    puzzle_errors_hint:   "[e] por montri erarojn.",
    confirm_quit_title:   "\u{108}esu ludon?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  da\u{16d}rigi",
};

// ── Toki Pona ─────────────────────────────────────────────────────────────────

pub const TP: Strings = Strings {
    menu_new_game:        "musi sin",
    menu_language:        "toki",
    menu_theme:           "kule",
    menu_quit:            "pini",
    difficulty_title:     "suli musi:",
    difficulty_easy:      "lili",
    difficulty_medium:    "insa",
    difficulty_hard:      "suli",
    symmetry_label:       "sama",
    language_title:       "toki seme:",
    theme_title:          "o kule e ni:",
    panel_time:           "  tenpo: {}",
    panel_mode:           "  nasin: {}",
    panel_errors:         "  pakala: {}",
    panel_count:          "  nanpa: {}",
    panel_filled:         "  pali: {}/81",
    panel_remaining:      "  awen:",
    panel_controls:       "  nasin luka",
    mode_notes:           "lipu",
    mode_solution:        "pona",
    toggle_on:            "lon",
    toggle_off:           "ala",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   tawa",
    ctrl_goto:            "  Enter  tawa ijo",
    ctrl_digit:           "  1-9    nanpa",
    ctrl_mode:            "  0      lipu\u{21d4}pona",
    ctrl_scan:            "  s      lukin",
    ctrl_errors:          "  e      pakala",
    ctrl_undo:            "  u/^Z   weka",
    ctrl_redo:            "  r/^Y   sin",
    ctrl_clear:           "  -      weka",
    ctrl_pause:           "  Spc    awen",
    ctrl_boss:            "  b      len",
    ctrl_quit:            "  Esc    pini",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} pana e seme \u{2014}",
    resize_too_small:     "lipu lili ala: {}\u{d7}{}",
    resize_required:      "wile: {}\u{d7}{}",
    resize_hint:          "pona e lipu ...",
    puzzle_invalid:       "musi li ike: {}",
    puzzle_few_givens:    "nasin {} taso (17 li wile).",
    puzzle_no_solution:   "musi li pona ala.",
    puzzle_has_errors:    "musi li jo e pakala.",
    puzzle_errors_hint:   "[e] tawa lukin pakala.",
    confirm_quit_title:   "pini musi?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  tawa sin",
};

// ── Leetspeak ─────────────────────────────────────────────────────────────────

pub const LEET: Strings = Strings {
    menu_new_game:        "N3W G4M3",
    menu_language:        "L4NGU4G3",
    menu_theme:           "TH3M3",
    menu_quit:            "QU1T",
    difficulty_title:     "CH00S3 D1FF:",
    difficulty_easy:      "34SY",
    difficulty_medium:    "M3D1UM",
    difficulty_hard:      "H4RD",
    symmetry_label:       "5YMM3TRY",
    language_title:       "L4NGU4G3:",
    theme_title:          "TH3M3:",
    panel_time:           "  71M3:  {}",
    panel_mode:           "  M0D3:  {}",
    panel_errors:         "  5H0W 3RR: {}",
    panel_count:          "  C0UNT: {}",
    panel_filled:         "  F1LL3D: {}/81",
    panel_remaining:      "  R3M41N1NG:",
    panel_controls:       "  C0NTR0LZ",
    mode_notes:           "N0T3Z",
    mode_solution:        "S0LUT10N",
    toggle_on:            "0n",
    toggle_off:           "0ff",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   m0v3",
    ctrl_goto:            "  Enter  g0t0",
    ctrl_digit:           "  1-9    d1g1t",
    ctrl_mode:            "  0      n0t3z\u{21d4}s0l",
    ctrl_scan:            "  s      sc4n",
    ctrl_errors:          "  e      3rr0rz",
    ctrl_undo:            "  u/^Z   und0",
    ctrl_redo:            "  r/^Y   r3d0",
    ctrl_clear:           "  -      cl34r",
    ctrl_pause:           "  Spc    p4us3",
    ctrl_boss:            "  b      b0ss k3y",
    ctrl_quit:            "  Esc    qu1t",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} pr355 4ny k3y \u{2014}",
    resize_too_small:     "T3RM1N4L T00 5M4LL: {}\u{d7}{}",
    resize_required:      "R3QU1R3D: {}\u{d7}{}",
    resize_hint:          "Pl34s3 r3s1z3 w1nd0w ...",
    puzzle_invalid:       "1NV4L1D PUZZL3: {}",
    puzzle_few_givens:    "0nly {} g1v3n c3llz (m1n. 17).",
    puzzle_no_solution:   "Puzzl3 h4z n0 s0lut10n.",
    puzzle_has_errors:    "Puzzl3 h4z 3rr0rz.",
    puzzle_errors_hint:   "[e] t0 r3v34l 3rr0rz.",
    confirm_quit_title:   "Qu1t g4m3?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[5p4c3]  t0 r35um3",
};

// ── Swahili ───────────────────────────────────────────────────────────────────

pub const SW: Strings = Strings {
    menu_new_game:        "Mchezo Mpya",
    menu_language:        "Lugha",
    menu_theme:           "Mandhari",
    menu_quit:            "Toka",
    difficulty_title:     "Chagua ugumu:",
    difficulty_easy:      "Rahisi",
    difficulty_medium:    "Wastani",
    difficulty_hard:      "Ngumu",
    symmetry_label:       "Usawa",
    language_title:       "Chagua lugha:",
    theme_title:          "Chagua mandhari:",
    panel_time:           "  Muda:  {}",
    panel_mode:           "  Hali:  {}",
    panel_errors:         "  Makosa: {}",
    panel_count:          "  Idadi: {}",
    panel_filled:         "  Imejaa: {}/81",
    panel_remaining:      "  Zilizobaki:",
    panel_controls:       "  Vidhibiti",
    mode_notes:           "Maelezo",
    mode_solution:        "Jibu",
    toggle_on:            "ndiyo",
    toggle_off:           "hapana",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   sogea",
    ctrl_goto:            "  Enter  nenda",
    ctrl_digit:           "  1-9    namba",
    ctrl_mode:            "  0      mael\u{21d4}jibu",
    ctrl_scan:            "  s      tafuta",
    ctrl_errors:          "  e      makosa",
    ctrl_undo:            "  u/^Z   tendua",
    ctrl_redo:            "  r/^Y   rudia",
    ctrl_clear:           "  -      futa",
    ctrl_pause:           "  Spc    simama",
    ctrl_boss:            "  b      ficha",
    ctrl_quit:            "  Esc    toka",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} bonyeza kitufe \u{2014}",
    resize_too_small:     "Terminali ndogo: {}\u{d7}{}",
    resize_required:      "Inahitajika: {}\u{d7}{}",
    resize_hint:          "Tafadhali panua dirisha ...",
    puzzle_invalid:       "Tatizo batili: {}",
    puzzle_few_givens:    "Ni {} tu iliyotolewa (kima. 17).",
    puzzle_no_solution:   "Tatizo halina jibu.",
    puzzle_has_errors:    "Tatizo lina makosa.",
    puzzle_errors_hint:   "[e] kuona makosa.",
    confirm_quit_title:   "Acha mchezo?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  endelea",
};

// ── Afrikaans ─────────────────────────────────────────────────────────────────

pub const AF: Strings = Strings {
    menu_new_game:        "Nuwe spel",
    menu_language:        "Taal",
    menu_theme:           "Tema",
    menu_quit:            "Verlaat",
    difficulty_title:     "Kies moeilikheid:",
    difficulty_easy:      "Maklik",
    difficulty_medium:    "Middelmatig",
    difficulty_hard:      "Moeilik",
    symmetry_label:       "Simmetrie",
    language_title:       "Kies taal:",
    theme_title:          "Kies tema:",
    panel_time:           "  Tyd:   {}",
    panel_mode:           "  Modus: {}",
    panel_errors:         "  Toon foute: {}",
    panel_count:          "  Telling: {}",
    panel_filled:         "  Gevul: {}/81",
    panel_remaining:      "  Oorblywend:",
    panel_controls:       "  Kontroles",
    mode_notes:           "Notas",
    mode_solution:        "Oplossing",
    toggle_on:            "ja",
    toggle_off:           "nee",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   beweeg",
    ctrl_goto:            "  Enter  gaan na",
    ctrl_digit:           "  1-9    syfer",
    ctrl_mode:            "  0      notas\u{21d4}opl",
    ctrl_scan:            "  s      soek",
    ctrl_errors:          "  e      foute",
    ctrl_undo:            "  u/^Z   ongedaan",
    ctrl_redo:            "  r/^Y   herhaal",
    ctrl_clear:           "  -      uitvee",
    ctrl_pause:           "  Spc    pouse",
    ctrl_boss:            "  b      baas",
    ctrl_quit:            "  Esc    verlaat",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} druk 'n toets \u{2014}",
    resize_too_small:     "Terminal te klein: {}\u{d7}{}",
    resize_required:      "Vereis: {}\u{d7}{}",
    resize_hint:          "Vergroot asseblief venster ...",
    puzzle_invalid:       "Ongeldige legkaart: {}",
    puzzle_few_givens:    "Net {} gegewe selle (min. 17).",
    puzzle_no_solution:   "Legkaart het geen oplossing.",
    puzzle_has_errors:    "Legkaart bevat foute.",
    puzzle_errors_hint:   "[e] om foute te wys.",
    confirm_quit_title:   "Spel verlaat?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  hervat",
};

// ── Mandarin Chinese in Pinyin romanisation ───────────────────────────────────
// Tone marks: macron=1st, acute=2nd, caron=3rd, grave=4th.

pub const PY: Strings = Strings {
    menu_new_game:        "X\u{12b}n y\u{f3}ux\u{ec}",
    menu_language:        "Y\u{1d4}y\u{e1}n",
    menu_theme:           "Zh\u{fa}t\u{ed}",
    menu_quit:            "Tu\u{ec}ch\u{16b}",
    difficulty_title:     "Xu\u{1ce}nz\u{e9} n\u{e1}nd\u{f9}:",
    difficulty_easy:      "R\u{f3}ngy\u{ec}",
    difficulty_medium:    "Zh\u{14d}ngd\u{11b}ng",
    difficulty_hard:      "K\u{f9}nn\u{e1}n",
    symmetry_label:       "Du\u{ec}ch\u{e8}n",
    language_title:       "Xu\u{1ce}nz\u{e9} y\u{1d4}y\u{e1}n:",
    theme_title:          "Xu\u{1ce}nz\u{e9} zh\u{fa}t\u{ed}:",
    panel_time:           "  Sh\u{ed}ji\u{101}n: {}",
    panel_mode:           "  M\u{f3}sh\u{ec}: {}",
    panel_errors:         "  Cu\u{f2}w\u{f9}: {}",
    panel_count:          "  J\u{ec}sh\u{f9}: {}",
    panel_filled:         "  Y\u{1d0}ti\u{e1}n: {}/81",
    panel_remaining:      "  Sh\u{e8}ngxi\u{e0}:",
    panel_controls:       "  C\u{101}ozu\u{f2}",
    mode_notes:           "B\u{1d0}j\u{ec}",
    mode_solution:        "Ji\u{11b}d\u{e1}",
    toggle_on:            "k\u{101}i",
    toggle_off:           "gu\u{101}n",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   y\u{ed}d\u{f2}ng",
    ctrl_goto:            "  Enter  ti\u{e0}o",
    ctrl_digit:           "  1-9    sh\u{f9}z\u{ec}",
    ctrl_mode:            "  0      b\u{1d0}j\u{ec}\u{21d4}ji\u{11b}",
    ctrl_scan:            "  s      s\u{1ce}o",
    ctrl_errors:          "  e      cu\u{f2}w\u{f9}",
    ctrl_undo:            "  u/^Z   ch\u{e8}xi\u{101}o",
    ctrl_redo:            "  r/^Y   ch\u{f3}ngzu\u{f2}",
    ctrl_clear:           "  -      q\u{12b}ngch\u{fa}",
    ctrl_pause:           "  Spc    z\u{e0}nt\u{ed}ng",
    ctrl_boss:            "  b      y\u{1d0}nc\u{e1}ng",
    ctrl_quit:            "  Esc    tu\u{ec}ch\u{16b}",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} \u{e0}n r\u{e8}ny\u{ec} ji\u{e0}n \u{2014}",
    resize_too_small:     "Zh\u{14d}ngdu\u{101}n t\u{e0}i xi\u{1ce}o: {}\u{d7}{}",
    resize_required:      "X\u{16b}y\u{e0}o: {}\u{d7}{}",
    resize_hint:          "Q\u{1d0}ng ti\u{e1}ozh\u{e8}ng chu\u{101}ngk\u{1d2}u ...",
    puzzle_invalid:       "T\u{ed} w\u{fa}xi\u{e0}o: {}",
    puzzle_few_givens:    "Zh\u{1d0} {} g\u{e8} (zu\u{ec}sh\u{e3}o 17).",
    puzzle_no_solution:   "T\u{ed} m\u{e9}i ji\u{11b}d\u{e1}.",
    puzzle_has_errors:    "T\u{ed} h\u{e1}ny\u{1d2}u cu\u{f2}w\u{f9}.",
    puzzle_errors_hint:   "[e] xi\u{1ce}nsh\u{ec} cu\u{f2}w\u{f9}.",
    confirm_quit_title:   "Tu\u{ec}ch\u{16b} y\u{f3}ux\u{ec}?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  j\u{ec}x\u{f9}",
};

// ── Indonesian (Bahasa Indonesia) ─────────────────────────────────────────────

pub const ID: Strings = Strings {
    menu_new_game:        "Permainan Baru",
    menu_language:        "Bahasa",
    menu_theme:           "Tema",
    menu_quit:            "Keluar",
    difficulty_title:     "Pilih kesulitan:",
    difficulty_easy:      "Mudah",
    difficulty_medium:    "Sedang",
    difficulty_hard:      "Sulit",
    symmetry_label:       "Simetri",
    language_title:       "Pilih bahasa:",
    theme_title:          "Pilih tema:",
    panel_time:           "  Waktu:  {}",
    panel_mode:           "  Mode:   {}",
    panel_errors:         "  Kesalahan: {}",
    panel_count:          "  Jumlah: {}",
    panel_filled:         "  Terisi: {}/81",
    panel_remaining:      "  Tersisa:",
    panel_controls:       "  Kontrol",
    mode_notes:           "Catatan",
    mode_solution:        "Solusi",
    toggle_on:            "ya",
    toggle_off:           "tidak",
    ctrl_move:            "  \u{2191}\u{2193}\u{2190}\u{2192}   gerak",
    ctrl_goto:            "  Enter  pergi ke",
    ctrl_digit:           "  1-9    angka",
    ctrl_mode:            "  0      cat\u{21d4}sol",
    ctrl_scan:            "  s      pindai",
    ctrl_errors:          "  e      kesalahan",
    ctrl_undo:            "  u/^Z   batalkan",
    ctrl_redo:            "  r/^Y   ulang",
    ctrl_clear:           "  -      hapus",
    ctrl_pause:           "  Spc    jeda",
    ctrl_boss:            "  b      kunci bos",
    ctrl_quit:            "  Esc    keluar",
    ctrl_hint:                   "  h      hint",

    hint_full_house_name:        "Full House",
    hint_full_house_explain:     "Only one empty cell remains in this unit.",
    hint_naked_single_name:      "Naked Single",
    hint_naked_single_explain:   "Only {digit} fits in this cell.",
    hint_hidden_single_name:     "Hidden Single",
    hint_hidden_single_explain:  "{digit} can only go here in this unit.",
    hint_notes_name:             "Add Notes",
    hint_notes_explain:          "Add pencil marks in this unit to continue.",
    hint_naked_pairs_name:       "Naked Pairs",
    hint_naked_pairs_explain:    "These two cells hold {digit}. Eliminate from others.",
    hint_hidden_pairs_name:      "Hidden Pairs",
    hint_hidden_pairs_explain:   "Only these cells can hold this pair.",
    hint_pointing_pairs_name:    "Pointing Pairs",
    hint_pointing_pairs_explain: "{digit} in this box points to this row/col.",
    hint_box_line_name:          "Box-Line Reduction",
    hint_box_line_explain:       "{digit} in this row/col is confined to this box.",
    hint_reveal_name:            "Reveal",
    hint_reveal_explain:         "No logical move found. Filling most constrained cell.",
    dismiss:              "\u{2014} tekan tombol \u{2014}",
    resize_too_small:     "Terminal terlalu kecil: {}\u{d7}{}",
    resize_required:      "Diperlukan: {}\u{d7}{}",
    resize_hint:          "Silakan perbesar jendela ...",
    puzzle_invalid:       "Puzzle tidak valid: {}",
    puzzle_few_givens:    "Hanya {} sel diberikan (min. 17).",
    puzzle_no_solution:   "Puzzle tidak punya solusi.",
    puzzle_has_errors:    "Puzzle mengandung kesalahan.",
    puzzle_errors_hint:   "[e] untuk lihat kesalahan.",
    confirm_quit_title:   "Keluar permainan?",
    confirm_quit_options: "[Y]es  [N]o",
    resume_hint:          "[Space]  lanjutkan",
};

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_LANGUAGES: &[Language] = &[
        Language::English,
        Language::German,
        Language::Spanish,
        Language::Italian,
        Language::French,
        Language::Slovenian,
        Language::Esperanto,
        Language::TokiPona,
        Language::Leet,
        Language::Swahili,
        Language::Afrikaans,
        Language::Pinyin,
        Language::Indonesian,
    ];

    fn assert_fits(s: &str, ctx: &str) {
        let len = s.chars().count();
        assert!(
            len <= 34,                          // was 18
            "Panel string too long ({} chars) in {}:\n  '{}'",
            len, ctx, s
        );
    }

    /// Every panel and control string must fit within the 34-char inner panel
    /// width. Static strings are checked directly; dynamic strings are tested
    /// with their worst-case substituted values.
    #[test]
    fn all_panel_strings_fit_34_chars() {
        for &lang in ALL_LANGUAGES {
            let s  = lang.strings();
            let nm = format!("{lang:?}");

            // ── Static strings ───────────────────────────────────────────────
            for (field, val) in [
                ("panel_remaining", s.panel_remaining),
                ("panel_controls",  s.panel_controls),
                ("ctrl_move",       s.ctrl_move),
                ("ctrl_goto",       s.ctrl_goto),
                ("ctrl_digit",      s.ctrl_digit),
                ("ctrl_mode",       s.ctrl_mode),
                ("ctrl_scan",       s.ctrl_scan),
                ("ctrl_errors",     s.ctrl_errors),
                ("ctrl_undo",       s.ctrl_undo),
                ("ctrl_redo",       s.ctrl_redo),
                ("ctrl_clear",      s.ctrl_clear),
                ("ctrl_pause",      s.ctrl_pause),
                ("ctrl_boss",       s.ctrl_boss),
                ("ctrl_quit",       s.ctrl_quit),
                ("ctrl_hint",       s.ctrl_hint),
            ] {
                assert_fits(val, &format!("{nm}.{field}"));
            }

            // ── panel_time — worst case "99:59" ─────────────────────────────
            assert_fits(
                &s.panel_time.replacen("{}", "99:59", 1),
                &format!("{nm}.panel_time"),
            );

            // ── panel_mode — with mode_notes and mode_solution ───────────────
            for mode_val in [s.mode_notes, s.mode_solution] {
                assert_fits(
                    &s.panel_mode.replacen("{}", mode_val, 1),
                    &format!("{nm}.panel_mode+'{mode_val}'"),
                );
            }

            // ── panel_errors — with toggle_on and toggle_off ─────────────────
            for toggle in [s.toggle_on, s.toggle_off] {
                assert_fits(
                    &s.panel_errors.replacen("{}", toggle, 1),
                    &format!("{nm}.panel_errors+'{toggle}'"),
                );
            }

            // ── panel_count — worst case two-digit number ────────────────────
            assert_fits(
                &s.panel_count.replacen("{}", "81", 1),
                &format!("{nm}.panel_count"),
            );

            // ── panel_filled — worst case "81/81" ───────────────────────────
            assert_fits(
                &s.panel_filled.replacen("{}", "81", 1),
                &format!("{nm}.panel_filled"),
            );
        }
    }

    #[test]
    fn hint_strings_non_empty_for_en_and_de() {
        assert!(!EN.hint_naked_single_name.is_empty());
        assert!(!EN.hint_naked_single_explain.is_empty());
        assert!(!DE.hint_naked_single_name.is_empty());
        assert!(!DE.hint_naked_single_explain.is_empty());
        assert!(!EN.ctrl_hint.is_empty());
    }

    /// Every language index must round-trip through from_index / as_index.
    #[test]
    fn language_index_round_trips() {
        for &lang in ALL_LANGUAGES {
            assert_eq!(Language::from_index(lang.as_index()), lang);
        }
    }

    /// from_code must accept all documented codes case-insensitively.
    #[test]
    fn language_from_code_accepts_all_codes() {
        for (code, expected) in [
            ("en",   Language::English),
            ("DE",   Language::German),
            ("Es",   Language::Spanish),
            ("it",   Language::Italian),
            ("fr",   Language::French),
            ("sl",   Language::Slovenian),
            ("eo",   Language::Esperanto),
            ("tp",   Language::TokiPona),
            ("leet", Language::Leet),
            ("1337", Language::Leet),
            ("sw",   Language::Swahili),
            ("af",   Language::Afrikaans),
            ("py",   Language::Pinyin),
            ("id",   Language::Indonesian),
        ] {
            assert_eq!(
                Language::from_code(code),
                Some(expected),
                "from_code({code:?}) failed"
            );
        }
        assert_eq!(Language::from_code("xx"), None);
    }
}
