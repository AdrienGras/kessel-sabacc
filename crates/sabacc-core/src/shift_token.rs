use crate::PlayerId;

/// The 16 different shift tokens available in Kessel Sabacc.
///
/// Each shift token can only be used once per game (not per round),
/// before a Draw or Stand action.
///
/// In Phase 1, shift tokens are defined but their logic is not implemented.
/// Playing a shift token will be rejected with `ShiftTokensDisabled`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShiftToken {
    /// Draw without paying 1 chip this turn.
    FreeDraw,
    /// Recover 2 invested chips this turn (minimum 1 invested).
    Refund,
    /// Recover 3 invested chips this turn.
    ExtraRefund,
    /// All other players pay 1 chip.
    GeneralTariff,
    /// A targeted player pays 2 chips.
    TargetTariff(PlayerId),
    /// The next player must Stand.
    Embargo,
    /// Sylop value becomes 0 until revelation (Sylop no longer matches).
    Markdown,
    /// Immunity against opponent shift token effects until revelation.
    Immunity,
    /// All players who Stand pay 2 chips.
    GeneralAudit,
    /// A targeted player who Stands pays 3 chips.
    TargetAudit(PlayerId),
    /// Impostor value fixed at 6 until revelation.
    MajorFraud,
    /// Take 1 chip from each other player.
    Embezzlement,
    /// Invert the Sabacc ranking until revelation (6/6 becomes the best).
    CookTheBooks,
    /// The targeted player discards and redraws a complete new hand.
    Exhaustion(PlayerId),
    /// Swap your hand with a targeted player.
    DirectTransaction(PlayerId),
    /// Roll 2 dice; the chosen value becomes the best Sabacc.
    PrimeSabacc,
}
