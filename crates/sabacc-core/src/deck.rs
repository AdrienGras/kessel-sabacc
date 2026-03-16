use rand::Rng;

use crate::card::{Card, Family};
use crate::error::GameError;

/// A deck for one card family (Sand or Blood).
///
/// Contains a draw pile and a discard pile. When the draw pile is exhausted,
/// the discard pile is reshuffled into it automatically.
#[derive(Debug, Clone, PartialEq)]
pub struct FamilyDeck {
    /// The family this deck belongs to.
    pub family: Family,
    /// Face-down draw pile. Last element is the top.
    pub draw_pile: Vec<Card>,
    /// Face-up discard pile. Last element is the top (visible).
    pub discard_pile: Vec<Card>,
}

impl FamilyDeck {
    /// Create a new deck for the given family with all 22 cards, shuffled.
    ///
    /// Each family deck contains:
    /// - 3 copies of each numbered card (1-6) = 18 cards
    /// - 2 Sylops = 2 cards
    /// - 2 Impostors = 2 cards
    /// - Total: 22 cards
    pub fn new(family: Family, rng: &mut impl Rng) -> Self {
        let mut cards = Vec::with_capacity(22);

        // 3 copies of each number 1-6
        for n in 1..=6 {
            for _ in 0..3 {
                cards.push(Card::number(family, n));
            }
        }

        // 2 Sylops
        for _ in 0..2 {
            cards.push(Card::sylop(family));
        }

        // 2 Impostors
        for _ in 0..2 {
            cards.push(Card::impostor(family));
        }

        shuffle(&mut cards, rng);

        Self {
            family,
            draw_pile: cards,
            discard_pile: Vec::new(),
        }
    }

    /// Draw the top card from the draw pile.
    ///
    /// If the draw pile is empty, reshuffles the discard pile into it first.
    /// Returns an error if both piles are empty.
    pub fn draw(&mut self, rng: &mut impl Rng) -> Result<Card, GameError> {
        if self.draw_pile.is_empty() {
            self.reshuffle(rng)?;
        }
        self.draw_pile
            .pop()
            .ok_or(GameError::DeckExhausted { family: self.family })
    }

    /// Draw the top card from the discard pile (face-up draw).
    pub fn draw_from_discard(&mut self) -> Result<Card, GameError> {
        self.discard_pile
            .pop()
            .ok_or(GameError::DiscardEmpty { family: self.family })
    }

    /// Place a card on top of the discard pile.
    pub fn discard(&mut self, card: Card) {
        self.discard_pile.push(card);
    }

    /// Peek at the top card of the discard pile without removing it.
    pub fn peek_discard(&self) -> Option<&Card> {
        self.discard_pile.last()
    }

    /// Reshuffle the discard pile into the draw pile.
    fn reshuffle(&mut self, rng: &mut impl Rng) -> Result<(), GameError> {
        if self.discard_pile.is_empty() {
            return Err(GameError::DeckExhausted { family: self.family });
        }
        self.draw_pile.append(&mut self.discard_pile);
        shuffle(&mut self.draw_pile, rng);
        Ok(())
    }

    /// Return the total number of cards in this deck (draw + discard).
    pub fn total_cards(&self) -> usize {
        self.draw_pile.len() + self.discard_pile.len()
    }
}

/// Fisher-Yates shuffle.
fn shuffle(cards: &mut [Card], rng: &mut impl Rng) {
    let len = cards.len();
    for i in (1..len).rev() {
        let j = rng.gen_range(0..=i);
        cards.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::CardValue;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn test_rng() -> SmallRng {
        SmallRng::seed_from_u64(42)
    }

    #[test]
    fn deck_has_22_cards() {
        let mut rng = test_rng();
        let deck = FamilyDeck::new(Family::Sand, &mut rng);
        assert_eq!(deck.total_cards(), 22);
        assert_eq!(deck.draw_pile.len(), 22);
        assert_eq!(deck.discard_pile.len(), 0);
    }

    #[test]
    fn deck_card_distribution() {
        let mut rng = test_rng();
        let deck = FamilyDeck::new(Family::Blood, &mut rng);

        let numbers: Vec<_> = deck
            .draw_pile
            .iter()
            .filter_map(|c| match c.value {
                CardValue::Number(n) => Some(n),
                _ => None,
            })
            .collect();
        assert_eq!(numbers.len(), 18);

        for n in 1..=6u8 {
            assert_eq!(numbers.iter().filter(|&&v| v == n).count(), 3);
        }

        let sylops = deck
            .draw_pile
            .iter()
            .filter(|c| c.value == CardValue::Sylop)
            .count();
        assert_eq!(sylops, 2);

        let impostors = deck
            .draw_pile
            .iter()
            .filter(|c| c.value == CardValue::Impostor)
            .count();
        assert_eq!(impostors, 2);
    }

    #[test]
    fn draw_reduces_pile() {
        let mut rng = test_rng();
        let mut deck = FamilyDeck::new(Family::Sand, &mut rng);
        let _ = deck.draw(&mut rng);
        assert_eq!(deck.draw_pile.len(), 21);
    }

    #[test]
    fn reshuffle_on_empty_draw() {
        let mut rng = test_rng();
        let mut deck = FamilyDeck::new(Family::Sand, &mut rng);

        // Draw all cards
        let mut cards = Vec::new();
        for _ in 0..22 {
            cards.push(deck.draw(&mut rng).unwrap());
        }

        // Discard some back
        for card in cards.into_iter().take(5) {
            deck.discard(card);
        }

        // Should reshuffle from discard
        let result = deck.draw(&mut rng);
        assert!(result.is_ok());
        assert_eq!(deck.total_cards(), 4); // 5 discarded, drew 1
    }

    #[test]
    fn error_when_both_piles_empty() {
        let mut rng = test_rng();
        let mut deck = FamilyDeck::new(Family::Sand, &mut rng);

        // Drain all cards
        for _ in 0..22 {
            let _ = deck.draw(&mut rng);
        }

        let result = deck.draw(&mut rng);
        assert!(matches!(result, Err(GameError::DeckExhausted { .. })));
    }

    #[test]
    fn draw_from_discard() {
        let mut rng = test_rng();
        let mut deck = FamilyDeck::new(Family::Sand, &mut rng);

        let card = deck.draw(&mut rng).unwrap();
        let card_clone = card.clone();
        deck.discard(card);

        let drawn = deck.draw_from_discard().unwrap();
        assert_eq!(drawn, card_clone);
    }

    #[test]
    fn empty_discard_error() {
        let mut rng = test_rng();
        let deck = FamilyDeck::new(Family::Sand, &mut rng);
        // Discard is empty at creation
        let mut deck = deck;
        let result = deck.draw_from_discard();
        assert!(matches!(result, Err(GameError::DiscardEmpty { .. })));
    }
}
