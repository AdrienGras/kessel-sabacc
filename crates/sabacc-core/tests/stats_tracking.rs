mod common;

use common::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use sabacc_core::bot::BotDifficulty;
use sabacc_core::game::{self, Action, GameConfig, GamePhase, TokenDistribution};
use sabacc_core::shift_token::ShiftToken;
use sabacc_core::turn::TurnAction;

#[test]
fn stats_track_stands_and_draws() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    let pid = state.players[state.current_player_idx].id;
    // Player 0 draws from SandDeck
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: pid,
            action: TurnAction::Draw(sabacc_core::turn::DrawSource::SandDeck),
        },
        &mut rng,
    )
    .unwrap();
    // Discard the drawn card
    state = game::apply_action(
        state,
        Action::ChooseDiscard {
            player_id: pid,
            choice: sabacc_core::turn::DiscardChoice::DiscardDrawn,
        },
        &mut rng,
    )
    .unwrap();

    // Player 1 stands
    let pid1 = state.players[state.current_player_idx].id;
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: pid1,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    assert_eq!(state.stats.get(0).unwrap().draws_count, 1);
    assert_eq!(state.stats.get(0).unwrap().stands_count, 0);
    assert_eq!(state.stats.get(1).unwrap().stands_count, 1);
    assert_eq!(state.stats.get(1).unwrap().draws_count, 0);
}

#[test]
fn stats_chips_history_grows_with_rounds() {
    use sabacc_core::bot::BasicBot;

    let mut rng = SmallRng::seed_from_u64(42);
    let config = GameConfig {
        players: vec![("P1".into(), true), ("P2".into(), true)],
        starting_chips: 6,
        buy_in: 100,
        enable_shift_tokens: false,
        token_distribution: TokenDistribution::None,
        bot_difficulty: BotDifficulty::Basic,
    };
    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    // Initial chips_history should have exactly 1 entry (baseline)
    assert_eq!(state.stats.get(0).unwrap().chips_history.len(), 1);
    assert_eq!(state.stats.get(0).unwrap().chips_history[0], 6);

    let bot = BasicBot;
    let mut rounds_completed = 0u8;

    for _ in 0..50_000 {
        match &state.phase {
            GamePhase::GameOver { .. } => break,
            GamePhase::Reveal { .. } | GamePhase::RoundEnd => {
                if matches!(state.phase, GamePhase::RoundEnd) {
                    rounds_completed += 1;
                }
                state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
            }
            _ => {
                state = game::advance_bots(state, &bot, &mut rng).unwrap();
            }
        }
    }

    // Each player's chips_history should have 1 (baseline) + N (one per round)
    for ps in &state.stats.player_stats {
        assert_eq!(
            ps.chips_history.len(),
            1 + rounds_completed as usize,
            "player {} chips_history length mismatch",
            ps.player_id
        );
    }
}

#[test]
fn stats_best_hand_tracked() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    // Set known hands: P0 gets Sabacc(3), P1 gets NonSabacc(4)
    set_hands(
        &mut state,
        vec![
            (0, make_hand(sand(3), blood(3))),
            (1, make_hand(sand(1), blood(5))),
        ],
    );

    // Fast-forward 3 turns (all stand)
    state = fast_forward_to_revelation(state, &mut rng);

    // Advance through Reveal -> RoundEnd
    state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();

    let p0_stats = state.stats.get(0).unwrap();
    assert_eq!(
        p0_stats.best_hand,
        Some(sabacc_core::hand::HandRank::Sabacc { pair_value: 3 })
    );
    assert_eq!(p0_stats.rounds_won, 1);
    assert_eq!(p0_stats.rounds_played, 1);
}

#[test]
fn stats_tokens_played_tracked() {
    let (mut state, mut rng) = game_with_tokens(
        2,
        6,
        vec![vec![ShiftToken::FreeDraw, ShiftToken::Markdown], vec![]],
        42,
    );

    // P0 plays FreeDraw
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::FreeDraw,
        },
        &mut rng,
    )
    .unwrap();
    assert_eq!(state.stats.get(0).unwrap().tokens_played, 1);

    // P0 stands, P1 stands (complete turn 1)
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 1,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // P0 plays Markdown in turn 2
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::Markdown,
        },
        &mut rng,
    )
    .unwrap();
    assert_eq!(state.stats.get(0).unwrap().tokens_played, 2);
    assert_eq!(state.stats.get(1).unwrap().tokens_played, 0);
}

#[test]
fn stats_tariff_vs_penalty_separation() {
    let (mut state, mut rng) = game_with_tokens(
        2,
        6,
        vec![vec![ShiftToken::GeneralTariff], vec![]],
        42,
    );

    // P0 plays GeneralTariff — P1 should lose 1 chip to tariff
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::GeneralTariff,
        },
        &mut rng,
    )
    .unwrap();

    let p1_stats = state.stats.get(1).unwrap();
    assert_eq!(p1_stats.chips_lost_to_tariffs, 1);
    assert_eq!(p1_stats.chips_lost_to_penalties, 0);
}
