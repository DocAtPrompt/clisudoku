// src/tui/seq_detect.rs
//
// Detects typed key sequences for easter eggs.
// Maintains a rolling buffer of the last N lowercase chars;
// after each push, checks for any registered sequence.
//
// Also tracks the Konami Code (↑↑↓↓←→←→) via directional inputs.

use std::collections::VecDeque;

const BUF_SIZE: usize = 20; // longest sequence: "zweiundvierzig" (14 chars)

/// Easter egg triggered by a typed sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EasterEgg {
    /// `iddqd` — Doom God Mode: fill all cells with the correct solution.
    GodMode,
    /// `idkfa` — Doom ammo cheat: fill correct notes into all empty cells.
    FillNotes,
    /// `xyzzy` — classic Adventure no-op: "Nothing happens."
    Xyzzy,
    /// `sudo`  — sudoers message.
    Sudo,
    /// `help`  — not a text adventure.
    Help,
    /// `42`    — Life, the Universe, and Everything.
    FortyTwo,
    /// ↑↑↓↓←→←→ — Konami Code: visual grid animation.
    KonamiCode,
    /// `matrix` — toggle Matrix Mode (green digit rendering).
    MatrixMode,
}

/// Arrow-key direction, used for Konami Code detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { Up, Down, Left, Right }

/// Konami Code sequence: ↑↑↓↓←→←→
const KONAMI: [Direction; 8] = [
    Direction::Up,    Direction::Up,
    Direction::Down,  Direction::Down,
    Direction::Left,  Direction::Right,
    Direction::Left,  Direction::Right,
];

pub struct SeqDetector {
    buf:          VecDeque<char>,
    konami_pos:   usize,    // progress through KONAMI sequence (0..=8)
}

impl Default for SeqDetector {
    fn default() -> Self {
        Self {
            buf:        VecDeque::with_capacity(BUF_SIZE),
            konami_pos: 0,
        }
    }
}

impl SeqDetector {
    /// Push a raw character and return any triggered easter egg.
    pub fn push(&mut self, c: char) -> Option<EasterEgg> {
        self.buf.push_back(c.to_ascii_lowercase());
        if self.buf.len() > BUF_SIZE {
            self.buf.pop_front();
        }
        // Any char keystroke resets Konami progress (Konami is arrow-keys only).
        self.konami_pos = 0;
        self.check()
    }

    /// Push a directional key and return any triggered easter egg.
    ///
    /// Char-based sequences are not checked here — only the Konami Code.
    pub fn push_direction(&mut self, dir: Direction) -> Option<EasterEgg> {
        if KONAMI[self.konami_pos] == dir {
            self.konami_pos += 1;
            if self.konami_pos == KONAMI.len() {
                self.konami_pos = 0;
                return Some(EasterEgg::KonamiCode);
            }
        } else {
            // Wrong key — restart from 0, but still check if this key starts the sequence.
            self.konami_pos = 0;
            if KONAMI[0] == dir {
                self.konami_pos = 1;
            }
        }
        None
    }

    fn tail_matches(&self, seq: &str) -> bool {
        let n = seq.len();
        if self.buf.len() < n { return false; }
        let start = self.buf.len() - n;
        self.buf.iter().skip(start).zip(seq.chars()).all(|(a, b)| *a == b)
    }

    fn check(&mut self) -> Option<EasterEgg> {
        let egg = if      self.tail_matches("iddqd")          { Some(EasterEgg::GodMode)   }
                  else if self.tail_matches("idkfa")          { Some(EasterEgg::FillNotes)  }
                  else if self.tail_matches("xyzzy")          { Some(EasterEgg::Xyzzy)      }
                  else if self.tail_matches("sudo")           { Some(EasterEgg::Sudo)        }
                  else if self.tail_matches("help")           { Some(EasterEgg::Help)        }
                  else if self.tail_matches("fortytwo")       { Some(EasterEgg::FortyTwo)   }
                  else if self.tail_matches("zweiundvierzig") { Some(EasterEgg::FortyTwo)   }
                  else if self.tail_matches("matrix")         { Some(EasterEgg::MatrixMode) }
                  else                                        { None                        };
        if egg.is_some() { self.buf.clear(); }
        egg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn detect(seq: &str) -> Option<EasterEgg> {
        let mut d = SeqDetector::default();
        let mut result = None;
        for c in seq.chars() { result = d.push(c); }
        result
    }

    fn detect_dirs(dirs: &[Direction]) -> Option<EasterEgg> {
        let mut d = SeqDetector::default();
        let mut result = None;
        for &dir in dirs { result = d.push_direction(dir); }
        result
    }

    #[test] fn iddqd()   { assert_eq!(detect("iddqd"), Some(EasterEgg::GodMode));   }
    #[test] fn idkfa()   { assert_eq!(detect("idkfa"), Some(EasterEgg::FillNotes)); }
    #[test] fn xyzzy()   { assert_eq!(detect("xyzzy"), Some(EasterEgg::Xyzzy));     }
    #[test] fn sudo()    { assert_eq!(detect("sudo"),  Some(EasterEgg::Sudo));       }
    #[test] fn help()    { assert_eq!(detect("help"),  Some(EasterEgg::Help));       }
    #[test] fn fortytwo_en() { assert_eq!(detect("fortytwo"),       Some(EasterEgg::FortyTwo)); }
    #[test] fn fortytwo_de() { assert_eq!(detect("zweiundvierzig"), Some(EasterEgg::FortyTwo)); }
    #[test] fn matrix()  { assert_eq!(detect("matrix"), Some(EasterEgg::MatrixMode)); }

    #[test]
    fn konami_full_sequence() {
        use Direction::*;
        assert_eq!(
            detect_dirs(&[Up, Up, Down, Down, Left, Right, Left, Right]),
            Some(EasterEgg::KonamiCode)
        );
    }

    #[test]
    fn konami_wrong_key_resets() {
        use Direction::*;
        // Interrupt after 3 correct keys, then complete from scratch
        let mut d = SeqDetector::default();
        for dir in [Up, Up, Down, Left] { d.push_direction(dir); }
        // Now start over correctly
        let mut result = None;
        for dir in [Up, Up, Down, Down, Left, Right, Left, Right] {
            result = d.push_direction(dir);
        }
        assert_eq!(result, Some(EasterEgg::KonamiCode));
    }

    #[test]
    fn konami_char_resets_progress() {
        use Direction::*;
        let mut d = SeqDetector::default();
        // Start Konami
        d.push_direction(Up);
        d.push_direction(Up);
        // Type a char — should reset konami progress
        d.push('a');
        // Complete sequence from scratch
        let mut result = None;
        for dir in [Up, Up, Down, Down, Left, Right, Left, Right] {
            result = d.push_direction(dir);
        }
        assert_eq!(result, Some(EasterEgg::KonamiCode));
    }

    #[test]
    fn no_false_positive() {
        assert_eq!(detect("hello"), None);
        assert_eq!(detect("idxxx"), None);
    }

    #[test]
    fn detects_after_garbage_prefix() {
        // Sequence embedded after unrelated chars.
        assert_eq!(detect("abciddqd"), Some(EasterEgg::GodMode));
    }

    #[test]
    fn clears_after_trigger() {
        let mut d = SeqDetector::default();
        for c in "iddqd".chars() { d.push(c); }
        // Buffer cleared — same sequence must be typed in full again.
        assert_eq!(d.push('d'), None);
    }
}
