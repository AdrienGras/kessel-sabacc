mod common;

use common::*;

use sabacc_core::card::{Card, Family};
use sabacc_core::game::{self, Action, GamePhase};
use sabacc_core::hand::HandRank;

/// NonSabacc penalty eliminates a player with insufficient chips.
#[test]
fn non_sabacc_penalty_eliminates_player() {
    let (mut state, mut rng) = started_game(2, 3, 42);

    // P0 gets Sabacc{2} (winner), P1 gets NonSabacc{5} (diff 5 > 3 chips → eliminated)
    set_hands(&mut state, vec![
        (0, make_hand(sand(2), blood(2))),
        (1, make_hand(sand(1), blood(6))),
    ]);

    // Play through 3 turns
    state = fast_forward_to_revelation(state, &mut rng);

    // Should be in Reveal
    match &state.phase {
        GamePhase::Reveal { results } => {
            let p0 = results.iter().find(|r| r.player_id == 0).unwrap();
            assert!(p0.is_winner);
            assert_eq!(p0.rank, HandRank::Sabacc { pair_value: 2 });

            let p1 = results.iter().find(|r| r.player_id == 1).unwrap();
            assert!(!p1.is_winner);
            assert_eq!(p1.rank, HandRank::NonSabacc { difference: 5 });
            assert_eq!(p1.penalty, 5);
        }
        other => panic!("expected Reveal, got {:?}", other),
    }

    // Advance: Reveal → RoundEnd
    state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
    assert!(matches!(state.phase, GamePhase::RoundEnd));

    // P1 should be eliminated (penalty 5 >= chips 3)
    assert!(state.players[1].is_eliminated);
    assert_eq!(state.players[1].chips, 0);

    // P0 should get invested chips back (pot → chips)
    assert_eq!(state.players[0].pot, 0);

    // Advance: RoundEnd → GameOver (only 1 player left)
    state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
    match &state.phase {
        GamePhase::GameOver { winner } => {
            assert_eq!(*winner, 0);
        }
        other => panic!("expected GameOver, got {:?}", other),
    }
}

/// Exact chip penalty (penalty == chips) eliminates the player.
#[test]
fn exact_chip_penalty_eliminates() {
    let (mut state, mut rng) = started_game(2, 3, 42);

    // P0 wins, P1 has NonSabacc{3} → penalty exactly 3
    set_hands(&mut state, vec![
        (0, make_hand(sand(4), blood(4))),
        (1, make_hand(sand(1), blood(4))), // diff = 3
    ]);

    state = fast_forward_to_revelation(state, &mut rng);
    state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap(); // → RoundEnd

    // P1: penalty 3 >= chips 3 → eliminated
    assert!(state.players[1].is_eliminated);
    assert_eq!(state.players[1].chips, 0);
}

/// Winner recovers invested chips after round resolution.
#[test]
fn winner_recovers_invested_chips() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    // Manually invest chips for P0 (simulate draws)
    state.players[0].chips = 4;
    state.players[0].pot = 2;

    set_hands(&mut state, vec![
        (0, make_hand(sand(1), blood(1))), // Sabacc{1} (strongest pair)
        (1, make_hand(sand(3), blood(5))), // NonSabacc{2}
    ]);

    state = fast_forward_to_revelation(state, &mut rng);
    state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap(); // → RoundEnd

    // Winner P0 gets pot back
    assert_eq!(state.players[0].chips, 6); // 4 + 2 pot
    assert_eq!(state.players[0].pot, 0);

    // Loser P1 loses pot and gets penalty
    assert_eq!(state.players[1].pot, 0);
}

/// Losing with a Sabacc hand costs only 1 chip penalty.
#[test]
fn losing_sabacc_penalty_is_one() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    set_hands(&mut state, vec![
        (0, make_hand(sand(1), blood(1))), // Sabacc{1} (best pair)
        (1, make_hand(sand(6), blood(6))), // Sabacc{6} (worst pair, but still Sabacc)
    ]);

    state = fast_forward_to_revelation(state, &mut rng);

    match &state.phase {
        GamePhase::Reveal { results } => {
            let p1 = results.iter().find(|r| r.player_id == 1).unwrap();
            assert!(!p1.is_winner);
            assert_eq!(p1.penalty, 1); // Losing Sabacc = 1 chip penalty
        }
        other => panic!("expected Reveal, got {:?}", other),
    }

    state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
    assert_eq!(state.players[1].chips, 5); // 6 - 1 penalty
    assert!(!state.players[1].is_eliminated);
}

/// Tied players both win (no penalty for either).
#[test]
fn tie_both_win_no_penalty() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    set_hands(&mut state, vec![
        (0, make_hand(sand(3), blood(3))),
        (1, make_hand(sand(3), blood(3))),
    ]);

    state = fast_forward_to_revelation(state, &mut rng);

    match &state.phase {
        GamePhase::Reveal { results } => {
            assert!(results[0].is_winner);
            assert!(results[1].is_winner);
            assert_eq!(results[0].penalty, 0);
            assert_eq!(results[1].penalty, 0);
        }
        other => panic!("expected Reveal, got {:?}", other),
    }
}

/// Eliminated player is skipped in the next round.
#[test]
fn eliminated_player_skipped_next_round() {
    let (mut state, mut rng) = started_game(3, 3, 42);

    // P0 wins, P1 gets eliminated (NonSabacc{5}), P2 survives (Sabacc{4}, penalty 1)
    set_hands(&mut state, vec![
        (0, make_hand(sand(1), blood(1))), // Sabacc{1}
        (1, make_hand(sand(1), blood(6))), // NonSabacc{5}
        (2, make_hand(sand(4), blood(4))), // Sabacc{4}
    ]);

    state = fast_forward_to_revelation(state, &mut rng);

    // Reveal → RoundEnd
    state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
    assert!(state.players[1].is_eliminated);

    // RoundEnd → new round (TurnAction)
    state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap();
    assert!(matches!(state.phase, GamePhase::TurnAction));

    // P1 should be skipped — current player should be P0 or P2
    let current_id = state.players[state.current_player_idx].id;
    assert_ne!(current_id, 1, "eliminated P1 should be skipped");

    // P1 should not have a hand
    assert!(state.players[1].hand.is_none());

    // P0 and P2 should have new hands
    assert!(state.players[0].hand.is_some());
    assert!(state.players[2].hand.is_some());
}

/// Elimination order is tracked correctly.
#[test]
fn elimination_order_tracked() {
    let (mut state, mut rng) = started_game(3, 3, 42);

    set_hands(&mut state, vec![
        (0, make_hand(sand(1), blood(1))), // Sabacc{1} - winner
        (1, make_hand(sand(1), blood(6))), // NonSabacc{5} - eliminated
        (2, make_hand(sand(4), blood(4))), // Sabacc{4} - survives
    ]);

    state = fast_forward_to_revelation(state, &mut rng);
    state = game::apply_action(state, Action::AdvanceRound, &mut rng).unwrap(); // → RoundEnd

    assert_eq!(state.elimination_order.len(), 1);
    assert_eq!(state.elimination_order[0].0, 1); // P1 eliminated
    assert_eq!(state.elimination_order[0].1, 1); // In round 1
}

/// PureSabacc beats everything.
#[test]
fn pure_sabacc_wins_against_all() {
    let (mut state, mut rng) = started_game(3, 6, 42);

    set_hands(&mut state, vec![
        (0, make_hand(Card::sylop(Family::Sand), Card::sylop(Family::Blood))), // PureSabacc
        (1, make_hand(sand(1), blood(1))), // Sabacc{1}
        (2, make_hand(Card::sylop(Family::Sand), blood(3))), // SylopSabacc{3}
    ]);

    state = fast_forward_to_revelation(state, &mut rng);

    match &state.phase {
        GamePhase::Reveal { results } => {
            let p0 = results.iter().find(|r| r.player_id == 0).unwrap();
            assert!(p0.is_winner);
            assert_eq!(p0.rank, HandRank::PureSabacc);

            // Both losers have Sabacc hands → penalty 1
            for pid in [1, 2] {
                let r = results.iter().find(|r| r.player_id == pid).unwrap();
                assert!(!r.is_winner);
                assert_eq!(r.penalty, 1);
            }
        }
        other => panic!("expected Reveal, got {:?}", other),
    }
}
