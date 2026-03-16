use crate::card::Family;

/// Source from which a player can draw a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawSource {
    /// Draw from the Sand deck (face-down).
    SandDeck,
    /// Draw from the Blood deck (face-down).
    BloodDeck,
    /// Draw from the Sand discard pile (face-up, top card).
    SandDiscard,
    /// Draw from the Blood discard pile (face-up, top card).
    BloodDiscard,
}

impl DrawSource {
    /// Get the family associated with this draw source.
    pub fn family(&self) -> Family {
        match self {
            DrawSource::SandDeck | DrawSource::SandDiscard => Family::Sand,
            DrawSource::BloodDeck | DrawSource::BloodDiscard => Family::Blood,
        }
    }

    /// Whether this source draws from the discard pile.
    pub fn is_discard(&self) -> bool {
        matches!(self, DrawSource::SandDiscard | DrawSource::BloodDiscard)
    }
}

/// What to do with the drawn card: keep it or discard it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscardChoice {
    /// Keep the drawn card, discard the card currently in hand.
    KeepDrawn,
    /// Discard the drawn card, keep the card currently in hand.
    DiscardDrawn,
}

/// An action a player can take during their turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnAction {
    /// Draw a card from the specified source. Costs 1 chip.
    Draw(DrawSource),
    /// Stand: do nothing. Free.
    Stand,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_source_family() {
        assert_eq!(DrawSource::SandDeck.family(), Family::Sand);
        assert_eq!(DrawSource::BloodDeck.family(), Family::Blood);
        assert_eq!(DrawSource::SandDiscard.family(), Family::Sand);
        assert_eq!(DrawSource::BloodDiscard.family(), Family::Blood);
    }

    #[test]
    fn draw_source_is_discard() {
        assert!(!DrawSource::SandDeck.is_discard());
        assert!(!DrawSource::BloodDeck.is_discard());
        assert!(DrawSource::SandDiscard.is_discard());
        assert!(DrawSource::BloodDiscard.is_discard());
    }
}
