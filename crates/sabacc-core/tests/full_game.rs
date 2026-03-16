use rand::rngs::SmallRng;
use rand::SeedableRng;

use sabacc_core::bot::BasicBot;
use sabacc_core::game::{self, Action, GameConfig, GamePhase};
use sabacc_core::turn::TurnAction;

/// Run a complete game with a fixed seed and verify it terminates.
#[test]
fn full_game_terminates_with_seed_42() {
    let mut rng = SmallRng::seed_from_u64(42);
    let config = GameConfig {
        players: vec![
            ("Alice".into(), false),
            ("Bot1".into(), true),
        ],
        starting_chips: 6,
        buy_in: 100,
        enable_shift_tokens: false,
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    let bot = BasicBot;
    let max_iterations = 10_000;
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            panic!("game did not terminate within {} iterations", max_iterations);
        }

        match &state.phase {
            GamePhase::GameOver { winner } => {
                // Verify winner exists and is not eliminated
                let winner_player = state.players.iter().find(|p| p.id == *winner).unwrap();
                assert!(!winner_player.is_eliminated);
                assert!(winner_player.chips > 0 || winner_player.pot > 0);
                break;
            }
            GamePhase::TurnAction => {
                let player = &state.players[state.current_player_idx];
                if player.is_bot {
                    state = game::advance_bots(state, &bot, &mut rng).unwrap();
                } else {
                    // Human player: always stand for simplicity
                    let pid = player.id;
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
            }
            GamePhase::ChoosingDiscard { player_id, .. } => {
                let pid = *player_id;
                let player = state.players.iter().find(|p| p.id == pid).unwrap();
                if player.is_bot {
                    state = game::advance_bots(state, &bot, &mut rng).unwrap();
                } else {
                    // Human: discard drawn card
                    state = game::apply_action(
                        state,
                        Action::ChooseDiscard {
                            player_id: pid,
                            choice: sabacc_core::turn::DiscardChoice::DiscardDrawn,
                        },
                        &mut rng,
                    )
                    .unwrap();
                }
            }
            GamePhase::ImpostorReveal { pending, .. } => {
                let pid = pending[0];
                let player = state.players.iter().find(|p| p.id == pid).unwrap();
                if player.is_bot {
                    state = game::advance_bots(state, &bot, &mut rng).unwrap();
                } else {
                    // Human: roll dice and pick first value
                    let die1: u8 = rand::Rng::gen_range(&mut rng, 1..=6);
                    let die2: u8 = rand::Rng::gen_range(&mut rng, 1..=6);

                    let hand = player.hand.as_ref().unwrap();
                    let sand_choice =
                        if hand.sand.value == sabacc_core::card::CardValue::Impostor {
                            Some(die1)
                        } else {
                            None
                        };
                    let blood_choice =
                        if hand.blood.value == sabacc_core::card::CardValue::Impostor {
                            Some(die1)
                        } else {
                            None
                        };

                    state = game::apply_action(
                        state,
                        Action::SubmitImpostorChoice(sabacc_core::scoring::ImpostorChoice {
                            player_id: pid,
                            die1,
                            die2,
                            sand_choice,
                            blood_choice,
                        }),
                        &mut rng,
                    )
                    .unwrap();
                }
            }
            GamePhase::Reveal { .. } => {
                state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
            }
            GamePhase::RoundEnd => {
                state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
            }
            GamePhase::Setup => {
                panic!("unexpected Setup phase during game");
            }
        }
    }
}

/// Verify chip conservation: chips are only destroyed by penalties.
#[test]
fn chip_conservation_across_rounds() {
    let mut rng = SmallRng::seed_from_u64(123);
    let config = GameConfig {
        players: vec![
            ("P1".into(), true),
            ("P2".into(), true),
            ("P3".into(), true),
        ],
        starting_chips: 6,
        buy_in: 100,
        enable_shift_tokens: false,
    };

    let initial_total_chips = 6u16 * 3;
    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    let bot = BasicBot;
    let max_iterations = 10_000;
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            panic!("game did not terminate");
        }

        // Check that total chips never increase
        let total: u16 = state
            .players
            .iter()
            .map(|p| p.total_chips() as u16)
            .sum();
        assert!(
            total <= initial_total_chips,
            "chips increased! {} > {}",
            total,
            initial_total_chips
        );

        match &state.phase {
            GamePhase::GameOver { .. } => break,
            _ => {
                state = game::advance_bots(state, &bot, &mut rng).unwrap();

                // If advance_bots didn't change phase (human turn, reveal, etc.)
                // handle remaining phases
                match &state.phase {
                    GamePhase::Reveal { .. } | GamePhase::RoundEnd => {
                        state =
                            game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Test with 4 players (max).
#[test]
fn four_player_game_terminates() {
    let mut rng = SmallRng::seed_from_u64(999);
    let config = GameConfig {
        players: vec![
            ("P1".into(), true),
            ("P2".into(), true),
            ("P3".into(), true),
            ("P4".into(), true),
        ],
        starting_chips: 4,
        buy_in: 50,
        enable_shift_tokens: false,
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    let bot = BasicBot;
    let max_iterations = 50_000;
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            panic!("4-player game did not terminate within {} iterations", max_iterations);
        }

        match &state.phase {
            GamePhase::GameOver { .. } => break,
            GamePhase::Reveal { .. } | GamePhase::RoundEnd => {
                state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
            }
            _ => {
                state = game::advance_bots(state, &bot, &mut rng).unwrap();
            }
        }
    }
}
