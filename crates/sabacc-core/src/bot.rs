use rand::Rng;

use crate::card::CardValue;
use crate::game::{Action, GamePhase, GameState};
use crate::scoring::ImpostorChoice;
use crate::shift_token::ShiftToken;
use crate::turn::{DiscardChoice, DrawSource, TurnAction};
use crate::PlayerId;

/// Trait for bot decision-making strategies.
pub trait BotStrategy {
    /// Choose a turn action (Draw or Stand) for the current bot player.
    fn choose_action(&self, state: &GameState, rng: &mut impl Rng) -> Action;

    /// Choose which card to discard after drawing.
    fn choose_discard(&self, state: &GameState, rng: &mut impl Rng) -> Action;

    /// Choose impostor die values.
    fn choose_impostor(&self, state: &GameState, rng: &mut impl Rng) -> Action;

    /// Optionally choose a shift token to play before Draw/Stand.
    /// Returns None if the bot decides not to play a token.
    fn choose_token(&self, state: &GameState, rng: &mut impl Rng) -> Option<Action>;

    /// Choose a PrimeSabacc dice value.
    fn choose_prime_sabacc(&self, state: &GameState, rng: &mut impl Rng) -> Action;
}

/// A basic bot that makes weighted random decisions.
///
/// - Prefers to draw when hand difference is high
/// - Prefers to stand when hand is already good
/// - For impostors, picks the die value closest to the other card
#[derive(Debug, Clone)]
pub struct BasicBot;

impl BotStrategy for BasicBot {
    fn choose_action(&self, state: &GameState, rng: &mut impl Rng) -> Action {
        let player = &state.players[state.current_player_idx];
        let pid = player.id;

        // Evaluate current hand quality
        let draw_chance = if let Some(ref hand) = player.hand {
            let sand_val = card_numeric_value(&hand.sand.value);
            let blood_val = card_numeric_value(&hand.blood.value);
            let diff = sand_val.abs_diff(blood_val);
            // Higher difference = more likely to draw
            // diff 0 (perfect) -> 10% chance to draw
            // diff 5 (worst) -> 90% chance to draw
            10 + diff as u32 * 16
        } else {
            50
        };

        let can_draw_free = state.free_draw_active;
        if !can_draw_free && (player.chips == 0 || rng.gen_range(0..100) >= draw_chance) {
            Action::PlayerAction {
                player_id: pid,
                action: TurnAction::Stand,
            }
        } else {
            // Pick a random draw source, preferring deck over discard
            let source = match rng.gen_range(0..4) {
                0 => DrawSource::SandDeck,
                1 => DrawSource::BloodDeck,
                2 if state.sand_deck.peek_discard().is_some() => DrawSource::SandDiscard,
                3 if state.blood_deck.peek_discard().is_some() => DrawSource::BloodDiscard,
                _ => {
                    if rng.gen_bool(0.5) {
                        DrawSource::SandDeck
                    } else {
                        DrawSource::BloodDeck
                    }
                }
            };

            Action::PlayerAction {
                player_id: pid,
                action: TurnAction::Draw(source),
            }
        }
    }

    fn choose_discard(&self, state: &GameState, rng: &mut impl Rng) -> Action {
        let (player_id, drawn_card) = match &state.phase {
            GamePhase::ChoosingDiscard {
                player_id,
                drawn_card,
            } => (*player_id, drawn_card),
            _ => panic!("choose_discard called in wrong phase"),
        };

        let player = state.players.iter().find(|p| p.id == player_id);

        let choice = if let Some(player) = player {
            if let Some(ref hand) = player.hand {
                let drawn_val = card_numeric_value(&drawn_card.value);
                let current_val = match drawn_card.family {
                    crate::card::Family::Sand => card_numeric_value(&hand.sand.value),
                    crate::card::Family::Blood => card_numeric_value(&hand.blood.value),
                };
                let other_val = match drawn_card.family {
                    crate::card::Family::Sand => card_numeric_value(&hand.blood.value),
                    crate::card::Family::Blood => card_numeric_value(&hand.sand.value),
                };

                // Keep the card that gets us closer to a match
                let diff_with_drawn = drawn_val.abs_diff(other_val);
                let diff_with_current = current_val.abs_diff(other_val);

                if diff_with_drawn < diff_with_current {
                    DiscardChoice::KeepDrawn
                } else if diff_with_drawn > diff_with_current {
                    DiscardChoice::DiscardDrawn
                } else {
                    // Tie: random
                    if rng.gen_bool(0.5) {
                        DiscardChoice::KeepDrawn
                    } else {
                        DiscardChoice::DiscardDrawn
                    }
                }
            } else {
                DiscardChoice::DiscardDrawn
            }
        } else {
            DiscardChoice::DiscardDrawn
        };

        Action::ChooseDiscard {
            player_id,
            choice,
        }
    }

    fn choose_impostor(&self, state: &GameState, rng: &mut impl Rng) -> Action {
        let (pending, _) = match &state.phase {
            GamePhase::ImpostorReveal {
                pending,
                submitted,
            } => (pending, submitted),
            _ => panic!("choose_impostor called in wrong phase"),
        };

        let pid = pending[0];
        let player = state.players.iter().find(|p| p.id == pid);

        let die1 = rng.gen_range(1..=6u8);
        let die2 = rng.gen_range(1..=6u8);

        let (sand_choice, blood_choice) = if let Some(player) = player {
            if let Some(ref hand) = player.hand {
                let sc = if hand.sand.value == CardValue::Impostor {
                    // Choose the die value closest to the blood card
                    let blood_val = card_numeric_value(&hand.blood.value);
                    Some(pick_closest(die1, die2, blood_val))
                } else {
                    None
                };

                let bc = if hand.blood.value == CardValue::Impostor {
                    let sand_val = card_numeric_value(&hand.sand.value);
                    Some(pick_closest(die1, die2, sand_val))
                } else {
                    None
                };

                (sc, bc)
            } else {
                (Some(die1), Some(die1))
            }
        } else {
            (Some(die1), Some(die1))
        };

        Action::SubmitImpostorChoice(ImpostorChoice {
            player_id: pid,
            die1,
            die2,
            sand_choice,
            blood_choice,
        })
    }

    fn choose_token(&self, state: &GameState, rng: &mut impl Rng) -> Option<Action> {
        let player = &state.players[state.current_player_idx];
        let pid = player.id;

        if player.shift_tokens.is_empty() {
            return None;
        }

        // ~30% chance to play a token per turn
        if rng.gen_range(0..100) >= 30 {
            return None;
        }

        let hand_diff = if let Some(ref hand) = player.hand {
            let s = card_numeric_value(&hand.sand.value);
            let b = card_numeric_value(&hand.blood.value);
            s.abs_diff(b)
        } else {
            3
        };

        let threat = most_threatening(state, pid);

        // Try tokens in priority order
        for token in &player.shift_tokens {
            let action = match token {
                // Self-buff priority: Immunity when hand is strong
                ShiftToken::Immunity if hand_diff <= 1 => Some(ShiftToken::Immunity),

                // FreeDraw when difference is high
                ShiftToken::FreeDraw if hand_diff >= 3 => Some(ShiftToken::FreeDraw),

                // Tariffs on turn 3
                ShiftToken::GeneralTariff if state.turn == 3 => Some(ShiftToken::GeneralTariff),
                ShiftToken::TargetTariff(_) if state.turn >= 2 => {
                    threat.map(ShiftToken::TargetTariff)
                }

                // Refund when invested
                ShiftToken::Refund if player.pot >= 2 => Some(ShiftToken::Refund),
                ShiftToken::ExtraRefund if player.pot >= 2 => Some(ShiftToken::ExtraRefund),

                // Embezzlement when multiple opponents
                ShiftToken::Embezzlement => {
                    let active = state.players.iter().filter(|p| !p.is_eliminated && p.id != pid).count();
                    if active >= 2 { Some(ShiftToken::Embezzlement) } else { None }
                }

                // Modifiers on turn 2+
                ShiftToken::CookTheBooks if state.turn >= 2 && hand_diff == 0 => {
                    // Only if we have a high-value sabacc
                    if let Some(ref hand) = player.hand {
                        let s = card_numeric_value(&hand.sand.value);
                        if s >= 4 { Some(ShiftToken::CookTheBooks) } else { None }
                    } else {
                        None
                    }
                }
                ShiftToken::Markdown if state.turn >= 2 => Some(ShiftToken::Markdown),
                ShiftToken::MajorFraud if state.turn >= 2 => Some(ShiftToken::MajorFraud),

                // Embargo
                ShiftToken::Embargo if state.turn >= 2 => Some(ShiftToken::Embargo),

                // Offensive tokens
                ShiftToken::Exhaustion(_) if state.turn >= 2 => {
                    threat.map(ShiftToken::Exhaustion)
                }
                ShiftToken::DirectTransaction(_) if hand_diff >= 4 => {
                    threat.map(ShiftToken::DirectTransaction)
                }

                // Audits
                ShiftToken::GeneralAudit if state.turn == 3 => Some(ShiftToken::GeneralAudit),
                ShiftToken::TargetAudit(_) if state.turn >= 2 => {
                    threat.map(ShiftToken::TargetAudit)
                }

                // PrimeSabacc on turn 3 with bad hand
                ShiftToken::PrimeSabacc if state.turn == 3 && hand_diff >= 2 => {
                    Some(ShiftToken::PrimeSabacc)
                }

                _ => None,
            };

            if let Some(chosen) = action {
                return Some(Action::PlayShiftToken {
                    player_id: pid,
                    token: chosen,
                });
            }
        }

        None
    }

    fn choose_prime_sabacc(&self, state: &GameState, _rng: &mut impl Rng) -> Action {
        let (player_id, die1, die2) = match &state.phase {
            GamePhase::PrimeSabaccChoice {
                player_id,
                die1,
                die2,
            } => (*player_id, *die1, *die2),
            _ => panic!("choose_prime_sabacc called in wrong phase"),
        };

        // Pick the die value closest to the other card in hand
        let player = state.players.iter().find(|p| p.id == player_id);
        let chosen = if let Some(player) = player {
            if let Some(ref hand) = player.hand {
                let sand_val = card_numeric_value(&hand.sand.value);
                let blood_val = card_numeric_value(&hand.blood.value);
                // Pick value closest to either card
                let target = sand_val.min(blood_val);
                pick_closest(die1, die2, target)
            } else {
                die1.min(die2)
            }
        } else {
            die1.min(die2)
        };

        Action::SubmitPrimeSabaccChoice {
            player_id,
            chosen_value: chosen,
        }
    }
}

/// Find the most threatening opponent (non-eliminated, most chips).
fn most_threatening(state: &GameState, self_id: PlayerId) -> Option<PlayerId> {
    state
        .players
        .iter()
        .filter(|p| !p.is_eliminated && p.id != self_id)
        .max_by_key(|p| p.total_chips())
        .map(|p| p.id)
}

/// Get a numeric value for comparison. Sylop=0, Impostor=7 (placeholder), Number=n.
fn card_numeric_value(value: &CardValue) -> u8 {
    match value {
        CardValue::Number(n) => *n,
        CardValue::Sylop => 0,
        CardValue::Impostor => 7,
    }
}

/// Pick the die value closest to a target.
fn pick_closest(die1: u8, die2: u8, target: u8) -> u8 {
    let d1 = die1.abs_diff(target);
    let d2 = die2.abs_diff(target);
    if d1 <= d2 {
        die1
    } else {
        die2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{self, GameConfig, TokenDistribution};
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn test_rng() -> SmallRng {
        SmallRng::seed_from_u64(42)
    }

    #[test]
    fn basic_bot_produces_valid_actions() {
        let mut rng = test_rng();
        let config = GameConfig {
            players: vec![
                ("Human".into(), false),
                ("Bot".into(), true),
            ],
            starting_chips: 6,
            buy_in: 100,
            enable_shift_tokens: false,
            token_distribution: TokenDistribution::None,
        };

        let state = game::new_game(config, &mut rng).unwrap();
        let state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

        // Human stands
        let state = game::apply_action(
            state,
            Action::PlayerAction {
                player_id: 0,
                action: TurnAction::Stand,
            },
            &mut rng,
        )
        .unwrap();

        // Bot should be current player
        assert_eq!(state.players[state.current_player_idx].id, 1);

        let bot = BasicBot;
        let action = bot.choose_action(&state, &mut rng);

        // Action should be valid (either Stand or Draw)
        match &action {
            Action::PlayerAction { player_id, action } => {
                assert_eq!(*player_id, 1);
                match action {
                    TurnAction::Stand | TurnAction::Draw(_) => {}
                }
            }
            _ => panic!("expected PlayerAction"),
        }
    }

    #[test]
    fn pick_closest_works() {
        assert_eq!(pick_closest(2, 5, 3), 2);
        assert_eq!(pick_closest(2, 5, 4), 5);
        assert_eq!(pick_closest(3, 3, 3), 3);
    }

    #[test]
    fn most_threatening_finds_richest() {
        let mut rng = test_rng();
        let config = GameConfig {
            players: vec![
                ("P1".into(), true),
                ("P2".into(), true),
                ("P3".into(), true),
            ],
            starting_chips: 6,
            buy_in: 100,
            enable_shift_tokens: false,
            token_distribution: TokenDistribution::None,
        };

        let mut state = game::new_game(config, &mut rng).unwrap();
        // Give P2 extra chips
        state.players[2].chips = 10;

        let threat = most_threatening(&state, 0);
        assert_eq!(threat, Some(2));
    }
}
