// src/tui/seq_detect.rs
//
// Detects typed key sequences for easter eggs.
// Maintains a rolling buffer of the last N lowercase chars;
// after each push, checks for any registered sequence.

use std::collections::VecDeque;

const BUF_SIZE: usize = 10;

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
}

pub struct SeqDetector {
    buf: VecDeque<char>,
}

impl Default for SeqDetector {
    fn default() -> Self {
        Self { buf: VecDeque::with_capacity(BUF_SIZE) }
    }
}

impl SeqDetector {
    /// Push a raw character and return any triggered easter egg.
    pub fn push(&mut self, c: char) -> Option<EasterEgg> {
        self.buf.push_back(c.to_ascii_lowercase());
        if self.buf.len() > BUF_SIZE {
            self.buf.pop_front();
        }
        self.check()
    }

    fn tail_matches(&self, seq: &str) -> bool {
        let n = seq.len();
        if self.buf.len() < n { return false; }
        let start = self.buf.len() - n;
        self.buf.iter().skip(start).zip(seq.chars()).all(|(a, b)| *a == b)
    }

    fn check(&mut self) -> Option<EasterEgg> {
        let egg = if      self.tail_matches("iddqd") { Some(EasterEgg::GodMode)  }
                  else if self.tail_matches("idkfa") { Some(EasterEgg::FillNotes) }
                  else if self.tail_matches("xyzzy") { Some(EasterEgg::Xyzzy)    }
                  else if self.tail_matches("sudo")  { Some(EasterEgg::Sudo)      }
                  else if self.tail_matches("help")  { Some(EasterEgg::Help)      }
                  else if self.tail_matches("42")    { Some(EasterEgg::FortyTwo)  }
                  else                               { None                       };
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

    #[test] fn iddqd()   { assert_eq!(detect("iddqd"), Some(EasterEgg::GodMode));   }
    #[test] fn idkfa()   { assert_eq!(detect("idkfa"), Some(EasterEgg::FillNotes)); }
    #[test] fn xyzzy()   { assert_eq!(detect("xyzzy"), Some(EasterEgg::Xyzzy));     }
    #[test] fn sudo()    { assert_eq!(detect("sudo"),  Some(EasterEgg::Sudo));       }
    #[test] fn help()    { assert_eq!(detect("help"),  Some(EasterEgg::Help));       }
    #[test] fn fortytwo(){ assert_eq!(detect("42"),    Some(EasterEgg::FortyTwo));   }

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
