use rand::Rng;

use crate::card::CardValue;
use crate::game::{Action, GamePhase, GameState};
use crate::scoring::ImpostorChoice;
use crate::shift_token::ShiftToken;
use crate::turn::{DiscardChoice, DrawSource, TurnAction};
use crate::PlayerId;

/// Bot difficulty level. Implements `BotStrategy` by delegation.
#[derive(Debug, Clone, PartialEq)]
pub enum BotDifficulty {
    /// Weighted-random decisions, forgiving for new players.
    Basic,
    /// EV-optimised strategy: deterministic thresholds, smart source selection, targeted tokens.
    Expert,
}

impl BotStrategy for BotDifficulty {
    fn choose_action(&self, state: &GameState, rng: &mut impl Rng) -> Action {
        match self {
            Self::Basic => BasicBot.choose_action(state, rng),
            Self::Expert => ExpertBot.choose_action(state, rng),
        }
    }

    fn choose_discard(&self, state: &GameState, rng: &mut impl Rng) -> Action {
        match self {
            Self::Basic => BasicBot.choose_discard(state, rng),
            Self::Expert => ExpertBot.choose_discard(state, rng),
        }
    }

    fn choose_impostor(&self, state: &GameState, rng: &mut impl Rng) -> Action {
        match self {
            Self::Basic => BasicBot.choose_impostor(state, rng),
            Self::Expert => ExpertBot.choose_impostor(state, rng),
        }
    }

    fn choose_token(&self, state: &GameState, rng: &mut impl Rng) -> Option<Action> {
        match self {
            Self::Basic => BasicBot.choose_token(state, rng),
            Self::Expert => ExpertBot.choose_token(state, rng),
        }
    }

    fn choose_prime_sabacc(&self, state: &GameState, rng: &mut impl Rng) -> Action {
        match self {
            Self::Basic => BasicBot.choose_prime_sabacc(state, rng),
            Self::Expert => ExpertBot.choose_prime_sabacc(state, rng),
        }
    }
}

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
        let draw_threshold = if let Some(ref hand) = player.hand {
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

        let can_draw_free = state.turn_state.free_draw_active;
        if !can_draw_free && (player.chips == 0 || rng.gen_range(0..100) >= draw_threshold) {
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
        default_impostor_choice(state, rng)
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
        default_prime_sabacc_choice(state)
    }
}

/// An expert bot that uses EV-optimised, deterministic strategy.
///
/// - Threshold-based draw/stand decisions depending on turn, chips, and hand difference
/// - Smart draw source: checks visible discards for guaranteed or likely improvements
/// - Deterministic token triggers (no random gate)
#[derive(Debug, Clone)]
pub struct ExpertBot;

impl BotStrategy for ExpertBot {
    fn choose_action(&self, state: &GameState, _rng: &mut impl Rng) -> Action {
        let player = &state.players[state.current_player_idx];
        let pid = player.id;
        let chips = player.chips;
        let turn = state.turn;

        let (sand_val, blood_val) = if let Some(ref hand) = player.hand {
            (
                card_numeric_value(&hand.sand.value),
                card_numeric_value(&hand.blood.value),
            )
        } else {
            return stand_action(pid);
        };

        let diff = sand_val.abs_diff(blood_val);

        // Sabacc override: D == 0 → Stand always
        // Exception: high pair on T1 with rich chips → may seek lower pair
        if diff == 0 {
            if turn == 1 && chips >= 6 && sand_val >= 5 {
                // Try to find a lower pair — draw from the worse family
                return self.smart_draw(state, pid, sand_val, blood_val);
            }
            return stand_action(pid);
        }

        // Can't draw without chips (and no FreeDraw)
        if chips == 0 && !state.turn_state.free_draw_active {
            return stand_action(pid);
        }

        // Threshold table: should_draw(diff, turn, chips)
        let should_draw = match chips {
            0..=2 => {
                // Survival mode: only draw on very bad hands
                matches!((turn, diff), (1, 5..) | (2, 5..))
                // T3 → Stand always
            }
            3..=5 => {
                // Normal: draw if D ≥ 3, but T3 requires chips ≥ 4
                if turn <= 2 {
                    diff >= 3
                } else {
                    diff >= 3 && chips >= 4
                }
            }
            _ => {
                // Rich (≥ 6): aggressive
                if turn <= 2 {
                    diff >= 2
                } else {
                    diff >= 3
                }
            }
        };

        if should_draw {
            self.smart_draw(state, pid, sand_val, blood_val)
        } else {
            stand_action(pid)
        }
    }

    fn choose_discard(&self, state: &GameState, rng: &mut impl Rng) -> Action {
        // Reuse BasicBot logic (already optimal)
        BasicBot.choose_discard(state, rng)
    }

    fn choose_impostor(&self, state: &GameState, rng: &mut impl Rng) -> Action {
        default_impostor_choice(state, rng)
    }

    fn choose_token(&self, state: &GameState, _rng: &mut impl Rng) -> Option<Action> {
        let player = &state.players[state.current_player_idx];
        let pid = player.id;

        if player.shift_tokens.is_empty() {
            return None;
        }

        let hand_diff = if let Some(ref hand) = player.hand {
            let s = card_numeric_value(&hand.sand.value);
            let b = card_numeric_value(&hand.blood.value);
            s.abs_diff(b)
        } else {
            3
        };

        let turn = state.turn;
        let chips = player.chips;
        let pot = player.pot;
        let threat = most_threatening(state, pid);
        let has_impostor = player.hand.as_ref().is_some_and(|h| {
            h.sand.value == CardValue::Impostor || h.blood.value == CardValue::Impostor
        });

        // Priority-ordered token checks (deterministic, no random gate)
        for token in &player.shift_tokens {
            let chosen = match token {
                // 1. FreeDraw: T1 and D ≥ 3 (free aggressive draw)
                ShiftToken::FreeDraw if turn == 1 && hand_diff >= 3 => {
                    Some(ShiftToken::FreeDraw)
                }

                // 2. Immunity: T1 (protect early investment)
                ShiftToken::Immunity if turn == 1 => Some(ShiftToken::Immunity),

                // 3. Embargo: T2+ and next player has ≥ own chips
                ShiftToken::Embargo if turn >= 2 => {
                    let next_has_more =
                        next_player_chips(state, pid).is_some_and(|c| c >= chips);
                    if next_has_more {
                        Some(ShiftToken::Embargo)
                    } else {
                        None
                    }
                }

                // 4. Refund: survival recovery
                ShiftToken::Refund if pot >= 2 && chips <= 3 => Some(ShiftToken::Refund),
                ShiftToken::ExtraRefund if pot >= 3 && chips <= 3 => {
                    Some(ShiftToken::ExtraRefund)
                }

                // 5. GeneralTariff: T2+ (maximize drain timing)
                ShiftToken::GeneralTariff if turn >= 2 => Some(ShiftToken::GeneralTariff),

                // 6. TargetTariff: T2+ targeting richest
                ShiftToken::TargetTariff(_) if turn >= 2 => {
                    threat.map(ShiftToken::TargetTariff)
                }

                // 7. GeneralAudit: T3 (punish Standers at reveal)
                ShiftToken::GeneralAudit if turn == 3 => Some(ShiftToken::GeneralAudit),

                // 8. TargetAudit: T3 targeting richest
                ShiftToken::TargetAudit(_) if turn == 3 => {
                    threat.map(ShiftToken::TargetAudit)
                }

                // 9. Exhaustion: T3 targeting richest (disrupt before reveal)
                ShiftToken::Exhaustion(_) if turn == 3 => threat.map(ShiftToken::Exhaustion),

                // 10. DirectTransaction: D ≥ 5 targeting threat (likely good hand)
                ShiftToken::DirectTransaction(_) if hand_diff >= 5 => {
                    threat.map(ShiftToken::DirectTransaction)
                }

                // 11. CookTheBooks: T3 and Sabacc with pair ≥ 4
                ShiftToken::CookTheBooks if turn == 3 && hand_diff == 0 => {
                    if let Some(ref hand) = player.hand {
                        let s = card_numeric_value(&hand.sand.value);
                        if s >= 4 {
                            Some(ShiftToken::CookTheBooks)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }

                // 12. MajorFraud: T2+ and holding Impostor
                ShiftToken::MajorFraud if turn >= 2 && has_impostor => {
                    Some(ShiftToken::MajorFraud)
                }

                // 13. Markdown: T2+
                ShiftToken::Markdown if turn >= 2 => Some(ShiftToken::Markdown),

                // 14. Embezzlement: T2+ and ≥ 3 opponents alive
                ShiftToken::Embezzlement if turn >= 2 => {
                    let alive = state
                        .players
                        .iter()
                        .filter(|p| !p.is_eliminated && p.id != pid)
                        .count();
                    if alive >= 3 {
                        Some(ShiftToken::Embezzlement)
                    } else {
                        None
                    }
                }

                // 15. PrimeSabacc: T3 and D ≥ 3 (hail mary)
                ShiftToken::PrimeSabacc if turn == 3 && hand_diff >= 3 => {
                    Some(ShiftToken::PrimeSabacc)
                }

                _ => None,
            };

            if let Some(token_to_play) = chosen {
                return Some(Action::PlayShiftToken {
                    player_id: pid,
                    token: token_to_play,
                });
            }
        }

        None
    }

    fn choose_prime_sabacc(&self, state: &GameState, _rng: &mut impl Rng) -> Action {
        default_prime_sabacc_choice(state)
    }
}

impl ExpertBot {
    /// Choose an intelligent draw source, checking visible discards first.
    fn smart_draw(&self, state: &GameState, pid: PlayerId, sand_val: u8, blood_val: u8) -> Action {
        let diff = sand_val.abs_diff(blood_val);

        // Check visible discards for guaranteed Sabacc or better improvement
        let sand_discard_val = state
            .sand_deck
            .peek_discard()
            .map(|c| card_numeric_value(&c.value));
        let blood_discard_val = state
            .blood_deck
            .peek_discard()
            .map(|c| card_numeric_value(&c.value));

        // If sand discard would replace our sand card → new diff = |discard_val - blood_val|
        let sand_discard_improvement = sand_discard_val.map(|dv| {
            let new_diff = dv.abs_diff(blood_val);
            (new_diff, DrawSource::SandDiscard)
        });
        // If blood discard would replace our blood card → new diff = |sand_val - discard_val|
        let blood_discard_improvement = blood_discard_val.map(|dv| {
            let new_diff = sand_val.abs_diff(dv);
            (new_diff, DrawSource::BloodDiscard)
        });

        // Find the best discard option
        let best_discard = [sand_discard_improvement, blood_discard_improvement]
            .into_iter()
            .flatten()
            .min_by_key(|(new_diff, _)| *new_diff);

        let source = if let Some((new_diff, source)) = best_discard {
            if new_diff == 0 {
                // Guaranteed Sabacc — always take it
                source
            } else if new_diff < diff.saturating_sub(1) {
                // Significantly better than current hand
                source
            } else {
                // Blind draw from the family with the higher card value
                if sand_val >= blood_val {
                    DrawSource::SandDeck
                } else {
                    DrawSource::BloodDeck
                }
            }
        } else {
            // No discards available — blind draw from worse family
            if sand_val >= blood_val {
                DrawSource::SandDeck
            } else {
                DrawSource::BloodDeck
            }
        };

        Action::PlayerAction {
            player_id: pid,
            action: TurnAction::Draw(source),
        }
    }
}

/// Default impostor choice: pick die closest to the other card's value.
fn default_impostor_choice(state: &GameState, rng: &mut impl Rng) -> Action {
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

/// Default PrimeSabacc choice: pick die closest to the minimum card value.
fn default_prime_sabacc_choice(state: &GameState) -> Action {
    let (player_id, die1, die2) = match &state.phase {
        GamePhase::PrimeSabaccChoice {
            player_id,
            die1,
            die2,
        } => (*player_id, *die1, *die2),
        _ => panic!("choose_prime_sabacc called in wrong phase"),
    };

    let player = state.players.iter().find(|p| p.id == player_id);
    let chosen = if let Some(player) = player {
        if let Some(ref hand) = player.hand {
            let sand_val = card_numeric_value(&hand.sand.value);
            let blood_val = card_numeric_value(&hand.blood.value);
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

/// Build a Stand action for the given player.
fn stand_action(pid: PlayerId) -> Action {
    Action::PlayerAction {
        player_id: pid,
        action: TurnAction::Stand,
    }
}


/// Get the chip count of the next non-eliminated player after the current one.
fn next_player_chips(state: &GameState, self_id: PlayerId) -> Option<u8> {
    let num = state.players.len();
    let self_idx = state
        .players
        .iter()
        .position(|p| p.id == self_id)
        .unwrap_or(0);
    for offset in 1..num {
        let idx = (self_idx + offset) % num;
        let p = &state.players[idx];
        if !p.is_eliminated && p.id != self_id {
            return Some(p.chips);
        }
    }
    None
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
            bot_difficulty: BotDifficulty::Basic,
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
            bot_difficulty: BotDifficulty::Basic,
        };

        let mut state = game::new_game(config, &mut rng).unwrap();
        // Give P2 extra chips
        state.players[2].chips = 10;

        let threat = most_threatening(&state, 0);
        assert_eq!(threat, Some(2));
    }

    // ── ExpertBot tests ─────────────────────────────────────────────

    /// Helper: build a started game, advance human to Stand, then mutate the bot's hand.
    fn expert_test_state(
        sand_val: CardValue,
        blood_val: CardValue,
        chips: u8,
        turn: u8,
        tokens: Vec<ShiftToken>,
    ) -> (GameState, SmallRng) {
        use crate::card::{Card, Family};
        use crate::hand::Hand;

        let mut rng = test_rng();
        let config = GameConfig {
            players: vec![
                ("Human".into(), false),
                ("Bot".into(), true),
            ],
            starting_chips: chips,
            buy_in: 100,
            enable_shift_tokens: !tokens.is_empty(),
            token_distribution: TokenDistribution::None,
            bot_difficulty: BotDifficulty::Expert,
        };

        let mut state = game::new_game(config, &mut rng).unwrap();
        state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

        // Advance to the desired turn (each turn = all players act)
        // We start at turn 1. For turns > 1, do dummy rounds.
        for _ in 1..turn {
            // Human stands
            let pid = state.players[state.current_player_idx].id;
            state = game::apply_action(
                state,
                Action::PlayerAction {
                    player_id: pid,
                    action: TurnAction::Stand,
                },
                &mut rng,
            )
            .unwrap();
            // Bot stands
            let pid = state.players[state.current_player_idx].id;
            state = game::apply_action(
                state,
                Action::PlayerAction {
                    player_id: pid,
                    action: TurnAction::Stand,
                },
                &mut rng,
            )
            .unwrap();
        }

        // Human stands on the target turn
        let pid = state.players[state.current_player_idx].id;
        state = game::apply_action(
            state,
            Action::PlayerAction {
                player_id: pid,
                action: TurnAction::Stand,
            },
            &mut rng,
        )
        .unwrap();

        // Now it's the bot's turn — set the hand and chips
        let bot = &mut state.players[1];
        bot.hand = Some(
            Hand::new(
                Card {
                    family: Family::Sand,
                    value: sand_val,
                },
                Card {
                    family: Family::Blood,
                    value: blood_val,
                },
            )
            .unwrap(),
        );
        bot.chips = chips;
        bot.shift_tokens = tokens;

        (state, rng)
    }

    #[test]
    fn expert_stands_on_sabacc() {
        let (state, mut rng) = expert_test_state(
            CardValue::Number(3),
            CardValue::Number(3),
            6,
            3,
            vec![],
        );
        let bot = ExpertBot;
        let action = bot.choose_action(&state, &mut rng);
        match action {
            Action::PlayerAction { action: TurnAction::Stand, .. } => {}
            other => panic!("expected Stand on Sabacc hand, got {other:?}"),
        }
    }

    #[test]
    fn expert_draws_high_diff() {
        let (state, mut rng) = expert_test_state(
            CardValue::Number(1),
            CardValue::Number(5),
            6,
            1,
            vec![],
        );
        let bot = ExpertBot;
        let action = bot.choose_action(&state, &mut rng);
        match action {
            Action::PlayerAction { action: TurnAction::Draw(_), .. } => {}
            other => panic!("expected Draw on D=4 T1, got {other:?}"),
        }
    }

    #[test]
    fn expert_conservative_low_chips() {
        // chips=1, D=3, T2 → Stand (survival mode, only draws D≥5)
        let (state, mut rng) = expert_test_state(
            CardValue::Number(1),
            CardValue::Number(4),
            1,
            2,
            vec![],
        );
        let bot = ExpertBot;
        let action = bot.choose_action(&state, &mut rng);
        match action {
            Action::PlayerAction { action: TurnAction::Stand, .. } => {}
            other => panic!("expected Stand with low chips, got {other:?}"),
        }
    }

    #[test]
    fn expert_prefers_discard_sabacc() {
        use crate::card::{Card, Family};
        // Set up: bot has sand=3, blood=5 (D=2). Put sand=5 on sand discard pile.
        // Sand discard 5 would give D = |5-5| = 0 → guaranteed Sabacc → should pick SandDiscard.
        let (mut state, mut rng) = expert_test_state(
            CardValue::Number(3),
            CardValue::Number(5),
            6,
            1,
            vec![],
        );
        // Place a Sand 5 on the sand discard pile
        state.sand_deck.discard(Card {
            family: Family::Sand,
            value: CardValue::Number(5),
        });
        let bot = ExpertBot;
        let action = bot.choose_action(&state, &mut rng);
        match action {
            Action::PlayerAction { action: TurnAction::Draw(DrawSource::SandDiscard), .. } => {}
            other => panic!("expected SandDiscard for guaranteed Sabacc, got {other:?}"),
        }
    }

    #[test]
    fn expert_token_free_draw_t1() {
        let (state, mut rng) = expert_test_state(
            CardValue::Number(1),
            CardValue::Number(5),
            6,
            1,
            vec![ShiftToken::FreeDraw],
        );
        let bot = ExpertBot;
        let token = bot.choose_token(&state, &mut rng);
        match token {
            Some(Action::PlayShiftToken { token: ShiftToken::FreeDraw, .. }) => {}
            other => panic!("expected FreeDraw token on T1 D≥3, got {other:?}"),
        }
    }

    #[test]
    fn expert_token_refund_survival() {
        let (mut state, mut rng) = expert_test_state(
            CardValue::Number(2),
            CardValue::Number(3),
            2,
            2,
            vec![ShiftToken::Refund],
        );
        // Need pot ≥ 2 for Refund trigger
        state.players[1].pot = 3;
        let bot = ExpertBot;
        let token = bot.choose_token(&state, &mut rng);
        match token {
            Some(Action::PlayShiftToken { token: ShiftToken::Refund, .. }) => {}
            other => panic!("expected Refund on survival mode, got {other:?}"),
        }
    }

    #[test]
    fn difficulty_delegates_correctly() {
        let (state, mut rng) = expert_test_state(
            CardValue::Number(3),
            CardValue::Number(3),
            6,
            1,
            vec![],
        );
        // BotDifficulty::Basic should give same result as BasicBot for same seed
        let mut rng1 = SmallRng::seed_from_u64(99);
        let mut rng2 = SmallRng::seed_from_u64(99);

        let basic_action = BasicBot.choose_action(&state, &mut rng1);
        let difficulty_action = BotDifficulty::Basic.choose_action(&state, &mut rng2);
        assert_eq!(basic_action, difficulty_action);
    }
}
