#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

use crate::solver::Strategy;

pub fn classify(used: &[Strategy]) -> Difficulty {
    let needs = |s: Strategy| used.contains(&s);
    if needs(Strategy::XWing) || needs(Strategy::HiddenPair)
        || needs(Strategy::NakedTriple) || needs(Strategy::BoxLineReduction) {
        Difficulty::Hard
    } else if needs(Strategy::NakedPair) || needs(Strategy::PointingPair) {
        Difficulty::Medium
    } else {
        Difficulty::Easy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::Strategy;

    #[test]
    fn easy_uses_only_singles() {
        let used = vec![Strategy::NakedSingle, Strategy::HiddenSingle];
        assert_eq!(classify(&used), Difficulty::Easy);
    }

    #[test]
    fn medium_uses_naked_pair() {
        let used = vec![Strategy::NakedSingle, Strategy::NakedPair];
        assert_eq!(classify(&used), Difficulty::Medium);
    }

    #[test]
    fn hard_uses_x_wing() {
        let used = vec![Strategy::NakedSingle, Strategy::XWing];
        assert_eq!(classify(&used), Difficulty::Hard);
    }
}
