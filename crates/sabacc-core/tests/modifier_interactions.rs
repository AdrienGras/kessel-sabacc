mod common;

use sabacc_core::card::{Card, Family};
use sabacc_core::hand::{Hand, HandRank};
use sabacc_core::scoring::{self, ActiveModifiers, PrimeSabaccModifier};

// ============================================================
// Markdown modifier
// ============================================================

#[test]
fn markdown_pure_sabacc_stays_pure() {
    let mods = ActiveModifiers {
        markdown_active: true,
        ..Default::default()
    };
    let hand = Hand::new(Card::sylop(Family::Sand), Card::sylop(Family::Blood)).unwrap();
    let rank = scoring::evaluate_hand(&hand, None, &mods, 0).unwrap();
    assert_eq!(rank, HandRank::PureSabacc);
}

#[test]
fn markdown_sylop_becomes_non_sabacc() {
    let mods = ActiveModifiers {
        markdown_active: true,
        ..Default::default()
    };
    let hand = Hand::new(Card::sylop(Family::Sand), Card::number(Family::Blood, 5)).unwrap();
    let rank = scoring::evaluate_hand(&hand, None, &mods, 0).unwrap();
    assert_eq!(rank, HandRank::NonSabacc { difference: 5 });
}

// ============================================================
// CookTheBooks modifier
// ============================================================

#[test]
fn cook_the_books_does_not_affect_non_sabacc() {
    let mods = ActiveModifiers {
        cook_the_books_active: true,
        ..Default::default()
    };
    // NonSabacc{1} should still beat NonSabacc{5} (closer to 0 = better)
    let players = vec![
        (0u8, HandRank::NonSabacc { difference: 1 }, 2u8),
        (1, HandRank::NonSabacc { difference: 5 }, 2),
    ];
    let results = scoring::resolve_round(&players, &mods);
    assert!(results[0].is_winner);
    assert!(!results[1].is_winner);
}

#[test]
fn cook_the_books_does_not_affect_sylop_sabacc() {
    let mods = ActiveModifiers {
        cook_the_books_active: true,
        ..Default::default()
    };
    // SylopSabacc{1} should still beat SylopSabacc{6}
    let players = vec![
        (0u8, HandRank::SylopSabacc { value: 1 }, 2u8),
        (1, HandRank::SylopSabacc { value: 6 }, 2),
    ];
    let results = scoring::resolve_round(&players, &mods);
    assert!(results[0].is_winner);
    assert!(!results[1].is_winner);
}

#[test]
fn cook_the_books_does_not_affect_pure_sabacc() {
    let mods = ActiveModifiers {
        cook_the_books_active: true,
        ..Default::default()
    };
    // PureSabacc always beats Sabacc{6} even with CookTheBooks
    let players = vec![
        (0u8, HandRank::PureSabacc, 2u8),
        (1, HandRank::Sabacc { pair_value: 6 }, 2),
    ];
    let results = scoring::resolve_round(&players, &mods);
    assert!(results[0].is_winner);
    assert!(!results[1].is_winner);
}

#[test]
fn cook_the_books_inverts_sabacc_pairs() {
    let mods = ActiveModifiers {
        cook_the_books_active: true,
        ..Default::default()
    };
    // Normally 1/1 > 6/6, but CookTheBooks inverts: 6/6 > 1/1
    let players = vec![
        (0u8, HandRank::Sabacc { pair_value: 6 }, 2u8),
        (1, HandRank::Sabacc { pair_value: 1 }, 2),
    ];
    let results = scoring::resolve_round(&players, &mods);
    assert!(results[0].is_winner);
    assert!(!results[1].is_winner);
}

// ============================================================
// MajorFraud modifier
// ============================================================

#[test]
fn major_fraud_impostor_matches_6() {
    let mods = ActiveModifiers {
        major_fraud_active: true,
        ..Default::default()
    };
    let hand = Hand::new(
        Card::impostor(Family::Sand),
        Card::number(Family::Blood, 6),
    )
    .unwrap();
    // No impostor choice needed with MajorFraud
    let rank = scoring::evaluate_hand(&hand, None, &mods, 0).unwrap();
    assert_eq!(rank, HandRank::Sabacc { pair_value: 6 });
}

#[test]
fn major_fraud_impostor_no_match() {
    let mods = ActiveModifiers {
        major_fraud_active: true,
        ..Default::default()
    };
    let hand = Hand::new(
        Card::impostor(Family::Sand),
        Card::number(Family::Blood, 3),
    )
    .unwrap();
    let rank = scoring::evaluate_hand(&hand, None, &mods, 0).unwrap();
    assert_eq!(rank, HandRank::NonSabacc { difference: 3 }); // 6 - 3 = 3
}

#[test]
fn major_fraud_double_impostor_sabacc_6() {
    let mods = ActiveModifiers {
        major_fraud_active: true,
        ..Default::default()
    };
    let hand = Hand::new(
        Card::impostor(Family::Sand),
        Card::impostor(Family::Blood),
    )
    .unwrap();
    // Both forced to 6 → Sabacc{6}
    let rank = scoring::evaluate_hand(&hand, None, &mods, 0).unwrap();
    assert_eq!(rank, HandRank::Sabacc { pair_value: 6 });
}

// ============================================================
// PrimeSabacc modifier
// ============================================================

#[test]
fn prime_sabacc_beats_sabacc_but_not_pure() {
    let mods = ActiveModifiers {
        prime_sabacc: Some(PrimeSabaccModifier {
            player_id: 1,
            chosen_value: 3,
        }),
        ..Default::default()
    };
    let players = vec![
        (0u8, HandRank::PureSabacc, 2u8),
        (1, HandRank::NonSabacc { difference: 5 }, 2), // overridden to PrimeSabacc
        (2, HandRank::Sabacc { pair_value: 1 }, 2),
    ];
    let results = scoring::resolve_round(&players, &mods);

    // P0 (PureSabacc) wins
    assert!(results[0].is_winner);
    // P1 (PrimeSabacc) loses but with sabacc penalty (1 chip)
    assert!(!results[1].is_winner);
    assert_eq!(results[1].penalty, 1);
    assert_eq!(results[1].rank, HandRank::PrimeSabacc { value: 3 });
    // P2 (Sabacc{1}) also loses with sabacc penalty
    assert!(!results[2].is_winner);
    assert_eq!(results[2].penalty, 1);
}

#[test]
fn cook_the_books_does_not_affect_prime_sabacc_tier() {
    let mods = ActiveModifiers {
        cook_the_books_active: true,
        prime_sabacc: Some(PrimeSabaccModifier {
            player_id: 0,
            chosen_value: 3,
        }),
        ..Default::default()
    };
    // PrimeSabacc tier is (0,1), CookTheBooks only inverts tier 2
    let players = vec![
        (0u8, HandRank::NonSabacc { difference: 5 }, 2u8),
        (1, HandRank::Sabacc { pair_value: 6 }, 2),
    ];
    let results = scoring::resolve_round(&players, &mods);
    // PrimeSabacc still beats Sabacc even with CookTheBooks
    assert!(results[0].is_winner);
    assert!(!results[1].is_winner);
}

// ============================================================
// Combined modifiers
// ============================================================

#[test]
fn markdown_plus_major_fraud_impostor_sylop() {
    let mods = ActiveModifiers {
        markdown_active: true,
        major_fraud_active: true,
        ..Default::default()
    };
    // Impostor Sand (forced to 6) + Sylop Blood (marked down to 0) = NonSabacc{6}
    let hand = Hand::new(
        Card::impostor(Family::Sand),
        Card::sylop(Family::Blood),
    )
    .unwrap();
    let rank = scoring::evaluate_hand(&hand, None, &mods, 0).unwrap();
    assert_eq!(rank, HandRank::NonSabacc { difference: 6 });
}
