use rand::Rng;

use crate::card::CardValue;
use crate::deck::FamilyDeck;
use crate::error::GameError;
use crate::hand::Hand;
use crate::player::Player;
use crate::scoring::{self, ActiveModifiers, ImpostorChoice, RoundResult};
use crate::PlayerId;

/// Deal initial hands to all active players and set up initial discard piles.
///
/// Each player receives one Sand and one Blood card.
/// One Sand and one Blood card are placed face-up as the initial discards.
pub fn deal_hands(
    players: &mut [Player],
    sand_deck: &mut FamilyDeck,
    blood_deck: &mut FamilyDeck,
    rng: &mut impl Rng,
) -> Result<(), GameError> {
    for player in players.iter_mut() {
        if player.is_eliminated {
            continue;
        }
        let sand = sand_deck.draw(rng)?;
        let blood = blood_deck.draw(rng)?;
        player.hand = Some(Hand::new(sand, blood)?);
    }

    // Initial face-up discards
    let sand_discard = sand_deck.draw(rng)?;
    sand_deck.discard(sand_discard);

    let blood_discard = blood_deck.draw(rng)?;
    blood_deck.discard(blood_discard);

    Ok(())
}

/// Check which active players have Impostors in their hand.
pub fn players_with_impostors(players: &[Player]) -> Vec<PlayerId> {
    let mut result = Vec::new();
    for player in players {
        if player.is_eliminated {
            continue;
        }
        if let Some(ref hand) = player.hand {
            let has_impostor = hand.sand.value == CardValue::Impostor
                || hand.blood.value == CardValue::Impostor;
            if has_impostor {
                result.push(player.id);
            }
        }
    }
    result
}

/// Evaluate all active players' hands and resolve the round.
///
/// Returns results with winners and penalties.
pub fn resolve(
    players: &[Player],
    impostor_choices: &[ImpostorChoice],
    modifiers: &ActiveModifiers,
) -> Result<Vec<RoundResult>, GameError> {
    let mut evaluated = Vec::new();

    for player in players {
        if player.is_eliminated {
            continue;
        }
        let hand = player.hand.as_ref().ok_or(GameError::PlayerNotFound {
            player_id: player.id,
        })?;

        let choice = impostor_choices
            .iter()
            .find(|c| c.player_id == player.id);

        // Check if impostor choice is needed
        let needs_impostor = hand.sand.value == CardValue::Impostor
            || hand.blood.value == CardValue::Impostor;
        if needs_impostor && choice.is_none() {
            return Err(GameError::ImpostorChoiceRequired {
                player_id: player.id,
            });
        }

        let rank = scoring::evaluate_hand(hand, choice, modifiers, player.id)?;
        evaluated.push((player.id, rank, player.pot));
    }

    Ok(scoring::resolve_round(&evaluated, modifiers))
}

/// Apply round results: winners get chips back, losers pay penalties.
/// Returns cards from all hands to the appropriate decks.
pub fn apply_results(
    players: &mut [Player],
    results: &[RoundResult],
    sand_deck: &mut FamilyDeck,
    blood_deck: &mut FamilyDeck,
) {
    for result in results {
        if let Some(player) = players.iter_mut().find(|p| p.id == result.player_id) {
            if result.is_winner {
                player.return_invested();
            } else {
                // Losers lose their invested chips (pot goes to 0, chips in pot are destroyed)
                player.pot = 0;
                player.apply_penalty(result.penalty);
            }
        }
    }

    // Return all hands to decks (Hand guarantees sand=Sand, blood=Blood)
    for player in players.iter_mut() {
        if let Some(hand) = player.hand.take() {
            sand_deck.discard(hand.sand);
            blood_deck.discard(hand.blood);
        }
    }
}

/// Check if the game is over (0 or 1 players remaining).
pub fn check_game_over(players: &[Player]) -> Option<PlayerId> {
    let active: Vec<_> = players
        .iter()
        .filter(|p| !p.is_eliminated)
        .collect();

    match active.len() {
        0 => None,
        1 => Some(active[0].id),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Card, Family};
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn test_rng() -> SmallRng {
        SmallRng::seed_from_u64(42)
    }

    #[test]
    fn deal_hands_gives_cards() {
        let mut rng = test_rng();
        let mut sand_deck = FamilyDeck::new(Family::Sand, &mut rng);
        let mut blood_deck = FamilyDeck::new(Family::Blood, &mut rng);
        let mut players = vec![
            Player::new(0, "Alice".into(), 6, false),
            Player::new(1, "Bob".into(), 6, true),
        ];

        deal_hands(&mut players, &mut sand_deck, &mut blood_deck, &mut rng).unwrap();

        assert!(players[0].hand.is_some());
        assert!(players[1].hand.is_some());
        // Each player drew 1 card per deck; initial discard drawn+returned = no net change
        assert_eq!(sand_deck.total_cards(), 22 - 2);
        assert_eq!(blood_deck.total_cards(), 22 - 2);
    }

    #[test]
    fn deal_hands_skips_eliminated() {
        let mut rng = test_rng();
        let mut sand_deck = FamilyDeck::new(Family::Sand, &mut rng);
        let mut blood_deck = FamilyDeck::new(Family::Blood, &mut rng);
        let mut players = vec![
            Player::new(0, "Alice".into(), 6, false),
            Player::new(1, "Bob".into(), 0, true),
        ];
        players[1].is_eliminated = true;

        deal_hands(&mut players, &mut sand_deck, &mut blood_deck, &mut rng).unwrap();

        assert!(players[0].hand.is_some());
        assert!(players[1].hand.is_none());
    }

    #[test]
    fn players_with_impostors_detection() {
        let mut players = vec![
            Player::new(0, "Alice".into(), 6, false),
            Player::new(1, "Bob".into(), 6, true),
        ];

        players[0].hand = Some(
            Hand::new(
                Card::impostor(Family::Sand),
                Card::number(Family::Blood, 3),
            )
            .unwrap(),
        );
        players[1].hand = Some(
            Hand::new(
                Card::number(Family::Sand, 2),
                Card::number(Family::Blood, 4),
            )
            .unwrap(),
        );

        let impostors = players_with_impostors(&players);
        assert_eq!(impostors, vec![0]);
    }

    #[test]
    fn check_game_over_one_remaining() {
        let mut players = vec![
            Player::new(0, "Alice".into(), 6, false),
            Player::new(1, "Bob".into(), 0, true),
        ];
        players[1].is_eliminated = true;

        assert_eq!(check_game_over(&players), Some(0));
    }

    #[test]
    fn check_game_over_multiple_remaining() {
        let players = vec![
            Player::new(0, "Alice".into(), 6, false),
            Player::new(1, "Bob".into(), 3, true),
        ];

        assert_eq!(check_game_over(&players), None);
    }
}
