use rand::Rng;

use crate::bot::BotDifficulty;
use crate::card::{Card, Family};
use crate::deck::FamilyDeck;
use crate::error::GameError;
use crate::player::Player;
use crate::round;
use crate::scoring::{ActiveModifiers, ImpostorChoice, RoundResult};
use crate::shift_token::ShiftToken;
use crate::turn::{DiscardChoice, DrawSource, TurnAction};
use crate::stats::GameStats;
use crate::PlayerId;

/// How shift tokens are distributed to players at game start.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenDistribution {
    /// Each player gets `tokens_per_player` random tokens from the 16 types.
    Random { tokens_per_player: usize },
    /// Each player gets a fixed set of tokens.
    Fixed(Vec<ShiftToken>),
    /// No tokens distributed (Phase 1 compatibility).
    None,
}

/// Pending audit effects to resolve at end of turn.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PendingAudit {
    /// Whether a GeneralAudit is active.
    pub general: bool,
    /// The player who played GeneralAudit (excluded from effect).
    pub general_source: Option<PlayerId>,
    /// Target player for TargetAudit.
    pub target: Option<PlayerId>,
}

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
    /// How tokens are distributed.
    pub token_distribution: TokenDistribution,
    /// Bot difficulty level.
    pub bot_difficulty: BotDifficulty,
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
            token_distribution: TokenDistribution::None,
            bot_difficulty: BotDifficulty::Basic,
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
    /// A player must choose a PrimeSabacc dice value.
    PrimeSabaccChoice {
        /// The player who played PrimeSabacc.
        player_id: PlayerId,
        /// First die value.
        die1: u8,
        /// Second die value.
        die2: u8,
    },
    /// Round ended, waiting to advance.
    RoundEnd,
    /// Game is over.
    GameOver {
        /// The winner's ID.
        winner: PlayerId,
    },
}

/// Per-turn ephemeral state that is reset between turns/rounds.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TurnEphemeral {
    /// Players who chose Stand this turn (for Audit resolution).
    pub stood_this_turn: Vec<PlayerId>,
    /// Player forced to Stand by Embargo (if any).
    pub embargoed_player: Option<PlayerId>,
    /// Whether a shift token has been played this turn by the current player.
    pub token_played_this_turn: bool,
    /// Whether FreeDraw is active for the current player's draw.
    pub free_draw_active: bool,
    /// Pending audit effects.
    pub pending_audit: PendingAudit,
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
    /// Per-turn ephemeral state.
    pub turn_state: TurnEphemeral,
    /// Order in which players were eliminated: (player_id, round_number).
    pub elimination_order: Vec<(PlayerId, u8)>,
    /// Aggregate game statistics.
    pub stats: GameStats,
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
    /// Play a shift token.
    PlayShiftToken {
        player_id: PlayerId,
        token: ShiftToken,
    },
    /// Submit a PrimeSabacc dice choice.
    SubmitPrimeSabaccChoice {
        player_id: PlayerId,
        chosen_value: u8,
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
    let stats = GameStats::new(
        &players.iter().map(|p| p.id).collect::<Vec<_>>(),
        config.starting_chips,
    );

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
        turn_state: TurnEphemeral::default(),
        elimination_order: Vec::new(),
        stats,
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
        Action::PlayShiftToken { player_id, token } => {
            if !state.config.enable_shift_tokens {
                Err(GameError::ShiftTokensDisabled)
            } else {
                apply_shift_token(state, player_id, token, rng)
            }
        }
        Action::SubmitPrimeSabaccChoice {
            player_id,
            chosen_value,
        } => apply_prime_sabacc_choice(state, player_id, chosen_value)
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
            let mut actions = Vec::new();

            // Shift token actions (if not already played this turn)
            if state.config.enable_shift_tokens
                && !state.turn_state.token_played_this_turn
                && !player.shift_tokens.is_empty()
            {
                for token in &player.shift_tokens {
                    actions.push(Action::PlayShiftToken {
                        player_id: pid,
                        token: token.clone(),
                    });
                }
            }

            // If embargoed, only Stand is available
            if state.turn_state.embargoed_player == Some(pid) {
                actions.push(Action::PlayerAction {
                    player_id: pid,
                    action: TurnAction::Stand,
                });
                return actions;
            }

            actions.push(Action::PlayerAction {
                player_id: pid,
                action: TurnAction::Stand,
            });

            // Can draw if player has chips or FreeDraw is active
            if player.chips > 0 || state.turn_state.free_draw_active {
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
        GamePhase::PrimeSabaccChoice {
            player_id,
            die1,
            die2,
        } => {
            let mut choices = vec![Action::SubmitPrimeSabaccChoice {
                player_id: *player_id,
                chosen_value: *die1,
            }];
            if die1 != die2 {
                choices.push(Action::SubmitPrimeSabaccChoice {
                    player_id: *player_id,
                    chosen_value: *die2,
                });
            }
            choices
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

                // Bot may play a token first
                if state.config.enable_shift_tokens && !state.turn_state.token_played_this_turn {
                    if let Some(token_action) = bot.choose_token(&state, rng) {
                        state = apply_action(state, token_action, rng)?;
                        // After token, loop back (might be PrimeSabaccChoice or still TurnAction)
                        continue;
                    }
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
            GamePhase::PrimeSabaccChoice { player_id, .. } => {
                let pid = *player_id;
                let player = state.players.iter().find(|p| p.id == pid);
                match player {
                    Some(p) if p.is_bot => {
                        let action = bot.choose_prime_sabacc(&state, rng);
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

    // Distribute shift tokens
    distribute_tokens(&mut state, rng);

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

/// Distribute shift tokens to players based on config.
fn distribute_tokens(state: &mut GameState, rng: &mut impl Rng) {
    match &state.config.token_distribution {
        TokenDistribution::None => {}
        TokenDistribution::Fixed(tokens) => {
            for player in &mut state.players {
                if !player.is_eliminated {
                    player.shift_tokens = tokens.clone();
                }
            }
        }
        TokenDistribution::Random { tokens_per_player } => {
            let all_types = ShiftToken::all_types();
            let n = *tokens_per_player;
            for player in &mut state.players {
                if !player.is_eliminated {
                    let mut available = all_types.clone();
                    shuffle_vec(&mut available, rng);
                    player.shift_tokens = available.into_iter().take(n).collect();
                }
            }
        }
    }
}

/// Shuffle a Vec using Fisher-Yates.
fn shuffle_vec<T>(items: &mut [T], rng: &mut impl Rng) {
    let len = items.len();
    for i in (1..len).rev() {
        let j = rng.gen_range(0..=i);
        items.swap(i, j);
    }
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

    // Embargo check: if this player is embargoed, they must Stand
    if let Some(embargoed_id) = state.turn_state.embargoed_player {
        if embargoed_id == player_id && !matches!(action, TurnAction::Stand) {
            return Err(GameError::InvalidActionForPhase {
                reason: "player is embargoed and must Stand".into(),
            });
        }
    }

    match action {
        TurnAction::Stand => {
            if let Some(ps) = state.stats.get_mut(player_id) {
                ps.stands_count += 1;
            }
            state.turn_state.stood_this_turn.push(player_id);
            // Clear embargo after the embargoed player stands
            if state.turn_state.embargoed_player == Some(player_id) {
                state.turn_state.embargoed_player = None;
            }
            advance_turn(&mut state, rng)?;
            Ok(state)
        }
        TurnAction::Draw(source) => {
            // Pay 1 chip (unless FreeDraw is active)
            if state.turn_state.free_draw_active {
                state.turn_state.free_draw_active = false;
            } else {
                state.players[state.current_player_idx].pay_chip()?;
            }

            // Draw the card
            let drawn = match source {
                DrawSource::SandDeck => state.sand_deck.draw(rng)?,
                DrawSource::BloodDeck => state.blood_deck.draw(rng)?,
                DrawSource::SandDiscard => state.sand_deck.draw_from_discard()?,
                DrawSource::BloodDiscard => state.blood_deck.draw_from_discard()?,
            };

            if let Some(ps) = state.stats.get_mut(player_id) {
                ps.draws_count += 1;
            }

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
            // Update per-player round stats
            for result in &results {
                if let Some(ps) = state.stats.get_mut(result.player_id) {
                    ps.rounds_played += 1;
                    if result.is_winner {
                        ps.rounds_won += 1;
                    }
                    if result.penalty > 0 {
                        ps.chips_lost_to_penalties += result.penalty as u16;
                    }
                    ps.update_best_hand(&result.rank);
                }
            }
            // Record chip history snapshots
            state.stats.record_round_chips(&state.players);

            // Track newly eliminated players
            let current_round = state.round;
            for player in &state.players {
                if player.is_eliminated
                    && !state
                        .elimination_order
                        .iter()
                        .any(|(pid, _)| *pid == player.id)
                {
                    state.elimination_order.push((player.id, current_round));
                }
            }
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

            state.turn_state = TurnEphemeral::default();

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
    // Reset per-player token state
    state.turn_state.token_played_this_turn = false;
    state.turn_state.free_draw_active = false;

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
        // Resolve audits at end of turn
        resolve_audits(state);

        // Reset per-turn state
        state.turn_state.stood_this_turn.clear();
        state.turn_state.embargoed_player = None;
        state.turn_state.pending_audit = PendingAudit::default();

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

/// Resolve pending audit effects at end of turn.
fn resolve_audits(state: &mut GameState) {
    let immune = &state.modifiers.immune_players;

    // GeneralAudit: all Stand players (except source, except immune) lose 2 chips
    if state.turn_state.pending_audit.general {
        let source = state.turn_state.pending_audit.general_source;
        for pid in &state.turn_state.stood_this_turn {
            if source == Some(*pid) || immune.contains(pid) {
                continue;
            }
            if let Some(player) = state.players.iter_mut().find(|p| p.id == *pid) {
                player.apply_penalty(2);
            }
            if let Some(ps) = state.stats.get_mut(*pid) {
                ps.chips_lost_to_tariffs += 2;
            }
        }
    }

    // TargetAudit: if target stood and is not immune, lose 3 chips
    if let Some(target_id) = state.turn_state.pending_audit.target {
        if state.turn_state.stood_this_turn.contains(&target_id) && !immune.contains(&target_id) {
            if let Some(player) = state.players.iter_mut().find(|p| p.id == target_id) {
                player.apply_penalty(3);
            }
            if let Some(ps) = state.stats.get_mut(target_id) {
                ps.chips_lost_to_tariffs += 3;
            }
        }
    }
}

/// Validate that a shift token can be played.
fn validate_shift_token(
    state: &GameState,
    player_id: PlayerId,
    token: &ShiftToken,
) -> Result<(), GameError> {
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
    if state.turn_state.token_played_this_turn {
        return Err(GameError::ShiftTokenAlreadyPlayed);
    }
    if !current.has_token(token) {
        return Err(GameError::ShiftTokenNotOwned { player_id });
    }
    if token.requires_target() {
        let target_id = match token {
            ShiftToken::TargetTariff(t)
            | ShiftToken::TargetAudit(t)
            | ShiftToken::Exhaustion(t)
            | ShiftToken::DirectTransaction(t) => *t,
            _ => unreachable!(),
        };
        if target_id == player_id {
            return Err(GameError::InvalidTokenTarget {
                player_id,
                reason: "cannot target yourself".into(),
            });
        }
        let target = state
            .players
            .iter()
            .find(|p| p.id == target_id)
            .ok_or(GameError::PlayerNotFound {
                player_id: target_id,
            })?;
        if target.is_eliminated {
            return Err(GameError::InvalidTokenTarget {
                player_id: target_id,
                reason: "target is eliminated".into(),
            });
        }
    }
    Ok(())
}

/// Apply economic token effects (FreeDraw, Refund, ExtraRefund, Embezzlement).
fn apply_economic_token(
    state: &mut GameState,
    player_id: PlayerId,
    token: &ShiftToken,
    immune: &[PlayerId],
) -> Result<(), GameError> {
    match token {
        ShiftToken::FreeDraw => {
            state.turn_state.free_draw_active = true;
        }
        ShiftToken::Refund => {
            if state.players[state.current_player_idx].pot == 0 {
                return Err(GameError::NoInvestedChips { player_id });
            }
            state.players[state.current_player_idx].refund_chips(2);
        }
        ShiftToken::ExtraRefund => {
            if state.players[state.current_player_idx].pot == 0 {
                return Err(GameError::NoInvestedChips { player_id });
            }
            state.players[state.current_player_idx].refund_chips(3);
        }
        ShiftToken::Embezzlement => {
            let mut stolen = 0u8;
            for i in 0..state.players.len() {
                let p = &state.players[i];
                if p.id != player_id && !p.is_eliminated && !immune.contains(&p.id) && p.chips > 0
                {
                    let affected_id = state.players[i].id;
                    state.players[i].chips -= 1;
                    stolen += 1;
                    if let Some(ps) = state.stats.get_mut(affected_id) {
                        ps.chips_lost_to_tariffs += 1;
                    }
                }
            }
            state.players[state.current_player_idx].chips += stolen;
        }
        _ => {}
    }
    Ok(())
}

/// Apply tariff token effects (GeneralTariff, TargetTariff).
fn apply_tariff_token(
    state: &mut GameState,
    player_id: PlayerId,
    token: &ShiftToken,
    immune: &[PlayerId],
) {
    match token {
        ShiftToken::GeneralTariff => {
            for i in 0..state.players.len() {
                let p = &state.players[i];
                if p.id != player_id && !p.is_eliminated && !immune.contains(&p.id) {
                    let affected_id = state.players[i].id;
                    state.players[i].apply_penalty(1);
                    if let Some(ps) = state.stats.get_mut(affected_id) {
                        ps.chips_lost_to_tariffs += 1;
                    }
                }
            }
        }
        ShiftToken::TargetTariff(target_id) => {
            let tid = *target_id;
            if !immune.contains(&tid) {
                if let Some(target) = state.players.iter_mut().find(|p| p.id == tid) {
                    target.apply_penalty(2);
                }
                if let Some(ps) = state.stats.get_mut(tid) {
                    ps.chips_lost_to_tariffs += 2;
                }
            }
        }
        _ => {}
    }
}

/// Apply audit token effects (GeneralAudit, TargetAudit).
fn apply_audit_token(state: &mut GameState, player_id: PlayerId, token: &ShiftToken) {
    match token {
        ShiftToken::GeneralAudit => {
            state.turn_state.pending_audit.general = true;
            state.turn_state.pending_audit.general_source = Some(player_id);
        }
        ShiftToken::TargetAudit(target_id) => {
            state.turn_state.pending_audit.target = Some(*target_id);
        }
        _ => {}
    }
}

/// Apply modifier token effects (Markdown, CookTheBooks, MajorFraud, Immunity).
fn apply_modifier_token(state: &mut GameState, player_id: PlayerId, token: &ShiftToken) {
    match token {
        ShiftToken::Markdown => state.modifiers.markdown_active = true,
        ShiftToken::CookTheBooks => state.modifiers.cook_the_books_active = true,
        ShiftToken::MajorFraud => state.modifiers.major_fraud_active = true,
        ShiftToken::Immunity => state.modifiers.immune_players.push(player_id),
        _ => {}
    }
}

/// Apply interaction token effects (Embargo, Exhaustion, DirectTransaction).
fn apply_interaction_token(
    state: &mut GameState,
    token: &ShiftToken,
    immune: &[PlayerId],
    rng: &mut impl Rng,
) -> Result<(), GameError> {
    match token {
        ShiftToken::Embargo => {
            let num = state.players.len();
            let mut next_idx = state.current_player_idx + 1;
            while next_idx < num {
                let p = &state.players[next_idx];
                if !p.is_eliminated {
                    if !immune.contains(&p.id) {
                        state.turn_state.embargoed_player = Some(p.id);
                    }
                    break;
                }
                next_idx += 1;
            }
        }
        ShiftToken::Exhaustion(target_id) => {
            let tid = *target_id;
            if !immune.contains(&tid) {
                if let Some(target) = state.players.iter_mut().find(|p| p.id == tid) {
                    if let Some(hand) = target.hand.take() {
                        state.sand_deck.discard(hand.sand);
                        state.blood_deck.discard(hand.blood);
                    }
                }
                let new_sand = state.sand_deck.draw(rng)?;
                let new_blood = state.blood_deck.draw(rng)?;
                let new_hand = crate::hand::Hand::new(new_sand, new_blood)?;
                if let Some(target) = state.players.iter_mut().find(|p| p.id == tid) {
                    target.hand = Some(new_hand);
                }
            }
        }
        ShiftToken::DirectTransaction(target_id) => {
            let tid = *target_id;
            if !immune.contains(&tid) {
                let player_idx = state.current_player_idx;
                let target_idx = state
                    .players
                    .iter()
                    .position(|p| p.id == tid)
                    .ok_or(GameError::PlayerNotFound { player_id: tid })?;
                let player_hand = state.players[player_idx].hand.take();
                let target_hand = state.players[target_idx].hand.take();
                state.players[player_idx].hand = target_hand;
                state.players[target_idx].hand = player_hand;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Apply a shift token action.
fn apply_shift_token(
    mut state: GameState,
    player_id: PlayerId,
    token: ShiftToken,
    rng: &mut impl Rng,
) -> Result<GameState, GameError> {
    validate_shift_token(&state, player_id, &token)?;

    let immune = state.modifiers.immune_players.clone();

    // PrimeSabacc has special handling (changes phase, returns early)
    if matches!(token, ShiftToken::PrimeSabacc) {
        let die1: u8 = rng.gen_range(1..=6);
        let die2: u8 = rng.gen_range(1..=6);
        state.players[state.current_player_idx].remove_token(&token)?;
        state.turn_state.token_played_this_turn = true;
        if let Some(ps) = state.stats.get_mut(player_id) {
            ps.tokens_played += 1;
        }
        state.phase = GamePhase::PrimeSabaccChoice {
            player_id,
            die1,
            die2,
        };
        return Ok(state);
    }

    // Apply token effect by category
    match &token {
        ShiftToken::FreeDraw | ShiftToken::Refund | ShiftToken::ExtraRefund | ShiftToken::Embezzlement => {
            apply_economic_token(&mut state, player_id, &token, &immune)?;
        }
        ShiftToken::GeneralTariff | ShiftToken::TargetTariff(_) => {
            apply_tariff_token(&mut state, player_id, &token, &immune);
        }
        ShiftToken::GeneralAudit | ShiftToken::TargetAudit(_) => {
            apply_audit_token(&mut state, player_id, &token);
        }
        ShiftToken::Markdown | ShiftToken::CookTheBooks | ShiftToken::MajorFraud | ShiftToken::Immunity => {
            apply_modifier_token(&mut state, player_id, &token);
        }
        ShiftToken::Embargo | ShiftToken::Exhaustion(_) | ShiftToken::DirectTransaction(_) => {
            apply_interaction_token(&mut state, &token, &immune, rng)?;
        }
        ShiftToken::PrimeSabacc => unreachable!(),
    }

    // Remove token and mark as played
    state.players[state.current_player_idx].remove_token(&token)?;
    state.turn_state.token_played_this_turn = true;
    if let Some(ps) = state.stats.get_mut(player_id) {
        ps.tokens_played += 1;
    }

    Ok(state)
}

/// Apply a PrimeSabacc dice choice.
fn apply_prime_sabacc_choice(
    mut state: GameState,
    player_id: PlayerId,
    chosen_value: u8,
) -> Result<GameState, GameError> {
    let (expected_pid, die1, die2) = match &state.phase {
        GamePhase::PrimeSabaccChoice {
            player_id,
            die1,
            die2,
        } => (*player_id, *die1, *die2),
        _ => {
            return Err(GameError::InvalidActionForPhase {
                reason: "not in PrimeSabaccChoice phase".into(),
            });
        }
    };

    if player_id != expected_pid {
        return Err(GameError::NotPlayerTurn { player_id });
    }

    if chosen_value != die1 && chosen_value != die2 {
        return Err(GameError::InvalidPrimeSabaccChoice {
            chosen: chosen_value,
            die1,
            die2,
        });
    }

    state.modifiers.prime_sabacc = Some(crate::scoring::PrimeSabaccModifier {
        player_id,
        chosen_value,
    });

    // Return to TurnAction phase
    state.phase = GamePhase::TurnAction;

    Ok(state)
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

/// A dummy RNG that panics if called.
///
/// Used in `apply_choose_discard` where `advance_turn` requires an `&mut impl Rng`
/// parameter but the code path never actually generates random numbers.
/// Panics on any call to ensure this invariant is upheld.
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
            token_distribution: TokenDistribution::None,
            bot_difficulty: BotDifficulty::Basic,
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
