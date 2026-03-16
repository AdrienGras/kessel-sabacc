use crate::card::{Card, Family};
use crate::error::GameError;

/// A player's hand: exactly one Sand card and one Blood card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hand {
    /// The Sand family card.
    pub sand: Card,
    /// The Blood family card.
    pub blood: Card,
}

impl Hand {
    /// Create a new hand, validating that sand is Sand and blood is Blood.
    pub fn new(sand: Card, blood: Card) -> Result<Self, GameError> {
        if sand.family != Family::Sand || blood.family != Family::Blood {
            return Err(GameError::InvalidHand {
                sand_family: sand.family,
                blood_family: blood.family,
            });
        }
        Ok(Self { sand, blood })
    }
}

/// The ranking of a hand, from strongest to weakest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandRank {
    /// Two Sylops (Sand Sylop + Blood Sylop) — strongest hand.
    PureSabacc,
    /// One Sylop + one numbered card — the Sylop copies the number, yielding 0 difference.
    SylopSabacc {
        /// The value of the numbered card.
        value: u8,
    },
    /// Two cards with the same numeric value — 0 difference.
    /// Lower pair_value beats higher (1/1 > 6/6).
    Sabacc {
        /// The matched pair value.
        pair_value: u8,
    },
    /// Cards with different values — ranked by how close to 0 the difference is.
    NonSabacc {
        /// Absolute difference between the two card values.
        difference: u8,
    },
}

impl HandRank {
    /// Return a comparable key where lower values are stronger hands.
    ///
    /// Layout: (tier, sub_value)
    /// - PureSabacc: (0, 0)
    /// - SylopSabacc: (1, value)
    /// - Sabacc: (2, pair_value)
    /// - NonSabacc: (3, difference)
    pub fn strength_key(&self) -> (u8, u8) {
        match self {
            HandRank::PureSabacc => (0, 0),
            HandRank::SylopSabacc { value } => (1, *value),
            HandRank::Sabacc { pair_value } => (2, *pair_value),
            HandRank::NonSabacc { difference } => (3, *difference),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_hand() {
        let hand = Hand::new(
            Card::number(Family::Sand, 3),
            Card::number(Family::Blood, 5),
        );
        assert!(hand.is_ok());
    }

    #[test]
    fn invalid_hand_swapped_families() {
        let hand = Hand::new(
            Card::number(Family::Blood, 3),
            Card::number(Family::Sand, 5),
        );
        assert!(hand.is_err());
    }

    #[test]
    fn strength_ordering() {
        let pure = HandRank::PureSabacc;
        let sylop = HandRank::SylopSabacc { value: 3 };
        let sabacc_low = HandRank::Sabacc { pair_value: 1 };
        let sabacc_high = HandRank::Sabacc { pair_value: 6 };
        let non_sabacc = HandRank::NonSabacc { difference: 2 };

        assert!(pure.strength_key() < sylop.strength_key());
        assert!(sylop.strength_key() < sabacc_low.strength_key());
        assert!(sabacc_low.strength_key() < sabacc_high.strength_key());
        assert!(sabacc_high.strength_key() < non_sabacc.strength_key());
    }

    #[test]
    fn sabacc_1_beats_sabacc_6() {
        let s1 = HandRank::Sabacc { pair_value: 1 };
        let s6 = HandRank::Sabacc { pair_value: 6 };
        assert!(s1.strength_key() < s6.strength_key());
    }
}
