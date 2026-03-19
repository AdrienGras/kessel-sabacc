use rand::rngs::SmallRng;
use rand::SeedableRng;

use sabacc_core::card::{Card, Family};
use sabacc_core::game::{self, Action, GameConfig, GamePhase, TokenDistribution};
use sabacc_core::hand::Hand;
use sabacc_core::shift_token::ShiftToken;
use sabacc_core::turn::TurnAction;
use sabacc_core::PlayerId;

/// Create a started game with N human players.
pub fn started_game(
    n_players: usize,
    chips: u8,
    seed: u64,
) -> (sabacc_core::game::GameState, SmallRng) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let players: Vec<(String, bool)> = (0..n_players)
        .map(|i| (format!("P{}", i), false))
        .collect();
    let config = GameConfig {
        players,
        starting_chips: chips,
        buy_in: 100,
        enable_shift_tokens: false,
        token_distribution: TokenDistribution::None,
    };
    let state = game::new_game(config, &mut rng).unwrap();
    let state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();
    (state, rng)
}

/// Create a started game with tokens enabled.
pub fn game_with_tokens(
    n_players: usize,
    chips: u8,
    tokens: Vec<Vec<ShiftToken>>,
    seed: u64,
) -> (sabacc_core::game::GameState, SmallRng) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let players: Vec<(String, bool)> = (0..n_players)
        .map(|i| (format!("P{}", i), false))
        .collect();
    let config = GameConfig {
        players,
        starting_chips: chips,
        buy_in: 100,
        enable_shift_tokens: true,
        token_distribution: TokenDistribution::None,
    };
    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();
    for (i, t) in tokens.into_iter().enumerate() {
        if i < state.players.len() {
            state.players[i].shift_tokens = t;
        }
    }
    (state, rng)
}

/// Override hands for specific players.
pub fn set_hands(state: &mut sabacc_core::game::GameState, hands: Vec<(PlayerId, Hand)>) {
    for (pid, hand) in hands {
        if let Some(player) = state.players.iter_mut().find(|p| p.id == pid) {
            player.hand = Some(hand);
        }
    }
}

/// All active players stand for one turn cycle.
pub fn all_stand(
    mut state: sabacc_core::game::GameState,
    rng: &mut SmallRng,
) -> sabacc_core::game::GameState {
    let n_active = state.players.iter().filter(|p| !p.is_eliminated).count();
    for _ in 0..n_active {
        let pid = state.players[state.current_player_idx].id;
        state = game::apply_action(
            state,
            Action::PlayerAction {
                player_id: pid,
                action: TurnAction::Stand,
            },
            rng,
        )
        .unwrap();
    }
    state
}

/// Fast-forward through 3 turns of all-stand to reach revelation.
pub fn fast_forward_to_revelation(
    mut state: sabacc_core::game::GameState,
    rng: &mut SmallRng,
) -> sabacc_core::game::GameState {
    for _ in 0..3 {
        state = all_stand(state, rng);
    }
    state
}

/// Shorthand to create a Hand from sand and blood cards.
pub fn make_hand(sand: Card, blood: Card) -> Hand {
    Hand::new(sand, blood).unwrap()
}

/// Shorthand for a numbered Sand card.
pub fn sand(n: u8) -> Card {
    Card::number(Family::Sand, n)
}

/// Shorthand for a numbered Blood card.
pub fn blood(n: u8) -> Card {
    Card::number(Family::Blood, n)
}
