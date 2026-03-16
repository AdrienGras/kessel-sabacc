use crate::card::CardValue;
use crate::error::GameError;
use crate::hand::{Hand, HandRank};
use crate::PlayerId;

/// Modifier for the PrimeSabacc shift token.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimeSabaccModifier {
    /// The player who activated PrimeSabacc.
    pub player_id: PlayerId,
    /// The dice value chosen by the player.
    pub chosen_value: u8,
}

/// Modifiers applied by active shift tokens.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ActiveModifiers {
    /// If true, Sylop value becomes 0 (doesn't match the other card).
    pub markdown_active: bool,
    /// If true, the hand ranking is inverted (6/6 beats 1/1).
    pub cook_the_books_active: bool,
    /// If true, Impostor value is forced to 6.
    pub major_fraud_active: bool,
    /// Players who are immune to opponent shift token effects.
    pub immune_players: Vec<PlayerId>,
    /// Active PrimeSabacc override, if any.
    pub prime_sabacc: Option<PrimeSabaccModifier>,
}

/// A player's choice for resolving an Impostor card.
#[derive(Debug, Clone, PartialEq)]
pub struct ImpostorChoice {
    /// The player making the choice.
    pub player_id: PlayerId,
    /// The two dice values rolled.
    pub die1: u8,
    pub die2: u8,
    /// Which die value the player chose for the Sand impostor (if any).
    pub sand_choice: Option<u8>,
    /// Which die value the player chose for the Blood impostor (if any).
    pub blood_choice: Option<u8>,
}

impl ImpostorChoice {
    /// Validate that chosen values are available from the dice roll.
    pub fn validate(&self) -> Result<(), GameError> {
        for choice in [self.sand_choice, self.blood_choice].iter().flatten() {
            if *choice != self.die1 && *choice != self.die2 {
                return Err(GameError::InvalidDieChoice {
                    chosen: *choice,
                    die1: self.die1,
                    die2: self.die2,
                });
            }
        }
        Ok(())
    }
}

/// The result for a single player after a round.
#[derive(Debug, Clone, PartialEq)]
pub struct RoundResult {
    /// The player's ID.
    pub player_id: PlayerId,
    /// The evaluated hand rank.
    pub rank: HandRank,
    /// Chips invested this round.
    pub invested: u8,
    /// Whether this player won the round.
    pub is_winner: bool,
    /// Penalty chips to lose (0 for winners, difference for non-sabacc losers, 1 for sabacc losers).
    pub penalty: u8,
}

/// Evaluate a hand into a HandRank.
///
/// If the hand contains Impostors, an `ImpostorChoice` must be provided
/// with the chosen die values. Sylop behavior depends on `ActiveModifiers`.
pub fn evaluate_hand(
    hand: &Hand,
    impostor_choice: Option<&ImpostorChoice>,
    modifiers: &ActiveModifiers,
) -> Result<HandRank, GameError> {
    let sand_value = resolve_card_value(&hand.sand.value, impostor_choice, true, modifiers)?;
    let blood_value = resolve_card_value(&hand.blood.value, impostor_choice, false, modifiers)?;

    match (sand_value, blood_value) {
        // Both Sylops -> Pure Sabacc (unless markdown makes them 0)
        (ResolvedValue::Sylop, ResolvedValue::Sylop) => {
            if modifiers.markdown_active {
                // Both marked down to 0, that's still a Pure Sabacc
                // (two Sylops always form Pure Sabacc regardless of markdown)
                Ok(HandRank::PureSabacc)
            } else {
                Ok(HandRank::PureSabacc)
            }
        }
        // One Sylop + one number
        (ResolvedValue::Sylop, ResolvedValue::Number(v))
        | (ResolvedValue::Number(v), ResolvedValue::Sylop) => {
            if modifiers.markdown_active {
                // Markdown: Sylop = 0, so difference = v
                Ok(HandRank::NonSabacc { difference: v })
            } else {
                Ok(HandRank::SylopSabacc { value: v })
            }
        }
        // Two numbers
        (ResolvedValue::Number(a), ResolvedValue::Number(b)) => {
            if a == b {
                Ok(HandRank::Sabacc { pair_value: a })
            } else {
                Ok(HandRank::NonSabacc {
                    difference: a.abs_diff(b),
                })
            }
        }
    }
}

/// Compare two hand ranks. Returns Ordering where Less means `a` is stronger.
pub fn compare_ranks(a: &HandRank, b: &HandRank, modifiers: &ActiveModifiers) -> std::cmp::Ordering {
    let key_a = adjusted_strength_key(a, modifiers);
    let key_b = adjusted_strength_key(b, modifiers);
    key_a.cmp(&key_b)
}

/// Resolve the round: determine winners, assign penalties.
///
/// Input: list of (player_id, hand_rank, chips_invested) for all active players.
/// Returns a RoundResult for each player.
pub fn resolve_round(
    players: &[(PlayerId, HandRank, u8)],
    modifiers: &ActiveModifiers,
) -> Vec<RoundResult> {
    if players.is_empty() {
        return Vec::new();
    }

    // Apply PrimeSabacc override if active
    let mut player_ranks: Vec<(PlayerId, HandRank, u8)> = players.to_vec();
    if let Some(ref prime) = modifiers.prime_sabacc {
        for entry in &mut player_ranks {
            if entry.0 == prime.player_id {
                entry.1 = HandRank::PrimeSabacc {
                    value: prime.chosen_value,
                };
            }
        }
    }

    // Find the best rank
    let best_key = player_ranks
        .iter()
        .map(|(_, rank, _)| adjusted_strength_key(rank, modifiers))
        .min()
        .unwrap_or((255, 255));

    let mut results = Vec::with_capacity(player_ranks.len());

    for (player_id, rank, invested) in &player_ranks {
        let key = adjusted_strength_key(rank, modifiers);
        let is_winner = key == best_key;

        let penalty = if is_winner {
            0
        } else {
            match rank {
                // Losing Sabacc hand -> 1 chip penalty
                HandRank::PureSabacc
                | HandRank::PrimeSabacc { .. }
                | HandRank::SylopSabacc { .. }
                | HandRank::Sabacc { .. } => 1,
                // Non-Sabacc -> penalty equal to difference
                HandRank::NonSabacc { difference } => *difference,
            }
        };

        results.push(RoundResult {
            player_id: *player_id,
            rank: rank.clone(),
            invested: *invested,
            is_winner,
            penalty,
        });
    }

    results
}

/// Internal resolved value after handling Impostors.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ResolvedValue {
    Number(u8),
    Sylop,
}

fn resolve_card_value(
    value: &CardValue,
    impostor_choice: Option<&ImpostorChoice>,
    is_sand: bool,
    modifiers: &ActiveModifiers,
) -> Result<ResolvedValue, GameError> {
    match value {
        CardValue::Number(n) => Ok(ResolvedValue::Number(*n)),
        CardValue::Sylop => Ok(ResolvedValue::Sylop),
        CardValue::Impostor => {
            if modifiers.major_fraud_active {
                // MajorFraud: impostor value forced to 6, no dice needed
                return Ok(ResolvedValue::Number(6));
            }
            let choice = impostor_choice.ok_or(GameError::ImpostorChoiceRequired {
                player_id: 0, // caller should handle this
            })?;
            let chosen = if is_sand {
                choice.sand_choice.ok_or(GameError::ImpostorChoiceRequired {
                    player_id: choice.player_id,
                })?
            } else {
                choice
                    .blood_choice
                    .ok_or(GameError::ImpostorChoiceRequired {
                        player_id: choice.player_id,
                    })?
            };
            Ok(ResolvedValue::Number(chosen))
        }
    }
}

/// Get the strength key adjusted for CookTheBooks modifier.
fn adjusted_strength_key(rank: &HandRank, modifiers: &ActiveModifiers) -> (u8, u8) {
    let (tier, sub) = rank.strength_key();
    if modifiers.cook_the_books_active && tier == 2 {
        // Invert Sabacc ranking: 6/6 becomes best (sub=1), 1/1 becomes worst (sub=6)
        (tier, 7 - sub)
    } else {
        (tier, sub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, Family};

    fn no_modifiers() -> ActiveModifiers {
        ActiveModifiers::default()
    }

    #[test]
    fn pure_sabacc() {
        let hand = Hand::new(Card::sylop(Family::Sand), Card::sylop(Family::Blood)).unwrap();
        let rank = evaluate_hand(&hand, None, &no_modifiers()).unwrap();
        assert_eq!(rank, HandRank::PureSabacc);
    }

    #[test]
    fn sylop_sabacc() {
        let hand = Hand::new(
            Card::sylop(Family::Sand),
            Card::number(Family::Blood, 3),
        )
        .unwrap();
        let rank = evaluate_hand(&hand, None, &no_modifiers()).unwrap();
        assert_eq!(rank, HandRank::SylopSabacc { value: 3 });
    }

    #[test]
    fn sylop_sabacc_blood_sylop() {
        let hand = Hand::new(
            Card::number(Family::Sand, 5),
            Card::sylop(Family::Blood),
        )
        .unwrap();
        let rank = evaluate_hand(&hand, None, &no_modifiers()).unwrap();
        assert_eq!(rank, HandRank::SylopSabacc { value: 5 });
    }

    #[test]
    fn sabacc_pair() {
        let hand = Hand::new(
            Card::number(Family::Sand, 4),
            Card::number(Family::Blood, 4),
        )
        .unwrap();
        let rank = evaluate_hand(&hand, None, &no_modifiers()).unwrap();
        assert_eq!(rank, HandRank::Sabacc { pair_value: 4 });
    }

    #[test]
    fn non_sabacc() {
        let hand = Hand::new(
            Card::number(Family::Sand, 6),
            Card::number(Family::Blood, 2),
        )
        .unwrap();
        let rank = evaluate_hand(&hand, None, &no_modifiers()).unwrap();
        assert_eq!(rank, HandRank::NonSabacc { difference: 4 });
    }

    #[test]
    fn impostor_makes_sabacc() {
        let hand = Hand::new(
            Card::impostor(Family::Sand),
            Card::number(Family::Blood, 3),
        )
        .unwrap();

        let choice = ImpostorChoice {
            player_id: 0,
            die1: 3,
            die2: 5,
            sand_choice: Some(3),
            blood_choice: None,
        };
        choice.validate().unwrap();

        let rank = evaluate_hand(&hand, Some(&choice), &no_modifiers()).unwrap();
        assert_eq!(rank, HandRank::Sabacc { pair_value: 3 });
    }

    #[test]
    fn impostor_non_sabacc() {
        let hand = Hand::new(
            Card::impostor(Family::Sand),
            Card::number(Family::Blood, 3),
        )
        .unwrap();

        let choice = ImpostorChoice {
            player_id: 0,
            die1: 2,
            die2: 5,
            sand_choice: Some(5),
            blood_choice: None,
        };
        choice.validate().unwrap();

        let rank = evaluate_hand(&hand, Some(&choice), &no_modifiers()).unwrap();
        assert_eq!(rank, HandRank::NonSabacc { difference: 2 });
    }

    #[test]
    fn double_impostor() {
        let hand = Hand::new(
            Card::impostor(Family::Sand),
            Card::impostor(Family::Blood),
        )
        .unwrap();

        let choice = ImpostorChoice {
            player_id: 0,
            die1: 4,
            die2: 4,
            sand_choice: Some(4),
            blood_choice: Some(4),
        };
        choice.validate().unwrap();

        let rank = evaluate_hand(&hand, Some(&choice), &no_modifiers()).unwrap();
        assert_eq!(rank, HandRank::Sabacc { pair_value: 4 });
    }

    #[test]
    fn sylop_plus_impostor() {
        let hand = Hand::new(
            Card::sylop(Family::Sand),
            Card::impostor(Family::Blood),
        )
        .unwrap();

        let choice = ImpostorChoice {
            player_id: 0,
            die1: 2,
            die2: 5,
            sand_choice: None,
            blood_choice: Some(2),
        };
        choice.validate().unwrap();

        let rank = evaluate_hand(&hand, Some(&choice), &no_modifiers()).unwrap();
        assert_eq!(rank, HandRank::SylopSabacc { value: 2 });
    }

    #[test]
    fn invalid_die_choice() {
        let choice = ImpostorChoice {
            player_id: 0,
            die1: 2,
            die2: 5,
            sand_choice: Some(3),
            blood_choice: None,
        };
        assert!(choice.validate().is_err());
    }

    #[test]
    fn compare_ranks_basic() {
        let mods = no_modifiers();
        let pure = HandRank::PureSabacc;
        let sabacc = HandRank::Sabacc { pair_value: 1 };

        assert_eq!(
            compare_ranks(&pure, &sabacc, &mods),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn sabacc_1_beats_sabacc_6() {
        let mods = no_modifiers();
        let s1 = HandRank::Sabacc { pair_value: 1 };
        let s6 = HandRank::Sabacc { pair_value: 6 };

        assert_eq!(
            compare_ranks(&s1, &s6, &mods),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn resolve_round_winner_gets_chips_back() {
        let mods = no_modifiers();
        let players = vec![
            (0, HandRank::Sabacc { pair_value: 1 }, 3),
            (1, HandRank::NonSabacc { difference: 4 }, 2),
        ];

        let results = resolve_round(&players, &mods);
        assert_eq!(results.len(), 2);

        let winner = &results[0];
        assert!(winner.is_winner);
        assert_eq!(winner.penalty, 0);

        let loser = &results[1];
        assert!(!loser.is_winner);
        assert_eq!(loser.penalty, 4); // difference
    }

    #[test]
    fn resolve_round_losing_sabacc_penalty_1() {
        let mods = no_modifiers();
        let players = vec![
            (0, HandRank::Sabacc { pair_value: 1 }, 3),
            (1, HandRank::Sabacc { pair_value: 6 }, 2),
        ];

        let results = resolve_round(&players, &mods);
        let loser = &results[1];
        assert!(!loser.is_winner);
        assert_eq!(loser.penalty, 1);
    }

    #[test]
    fn resolve_round_tie() {
        let mods = no_modifiers();
        let players = vec![
            (0, HandRank::Sabacc { pair_value: 3 }, 2),
            (1, HandRank::Sabacc { pair_value: 3 }, 2),
        ];

        let results = resolve_round(&players, &mods);
        assert!(results[0].is_winner);
        assert!(results[1].is_winner);
        assert_eq!(results[0].penalty, 0);
        assert_eq!(results[1].penalty, 0);
    }

    #[test]
    fn cook_the_books_inverts_sabacc() {
        let mods = ActiveModifiers {
            cook_the_books_active: true,
            ..Default::default()
        };
        let s1 = HandRank::Sabacc { pair_value: 1 };
        let s6 = HandRank::Sabacc { pair_value: 6 };

        // With CookTheBooks, 6/6 should beat 1/1
        assert_eq!(
            compare_ranks(&s6, &s1, &mods),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn markdown_breaks_sylop() {
        let mods = ActiveModifiers {
            markdown_active: true,
            ..Default::default()
        };
        let hand = Hand::new(
            Card::sylop(Family::Sand),
            Card::number(Family::Blood, 3),
        )
        .unwrap();

        let rank = evaluate_hand(&hand, None, &mods).unwrap();
        assert_eq!(rank, HandRank::NonSabacc { difference: 3 });
    }
}
