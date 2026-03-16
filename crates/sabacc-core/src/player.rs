use crate::error::GameError;
use crate::hand::Hand;
use crate::shift_token::ShiftToken;
use crate::PlayerId;

/// A player in the Sabacc game.
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    /// Unique player identifier.
    pub id: PlayerId,
    /// Display name.
    pub name: String,
    /// Remaining chips (reserve).
    pub chips: u8,
    /// Chips invested in the current round.
    pub pot: u8,
    /// The player's current hand (None before dealing).
    pub hand: Option<Hand>,
    /// Available shift tokens (always empty in Phase 1).
    pub shift_tokens: Vec<ShiftToken>,
    /// Whether this player has been eliminated.
    pub is_eliminated: bool,
    /// Whether this player is a bot.
    pub is_bot: bool,
}

impl Player {
    /// Create a new player with the given id, name, and starting chips.
    pub fn new(id: PlayerId, name: String, chips: u8, is_bot: bool) -> Self {
        Self {
            id,
            name,
            chips,
            pot: 0,
            hand: None,
            shift_tokens: Vec::new(),
            is_eliminated: false,
            is_bot,
        }
    }

    /// Pay one chip from reserve to pot (e.g., for drawing).
    pub fn pay_chip(&mut self) -> Result<(), GameError> {
        if self.chips == 0 {
            return Err(GameError::InsufficientChips {
                player_id: self.id,
                available: 0,
                required: 1,
            });
        }
        self.chips -= 1;
        self.pot += 1;
        Ok(())
    }

    /// Return all invested chips back to reserve (winner of the round).
    pub fn return_invested(&mut self) {
        self.chips += self.pot;
        self.pot = 0;
    }

    /// Apply a penalty: lose `amount` chips from reserve. Chips are destroyed.
    /// If the player doesn't have enough chips, they lose all remaining
    /// and are eliminated.
    pub fn apply_penalty(&mut self, amount: u8) {
        if amount >= self.chips {
            self.chips = 0;
            self.is_eliminated = true;
        } else {
            self.chips -= amount;
        }
    }

    /// Total chips this player has (reserve + invested).
    pub fn total_chips(&self) -> u8 {
        self.chips + self.pot
    }

    /// Check if the player has a token of the given type.
    pub fn has_token(&self, token: &ShiftToken) -> bool {
        self.shift_tokens.iter().any(|t| t.matches_type(token))
    }

    /// Remove a token of the given type from the player's inventory.
    pub fn remove_token(&mut self, token: &ShiftToken) -> Result<(), GameError> {
        let pos = self
            .shift_tokens
            .iter()
            .position(|t| t.matches_type(token));
        match pos {
            Some(idx) => {
                self.shift_tokens.remove(idx);
                Ok(())
            }
            None => Err(GameError::ShiftTokenNotOwned {
                player_id: self.id,
            }),
        }
    }

    /// Refund chips from pot back to reserve. Returns actual amount refunded.
    pub fn refund_chips(&mut self, amount: u8) -> u8 {
        let refunded = amount.min(self.pot);
        self.pot -= refunded;
        self.chips += refunded;
        refunded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_player() {
        let p = Player::new(0, "Alice".into(), 6, false);
        assert_eq!(p.id, 0);
        assert_eq!(p.chips, 6);
        assert_eq!(p.pot, 0);
        assert!(!p.is_eliminated);
        assert!(!p.is_bot);
    }

    #[test]
    fn pay_chip_success() {
        let mut p = Player::new(0, "Alice".into(), 6, false);
        assert!(p.pay_chip().is_ok());
        assert_eq!(p.chips, 5);
        assert_eq!(p.pot, 1);
    }

    #[test]
    fn pay_chip_insufficient() {
        let mut p = Player::new(0, "Alice".into(), 0, false);
        assert!(p.pay_chip().is_err());
    }

    #[test]
    fn return_invested() {
        let mut p = Player::new(0, "Alice".into(), 6, false);
        p.pay_chip().unwrap();
        p.pay_chip().unwrap();
        assert_eq!(p.chips, 4);
        assert_eq!(p.pot, 2);
        p.return_invested();
        assert_eq!(p.chips, 6);
        assert_eq!(p.pot, 0);
    }

    #[test]
    fn apply_penalty_partial() {
        let mut p = Player::new(0, "Alice".into(), 6, false);
        p.apply_penalty(3);
        assert_eq!(p.chips, 3);
        assert!(!p.is_eliminated);
    }

    #[test]
    fn apply_penalty_elimination() {
        let mut p = Player::new(0, "Alice".into(), 2, false);
        p.apply_penalty(5);
        assert_eq!(p.chips, 0);
        assert!(p.is_eliminated);
    }

    #[test]
    fn apply_penalty_exact() {
        let mut p = Player::new(0, "Alice".into(), 3, false);
        p.apply_penalty(3);
        assert_eq!(p.chips, 0);
        assert!(p.is_eliminated);
    }
}
