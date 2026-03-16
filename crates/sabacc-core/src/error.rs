use crate::PlayerId;

/// All possible errors that can occur during a Sabacc game.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum GameError {
    /// The attempted action is not valid in the current game phase.
    #[error("invalid action for current phase: {reason}")]
    InvalidActionForPhase { reason: String },

    /// The specified player was not found.
    #[error("player {player_id} not found")]
    PlayerNotFound { player_id: PlayerId },

    /// It is not this player's turn.
    #[error("not player {player_id}'s turn")]
    NotPlayerTurn { player_id: PlayerId },

    /// The player does not have enough chips.
    #[error("player {player_id} has insufficient chips ({available} < {required})")]
    InsufficientChips {
        player_id: PlayerId,
        available: u8,
        required: u8,
    },

    /// The deck is empty and cannot be reshuffled.
    #[error("{family:?} deck is empty and discard pile has no cards to reshuffle")]
    DeckExhausted { family: crate::card::Family },

    /// The discard pile is empty, cannot draw from it.
    #[error("{family:?} discard pile is empty")]
    DiscardEmpty { family: crate::card::Family },

    /// Invalid card value (must be 1-6 for numbered cards).
    #[error("invalid card number: {value} (must be 1-6)")]
    InvalidCardNumber { value: u8 },

    /// A hand must contain exactly one Sand and one Blood card.
    #[error("invalid hand: sand card has family {sand_family:?}, blood card has family {blood_family:?}")]
    InvalidHand {
        sand_family: crate::card::Family,
        blood_family: crate::card::Family,
    },

    /// The player has already been eliminated.
    #[error("player {player_id} is eliminated")]
    PlayerEliminated { player_id: PlayerId },

    /// Impostor choice is required but not provided.
    #[error("impostor choice required for player {player_id}")]
    ImpostorChoiceRequired { player_id: PlayerId },

    /// The chosen die value is not available from the dice roll.
    #[error("die value {chosen} not available from roll ({die1}, {die2})")]
    InvalidDieChoice { chosen: u8, die1: u8, die2: u8 },

    /// Invalid game configuration.
    #[error("invalid config: {reason}")]
    InvalidConfig { reason: String },

    /// ShiftTokens are not enabled in this game configuration.
    #[error("shift tokens are not enabled")]
    ShiftTokensDisabled,
}
