// src/pattern/mod.rs

/// A visual pattern mask for Designer Sudoku generation.
///
/// `mask[r * 9 + c]` is `true` when cell (r, c) may be a given.
#[derive(Debug, Clone)]
pub struct Pattern {
    pub name_en:    &'static str,
    pub name_de:    &'static str,
    pub mask:       [bool; 81],
    pub cell_count: usize,
}

impl Pattern {
    /// Parse an 81-char CLI string:
    ///   '1' or '*' → pattern cell (may be given)
    ///   '.' or '0' → always empty
    pub fn from_cli_str(s: &str) -> Result<Self, String> {
        if s.chars().count() != 81 {
            return Err(format!(
                "Pattern must be exactly 81 characters, got {}",
                s.chars().count()
            ));
        }
        let mut mask = [false; 81];
        for (i, ch) in s.chars().enumerate() {
            mask[i] = matches!(ch, '1' | '*');
        }
        let cell_count = mask.iter().filter(|&&b| b).count();
        Ok(Pattern {
            name_en: "Custom",
            name_de: "Benutzerdefiniert",
            mask,
            cell_count,
        })
    }
}

// ── Internal mask builder (used only in const context below) ──────────────────

const fn mask_from_bytes(s: &[u8; 81]) -> [bool; 81] {
    let mut m = [false; 81];
    let mut i = 0;
    while i < 81 {
        m[i] = s[i] == b'1';
        i += 1;
    }
    m
}

// Helper: count true bits in a const mask
const fn count_bits(m: &[bool; 81]) -> usize {
    let mut n = 0;
    let mut i = 0;
    while i < 81 { if m[i] { n += 1; } i += 1; }
    n
}

macro_rules! pat {
    ($en:expr, $de:expr, $bits:expr) => {{
        const M: [bool; 81] = mask_from_bytes($bits);
        Pattern { name_en: $en, name_de: $de, mask: M, cell_count: count_bits(&M) }
    }};
}

// ── 28 built-in patterns, sorted by cell_count descending ─────────────────────

pub static PATTERNS: &[Pattern] = &[
    // 1. Holy Crap — 46
    pat!("Holy Crap",    "Heilige Kuh",
         b"001101100110000011111101111100000001101111101111010111001111100011111110010000010"),
    // 2. Checker — 41
    pat!("Checker",      "Schachbrett",
         b"101010101010101010101010101010101010101010101010101010101010101010101010101010101"),
    // 3. Rudolph — 41
    pat!("Rudolph",      "Rudolf",
         b"100001000111111000001100000011100000001111110001111111001111111001010101001010101"),
    // 4. Bug or Feature? — 41
    pat!("Bug or Feature?", "Bug oder Feature?",
         b"000101000110111011011111110000101000011101110000101000011111110110111011000010000"),
    // 5. Heart — 40
    pat!("Heart",        "Herz",
         b"011000110111101111110111011110010011110000011011000110001101100000111000000010000"),
    // 6. Wind Up — 40
    pat!("Wind Up",      "Aufziehen",
         b"000010000111101111000010000001111000011011001010011111011111100001001000011111110"),
    // 7. Shit Happens — 39
    pat!("Shit Happens", "Shit Happens",
         b"000001000010000000000110010001110000001001100011111110011111110011100011111111111"),
    // 8. Ripples — 37
    pat!("Ripples",      "Wellen",
         b"000111000011000110010111010101000101101010101101000101010111010011000110000111000"),
    // 9. Mamihlapinatapai — 36
    pat!("Mamihlapinatapai", "Mamihlapinatapai",
         b"011000110000000000011000110100101001100101001110101101110101101100101001011000110"),
    // 10. Fire Fighter — 36
    pat!("Fire Fighter",  "Feuerwehr",
         b"000111000001101100011000110010010010111111111010000010010101010010000010001111100"),
    // 11. Asterisk — 33
    pat!("Asterisk",     "Stern",
         b"100010001010010010001010100000111000111111111000111000001010100010010010100010001"),
    // 12. Border — 32
    pat!("Border",       "Rahmen",
         b"111111111100000001100000001100000001100000001100000001100000001100000001111111111"),
    // 13. Rainy Day — 32
    pat!("Rainy Day",    "Regentag",
         b"000111000011111110010010010111111111101010101000010000000010000000010000000110000"),
    // 14. Smiley — 31
    pat!("Smiley",       "Smiley",
         b"001111100010000010100101001100000001100000001101000101100111001010000010001111100"),
    // 15. Badley — 31
    pat!("Badley",       "Traurig",
         b"001111100010000010100101001100000001100000001100111001101000101010000010001111100"),
    // 16. Bigger Fish to Fry — 31
    pat!("Bigger Fish to Fry", "Größere Sorgen",
         b"000100000011111001110001011101001110100000010100000110010001011001110001000001000"),
    // 17. Anchor — 30
    pat!("Anchor",       "Anker",
         b"000111000000111000000010000001111100000010000010010010111010111010010010001101100"),
    // 18. Lost My Cherries — 30
    pat!("Lost My Cherries", "Meine Kirschen",
         b"000000111000001100000011100000110100000100100011100110100101001100101001011000110"),
    // 19. Joshua — 29
    pat!("Joshua",       "Josua",
         b"000010000000111000000101000010101000011101010000101110000101110000101000001111100"),
    // 20. Five to Twelve — 29
    pat!("Five to Twelve", "Fünf vor Zwölf",
         b"000111000011010110010110010100010001100010001100000001010000010011000110000111000"),
    // 21. Diamond — 28
    pat!("Diamond",      "Diamant",
         b"010010010100101001001000100010010010100101001010010010001000100100101001010010010"),
    // 22. An Apple a Day — 28
    pat!("An Apple a Day", "Täglich ein Apfel",
         b"000001000000010000011111110110000011100100001101000001100000001010000010001111100"),
    // 23. Wave — 27
    pat!("Wave",         "Welle",
         b"001001001010010010100100100010010010001001001010010010100100100010010010001001001"),
    // 24. Per Aspera ad Astra — 27
    pat!("Per Aspera ad Astra", "Per Aspera ad Astra",
         b"000010000000111000000101000000111000000101000000111000001111100001010100011010110"),
    // 25. Minion — 27
    pat!("Minion",       "Minion",
         b"000111000001000100010010010010101010010010010010000010010111010001000100000111000"),
    // 26. 42 — 26
    pat!("42",           "42",
         b"000000000101000110101001001101000001101000010111100100001001000001001111000000000"),
    // 27. Home Office — 26
    pat!("Home Office",  "Home Office",
         b"000010000000111000001101100110000011010000010010000010010001010010101010010100010"),
    // 28. We Are Connected Wirelessly — 22
    pat!("We Are Connected Wirelessly", "Kabellos verbunden",
         b"001111100010000010100000001001111100010000010000000000000111000001000100000010000"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_count_is_28() {
        assert_eq!(PATTERNS.len(), 28);
    }

    #[test]
    fn all_patterns_have_at_least_17_cells() {
        for p in PATTERNS {
            assert!(
                p.cell_count >= 17,
                "Pattern '{}' has only {} cells — below the minimum of 17",
                p.name_en, p.cell_count
            );
        }
    }

    #[test]
    fn patterns_sorted_descending_by_cell_count() {
        for i in 1..PATTERNS.len() {
            assert!(
                PATTERNS[i - 1].cell_count >= PATTERNS[i].cell_count,
                "Sort broken at index {}: {} ({}) should be >= {} ({})",
                i,
                PATTERNS[i - 1].name_en, PATTERNS[i - 1].cell_count,
                PATTERNS[i].name_en,     PATTERNS[i].cell_count
            );
        }
    }

    #[test]
    fn cell_count_matches_mask() {
        for p in PATTERNS {
            let counted = p.mask.iter().filter(|&&b| b).count();
            assert_eq!(
                counted, p.cell_count,
                "Pattern '{}': declared cell_count={} but mask has {} true bits",
                p.name_en, p.cell_count, counted
            );
        }
    }

    #[test]
    fn from_cli_str_valid() {
        let s = "1".repeat(81);
        let p = Pattern::from_cli_str(&s).unwrap();
        assert_eq!(p.cell_count, 81);
        assert!(p.mask.iter().all(|&b| b));
    }

    #[test]
    fn from_cli_str_too_short() {
        assert!(Pattern::from_cli_str("111").is_err());
    }

    #[test]
    fn from_cli_str_too_long() {
        assert!(Pattern::from_cli_str(&"1".repeat(82)).is_err());
    }

    #[test]
    fn from_cli_str_accepts_dots_and_stars() {
        let s = "*".repeat(40) + &".".repeat(41);
        let p = Pattern::from_cli_str(&s).unwrap();
        assert_eq!(p.cell_count, 40);
    }

    #[test]
    fn from_cli_str_accepts_zeros() {
        let s = "1".repeat(40) + &"0".repeat(41);
        let p = Pattern::from_cli_str(&s).unwrap();
        assert_eq!(p.cell_count, 40);
    }
}
