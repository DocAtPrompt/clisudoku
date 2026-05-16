#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
    /// Requires at least one Tier-2 expert technique (Jellyfish, Skyscraper, XY-Chain, …).
    /// The Expert solver can solve it; the Extreme solver cannot.
    Expert,
    /// Maximally-reduced puzzle: as few givens as possible (targeting 17),
    /// solved using full backtracking — no strategy cap.
    BareMinimum,
}

use crate::solver::Strategy;

impl Difficulty {
    pub fn to_db_str(&self) -> &'static str {
        match self {
            Difficulty::Easy        => "Easy",
            Difficulty::Medium      => "Medium",
            Difficulty::Hard        => "Hard",
            Difficulty::Extreme     => "Extreme",
            Difficulty::Expert      => "Expert",
            Difficulty::BareMinimum => "Sparse",
        }
    }
}

/// Classify a solved puzzle by the hardest strategy used.
///
/// Priority (highest first): Expert > Extreme (Swordfish/Backtracking) > Hard > Medium > Easy.
/// `Expert` is checked first because a puzzle requiring Tier-2 techniques (Jellyfish,
/// Skyscraper, XY-Chain, …) cannot be solved by the Extreme solver at all. When both
/// `Strategy::Expert` and `Strategy::Swordfish` appear, `Expert` wins.
pub fn classify(used: &[Strategy]) -> Difficulty {
    let needs = |s: Strategy| used.contains(&s);
    if needs(Strategy::Expert) {
        Difficulty::Expert
    } else if needs(Strategy::Swordfish) || needs(Strategy::Backtracking) {
        Difficulty::Extreme
    } else if needs(Strategy::XWing)
        || needs(Strategy::HiddenPair)
        || needs(Strategy::NakedTriple)
        || needs(Strategy::BoxLineReduction)
    {
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

    #[test]
    fn swordfish_classifies_as_extreme() {
        let used = vec![Strategy::NakedSingle, Strategy::Swordfish];
        assert_eq!(classify(&used), Difficulty::Extreme);
    }

    #[test]
    fn backtracking_classifies_as_extreme() {
        let used = vec![Strategy::NakedSingle, Strategy::Backtracking];
        assert_eq!(classify(&used), Difficulty::Extreme);
    }

    #[test]
    fn x_wing_alone_classifies_as_hard() {
        let used = vec![Strategy::NakedSingle, Strategy::XWing];
        assert_eq!(classify(&used), Difficulty::Hard);
    }

    #[test]
    fn expert_strategy_classifies_as_expert() {
        let used = vec![Strategy::NakedSingle, Strategy::Expert];
        assert_eq!(classify(&used), Difficulty::Expert);
    }

    #[test]
    fn expert_dominates_swordfish_in_combined_use() {
        let used = vec![Strategy::NakedSingle, Strategy::Swordfish, Strategy::Expert];
        assert_eq!(classify(&used), Difficulty::Expert);
    }

    #[test]
    fn to_db_str_covers_all_variants() {
        assert_eq!(Difficulty::Easy.to_db_str(), "Easy");
        assert_eq!(Difficulty::Medium.to_db_str(), "Medium");
        assert_eq!(Difficulty::Hard.to_db_str(), "Hard");
        assert_eq!(Difficulty::Extreme.to_db_str(), "Extreme");
        assert_eq!(Difficulty::Expert.to_db_str(), "Expert");
        assert_eq!(Difficulty::BareMinimum.to_db_str(), "Sparse");
    }
}
