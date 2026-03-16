use rand::Rng;

use crate::card::{Card, Family};
use crate::deck::FamilyDeck;
use crate::error::GameError;
use crate::player::Player;
use crate::round;
use crate::scoring::{ActiveModifiers, ImpostorChoice, RoundResult};
use crate::shift_token::ShiftToken;
use crate::turn::{DiscardChoice, DrawSource, TurnAction};
use crate::PlayerId;

/// Configuration for a new game.
#[derive(Debug, Clone, PartialEq)]
pub struct GameConfig {
    /// Player names and bot status. First element is the human player.
    pub players: Vec<(String, bool)>,
    /// Starting chips per player.
    pub starting_chips: u8,
    /// Credits buy-in per player.
    pub buy_in: u32,
    /// Whether shift tokens are enabled (Phase 2).
    pub enable_shift_tokens: bool,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            players: vec![
                ("Player".into(), false),
                ("Bot 1".into(), true),
            ],
            starting_chips: 6,
            buy_in: 100,
            enable_shift_tokens: false,
        }
    }
}

/// The current phase of the game.
#[derive(Debug, Clone, PartialEq)]
pub enum GamePhase {
    /// Game not yet started.
    Setup,
    /// A player must choose Draw or Stand.
    TurnAction,
    /// A player has drawn a card and must choose what to discard.
    ChoosingDiscard {
        /// The player making the choice.
        player_id: PlayerId,
        /// The card that was drawn.
        drawn_card: Card,
    },
    /// One or more players have Impostors and must choose die values.
    ImpostorReveal {
        /// Players who still need to submit their impostor choices.
        pending: Vec<PlayerId>,
        /// Choices already submitted.
        submitted: Vec<ImpostorChoice>,
    },
    /// All hands revealed, showing results.
    Reveal {
        /// The round results.
        results: Vec<RoundResult>,
    },
    /// Round ended, waiting to advance.
    RoundEnd,
    /// Game is over.
    GameOver {
        /// The winner's ID.
        winner: PlayerId,
    },
}

/// The complete game state.
#[derive(Debug, Clone, PartialEq)]
pub struct GameState {
    /// All players in the game.
    pub players: Vec<Player>,
    /// The Sand deck.
    pub sand_deck: FamilyDeck,
    /// The Blood deck.
    pub blood_deck: FamilyDeck,
    /// Current round number (starts at 1).
    pub round: u8,
    /// Current turn within the round (1, 2, or 3).
    pub turn: u8,
    /// Index of the current player in the players vec.
    pub current_player_idx: usize,
    /// Current game phase.
    pub phase: GamePhase,
    /// Credits in the pot.
    pub credits_in_pot: u32,
    /// Active modifiers from shift tokens (default in Phase 1).
    pub modifiers: ActiveModifiers,
    /// Game configuration.
    pub config: GameConfig,
}

/// An action that can be applied to the game state.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Start the game from Setup phase.
    StartGame,
    /// A player performs a turn action (Draw or Stand).
    PlayerAction {
        player_id: PlayerId,
        action: TurnAction,
    },
    /// A player chooses what to discard after drawing.
    ChooseDiscard {
        player_id: PlayerId,
        choice: DiscardChoice,
    },
    /// A player submits their impostor die choice.
    SubmitImpostorChoice(ImpostorChoice),
    /// Advance to the next round after viewing results.
    AdvanceRound,
    /// Play a shift token (rejected in Phase 1).
    PlayShiftToken {
        player_id: PlayerId,
        token: ShiftToken,
    },
}

/// Create a new game state from configuration.
pub fn new_game(config: GameConfig, rng: &mut impl Rng) -> Result<GameState, GameError> {
    if config.players.len() < 2 || config.players.len() > 4 {
        return Err(GameError::InvalidConfig {
            reason: format!(
                "need 2-4 players, got {}",
                config.players.len()
            ),
        });
    }

    let players: Vec<Player> = config
        .players
        .iter()
        .enumerate()
        .map(|(i, (name, is_bot))| Player::new(i as PlayerId, name.clone(), config.starting_chips, *is_bot))
        .collect();

    let credits_in_pot = config.buy_in * players.len() as u32;

    Ok(GameState {
        players,
        sand_deck: FamilyDeck::new(Family::Sand, rng),
        blood_deck: FamilyDeck::new(Family::Blood, rng),
        round: 0,
        turn: 0,
        current_player_idx: 0,
        phase: GamePhase::Setup,
        credits_in_pot,
        modifiers: ActiveModifiers::default(),
        config,
    })
}

/// Apply an action to the game state, returning the new state or an error.
pub fn apply_action(
    state: GameState,
    action: Action,
    rng: &mut impl Rng,
) -> Result<GameState, GameError> {
    match action {
        Action::StartGame => apply_start_game(state, rng),
        Action::PlayerAction { player_id, action } => {
            apply_player_action(state, player_id, action, rng)
        }
        Action::ChooseDiscard { player_id, choice } => {
            apply_choose_discard(state, player_id, choice)
        }
        Action::SubmitImpostorChoice(choice) => apply_impostor_choice(state, choice, rng),
        Action::AdvanceRound => apply_advance_round(state, rng),
        Action::PlayShiftToken { .. } => {
            if !state.config.enable_shift_tokens {
                Err(GameError::ShiftTokensDisabled)
            } else {
                Err(GameError::InvalidActionForPhase {
                    reason: "shift tokens not yet implemented".into(),
                })
            }
        }
    }
}

/// Get the list of available actions for the current state.
pub fn available_actions(state: &GameState) -> Vec<Action> {
    match &state.phase {
        GamePhase::Setup => vec![Action::StartGame],
        GamePhase::TurnAction => {
            let player = &state.players[state.current_player_idx];
            if player.is_eliminated {
                return vec![];
            }
            let pid = player.id;
            let mut actions = vec![Action::PlayerAction {
                player_id: pid,
                action: TurnAction::Stand,
            }];

            // Can only draw if player has chips
            if player.chips > 0 {
                for source in [
                    DrawSource::SandDeck,
                    DrawSource::BloodDeck,
                    DrawSource::SandDiscard,
                    DrawSource::BloodDiscard,
                ] {
                    // Only offer discard sources if there's a card to draw
                    let available = match source {
                        DrawSource::SandDiscard => state.sand_deck.peek_discard().is_some(),
                        DrawSource::BloodDiscard => state.blood_deck.peek_discard().is_some(),
                        _ => true,
                    };
                    if available {
                        actions.push(Action::PlayerAction {
                            player_id: pid,
                            action: TurnAction::Draw(source),
                        });
                    }
                }
            }

            actions
        }
        GamePhase::ChoosingDiscard { player_id, .. } => {
            vec![
                Action::ChooseDiscard {
                    player_id: *player_id,
                    choice: DiscardChoice::KeepDrawn,
                },
                Action::ChooseDiscard {
                    player_id: *player_id,
                    choice: DiscardChoice::DiscardDrawn,
                },
            ]
        }
        GamePhase::ImpostorReveal { .. } => {
            // Impostor choices require dice rolls; caller must construct them manually
            vec![]
        }
        GamePhase::Reveal { .. } | GamePhase::RoundEnd => {
            vec![Action::AdvanceRound]
        }
        GamePhase::GameOver { .. } => vec![],
    }
}

/// Advance bot players automatically. Returns state after all bot actions.
pub fn advance_bots(
    mut state: GameState,
    bot: &impl crate::bot::BotStrategy,
    rng: &mut impl Rng,
) -> Result<GameState, GameError> {
    loop {
        match &state.phase {
            GamePhase::TurnAction => {
                let player = &state.players[state.current_player_idx];
                if !player.is_bot || player.is_eliminated {
                    return Ok(state);
                }
                let action = bot.choose_action(&state, rng);
                state = apply_action(state, action, rng)?;
            }
            GamePhase::ChoosingDiscard { player_id, .. } => {
                let pid = *player_id;
                let player = state.players.iter().find(|p| p.id == pid);
                match player {
                    Some(p) if p.is_bot => {
                        let action = bot.choose_discard(&state, rng);
                        state = apply_action(state, action, rng)?;
                    }
                    _ => return Ok(state),
                }
            }
            GamePhase::ImpostorReveal { pending, .. } => {
                if pending.is_empty() {
                    return Ok(state);
                }
                let pid = pending[0];
                let player = state.players.iter().find(|p| p.id == pid);
                match player {
                    Some(p) if p.is_bot => {
                        let action = bot.choose_impostor(&state, rng);
                        state = apply_action(state, action, rng)?;
                    }
                    _ => return Ok(state),
                }
            }
            _ => return Ok(state),
        }
    }
}

// --- Internal helpers ---

fn apply_start_game(
    mut state: GameState,
    rng: &mut impl Rng,
) -> Result<GameState, GameError> {
    if !matches!(state.phase, GamePhase::Setup) {
        return Err(GameError::InvalidActionForPhase {
            reason: "game already started".into(),
        });
    }

    state.round = 1;
    state.turn = 1;
    state.current_player_idx = 0;

    round::deal_hands(
        &mut state.players,
        &mut state.sand_deck,
        &mut state.blood_deck,
        rng,
    )?;

    state.phase = GamePhase::TurnAction;
    // Skip eliminated players
    skip_eliminated(&mut state);

    Ok(state)
}

fn apply_player_action(
    mut state: GameState,
    player_id: PlayerId,
    action: TurnAction,
    rng: &mut impl Rng,
) -> Result<GameState, GameError> {
    if !matches!(state.phase, GamePhase::TurnAction) {
        return Err(GameError::InvalidActionForPhase {
            reason: "not in TurnAction phase".into(),
        });
    }

    let current = &state.players[state.current_player_idx];
    if current.id != player_id {
        return Err(GameError::NotPlayerTurn { player_id });
    }
    if current.is_eliminated {
        return Err(GameError::PlayerEliminated { player_id });
    }

    match action {
        TurnAction::Stand => {
            advance_turn(&mut state, rng)?;
            Ok(state)
        }
        TurnAction::Draw(source) => {
            // Pay 1 chip
            state.players[state.current_player_idx].pay_chip()?;

            // Draw the card
            let drawn = match source {
                DrawSource::SandDeck => state.sand_deck.draw(rng)?,
                DrawSource::BloodDeck => state.blood_deck.draw(rng)?,
                DrawSource::SandDiscard => state.sand_deck.draw_from_discard()?,
                DrawSource::BloodDiscard => state.blood_deck.draw_from_discard()?,
            };

            state.phase = GamePhase::ChoosingDiscard {
                player_id,
                drawn_card: drawn,
            };

            Ok(state)
        }
    }
}

fn apply_choose_discard(
    mut state: GameState,
    player_id: PlayerId,
    choice: DiscardChoice,
) -> Result<GameState, GameError> {
    let (expected_pid, drawn_card) = match &state.phase {
        GamePhase::ChoosingDiscard {
            player_id: pid,
            drawn_card,
        } => (*pid, drawn_card.clone()),
        _ => {
            return Err(GameError::InvalidActionForPhase {
                reason: "not in ChoosingDiscard phase".into(),
            });
        }
    };

    if player_id != expected_pid {
        return Err(GameError::NotPlayerTurn { player_id });
    }

    let player = &mut state.players[state.current_player_idx];
    let hand = player.hand.as_mut().ok_or(GameError::PlayerNotFound {
        player_id,
    })?;

    let drawn_family = drawn_card.family;

    // Determine which card in hand to swap based on the drawn card's family
    match drawn_family {
        Family::Sand => match choice {
            DiscardChoice::KeepDrawn => {
                let old = std::mem::replace(&mut hand.sand, drawn_card);
                state.sand_deck.discard(old);
            }
            DiscardChoice::DiscardDrawn => {
                state.sand_deck.discard(drawn_card);
            }
        },
        Family::Blood => match choice {
            DiscardChoice::KeepDrawn => {
                let old = std::mem::replace(&mut hand.blood, drawn_card);
                state.blood_deck.discard(old);
            }
            DiscardChoice::DiscardDrawn => {
                state.blood_deck.discard(drawn_card);
            }
        },
    }

    // Use a dummy rng for advancing (no randomness needed)
    let mut dummy_rng = DummyRng;
    advance_turn(&mut state, &mut dummy_rng)?;

    Ok(state)
}

fn apply_impostor_choice(
    mut state: GameState,
    choice: ImpostorChoice,
    _rng: &mut impl Rng,
) -> Result<GameState, GameError> {
    let (mut pending, mut submitted) = match &state.phase {
        GamePhase::ImpostorReveal {
            pending,
            submitted,
        } => (pending.clone(), submitted.clone()),
        _ => {
            return Err(GameError::InvalidActionForPhase {
                reason: "not in ImpostorReveal phase".into(),
            });
        }
    };

    let pid = choice.player_id;
    if !pending.contains(&pid) {
        return Err(GameError::InvalidActionForPhase {
            reason: format!("player {} is not pending impostor choice", pid),
        });
    }

    choice.validate()?;

    pending.retain(|&id| id != pid);
    submitted.push(choice);

    if pending.is_empty() {
        // All impostors resolved, go to reveal
        let results = round::resolve(&state.players, &submitted, &state.modifiers)?;
        state.phase = GamePhase::Reveal { results };
    } else {
        state.phase = GamePhase::ImpostorReveal {
            pending,
            submitted,
        };
    }

    Ok(state)
}

fn apply_advance_round(
    mut state: GameState,
    rng: &mut impl Rng,
) -> Result<GameState, GameError> {
    match &state.phase {
        GamePhase::Reveal { results } => {
            // Apply results
            let results = results.clone();
            round::apply_results(
                &mut state.players,
                &results,
                &mut state.sand_deck,
                &mut state.blood_deck,
            );
            state.phase = GamePhase::RoundEnd;
            Ok(state)
        }
        GamePhase::RoundEnd => {
            // Check for game over
            if let Some(winner) = round::check_game_over(&state.players) {
                state.phase = GamePhase::GameOver { winner };
                return Ok(state);
            }

            // Start new round
            state.round += 1;
            state.turn = 1;
            state.current_player_idx = 0;
            state.modifiers = ActiveModifiers::default();

            round::deal_hands(
                &mut state.players,
                &mut state.sand_deck,
                &mut state.blood_deck,
                rng,
            )?;

            state.phase = GamePhase::TurnAction;
            skip_eliminated(&mut state);

            Ok(state)
        }
        _ => Err(GameError::InvalidActionForPhase {
            reason: "not in Reveal or RoundEnd phase".into(),
        }),
    }
}

/// Advance to the next player/turn, or begin revelation if turn 3 is complete.
fn advance_turn(state: &mut GameState, _rng: &mut impl Rng) -> Result<(), GameError> {
    // Move to next player
    state.current_player_idx += 1;

    // Skip eliminated players
    while state.current_player_idx < state.players.len()
        && state.players[state.current_player_idx].is_eliminated
    {
        state.current_player_idx += 1;
    }

    // If we've gone through all players, advance the turn
    if state.current_player_idx >= state.players.len() {
        if state.turn >= 3 {
            // End of turn 3: begin revelation
            begin_revelation(state)?;
        } else {
            state.turn += 1;
            state.current_player_idx = 0;
            skip_eliminated(state);
            state.phase = GamePhase::TurnAction;
        }
    } else {
        state.phase = GamePhase::TurnAction;
    }

    Ok(())
}

/// Begin the revelation phase after turn 3.
fn begin_revelation(state: &mut GameState) -> Result<(), GameError> {
    let impostors = round::players_with_impostors(&state.players);

    if impostors.is_empty() {
        // No impostors: resolve directly
        let results = round::resolve(&state.players, &[], &state.modifiers)?;
        state.phase = GamePhase::Reveal { results };
    } else {
        state.phase = GamePhase::ImpostorReveal {
            pending: impostors,
            submitted: Vec::new(),
        };
    }

    Ok(())
}

/// Skip to the first non-eliminated player from current_player_idx.
fn skip_eliminated(state: &mut GameState) {
    while state.current_player_idx < state.players.len()
        && state.players[state.current_player_idx].is_eliminated
    {
        state.current_player_idx += 1;
    }
}

/// A dummy RNG that panics if used. Used in code paths that don't need randomness.
struct DummyRng;

impl rand::RngCore for DummyRng {
    fn next_u32(&mut self) -> u32 {
        panic!("DummyRng should not be called")
    }

    fn next_u64(&mut self) -> u64 {
        panic!("DummyRng should not be called")
    }

    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        panic!("DummyRng should not be called")
    }

    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
        panic!("DummyRng should not be called")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn test_rng() -> SmallRng {
        SmallRng::seed_from_u64(42)
    }

    fn default_config() -> GameConfig {
        GameConfig {
            players: vec![
                ("Alice".into(), false),
                ("Bob".into(), true),
            ],
            starting_chips: 6,
            buy_in: 100,
            enable_shift_tokens: false,
        }
    }

    #[test]
    fn new_game_creates_valid_state() {
        let mut rng = test_rng();
        let config = default_config();
        let state = new_game(config, &mut rng).unwrap();

        assert_eq!(state.players.len(), 2);
        assert_eq!(state.phase, GamePhase::Setup);
        assert_eq!(state.credits_in_pot, 200);
    }

    #[test]
    fn invalid_player_count() {
        let mut rng = test_rng();
        let config = GameConfig {
            players: vec![("Solo".into(), false)],
            ..default_config()
        };
        assert!(new_game(config, &mut rng).is_err());
    }

    #[test]
    fn start_game_transitions_to_turn_action() {
        let mut rng = test_rng();
        let state = new_game(default_config(), &mut rng).unwrap();
        let state = apply_action(state, Action::StartGame, &mut rng).unwrap();

        assert!(matches!(state.phase, GamePhase::TurnAction));
        assert_eq!(state.round, 1);
        assert_eq!(state.turn, 1);
        assert!(state.players[0].hand.is_some());
        assert!(state.players[1].hand.is_some());
    }

    #[test]
    fn stand_advances_to_next_player() {
        let mut rng = test_rng();
        let state = new_game(default_config(), &mut rng).unwrap();
        let state = apply_action(state, Action::StartGame, &mut rng).unwrap();

        let pid = state.players[state.current_player_idx].id;
        let state = apply_action(
            state,
            Action::PlayerAction {
                player_id: pid,
                action: TurnAction::Stand,
            },
            &mut rng,
        )
        .unwrap();

        // Should have advanced to next player
        assert!(matches!(state.phase, GamePhase::TurnAction));
        assert_eq!(state.current_player_idx, 1);
    }

    #[test]
    fn draw_transitions_to_choosing_discard() {
        let mut rng = test_rng();
        let state = new_game(default_config(), &mut rng).unwrap();
        let state = apply_action(state, Action::StartGame, &mut rng).unwrap();

        let pid = state.players[state.current_player_idx].id;
        let state = apply_action(
            state,
            Action::PlayerAction {
                player_id: pid,
                action: TurnAction::Draw(DrawSource::SandDeck),
            },
            &mut rng,
        )
        .unwrap();

        assert!(matches!(
            state.phase,
            GamePhase::ChoosingDiscard { .. }
        ));
        // Player should have paid 1 chip
        assert_eq!(state.players[0].chips, 5);
        assert_eq!(state.players[0].pot, 1);
    }

    #[test]
    fn wrong_player_rejected() {
        let mut rng = test_rng();
        let state = new_game(default_config(), &mut rng).unwrap();
        let state = apply_action(state, Action::StartGame, &mut rng).unwrap();

        let result = apply_action(
            state,
            Action::PlayerAction {
                player_id: 1, // Not player 0's turn
                action: TurnAction::Stand,
            },
            &mut rng,
        );

        assert!(matches!(result, Err(GameError::NotPlayerTurn { .. })));
    }

    #[test]
    fn shift_token_rejected_phase1() {
        let mut rng = test_rng();
        let state = new_game(default_config(), &mut rng).unwrap();
        let state = apply_action(state, Action::StartGame, &mut rng).unwrap();

        let result = apply_action(
            state,
            Action::PlayShiftToken {
                player_id: 0,
                token: ShiftToken::FreeDraw,
            },
            &mut rng,
        );

        assert!(matches!(result, Err(GameError::ShiftTokensDisabled)));
    }

    #[test]
    fn full_turn_cycle() {
        let mut rng = test_rng();
        let state = new_game(default_config(), &mut rng).unwrap();
        let mut state = apply_action(state, Action::StartGame, &mut rng).unwrap();

        // Play through 3 turns of 2 players standing
        for expected_turn in 1..=3 {
            assert_eq!(state.turn, expected_turn);
            for _ in 0..2 {
                let pid = state.players[state.current_player_idx].id;
                state = apply_action(
                    state,
                    Action::PlayerAction {
                        player_id: pid,
                        action: TurnAction::Stand,
                    },
                    &mut rng,
                )
                .unwrap();
            }
        }

        // After 3 turns, should be in reveal or impostor reveal
        assert!(
            matches!(state.phase, GamePhase::Reveal { .. })
                || matches!(state.phase, GamePhase::ImpostorReveal { .. })
        );
    }

    #[test]
    fn available_actions_setup() {
        let mut rng = test_rng();
        let state = new_game(default_config(), &mut rng).unwrap();
        let actions = available_actions(&state);
        assert_eq!(actions, vec![Action::StartGame]);
    }

    #[test]
    fn available_actions_turn() {
        let mut rng = test_rng();
        let state = new_game(default_config(), &mut rng).unwrap();
        let state = apply_action(state, Action::StartGame, &mut rng).unwrap();
        let actions = available_actions(&state);

        // Should have Stand + up to 4 draw sources
        assert!(actions.len() >= 2);
        assert!(actions.contains(&Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Stand,
        }));
    }
}
