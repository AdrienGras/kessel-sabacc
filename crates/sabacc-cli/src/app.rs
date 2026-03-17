/// Application state and pure update function (Elm architecture).
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

use sabacc_core::bot::BasicBot;
use sabacc_core::card::Card;
use sabacc_core::game::{self, Action, GameConfig, GamePhase, GameState, TokenDistribution};
use sabacc_core::player::Player;
use sabacc_core::shift_token::ShiftToken;
use sabacc_core::turn::{DiscardChoice, DrawSource, TurnAction};
use sabacc_core::PlayerId;

use crate::animation::{Animation, AnimationQueue};
use crate::events::AppEvent;

// ── Log ──────────────────────────────────────────────────────────────

/// A single log entry.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub text: String,
    pub is_error: bool,
}

// ── TUI State ────────────────────────────────────────────────────────

/// Which overlay is currently displayed on top of the main layout.
#[derive(Debug, Clone)]
pub enum Overlay {
    SourcePicker,
    TokenPicker,
    TargetPicker {
        token: ShiftToken,
    },
    DiscardChoice {
        drawn: Card,
        current: Card,
    },
    ImpostorChoice {
        die1: u8,
        die2: u8,
        for_sand: bool,
        sand_choice: Option<u8>,
    },
    PrimeSabaccChoice {
        die1: u8,
        die2: u8,
    },
    QuitConfirm,
    GameOver {
        winner_name: String,
        is_human: bool,
    },
}

/// Focus state for keyboard navigation.
#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    ActionBar,
    Setup,
}

/// UI-only state (not game logic).
#[derive(Debug, Clone)]
pub struct TuiState {
    pub focus: Focus,
    pub selected_action: usize,
    pub selected_token: usize,
    pub selected_target: usize,
    pub selected_source: usize,
    pub selected_discard: usize,
    pub selected_die: usize,
    pub overlay: Option<Overlay>,
    #[allow(dead_code)]
    pub should_quit: bool,
    pub show_help: bool,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub log_scroll_offset: usize,
    pub log_auto_scroll: bool,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            focus: Focus::Setup,
            selected_action: 0,
            selected_token: 0,
            selected_target: 0,
            selected_source: 0,
            selected_discard: 0,
            selected_die: 0,
            overlay: None,
            should_quit: false,
            show_help: false,
            terminal_width: 80,
            terminal_height: 24,
            log_scroll_offset: 0,
            log_auto_scroll: true,
        }
    }
}

/// Setup screen fields.
#[derive(Debug, Clone)]
pub struct SetupState {
    pub player_name: String,
    pub num_bots: u8,
    pub buy_in_index: usize,
    pub tokens_enabled: bool,
    pub tokens_per_player: u8,
    pub selected_field: usize,
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            player_name: "Player".into(),
            num_bots: 3,
            buy_in_index: 1, // 100 credits
            tokens_enabled: true,
            tokens_per_player: 4,
            selected_field: 0,
        }
    }
}

impl SetupState {
    pub const BUY_IN_OPTIONS: [u32; 4] = [50, 100, 150, 200];
    pub const CHIPS_OPTIONS: [u8; 4] = [4, 5, 6, 8];
    pub const NUM_FIELDS: usize = 6; // name, bots, buy_in, tokens, tokens_count, start

    pub fn buy_in(&self) -> u32 {
        Self::BUY_IN_OPTIONS[self.buy_in_index]
    }

    pub fn starting_chips(&self) -> u8 {
        Self::CHIPS_OPTIONS[self.buy_in_index]
    }
}

/// Which screen the app is on.
#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Setup,
    Playing,
}

// ── AppState ─────────────────────────────────────────────────────────

/// Top-level application state.
#[derive(Debug, Clone)]
pub struct AppState {
    pub screen: Screen,
    pub setup: SetupState,
    pub game: Option<GameState>,
    pub tui: TuiState,
    pub animations: AnimationQueue,
    pub log: Vec<LogEntry>,
    pub rng: SmallRng,
    pub revealed_players: Vec<PlayerId>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            screen: Screen::Setup,
            setup: SetupState::default(),
            game: None,
            tui: TuiState::default(),
            animations: AnimationQueue::new(),
            log: Vec::new(),
            rng: SmallRng::from_entropy(),
            revealed_players: Vec::new(),
        }
    }

    /// Shortcut: create a quick-start state (skip setup).
    #[allow(dead_code)]
    pub fn quick_start(name: String, num_bots: u8, buy_in: u32, tokens: bool) -> Self {
        let mut state = Self::new();
        state.setup.player_name = name;
        state.setup.num_bots = num_bots;
        // Find buy-in index
        state.setup.buy_in_index = SetupState::BUY_IN_OPTIONS
            .iter()
            .position(|&b| b == buy_in)
            .unwrap_or(1);
        state.setup.tokens_enabled = tokens;
        start_game(state)
    }

    pub fn is_animating(&self) -> bool {
        self.animations.is_animating()
    }

    /// Returns the human player (always id 0).
    #[allow(dead_code)]
    pub fn human_player(&self) -> Option<&Player> {
        self.game.as_ref().and_then(|g| g.players.first())
    }

    /// Returns true if it's the human player's turn.
    pub fn is_human_turn(&self) -> bool {
        self.game.as_ref().is_some_and(|g| {
            g.players
                .get(g.current_player_idx)
                .is_some_and(|player| !player.is_bot && !player.is_eliminated)
        })
    }

    #[allow(dead_code)]
    pub fn current_player(&self) -> Option<&Player> {
        self.game
            .as_ref()
            .and_then(|g| g.players.get(g.current_player_idx))
    }

    pub fn push_log(&mut self, text: String) {
        self.log.push(LogEntry {
            text,
            is_error: false,
        });
        if !self.tui.log_auto_scroll {
            self.tui.log_scroll_offset += 1;
        }
    }

    pub fn push_error(&mut self, text: String) {
        self.log.push(LogEntry {
            text,
            is_error: true,
        });
        if !self.tui.log_auto_scroll {
            self.tui.log_scroll_offset += 1;
        }
    }
}

// ── Command ──────────────────────────────────────────────────────────

/// Side-effect commands returned by `update()`.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    None,
    RunBots,
    Quit,
}

// ── update() ─────────────────────────────────────────────────────────

/// Pure update function: state × event → (state, command).
pub fn update(mut state: AppState, event: AppEvent) -> (AppState, Command) {
    match event {
        AppEvent::Resize(w, h) => {
            state.tui.terminal_width = w;
            state.tui.terminal_height = h;
            (state, Command::None)
        }
        AppEvent::Tick => {
            let messages = state.animations.tick(33);
            for msg in messages {
                state.push_log(msg);
            }

            // When animations finish, check if bots need to act
            if !state.is_animating() {
                if let Some(ref g) = state.game {
                    let needs_bots = match &g.phase {
                        GamePhase::TurnAction => !state.is_human_turn(),
                        GamePhase::ImpostorReveal { pending, .. } => {
                            !pending.contains(&0u8) // human id = 0
                        }
                        GamePhase::PrimeSabaccChoice { player_id, .. } => *player_id != 0,
                        _ => false,
                    };
                    if needs_bots {
                        return (state, Command::RunBots);
                    }
                }
            }

            (state, Command::None)
        }
        AppEvent::Key(key) => update_key(state, key),
    }
}

fn update_key(mut state: AppState, key: KeyEvent) -> (AppState, Command) {
    // Global: Ctrl+C always quits immediately
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return (state, Command::Quit);
    }

    // Space skips animations
    if state.is_animating() && key.code == KeyCode::Char(' ') {
        let messages = state.animations.skip_all();
        for msg in messages {
            state.push_log(msg);
        }
        // After skip, check if bots need to run
        if let Some(ref g) = state.game {
            if matches!(g.phase, GamePhase::TurnAction) && !state.is_human_turn() {
                return (state, Command::RunBots);
            }
        }
        return (state, Command::None);
    }

    // Ignore input during animations (except space/ctrl+c above)
    if state.is_animating() {
        return (state, Command::None);
    }

    match state.screen {
        Screen::Setup => update_setup(state, key),
        Screen::Playing => update_playing(state, key),
    }
}

// ── Setup screen ─────────────────────────────────────────────────────

fn update_setup(mut state: AppState, key: KeyEvent) -> (AppState, Command) {
    let field = state.setup.selected_field;

    match key.code {
        KeyCode::Tab | KeyCode::Down => {
            state.setup.selected_field = (field + 1) % SetupState::NUM_FIELDS;
        }
        KeyCode::BackTab | KeyCode::Up => {
            state.setup.selected_field =
                (field + SetupState::NUM_FIELDS - 1) % SetupState::NUM_FIELDS;
        }
        KeyCode::Enter => {
            if field == SetupState::NUM_FIELDS - 1 {
                // Start button
                state = start_game(state);
                return (state, Command::RunBots);
            }
        }
        KeyCode::Left => match field {
            1 => state.setup.num_bots = state.setup.num_bots.saturating_sub(1).max(1),
            2 => {
                state.setup.buy_in_index = state.setup.buy_in_index.saturating_sub(1);
            }
            3 => state.setup.tokens_enabled = !state.setup.tokens_enabled,
            4 if state.setup.tokens_enabled => {
                state.setup.tokens_per_player =
                    state.setup.tokens_per_player.saturating_sub(1).max(1);
            }
            _ => {}
        },
        KeyCode::Right => match field {
            1 => state.setup.num_bots = (state.setup.num_bots + 1).min(7),
            2 => {
                state.setup.buy_in_index =
                    (state.setup.buy_in_index + 1).min(SetupState::BUY_IN_OPTIONS.len() - 1);
            }
            3 => state.setup.tokens_enabled = !state.setup.tokens_enabled,
            4 if state.setup.tokens_enabled => {
                state.setup.tokens_per_player = (state.setup.tokens_per_player + 1).min(8);
            }
            _ => {}
        },
        KeyCode::Char(c) if field == 0 => {
            state.setup.player_name.push(c);
        }
        KeyCode::Backspace if field == 0 => {
            state.setup.player_name.pop();
        }
        KeyCode::Char('q') if field != 0 => {
            return (state, Command::Quit);
        }
        KeyCode::Esc => {
            return (state, Command::Quit);
        }
        _ => {}
    }

    (state, Command::None)
}

fn start_game(mut state: AppState) -> AppState {
    let mut players = vec![(state.setup.player_name.clone(), false)];
    let bot_names = [
        "Lando", "Han", "Chewie", "Qi'ra", "Beckett", "L3-37", "Dryden",
    ];
    for i in 0..state.setup.num_bots as usize {
        players.push((bot_names[i % bot_names.len()].into(), true));
    }

    let config = GameConfig {
        players,
        starting_chips: state.setup.starting_chips(),
        buy_in: state.setup.buy_in(),
        enable_shift_tokens: state.setup.tokens_enabled,
        token_distribution: if state.setup.tokens_enabled {
            TokenDistribution::Random {
                tokens_per_player: state.setup.tokens_per_player as usize,
            }
        } else {
            TokenDistribution::None
        },
    };

    match game::new_game(config, &mut state.rng) {
        Ok(game_state) => {
            // Apply StartGame to deal cards
            match game::apply_action(game_state, Action::StartGame, &mut state.rng) {
                Ok(started) => {
                    state.game = Some(started);
                    state.screen = Screen::Playing;
                    state.tui.focus = Focus::ActionBar;
                    state.push_log("Partie lancée !".into());
                }
                Err(e) => state.push_error(format!("Erreur au démarrage: {e}")),
            }
        }
        Err(e) => state.push_error(format!("Erreur de configuration: {e}")),
    }

    state
}

// ── Playing screen ───────────────────────────────────────────────────

fn update_playing(mut state: AppState, key: KeyEvent) -> (AppState, Command) {
    // Help toggle
    if key.code == KeyCode::Char('?') {
        state.tui.show_help = !state.tui.show_help;
        return (state, Command::None);
    }

    // If help is shown, any key closes it
    if state.tui.show_help {
        state.tui.show_help = false;
        return (state, Command::None);
    }

    // Log scroll: PageUp/PageDown
    if key.code == KeyCode::PageUp {
        let page = 10; // approximate visible height
        let max_offset = state.log.len().saturating_sub(page);
        state.tui.log_scroll_offset = (state.tui.log_scroll_offset + page).min(max_offset);
        state.tui.log_auto_scroll = false;
        return (state, Command::None);
    }
    if key.code == KeyCode::PageDown {
        let page = 10;
        state.tui.log_scroll_offset = state.tui.log_scroll_offset.saturating_sub(page);
        if state.tui.log_scroll_offset == 0 {
            state.tui.log_auto_scroll = true;
        }
        return (state, Command::None);
    }

    // Handle overlays first
    if state.tui.overlay.is_some() {
        return update_overlay(state, key);
    }

    // Quit confirmation
    if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
        state.tui.overlay = Some(Overlay::QuitConfirm);
        return (state, Command::None);
    }

    let game = match state.game {
        Some(ref g) => g,
        None => return (state, Command::None),
    };

    match &game.phase {
        GamePhase::TurnAction if state.is_human_turn() => update_turn_action(state, key),
        GamePhase::Reveal { .. } | GamePhase::RoundEnd => {
            if key.code == KeyCode::Enter {
                state = apply_game_action(state, Action::AdvanceRound);
                // After advancing, check if bots should play
                if !state.is_human_turn() {
                    return (state, Command::RunBots);
                }
            }
            (state, Command::None)
        }
        GamePhase::GameOver { .. } => {
            if key.code == KeyCode::Enter {
                // New game
                let mut new_state = AppState::new();
                new_state.setup = state.setup.clone();
                return (new_state, Command::None);
            }
            if key.code == KeyCode::Char('q') {
                return (state, Command::Quit);
            }
            (state, Command::None)
        }
        GamePhase::ImpostorReveal { pending, .. } => {
            let human_id = 0u8;
            if pending.contains(&human_id) {
                // Human needs to submit impostor choice — open overlay
                if let Some(hand) = state
                    .game
                    .as_ref()
                    .and_then(|g| g.players.first())
                    .and_then(|p| p.hand.as_ref())
                {
                    let has_sand_impostor =
                        matches!(hand.sand.value, sabacc_core::card::CardValue::Impostor);
                    let die1 = (state.rng.next_u64() % 6 + 1) as u8;
                    let die2 = (state.rng.next_u64() % 6 + 1) as u8;
                    state.tui.overlay = Some(Overlay::ImpostorChoice {
                        die1,
                        die2,
                        for_sand: has_sand_impostor,
                        sand_choice: None,
                    });
                }
                (state, Command::None)
            } else {
                // Only bots pending — let them handle it
                (state, Command::RunBots)
            }
        }
        GamePhase::PrimeSabaccChoice {
            player_id,
            die1,
            die2,
        } => {
            let pid = *player_id;
            let d1 = *die1;
            let d2 = *die2;
            if pid == 0 {
                state.tui.overlay = Some(Overlay::PrimeSabaccChoice { die1: d1, die2: d2 });
                (state, Command::None)
            } else {
                (state, Command::RunBots)
            }
        }
        _ => (state, Command::None),
    }
}

fn update_turn_action(mut state: AppState, key: KeyEvent) -> (AppState, Command) {
    let has_tokens = state.game.as_ref().is_some_and(|g| {
        g.config.enable_shift_tokens
            && !g.token_played_this_turn
            && g.players
                .first()
                .is_some_and(|p| !p.shift_tokens.is_empty())
    });
    let max_actions = if has_tokens { 3 } else { 2 };

    match key.code {
        KeyCode::Tab | KeyCode::Right => {
            state.tui.selected_action = (state.tui.selected_action + 1) % max_actions;
        }
        KeyCode::BackTab | KeyCode::Left => {
            state.tui.selected_action = (state.tui.selected_action + max_actions - 1) % max_actions;
        }
        KeyCode::Char('s') if has_tokens => {
            state.tui.overlay = Some(Overlay::TokenPicker);
            state.tui.selected_token = 0;
        }
        KeyCode::Char('1') | KeyCode::Char('2') | KeyCode::Char('3') | KeyCode::Char('4') => {
            // Direct source selection for draw
            let source_idx = (key.code.to_string().parse::<usize>().unwrap_or(1)) - 1;
            state.tui.selected_source = source_idx;
            state.tui.overlay = Some(Overlay::SourcePicker);
            state = confirm_source_pick(state);
            if !state.is_human_turn() {
                return (state, Command::RunBots);
            }
        }
        KeyCode::Enter => match state.tui.selected_action {
            0 => {
                // Draw → open SourcePicker
                state.tui.overlay = Some(Overlay::SourcePicker);
                state.tui.selected_source = 0;
            }
            1 => {
                // Stand
                state = apply_game_action(
                    state,
                    Action::PlayerAction {
                        player_id: 0,
                        action: TurnAction::Stand,
                    },
                );
                state.push_log("Vous: Stand".into());
                if !state.is_human_turn() {
                    return (state, Command::RunBots);
                }
            }
            2 if has_tokens => {
                // Token
                state.tui.overlay = Some(Overlay::TokenPicker);
                state.tui.selected_token = 0;
            }
            _ => {}
        },
        _ => {}
    }

    (state, Command::None)
}

// ── Overlays ─────────────────────────────────────────────────────────

fn update_overlay(mut state: AppState, key: KeyEvent) -> (AppState, Command) {
    if key.code == KeyCode::Esc {
        state.tui.overlay = None;
        return (state, Command::None);
    }

    let overlay = state.tui.overlay.clone();
    match overlay {
        Some(Overlay::QuitConfirm) => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => return (state, Command::Quit),
            _ => {
                state.tui.overlay = None;
            }
        },
        Some(Overlay::SourcePicker) => match key.code {
            KeyCode::Tab | KeyCode::Right | KeyCode::Down => {
                state.tui.selected_source = (state.tui.selected_source + 1) % 4;
            }
            KeyCode::BackTab | KeyCode::Left | KeyCode::Up => {
                state.tui.selected_source = (state.tui.selected_source + 3) % 4;
            }
            KeyCode::Char('1') => state.tui.selected_source = 0,
            KeyCode::Char('2') => state.tui.selected_source = 1,
            KeyCode::Char('3') => state.tui.selected_source = 2,
            KeyCode::Char('4') => state.tui.selected_source = 3,
            KeyCode::Enter => {
                state = confirm_source_pick(state);
                if !state.is_human_turn() {
                    return (state, Command::RunBots);
                }
            }
            _ => {}
        },
        Some(Overlay::DiscardChoice { .. }) => match key.code {
            KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                state.tui.selected_discard = 1 - state.tui.selected_discard;
            }
            KeyCode::Enter => {
                let choice = if state.tui.selected_discard == 0 {
                    DiscardChoice::KeepDrawn
                } else {
                    DiscardChoice::DiscardDrawn
                };
                state.tui.overlay = None;
                state = apply_game_action(
                    state,
                    Action::ChooseDiscard {
                        player_id: 0,
                        choice,
                    },
                );
                if state.tui.selected_discard == 0 {
                    state.push_log("Vous: garde piochée".into());
                } else {
                    state.push_log("Vous: défausse".into());
                }
                if !state.is_human_turn() {
                    return (state, Command::RunBots);
                }
            }
            _ => {}
        },
        Some(Overlay::TokenPicker) => {
            let token_count = state
                .game
                .as_ref()
                .and_then(|g| g.players.first())
                .map_or(0, |p| p.shift_tokens.len());

            if token_count == 0 {
                state.tui.overlay = None;
                return (state, Command::None);
            }

            match key.code {
                KeyCode::Tab | KeyCode::Down => {
                    state.tui.selected_token = (state.tui.selected_token + 1) % token_count;
                }
                KeyCode::BackTab | KeyCode::Up => {
                    state.tui.selected_token =
                        (state.tui.selected_token + token_count - 1) % token_count;
                }
                KeyCode::Enter => {
                    let token = state
                        .game
                        .as_ref()
                        .and_then(|g| g.players.first())
                        .and_then(|p| p.shift_tokens.get(state.tui.selected_token))
                        .cloned();

                    if let Some(token) = token {
                        if token.requires_target() {
                            state.tui.overlay = Some(Overlay::TargetPicker {
                                token: token.clone(),
                            });
                            state.tui.selected_target = 0;
                        } else {
                            state.tui.overlay = None;
                            state.push_log(format!("Vous: {:?}", token));
                            state = apply_game_action(
                                state,
                                Action::PlayShiftToken {
                                    player_id: 0,
                                    token,
                                },
                            );
                            if !state.is_human_turn() {
                                return (state, Command::RunBots);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        Some(Overlay::TargetPicker { ref token }) => {
            let targets: Vec<PlayerId> = state.game.as_ref().map_or(Vec::new(), |g| {
                g.players
                    .iter()
                    .filter(|p| p.id != 0 && !p.is_eliminated)
                    .map(|p| p.id)
                    .collect()
            });

            if targets.is_empty() {
                state.tui.overlay = None;
                return (state, Command::None);
            }

            let token = token.clone();
            match key.code {
                KeyCode::Tab | KeyCode::Down => {
                    state.tui.selected_target = (state.tui.selected_target + 1) % targets.len();
                }
                KeyCode::BackTab | KeyCode::Up => {
                    state.tui.selected_target =
                        (state.tui.selected_target + targets.len() - 1) % targets.len();
                }
                KeyCode::Enter => {
                    let target_id = targets[state.tui.selected_target];
                    let targeted_token = apply_target_to_token(&token, target_id);
                    let target_name = state
                        .game
                        .as_ref()
                        .and_then(|g| g.players.iter().find(|p| p.id == target_id))
                        .map_or("?".into(), |p| p.name.clone());

                    state.tui.overlay = None;
                    state.push_log(format!("Vous: {:?}→{}", token, target_name));
                    state = apply_game_action(
                        state,
                        Action::PlayShiftToken {
                            player_id: 0,
                            token: targeted_token,
                        },
                    );
                    if !state.is_human_turn() {
                        return (state, Command::RunBots);
                    }
                }
                _ => {}
            }
        }
        Some(Overlay::ImpostorChoice {
            die1,
            die2,
            for_sand,
            sand_choice,
        }) => match key.code {
            KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                state.tui.selected_die = 1 - state.tui.selected_die;
            }
            KeyCode::Enter => {
                let chosen = if state.tui.selected_die == 0 {
                    die1
                } else {
                    die2
                };

                let choice = sabacc_core::scoring::ImpostorChoice {
                    player_id: 0,
                    die1,
                    die2,
                    sand_choice: if for_sand { Some(chosen) } else { sand_choice },
                    blood_choice: if for_sand { None } else { Some(chosen) },
                };

                state.tui.overlay = None;
                state.tui.selected_die = 0;
                state.push_log(format!("Imposteur→{chosen}"));
                state = apply_game_action(state, Action::SubmitImpostorChoice(choice));
                if !state.is_human_turn() {
                    return (state, Command::RunBots);
                }
            }
            _ => {}
        },
        Some(Overlay::PrimeSabaccChoice { die1, die2 }) => match key.code {
            KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down => {
                state.tui.selected_die = 1 - state.tui.selected_die;
            }
            KeyCode::Enter => {
                let chosen = if state.tui.selected_die == 0 {
                    die1
                } else {
                    die2
                };
                state.tui.overlay = None;
                state.tui.selected_die = 0;
                state.push_log(format!("PrimeSabacc→{chosen}"));
                state = apply_game_action(
                    state,
                    Action::SubmitPrimeSabaccChoice {
                        player_id: 0,
                        chosen_value: chosen,
                    },
                );
                if !state.is_human_turn() {
                    return (state, Command::RunBots);
                }
            }
            _ => {}
        },
        Some(Overlay::GameOver { .. }) => match key.code {
            KeyCode::Enter => {
                let mut new_state = AppState::new();
                new_state.setup = state.setup.clone();
                return (new_state, Command::None);
            }
            KeyCode::Char('q') => return (state, Command::Quit),
            _ => {}
        },
        None => {}
    }

    (state, Command::None)
}

// ── Helpers ──────────────────────────────────────────────────────────

fn confirm_source_pick(mut state: AppState) -> AppState {
    let sources = [
        DrawSource::SandDeck,
        DrawSource::BloodDeck,
        DrawSource::SandDiscard,
        DrawSource::BloodDiscard,
    ];
    let source = sources[state.tui.selected_source];
    let source_name = match source {
        DrawSource::SandDeck => "Deck S",
        DrawSource::BloodDeck => "Deck B",
        DrawSource::SandDiscard => "Déf S",
        DrawSource::BloodDiscard => "Déf B",
    };

    state.tui.overlay = None;
    state.push_log(format!("Vous: Draw {source_name}"));
    state = apply_game_action(
        state,
        Action::PlayerAction {
            player_id: 0,
            action: TurnAction::Draw(source),
        },
    );

    // If we're now in ChoosingDiscard, open that overlay
    if let Some(ref g) = state.game {
        if let GamePhase::ChoosingDiscard {
            drawn_card,
            player_id,
        } = &g.phase
        {
            // Find the current card of the same family in hand
            let current_card = g
                .players
                .iter()
                .find(|p| p.id == *player_id)
                .and_then(|p| p.hand.as_ref())
                .map(|h| match drawn_card.family {
                    sabacc_core::card::Family::Sand => h.sand.clone(),
                    sabacc_core::card::Family::Blood => h.blood.clone(),
                })
                .unwrap_or_else(|| drawn_card.clone());

            state.tui.overlay = Some(Overlay::DiscardChoice {
                drawn: drawn_card.clone(),
                current: current_card,
            });
            state.tui.selected_discard = 0;
        }
    }

    state
}

fn apply_game_action(mut state: AppState, action: Action) -> AppState {
    if let Some(game_state) = state.game.take() {
        match game::apply_action(game_state, action, &mut state.rng) {
            Ok(new_game) => {
                // Check for phase transitions
                check_phase_transitions(&mut state, &new_game);
                state.game = Some(new_game);
            }
            Err(e) => {
                state.push_error(format!("Erreur: {e}"));
                // Restore game state on error — re-create from error
                // The game state was consumed, but we can't recover it.
                // This is a design limitation — we should clone before.
                // For now, log the error. The game is lost on error.
                state.push_error("État du jeu perdu suite à une erreur.".into());
            }
        }
    }
    state
}

fn check_phase_transitions(state: &mut AppState, game: &GameState) {
    match &game.phase {
        GamePhase::Reveal { results } => {
            // Queue reveal animations
            for result in results {
                let player = game.players.iter().find(|p| p.id == result.player_id);
                let name = player.map_or("?".into(), |p| p.name.clone());
                let rank_str = short_rank(&result.rank);
                let is_winner = result.is_winner;

                state.animations.push(Animation::PlayerHighlight {
                    player_id: result.player_id,
                    color: ratatui::style::Color::Yellow,
                    duration_ms: 300,
                });
                state.animations.push(Animation::CardReveal {
                    player_id: result.player_id,
                    delay_ms: 500,
                });

                let status = if is_winner {
                    "GAGNE".to_string()
                } else if result.penalty > 0 {
                    format!("-{}", result.penalty)
                } else {
                    "OK".to_string()
                };
                state.animations.push(Animation::LogMessage {
                    text: format!("{name}: {rank_str} — {status}"),
                });
                state.animations.push(Animation::Pause { duration_ms: 300 });
            }

            // Find winner
            if let Some(winner) = results.iter().find(|r| r.is_winner) {
                let name = game
                    .players
                    .iter()
                    .find(|p| p.id == winner.player_id)
                    .map_or("?".into(), |p| p.name.clone());
                state.animations.push(Animation::LogMessage {
                    text: format!("Gagnant: {name}"),
                });
            }

            state.revealed_players = game.players.iter().map(|p| p.id).collect();
        }
        GamePhase::GameOver { winner } => {
            let player = game.players.iter().find(|p| p.id == *winner);
            let name = player.map_or("?".into(), |p| p.name.clone());
            let is_human = player.is_some_and(|p| !p.is_bot);
            state.tui.overlay = Some(Overlay::GameOver {
                winner_name: name,
                is_human,
            });
        }
        _ => {}
    }
}

fn apply_target_to_token(token: &ShiftToken, target: PlayerId) -> ShiftToken {
    match token {
        ShiftToken::TargetTariff(_) => ShiftToken::TargetTariff(target),
        ShiftToken::TargetAudit(_) => ShiftToken::TargetAudit(target),
        ShiftToken::Exhaustion(_) => ShiftToken::Exhaustion(target),
        ShiftToken::DirectTransaction(_) => ShiftToken::DirectTransaction(target),
        other => other.clone(),
    }
}

/// Runs bot turns one at a time, logging each action with animations.
/// Loops until it's the human's turn, a non-bot phase, or an error.
pub fn run_bots(mut state: AppState) -> AppState {
    use sabacc_core::bot::BotStrategy;
    let bot = BasicBot;

    loop {
        let game_state = match state.game.take() {
            Some(g) => g,
            None => return state,
        };

        match &game_state.phase {
            GamePhase::TurnAction => {
                let player = &game_state.players[game_state.current_player_idx];
                if !player.is_bot || player.is_eliminated {
                    state.game = Some(game_state);
                    return state;
                }

                let bot_name = player.name.clone();

                // Bot may play a token first
                if game_state.config.enable_shift_tokens && !game_state.token_played_this_turn {
                    if let Some(token_action) = bot.choose_token(&game_state, &mut state.rng) {
                        let token_desc = describe_action(&token_action, &game_state);
                        match game::apply_action(game_state, token_action, &mut state.rng) {
                            Ok(new_state) => {
                                state.animations.push(Animation::LogMessage {
                                    text: format!("{bot_name}: {token_desc}"),
                                });
                                state.animations.push(Animation::Pause { duration_ms: 150 });
                                state.game = Some(new_state);
                                continue; // Loop back — bot still needs Draw/Stand
                            }
                            Err(e) => {
                                state.push_error(format!("Bot token error: {e}"));
                                return state;
                            }
                        }
                    }
                }

                let action = bot.choose_action(&game_state, &mut state.rng);
                let action_desc = describe_action(&action, &game_state);

                match game::apply_action(game_state, action, &mut state.rng) {
                    Ok(new_state) => {
                        state.game = Some(new_state);
                        // Handle ChoosingDiscard immediately
                        state = advance_bot_discard(&bot, state, &bot_name);
                        state.animations.push(Animation::LogMessage {
                            text: format!("{bot_name}: {action_desc}"),
                        });
                        state.animations.push(Animation::Pause { duration_ms: 200 });
                        continue; // Next bot or human
                    }
                    Err(e) => {
                        state.push_error(format!("Bot error ({bot_name}): {e}"));
                        return state;
                    }
                }
            }
            GamePhase::ChoosingDiscard { player_id, .. } => {
                let pid = *player_id;
                let player = game_state.players.iter().find(|p| p.id == pid);
                if player.is_some_and(|p| p.is_bot) {
                    let action = bot.choose_discard(&game_state, &mut state.rng);
                    match game::apply_action(game_state, action, &mut state.rng) {
                        Ok(new_state) => {
                            state.game = Some(new_state);
                            continue;
                        }
                        Err(e) => {
                            state.push_error(format!("Bot discard error: {e}"));
                            return state;
                        }
                    }
                } else {
                    state.game = Some(game_state);
                    return state;
                }
            }
            GamePhase::ImpostorReveal { pending, .. } => {
                if pending.is_empty() {
                    state.game = Some(game_state);
                    return state;
                }
                let pid = pending[0];
                if let Some(player) = game_state.players.iter().find(|p| p.id == pid) {
                    if player.is_bot {
                        let bot_name = player.name.clone();
                        let action = bot.choose_impostor(&game_state, &mut state.rng);
                        match game::apply_action(game_state, action, &mut state.rng) {
                            Ok(new_state) => {
                                state.animations.push(Animation::LogMessage {
                                    text: format!("{bot_name}: Imposteur"),
                                });
                                check_phase_transitions(&mut state, &new_state);
                                state.game = Some(new_state);
                                continue;
                            }
                            Err(e) => {
                                state.push_error(format!("Bot impostor error: {e}"));
                                return state;
                            }
                        }
                    } else {
                        state.game = Some(game_state);
                        return state;
                    }
                } else {
                    state.game = Some(game_state);
                    return state;
                }
            }
            GamePhase::PrimeSabaccChoice { player_id, .. } => {
                let pid = *player_id;
                if let Some(player) = game_state.players.iter().find(|p| p.id == pid) {
                    if player.is_bot {
                        let bot_name = player.name.clone();
                        let action = bot.choose_prime_sabacc(&game_state, &mut state.rng);
                        match game::apply_action(game_state, action, &mut state.rng) {
                            Ok(new_state) => {
                                state.animations.push(Animation::LogMessage {
                                    text: format!("{bot_name}: PrimeSabacc"),
                                });
                                check_phase_transitions(&mut state, &new_state);
                                state.game = Some(new_state);
                                continue;
                            }
                            Err(e) => {
                                state.push_error(format!("Bot prime error: {e}"));
                                return state;
                            }
                        }
                    } else {
                        state.game = Some(game_state);
                        return state;
                    }
                } else {
                    state.game = Some(game_state);
                    return state;
                }
            }
            _ => {
                // Phase not handled by bots — check for transitions and return
                check_phase_transitions(&mut state, &game_state);
                state.game = Some(game_state);
                return state;
            }
        }
    }
}

/// Handles the ChoosingDiscard phase for a bot after a Draw.
fn advance_bot_discard(bot: &BasicBot, mut state: AppState, bot_name: &str) -> AppState {
    use sabacc_core::bot::BotStrategy;

    if let Some(game_state) = state.game.take() {
        if let GamePhase::ChoosingDiscard { player_id, .. } = &game_state.phase {
            let pid = *player_id;
            let player = game_state.players.iter().find(|p| p.id == pid);
            if player.is_some_and(|p| p.is_bot) {
                let action = bot.choose_discard(&game_state, &mut state.rng);
                match game::apply_action(game_state, action, &mut state.rng) {
                    Ok(new_state) => {
                        state.game = Some(new_state);
                        return state;
                    }
                    Err(e) => {
                        state.push_error(format!("Bot discard error ({bot_name}): {e}"));
                        return state;
                    }
                }
            }
            state.game = Some(game_state);
        } else {
            state.game = Some(game_state);
        }
    }
    state
}

/// Describes a game action for log display.
fn describe_action(action: &Action, _game: &GameState) -> String {
    match action {
        Action::PlayerAction {
            action: turn_action,
            ..
        } => match turn_action {
            TurnAction::Draw(source) => {
                let source_name = match source {
                    DrawSource::SandDeck => "Deck S",
                    DrawSource::BloodDeck => "Deck B",
                    DrawSource::SandDiscard => "Déf S",
                    DrawSource::BloodDiscard => "Déf B",
                };
                format!("Draw {source_name}")
            }
            TurnAction::Stand => "Stand".into(),
        },
        Action::PlayShiftToken { token, .. } => format!("{:?}", token),
        _ => "action".into(),
    }
}

/// Short display for HandRank (fits ~23 chars in log).
fn short_rank(rank: &sabacc_core::hand::HandRank) -> String {
    use sabacc_core::hand::HandRank;
    match rank {
        HandRank::PureSabacc => "PureSabacc".into(),
        HandRank::PrimeSabacc { value } => format!("PrimeSabacc({value})"),
        HandRank::SylopSabacc { value } => format!("SylopSabacc({value})"),
        HandRank::Sabacc { pair_value } => format!("Sabacc({pair_value})"),
        HandRank::NonSabacc { difference } => format!("Non({difference})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn test_quit_on_ctrl_c() {
        let state = AppState::new();
        let event = AppEvent::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        });
        let (_, cmd) = update(state, event);
        assert_eq!(cmd, Command::Quit);
    }

    #[test]
    fn test_setup_navigation() {
        let mut state = AppState::new();
        assert_eq!(state.setup.selected_field, 0);

        let (state, _) = update(state, AppEvent::Key(key(KeyCode::Tab)));
        assert_eq!(state.setup.selected_field, 1);

        let (state, _) = update(state, AppEvent::Key(key(KeyCode::Tab)));
        assert_eq!(state.setup.selected_field, 2);
    }

    #[test]
    fn test_setup_bots_adjustment() {
        let mut state = AppState::new();
        state.setup.selected_field = 1; // bots field
        state.setup.num_bots = 3;

        let (state, _) = update(state, AppEvent::Key(key(KeyCode::Right)));
        assert_eq!(state.setup.num_bots, 4);

        let (state, _) = update(state, AppEvent::Key(key(KeyCode::Left)));
        assert_eq!(state.setup.num_bots, 3);
    }

    #[test]
    fn test_resize_event() {
        let state = AppState::new();
        let (state, _) = update(state, AppEvent::Resize(120, 40));
        assert_eq!(state.tui.terminal_width, 120);
        assert_eq!(state.tui.terminal_height, 40);
    }
}
