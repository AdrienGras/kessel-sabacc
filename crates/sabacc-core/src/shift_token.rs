use crate::PlayerId;

/// The 16 different shift tokens available in Kessel Sabacc.
///
/// Each shift token can only be used once per game (not per round),
/// before a Draw or Stand action.
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

impl std::fmt::Display for ShiftToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShiftToken::FreeDraw => write!(f, "FreeDraw"),
            ShiftToken::Refund => write!(f, "Refund"),
            ShiftToken::ExtraRefund => write!(f, "ExtraRefund"),
            ShiftToken::GeneralTariff => write!(f, "GeneralTariff"),
            ShiftToken::TargetTariff(_) => write!(f, "TargetTariff"),
            ShiftToken::Embargo => write!(f, "Embargo"),
            ShiftToken::Markdown => write!(f, "Markdown"),
            ShiftToken::Immunity => write!(f, "Immunity"),
            ShiftToken::GeneralAudit => write!(f, "GeneralAudit"),
            ShiftToken::TargetAudit(_) => write!(f, "TargetAudit"),
            ShiftToken::MajorFraud => write!(f, "MajorFraud"),
            ShiftToken::Embezzlement => write!(f, "Embezzlement"),
            ShiftToken::CookTheBooks => write!(f, "CookTheBooks"),
            ShiftToken::Exhaustion(_) => write!(f, "Exhaustion"),
            ShiftToken::DirectTransaction(_) => write!(f, "DirectTransaction"),
            ShiftToken::PrimeSabacc => write!(f, "PrimeSabacc"),
        }
    }
}

impl ShiftToken {
    /// Short description of what this token does.
    pub fn description(&self) -> &'static str {
        match self {
            ShiftToken::FreeDraw => "Draw without paying 1 chip",
            ShiftToken::Refund => "Recover 2 invested chips",
            ShiftToken::ExtraRefund => "Recover 3 invested chips",
            ShiftToken::GeneralTariff => "All others pay 1 chip",
            ShiftToken::TargetTariff(_) => "Targeted player pays 2 chips",
            ShiftToken::Embargo => "Next player must Stand",
            ShiftToken::Markdown => "Sylop = 0 (no match)",
            ShiftToken::Immunity => "Immune to opponent tokens",
            ShiftToken::GeneralAudit => "Standing players pay 2 chips",
            ShiftToken::TargetAudit(_) => "Targeted standing pays 3 chips",
            ShiftToken::MajorFraud => "Impostor locked at 6",
            ShiftToken::Embezzlement => "Take 1 chip from each opponent",
            ShiftToken::CookTheBooks => "Reverse Sabacc ranking",
            ShiftToken::Exhaustion(_) => "Target redraws a new hand",
            ShiftToken::DirectTransaction(_) => "Swap hand with target",
            ShiftToken::PrimeSabacc => "Dice → value = best Sabacc",
        }
    }

    /// Compare token types by discriminant, ignoring inner values.
    pub fn matches_type(&self, other: &ShiftToken) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }

    /// Whether this token requires a target player ID when played.
    pub fn requires_target(&self) -> bool {
        matches!(
            self,
            ShiftToken::TargetTariff(_)
                | ShiftToken::TargetAudit(_)
                | ShiftToken::Exhaustion(_)
                | ShiftToken::DirectTransaction(_)
        )
    }

    /// Get all 16 token types (targeted tokens use placeholder id 0).
    pub fn all_types() -> Vec<ShiftToken> {
        vec![
            ShiftToken::FreeDraw,
            ShiftToken::Refund,
            ShiftToken::ExtraRefund,
            ShiftToken::GeneralTariff,
            ShiftToken::TargetTariff(0),
            ShiftToken::Embargo,
            ShiftToken::Markdown,
            ShiftToken::Immunity,
            ShiftToken::GeneralAudit,
            ShiftToken::TargetAudit(0),
            ShiftToken::MajorFraud,
            ShiftToken::Embezzlement,
            ShiftToken::CookTheBooks,
            ShiftToken::Exhaustion(0),
            ShiftToken::DirectTransaction(0),
            ShiftToken::PrimeSabacc,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_type_same() {
        assert!(ShiftToken::FreeDraw.matches_type(&ShiftToken::FreeDraw));
        assert!(ShiftToken::TargetTariff(0).matches_type(&ShiftToken::TargetTariff(1)));
    }

    #[test]
    fn matches_type_different() {
        assert!(!ShiftToken::FreeDraw.matches_type(&ShiftToken::Refund));
        assert!(!ShiftToken::TargetTariff(0).matches_type(&ShiftToken::TargetAudit(0)));
    }

    #[test]
    fn requires_target_correct() {
        assert!(!ShiftToken::FreeDraw.requires_target());
        assert!(!ShiftToken::GeneralTariff.requires_target());
        assert!(ShiftToken::TargetTariff(0).requires_target());
        assert!(ShiftToken::TargetAudit(0).requires_target());
        assert!(ShiftToken::Exhaustion(0).requires_target());
        assert!(ShiftToken::DirectTransaction(0).requires_target());
    }

    #[test]
    fn all_types_has_16() {
        assert_eq!(ShiftToken::all_types().len(), 16);
    }
}
