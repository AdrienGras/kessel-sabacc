mod common;

use common::*;

use sabacc_core::card::{Card, CardValue, Family};
use sabacc_core::game::{self, Action, GamePhase};
use sabacc_core::hand::HandRank;
use sabacc_core::scoring::ImpostorChoice;

/// Single impostor: inject hand, play to revelation, submit choice, verify rank.
#[test]
fn single_impostor_full_flow() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    // Inject hands: P0 has Impostor Sand + Blood 3, P1 has Sand 2 + Blood 2 (Sabacc{2})
    set_hands(&mut state, vec![
        (0, make_hand(Card::impostor(Family::Sand), blood(3))),
        (1, make_hand(sand(2), blood(2))),
    ]);

    // Fast-forward 3 turns of Stand
    state = fast_forward_to_revelation(state, &mut rng);

    // Should be in ImpostorReveal with P0 pending
    match &state.phase {
        GamePhase::ImpostorReveal { pending, .. } => {
            assert!(pending.contains(&0));
            assert!(!pending.contains(&1));
        }
        other => panic!("expected ImpostorReveal, got {:?}", other),
    }

    // Submit impostor choice: die1=3, die2=5, choose 3 for sand → Sabacc{3}
    state = game::apply_action(
        state,
        Action::SubmitImpostorChoice(ImpostorChoice {
            player_id: 0,
            die1: 3,
            die2: 5,
            sand_choice: Some(3),
            blood_choice: None,
        }),
        &mut rng,
    )
    .unwrap();

    // Should now be in Reveal
    match &state.phase {
        GamePhase::Reveal { results } => {
            let p0_result = results.iter().find(|r| r.player_id == 0).unwrap();
            assert_eq!(p0_result.rank, HandRank::Sabacc { pair_value: 3 });
        }
        other => panic!("expected Reveal, got {:?}", other),
    }
}

/// Two players with impostors: both must submit before Reveal.
#[test]
fn two_players_with_impostors() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    // Both players have impostors
    set_hands(&mut state, vec![
        (0, make_hand(Card::impostor(Family::Sand), blood(4))),
        (1, make_hand(Card::impostor(Family::Sand), blood(2))),
    ]);

    state = fast_forward_to_revelation(state, &mut rng);

    match &state.phase {
        GamePhase::ImpostorReveal { pending, .. } => {
            assert_eq!(pending.len(), 2);
        }
        other => panic!("expected ImpostorReveal, got {:?}", other),
    }

    // P0 submits
    state = game::apply_action(
        state,
        Action::SubmitImpostorChoice(ImpostorChoice {
            player_id: 0,
            die1: 4,
            die2: 2,
            sand_choice: Some(4),
            blood_choice: None,
        }),
        &mut rng,
    )
    .unwrap();

    // Still in ImpostorReveal, P1 pending
    match &state.phase {
        GamePhase::ImpostorReveal { pending, .. } => {
            assert_eq!(pending.len(), 1);
            assert_eq!(pending[0], 1);
        }
        other => panic!("expected ImpostorReveal with 1 pending, got {:?}", other),
    }

    // P1 submits
    state = game::apply_action(
        state,
        Action::SubmitImpostorChoice(ImpostorChoice {
            player_id: 1,
            die1: 2,
            die2: 5,
            sand_choice: Some(2),
            blood_choice: None,
        }),
        &mut rng,
    )
    .unwrap();

    // Now should be in Reveal
    assert!(matches!(state.phase, GamePhase::Reveal { .. }));
}

/// Double impostor (Sand + Blood): both choices required.
#[test]
fn double_impostor_both_choices() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    set_hands(&mut state, vec![
        (0, make_hand(Card::impostor(Family::Sand), Card::impostor(Family::Blood))),
        (1, make_hand(sand(3), blood(3))),
    ]);

    state = fast_forward_to_revelation(state, &mut rng);

    // Verify P0 has double impostor
    let p0_hand = state.players[0].hand.as_ref().unwrap();
    assert_eq!(p0_hand.sand.value, CardValue::Impostor);
    assert_eq!(p0_hand.blood.value, CardValue::Impostor);

    // Submit with both sand and blood choices = 4 → Sabacc{4}
    state = game::apply_action(
        state,
        Action::SubmitImpostorChoice(ImpostorChoice {
            player_id: 0,
            die1: 4,
            die2: 6,
            sand_choice: Some(4),
            blood_choice: Some(4),
        }),
        &mut rng,
    )
    .unwrap();

    match &state.phase {
        GamePhase::Reveal { results } => {
            let p0_result = results.iter().find(|r| r.player_id == 0).unwrap();
            assert_eq!(p0_result.rank, HandRank::Sabacc { pair_value: 4 });
        }
        other => panic!("expected Reveal, got {:?}", other),
    }
}

/// Invalid die choice is rejected.
#[test]
fn invalid_die_choice_rejected() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    set_hands(&mut state, vec![
        (0, make_hand(Card::impostor(Family::Sand), blood(3))),
        (1, make_hand(sand(2), blood(2))),
    ]);

    state = fast_forward_to_revelation(state, &mut rng);

    // Submit with invalid choice (4 is not die1=2 or die2=5)
    let result = game::apply_action(
        state,
        Action::SubmitImpostorChoice(ImpostorChoice {
            player_id: 0,
            die1: 2,
            die2: 5,
            sand_choice: Some(4), // invalid!
            blood_choice: None,
        }),
        &mut rng,
    );

    assert!(result.is_err());
}

/// No impostors → skip ImpostorReveal, go straight to Reveal.
#[test]
fn no_impostors_skip_to_reveal() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    set_hands(&mut state, vec![
        (0, make_hand(sand(3), blood(3))),
        (1, make_hand(sand(5), blood(2))),
    ]);

    state = fast_forward_to_revelation(state, &mut rng);

    // Should go directly to Reveal (no ImpostorReveal)
    match &state.phase {
        GamePhase::Reveal { results } => {
            assert_eq!(results.len(), 2);
            // P0 has Sabacc{3}, P1 has NonSabacc{3}
            let p0 = results.iter().find(|r| r.player_id == 0).unwrap();
            assert_eq!(p0.rank, HandRank::Sabacc { pair_value: 3 });
            assert!(p0.is_winner);
        }
        other => panic!("expected Reveal, got {:?}", other),
    }
}

/// Impostor + Sylop with Markdown active.
#[test]
fn impostor_sylop_with_markdown() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    // Enable shift tokens and set Markdown active via modifier
    state.config.enable_shift_tokens = true;
    state.modifiers.markdown_active = true;

    set_hands(&mut state, vec![
        (0, make_hand(Card::impostor(Family::Sand), Card::sylop(Family::Blood))),
        (1, make_hand(sand(1), blood(1))),
    ]);

    state = fast_forward_to_revelation(state, &mut rng);

    // P0 has impostor, needs choice
    state = game::apply_action(
        state,
        Action::SubmitImpostorChoice(ImpostorChoice {
            player_id: 0,
            die1: 3,
            die2: 5,
            sand_choice: Some(3),
            blood_choice: None, // Sylop doesn't need impostor choice
        }),
        &mut rng,
    )
    .unwrap();

    match &state.phase {
        GamePhase::Reveal { results } => {
            let p0 = results.iter().find(|r| r.player_id == 0).unwrap();
            // Sylop is marked down to 0, Impostor chose 3 → NonSabacc{3}
            assert_eq!(p0.rank, HandRank::NonSabacc { difference: 3 });
        }
        other => panic!("expected Reveal, got {:?}", other),
    }
}
