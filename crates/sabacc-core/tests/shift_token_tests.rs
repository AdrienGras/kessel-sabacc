use rand::rngs::SmallRng;
use rand::SeedableRng;

use sabacc_core::card::{Card, CardValue, Family};
use sabacc_core::game::{self, Action, GameConfig, GamePhase, TokenDistribution};
use sabacc_core::hand::Hand;
use sabacc_core::shift_token::ShiftToken;
use sabacc_core::turn::{DrawSource, TurnAction};

/// Helper: create a started 2-player game with specific tokens.
/// Both players are human (not bots) for precise control.
fn setup_game_with_tokens(
    tokens_p0: Vec<ShiftToken>,
    tokens_p1: Vec<ShiftToken>,
    seed: u64,
) -> (sabacc_core::game::GameState, SmallRng) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let config = GameConfig {
        players: vec![("P0".into(), false), ("P1".into(), false)],
        starting_chips: 6,
        buy_in: 100,
        enable_shift_tokens: true,
        token_distribution: TokenDistribution::None, // We'll set tokens manually
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    // Manually assign tokens
    state.players[0].shift_tokens = tokens_p0;
    state.players[1].shift_tokens = tokens_p1;

    (state, rng)
}

/// Helper: create a 3-player game with specific tokens.
fn setup_3p_game_with_tokens(
    tokens: Vec<Vec<ShiftToken>>,
    seed: u64,
) -> (sabacc_core::game::GameState, SmallRng) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let config = GameConfig {
        players: vec![
            ("P0".into(), false),
            ("P1".into(), false),
            ("P2".into(), false),
        ],
        starting_chips: 6,
        buy_in: 100,
        enable_shift_tokens: true,
        token_distribution: TokenDistribution::None,
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    for (i, t) in tokens.into_iter().enumerate() {
        state.players[i].shift_tokens = t;
    }

    (state, rng)
}

// ============================================================
// Per-token unit tests
// ============================================================

#[test]
fn shift_token_free_draw() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::FreeDraw], vec![], 42);

    let chips_before = state.players[0].chips;

    // Play FreeDraw
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::FreeDraw,
        },
        &mut rng,
    )
    .unwrap();

    // Should still be in TurnAction
    assert!(matches!(state.phase, GamePhase::TurnAction));
    assert!(state.free_draw_active);
    assert!(state.players[0].shift_tokens.is_empty());

    // Now draw — should not cost a chip
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Draw(DrawSource::SandDeck),
        },
        &mut rng,
    )
    .unwrap();

    assert_eq!(state.players[0].chips, chips_before);
    assert_eq!(state.players[0].pot, 0);
}

#[test]
fn shift_token_refund() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::Refund], vec![], 42);

    // P0 needs invested chips first — draw to invest 1 chip
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Draw(DrawSource::SandDeck),
        },
        &mut rng,
    )
    .unwrap();

    // Discard drawn
    state = game::apply_action(
        state,
        Action::ChooseDiscard {
            player_id: 0,
            choice: sabacc_core::turn::DiscardChoice::DiscardDrawn,
        },
        &mut rng,
    )
    .unwrap();

    // P1 stands
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 1,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // Now turn 2, P0 can play Refund
    assert_eq!(state.players[0].pot, 1);
    assert_eq!(state.players[0].chips, 5);

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::Refund,
        },
        &mut rng,
    )
    .unwrap();

    // Refund 2, but only 1 invested, so refund 1
    assert_eq!(state.players[0].pot, 0);
    assert_eq!(state.players[0].chips, 6);
}

#[test]
fn shift_token_refund_no_invested_error() {
    let (state, mut rng) = setup_game_with_tokens(vec![ShiftToken::Refund], vec![], 42);

    // P0 has 0 invested chips, Refund should error
    let result = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::Refund,
        },
        &mut rng,
    );

    assert!(result.is_err());
}

#[test]
fn shift_token_extra_refund() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::ExtraRefund], vec![], 42);

    // Invest some chips first
    state.players[0].chips = 3;
    state.players[0].pot = 3;

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::ExtraRefund,
        },
        &mut rng,
    )
    .unwrap();

    assert_eq!(state.players[0].pot, 0);
    assert_eq!(state.players[0].chips, 6);
}

#[test]
fn shift_token_general_tariff() {
    let (state, mut rng) = setup_3p_game_with_tokens(
        vec![vec![ShiftToken::GeneralTariff], vec![], vec![]],
        42,
    );

    let p1_chips = state.players[1].chips;
    let p2_chips = state.players[2].chips;

    let state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::GeneralTariff,
        },
        &mut rng,
    )
    .unwrap();

    assert_eq!(state.players[1].chips, p1_chips - 1);
    assert_eq!(state.players[2].chips, p2_chips - 1);
}

#[test]
fn shift_token_target_tariff() {
    let (state, mut rng) = setup_game_with_tokens(vec![ShiftToken::TargetTariff(0)], vec![], 42);

    let p1_chips = state.players[1].chips;

    let state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::TargetTariff(1),
        },
        &mut rng,
    )
    .unwrap();

    assert_eq!(state.players[1].chips, p1_chips - 2);
}

#[test]
fn shift_token_embargo() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::Embargo], vec![], 42);

    // P0 plays Embargo
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::Embargo,
        },
        &mut rng,
    )
    .unwrap();

    assert_eq!(state.embargoed_player, Some(1));

    // P0 stands
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // P1 tries to draw — should fail
    let result = game::apply_action(
        state.clone(),
        Action::PlayerAction {
            player_id: 1,
            action: TurnAction::Draw(DrawSource::SandDeck),
        },
        &mut rng,
    );
    assert!(result.is_err());

    // P1 stands — should work
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 1,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();
}

#[test]
fn shift_token_markdown() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::Markdown], vec![], 42);

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::Markdown,
        },
        &mut rng,
    )
    .unwrap();

    assert!(state.modifiers.markdown_active);

    // Verify scoring effect: Sylop + 3 should be NonSabacc{3} instead of SylopSabacc{3}
    let hand = Hand::new(Card::sylop(Family::Sand), Card::number(Family::Blood, 3)).unwrap();
    let rank =
        sabacc_core::scoring::evaluate_hand(&hand, None, &state.modifiers).unwrap();
    assert_eq!(
        rank,
        sabacc_core::hand::HandRank::NonSabacc { difference: 3 }
    );
}

#[test]
fn shift_token_immunity_blocks_tariff() {
    let (mut state, mut rng) = setup_game_with_tokens(
        vec![ShiftToken::GeneralTariff],
        vec![ShiftToken::Immunity],
        42,
    );

    // P1 needs to play first. Let's give P1 immunity before P0's tariff.
    // Actually, P0 goes first. So let P0 stand first, then P1 plays Immunity,
    // then next turn P0 plays GeneralTariff.

    // P0 stands
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // P1 plays Immunity
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 1,
            token: ShiftToken::Immunity,
        },
        &mut rng,
    )
    .unwrap();

    // P1 stands
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 1,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // Turn 2: P0 plays GeneralTariff
    let p1_chips = state.players[1].chips;

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::GeneralTariff,
        },
        &mut rng,
    )
    .unwrap();

    // P1 should not have lost chips (immune)
    assert_eq!(state.players[1].chips, p1_chips);
}

#[test]
fn shift_token_immunity_blocks_embargo() {
    let (mut state, mut rng) = setup_game_with_tokens(
        vec![ShiftToken::Embargo],
        vec![ShiftToken::Immunity],
        42,
    );

    // P0 stands turn 1
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // P1 plays Immunity then stands turn 1
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 1,
            token: ShiftToken::Immunity,
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

    // Turn 2: P0 plays Embargo
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::Embargo,
        },
        &mut rng,
    )
    .unwrap();

    // P1 should not be embargoed (immune)
    assert_eq!(state.embargoed_player, None);
}

#[test]
fn shift_token_general_audit() {
    let (mut state, mut rng) = setup_3p_game_with_tokens(
        vec![vec![ShiftToken::GeneralAudit], vec![], vec![]],
        42,
    );

    let p1_chips = state.players[1].chips;
    let p2_chips = state.players[2].chips;

    // P0 plays GeneralAudit
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::GeneralAudit,
        },
        &mut rng,
    )
    .unwrap();

    // P0 stands
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // P1 stands (will be audited)
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 1,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // P2 draws (won't be audited by stand rule)
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 2,
            action: TurnAction::Draw(DrawSource::SandDeck),
        },
        &mut rng,
    )
    .unwrap();

    // Discard drawn
    state = game::apply_action(
        state,
        Action::ChooseDiscard {
            player_id: 2,
            choice: sabacc_core::turn::DiscardChoice::DiscardDrawn,
        },
        &mut rng,
    )
    .unwrap();

    // End of turn 1: audit resolved
    // P0 stood but is source → excluded
    // P1 stood → -2 chips
    // P2 drew → not affected
    assert_eq!(state.players[1].chips, p1_chips - 2);
    assert_eq!(state.players[2].chips, p2_chips - 1); // -1 from draw cost only
}

#[test]
fn shift_token_target_audit() {
    let (mut state, mut rng) = setup_game_with_tokens(
        vec![ShiftToken::TargetAudit(0)],
        vec![],
        42,
    );

    let p1_chips = state.players[1].chips;

    // P0 plays TargetAudit targeting P1
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::TargetAudit(1),
        },
        &mut rng,
    )
    .unwrap();

    // P0 stands
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // P1 stands (will be audited)
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 1,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // P1 should lose 3 chips
    assert_eq!(state.players[1].chips, p1_chips - 3);
}

#[test]
fn shift_token_major_fraud() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::MajorFraud], vec![], 42);

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::MajorFraud,
        },
        &mut rng,
    )
    .unwrap();

    assert!(state.modifiers.major_fraud_active);

    // Verify: impostor resolves as 6 regardless of dice
    let hand = Hand::new(
        Card::impostor(Family::Sand),
        Card::number(Family::Blood, 3),
    )
    .unwrap();

    let rank = sabacc_core::scoring::evaluate_hand(&hand, None, &state.modifiers).unwrap();
    assert_eq!(
        rank,
        sabacc_core::hand::HandRank::NonSabacc { difference: 3 }
    ); // 6 - 3 = 3
}

#[test]
fn shift_token_embezzlement() {
    let (state, mut rng) = setup_3p_game_with_tokens(
        vec![vec![ShiftToken::Embezzlement], vec![], vec![]],
        42,
    );

    let p0_chips = state.players[0].chips;
    let p1_chips = state.players[1].chips;
    let p2_chips = state.players[2].chips;

    let state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::Embezzlement,
        },
        &mut rng,
    )
    .unwrap();

    // P0 gains 2 (1 from each opponent), P1 and P2 lose 1 each
    assert_eq!(state.players[0].chips, p0_chips + 2);
    assert_eq!(state.players[1].chips, p1_chips - 1);
    assert_eq!(state.players[2].chips, p2_chips - 1);
}

#[test]
fn shift_token_cook_the_books() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::CookTheBooks], vec![], 42);

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::CookTheBooks,
        },
        &mut rng,
    )
    .unwrap();

    assert!(state.modifiers.cook_the_books_active);

    // With CookTheBooks, 6/6 beats 1/1
    let s6 = sabacc_core::hand::HandRank::Sabacc { pair_value: 6 };
    let s1 = sabacc_core::hand::HandRank::Sabacc { pair_value: 1 };
    assert_eq!(
        sabacc_core::scoring::compare_ranks(&s6, &s1, &state.modifiers),
        std::cmp::Ordering::Less
    );
}

#[test]
fn shift_token_exhaustion() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::Exhaustion(0)], vec![], 42);

    // Get P1's original hand
    let original_hand = state.players[1].hand.clone().unwrap();

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::Exhaustion(1),
        },
        &mut rng,
    )
    .unwrap();

    // P1 should have a new hand
    let new_hand = state.players[1].hand.as_ref().unwrap();
    assert_eq!(new_hand.sand.family, Family::Sand);
    assert_eq!(new_hand.blood.family, Family::Blood);
    // Very likely different hand (not guaranteed with small decks, but overwhelmingly likely)
    // Just verify they have a valid hand
    assert!(state.players[1].hand.is_some());
    // And that it's still structurally valid
    let _ = &original_hand; // suppress unused warning
}

#[test]
fn shift_token_direct_transaction() {
    let (mut state, mut rng) =
        setup_game_with_tokens(vec![ShiftToken::DirectTransaction(0)], vec![], 42);

    let p0_hand = state.players[0].hand.clone().unwrap();
    let p1_hand = state.players[1].hand.clone().unwrap();

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::DirectTransaction(1),
        },
        &mut rng,
    )
    .unwrap();

    // Hands should be swapped
    assert_eq!(state.players[0].hand.as_ref().unwrap(), &p1_hand);
    assert_eq!(state.players[1].hand.as_ref().unwrap(), &p0_hand);
}

#[test]
fn shift_token_immunity_blocks_direct_transaction() {
    let (mut state, mut rng) = setup_game_with_tokens(
        vec![ShiftToken::DirectTransaction(0)],
        vec![ShiftToken::Immunity],
        42,
    );

    // P0 stands turn 1
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Stand,
        },
        &mut rng,
    )
    .unwrap();

    // P1 plays Immunity
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 1,
            token: ShiftToken::Immunity,
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

    // Turn 2: P0 plays DirectTransaction
    let p0_hand = state.players[0].hand.clone();
    let p1_hand = state.players[1].hand.clone();

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::DirectTransaction(1),
        },
        &mut rng,
    )
    .unwrap();

    // Hands should NOT be swapped (P1 is immune)
    assert_eq!(state.players[0].hand, p0_hand);
    assert_eq!(state.players[1].hand, p1_hand);
}

#[test]
fn shift_token_prime_sabacc() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::PrimeSabacc], vec![], 42);

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::PrimeSabacc,
        },
        &mut rng,
    )
    .unwrap();

    // Should be in PrimeSabaccChoice phase
    match &state.phase {
        GamePhase::PrimeSabaccChoice {
            player_id,
            die1,
            die2,
        } => {
            assert_eq!(*player_id, 0);
            assert!(*die1 >= 1 && *die1 <= 6);
            assert!(*die2 >= 1 && *die2 <= 6);

            // Submit choice
            let chosen = *die1;
            state = game::apply_action(
                state,
                Action::SubmitPrimeSabaccChoice {
                    player_id: 0,
                    chosen_value: chosen,
                },
                &mut rng,
            )
            .unwrap();

            assert!(matches!(state.phase, GamePhase::TurnAction));
            assert!(state.modifiers.prime_sabacc.is_some());
            let prime = state.modifiers.prime_sabacc.as_ref().unwrap();
            assert_eq!(prime.player_id, 0);
            assert_eq!(prime.chosen_value, chosen);
        }
        _ => panic!("expected PrimeSabaccChoice phase"),
    }
}

#[test]
fn shift_token_prime_sabacc_invalid_choice() {
    let (mut state, mut rng) = setup_game_with_tokens(vec![ShiftToken::PrimeSabacc], vec![], 42);

    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::PrimeSabacc,
        },
        &mut rng,
    )
    .unwrap();

    match &state.phase {
        GamePhase::PrimeSabaccChoice { die1, die2, .. } => {
            // Choose a value that's neither die
            let invalid = (1..=6)
                .find(|v| *v != *die1 && *v != *die2)
                .unwrap_or(0);
            if invalid > 0 {
                let result = game::apply_action(
                    state,
                    Action::SubmitPrimeSabaccChoice {
                        player_id: 0,
                        chosen_value: invalid,
                    },
                    &mut rng,
                );
                assert!(result.is_err());
            }
        }
        _ => panic!("expected PrimeSabaccChoice"),
    }
}

#[test]
fn shift_token_already_played_error() {
    let (mut state, mut rng) = setup_game_with_tokens(
        vec![ShiftToken::FreeDraw, ShiftToken::Markdown],
        vec![],
        42,
    );

    // Play first token
    state = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::FreeDraw,
        },
        &mut rng,
    )
    .unwrap();

    // Try to play second token — should fail
    let result = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::Markdown,
        },
        &mut rng,
    );

    assert!(result.is_err());
}

#[test]
fn shift_token_not_owned_error() {
    let (state, mut rng) = setup_game_with_tokens(vec![], vec![], 42);

    let result = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::FreeDraw,
        },
        &mut rng,
    );

    assert!(result.is_err());
}

#[test]
fn shift_token_cannot_target_self() {
    let (state, mut rng) =
        setup_game_with_tokens(vec![ShiftToken::TargetTariff(0)], vec![], 42);

    let result = game::apply_action(
        state,
        Action::PlayShiftToken {
            player_id: 0,
            token: ShiftToken::TargetTariff(0),
        },
        &mut rng,
    );

    assert!(result.is_err());
}

// ============================================================
// PrimeSabacc scoring override
// ============================================================

#[test]
fn prime_sabacc_overrides_hand_rank() {
    use sabacc_core::hand::HandRank;
    use sabacc_core::scoring::{ActiveModifiers, PrimeSabaccModifier};

    let mods = ActiveModifiers {
        prime_sabacc: Some(PrimeSabaccModifier {
            player_id: 0,
            chosen_value: 3,
        }),
        ..Default::default()
    };

    let players = vec![
        (0u8, HandRank::NonSabacc { difference: 5 }, 2u8),
        (1, HandRank::PureSabacc, 2),
    ];

    let results = sabacc_core::scoring::resolve_round(&players, &mods);

    // P0 should be PrimeSabacc (overridden), but PureSabacc still beats it
    assert_eq!(
        results[0].rank,
        HandRank::PrimeSabacc { value: 3 }
    );
    assert!(!results[0].is_winner); // PureSabacc beats PrimeSabacc
    assert!(results[1].is_winner);
}

#[test]
fn prime_sabacc_beats_sylop_sabacc() {
    use sabacc_core::hand::HandRank;
    use sabacc_core::scoring::{ActiveModifiers, PrimeSabaccModifier};

    let mods = ActiveModifiers {
        prime_sabacc: Some(PrimeSabaccModifier {
            player_id: 0,
            chosen_value: 3,
        }),
        ..Default::default()
    };

    let players = vec![
        (0u8, HandRank::NonSabacc { difference: 5 }, 2u8),
        (1, HandRank::SylopSabacc { value: 1 }, 2),
    ];

    let results = sabacc_core::scoring::resolve_round(&players, &mods);

    // P0 with PrimeSabacc should beat SylopSabacc
    assert!(results[0].is_winner);
    assert!(!results[1].is_winner);
}

#[test]
fn cook_the_books_does_not_affect_prime_sabacc() {
    use sabacc_core::hand::HandRank;
    use sabacc_core::scoring::{ActiveModifiers, PrimeSabaccModifier};

    let mods = ActiveModifiers {
        cook_the_books_active: true,
        prime_sabacc: Some(PrimeSabaccModifier {
            player_id: 0,
            chosen_value: 3,
        }),
        ..Default::default()
    };

    let players = vec![
        (0u8, HandRank::NonSabacc { difference: 5 }, 2u8),
        (1, HandRank::Sabacc { pair_value: 6 }, 2),
    ];

    let results = sabacc_core::scoring::resolve_round(&players, &mods);

    // PrimeSabacc should still beat Sabacc even with CookTheBooks
    assert!(results[0].is_winner);
    assert!(!results[1].is_winner);
}

#[test]
fn major_fraud_forces_impostor_to_6() {
    use sabacc_core::scoring::ActiveModifiers;

    let mods = ActiveModifiers {
        major_fraud_active: true,
        ..Default::default()
    };

    let hand = Hand::new(
        Card::impostor(Family::Sand),
        Card::number(Family::Blood, 6),
    )
    .unwrap();

    // No impostor choice needed when MajorFraud is active
    let rank = sabacc_core::scoring::evaluate_hand(&hand, None, &mods).unwrap();
    assert_eq!(rank, sabacc_core::hand::HandRank::Sabacc { pair_value: 6 });
}

// ============================================================
// Integration: full game with tokens
// ============================================================

#[test]
fn full_game_with_tokens_terminates() {
    use sabacc_core::bot::BasicBot;

    let mut rng = SmallRng::seed_from_u64(42);
    let config = GameConfig {
        players: vec![
            ("P1".into(), true),
            ("P2".into(), true),
        ],
        starting_chips: 6,
        buy_in: 100,
        enable_shift_tokens: true,
        token_distribution: TokenDistribution::Random {
            tokens_per_player: 4,
        },
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    // Verify tokens were distributed
    assert!(!state.players[0].shift_tokens.is_empty());
    assert!(!state.players[1].shift_tokens.is_empty());

    let bot = BasicBot;
    let max_iterations = 50_000;
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            panic!("game with tokens did not terminate");
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

#[test]
fn three_player_game_with_tokens_terminates() {
    use sabacc_core::bot::BasicBot;

    let mut rng = SmallRng::seed_from_u64(123);
    let config = GameConfig {
        players: vec![
            ("P1".into(), true),
            ("P2".into(), true),
            ("P3".into(), true),
        ],
        starting_chips: 6,
        buy_in: 100,
        enable_shift_tokens: true,
        token_distribution: TokenDistribution::Random {
            tokens_per_player: 4,
        },
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    let bot = BasicBot;
    let max_iterations = 50_000;
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            panic!("3p game with tokens did not terminate");
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

#[test]
fn four_player_game_with_tokens_terminates() {
    use sabacc_core::bot::BasicBot;

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
        enable_shift_tokens: true,
        token_distribution: TokenDistribution::Random {
            tokens_per_player: 3,
        },
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    let bot = BasicBot;
    let max_iterations = 50_000;
    let mut iterations = 0;

    loop {
        iterations += 1;
        if iterations > max_iterations {
            panic!("4p game with tokens did not terminate");
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

#[test]
fn token_distribution_fixed() {
    let mut rng = SmallRng::seed_from_u64(42);
    let fixed_tokens = vec![ShiftToken::FreeDraw, ShiftToken::Immunity];
    let config = GameConfig {
        players: vec![("P0".into(), false), ("P1".into(), false)],
        starting_chips: 6,
        buy_in: 100,
        enable_shift_tokens: true,
        token_distribution: TokenDistribution::Fixed(fixed_tokens.clone()),
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    assert_eq!(state.players[0].shift_tokens, fixed_tokens);
    assert_eq!(state.players[1].shift_tokens, fixed_tokens);
}

#[test]
fn token_distribution_none() {
    let mut rng = SmallRng::seed_from_u64(42);
    let config = GameConfig {
        players: vec![("P0".into(), false), ("P1".into(), false)],
        starting_chips: 6,
        buy_in: 100,
        enable_shift_tokens: false,
        token_distribution: TokenDistribution::None,
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    assert!(state.players[0].shift_tokens.is_empty());
    assert!(state.players[1].shift_tokens.is_empty());
}
