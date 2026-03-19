# Game Stats Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Track per-player game statistics and display them with a chip history line chart on the GameOver screen.

**Architecture:** New `stats.rs` module in sabacc-core with `PlayerStats` and `GameStats` structs. Stats are updated in `apply_action()` at each event. The TUI GameOver overlay is enriched with standings, human stats, and a Ratatui `Chart` widget showing chip evolution per round.

**Tech Stack:** Rust, sabacc-core (pure logic), sabacc-cli (Ratatui 0.29 with Chart/Dataset/Axis widgets)

**Spec:** `docs/superpowers/specs/2026-03-19-game-stats-design.md`

---

## File Structure

| File | Responsibility |
|------|---------------|
| `crates/sabacc-core/src/stats.rs` | **New.** `PlayerStats`, `GameStats` structs + update methods |
| `crates/sabacc-core/src/lib.rs` | Add `pub mod stats` export |
| `crates/sabacc-core/src/hand.rs` | Add `Display` impl for `HandRank` |
| `crates/sabacc-core/src/game.rs` | Add `stats` field to `GameState`, update stats in action handlers |
| `crates/sabacc-core/tests/stats_tracking.rs` | **New.** Integration tests for stats tracking |
| `crates/sabacc-cli/src/app.rs` | Enrich `StandingEntry`/`GameOverStats`, populate from core stats |
| `crates/sabacc-cli/src/widgets/results.rs` | 2-column GameOver layout with Chart widget |

---

### Task 1: Create `stats.rs` with `PlayerStats` and `GameStats`

**Files:**
- Create: `crates/sabacc-core/src/stats.rs`
- Modify: `crates/sabacc-core/src/lib.rs` (add module export after `pub mod scoring;`)

- [ ] **Step 1: Create `stats.rs` with structs and methods**

```rust
use crate::hand::HandRank;
use crate::player::Player;
use crate::PlayerId;

/// Per-player statistics tracked during the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerStats {
    pub player_id: PlayerId,
    pub rounds_won: u8,
    pub rounds_played: u8,
    pub draws_count: u16,
    pub stands_count: u16,
    pub tokens_played: u8,
    pub chips_lost_to_penalties: u16,
    pub chips_lost_to_tariffs: u16,
    pub best_hand: Option<HandRank>,
    /// Chips at each round boundary. Index 0 = game start, index N = after round N.
    pub chips_history: Vec<u8>,
}

impl PlayerStats {
    /// Create a new PlayerStats with starting chip baseline.
    pub fn new(player_id: PlayerId, starting_chips: u8) -> Self {
        Self {
            player_id,
            rounds_won: 0,
            rounds_played: 0,
            draws_count: 0,
            stands_count: 0,
            tokens_played: 0,
            chips_lost_to_penalties: 0,
            chips_lost_to_tariffs: 0,
            best_hand: None,
            chips_history: vec![starting_chips],
        }
    }

    /// Update best_hand if the new rank is stronger (lower strength_key).
    pub fn update_best_hand(&mut self, rank: &HandRank) {
        let dominated = match &self.best_hand {
            None => true,
            Some(current) => rank.strength_key() < current.strength_key(),
        };
        if dominated {
            self.best_hand = Some(rank.clone());
        }
    }
}

/// Aggregate game statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameStats {
    pub player_stats: Vec<PlayerStats>,
}

impl GameStats {
    /// Create stats for all players with starting chip baselines.
    pub fn new(player_ids: &[PlayerId], starting_chips: u8) -> Self {
        Self {
            player_stats: player_ids
                .iter()
                .map(|&id| PlayerStats::new(id, starting_chips))
                .collect(),
        }
    }

    /// Get mutable stats for a player.
    pub fn get_mut(&mut self, player_id: PlayerId) -> Option<&mut PlayerStats> {
        self.player_stats.iter_mut().find(|s| s.player_id == player_id)
    }

    /// Get stats for a player.
    pub fn get(&self, player_id: PlayerId) -> Option<&PlayerStats> {
        self.player_stats.iter().find(|s| s.player_id == player_id)
    }

    /// Record chip snapshots for all players at end of round.
    pub fn record_round_chips(&mut self, players: &[Player]) {
        for player in players {
            if let Some(stats) = self.get_mut(player.id) {
                stats.chips_history.push(player.chips);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hand::HandRank;

    #[test]
    fn update_best_hand_from_none() {
        let mut ps = PlayerStats::new(0, 6);
        let rank = HandRank::Sabacc { pair_value: 3 };
        ps.update_best_hand(&rank);
        assert_eq!(ps.best_hand, Some(HandRank::Sabacc { pair_value: 3 }));
    }

    #[test]
    fn update_best_hand_stronger_replaces() {
        let mut ps = PlayerStats::new(0, 6);
        ps.best_hand = Some(HandRank::Sabacc { pair_value: 3 });
        ps.update_best_hand(&HandRank::PureSabacc);
        assert_eq!(ps.best_hand, Some(HandRank::PureSabacc));
    }

    #[test]
    fn update_best_hand_weaker_does_not_replace() {
        let mut ps = PlayerStats::new(0, 6);
        ps.best_hand = Some(HandRank::Sabacc { pair_value: 1 });
        ps.update_best_hand(&HandRank::NonSabacc { difference: 3 });
        assert_eq!(ps.best_hand, Some(HandRank::Sabacc { pair_value: 1 }));
    }

    #[test]
    fn chips_history_starts_with_baseline() {
        let ps = PlayerStats::new(0, 6);
        assert_eq!(ps.chips_history, vec![6]);
    }

    #[test]
    fn game_stats_new_creates_per_player() {
        let stats = GameStats::new(&[0, 1, 2], 6);
        assert_eq!(stats.player_stats.len(), 3);
        assert_eq!(stats.get(0).unwrap().player_id, 0);
        assert_eq!(stats.get(2).unwrap().chips_history, vec![6]);
    }
}
```

- [ ] **Step 2: Add module export to `lib.rs`**

Add `pub mod stats;` after `pub mod scoring;` in `crates/sabacc-core/src/lib.rs`.

- [ ] **Step 3: Verify it compiles and unit tests pass**

Run: `cargo test -p sabacc-core`
Expected: compiles cleanly, 5 new unit tests pass

- [ ] **Step 4: Commit**

```
git add crates/sabacc-core/src/stats.rs crates/sabacc-core/src/lib.rs
git commit -m "✨ feat: add PlayerStats and GameStats structs to sabacc-core"
```

---

### Task 2: Add `Display` for `HandRank`

**Files:**
- Modify: `crates/sabacc-core/src/hand.rs` (after `impl HandRank` block, ~line 71)

- [ ] **Step 1: Add Display impl after HandRank impl block**

```rust
impl std::fmt::Display for HandRank {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandRank::PureSabacc => write!(f, "Pure Sabacc"),
            HandRank::PrimeSabacc { value } => write!(f, "Prime Sabacc ({})", value),
            HandRank::SylopSabacc { value } => write!(f, "Sylop Sabacc ({})", value),
            HandRank::Sabacc { pair_value } => write!(f, "Sabacc ({})", pair_value),
            HandRank::NonSabacc { difference } => write!(f, "Non-Sabacc (diff {})", difference),
        }
    }
}
```

- [ ] **Step 2: Verify it compiles and tests pass**

Run: `cargo test -p sabacc-core`

- [ ] **Step 3: Commit**

```
git add crates/sabacc-core/src/hand.rs
git commit -m "✨ feat: add Display impl for HandRank"
```

---

### Task 3: Wire `GameStats` into `GameState` and action handlers

**Files:**
- Modify: `crates/sabacc-core/src/game.rs`
  - Line 111: `GameState` struct — add `stats` field
  - Lines 197-214: `new_game()` — initialize stats
  - Lines 515-523: Stand handler — `stands_count`
  - Lines 524-546: Draw handler — `draws_count`
  - Lines 787-1015: `apply_shift_token` — `tokens_played` + tariff tracking
  - Lines 760-784: `resolve_audits` — audit tariff tracking
  - Lines 650-714: `apply_advance_round` — round-end stats + chips_history

- [ ] **Step 1: Add `stats` field to `GameState`**

Add `use crate::stats::GameStats;` to imports. Add field after `elimination_order`:
```rust
    /// Per-player game statistics.
    pub stats: GameStats,
```

- [ ] **Step 2: Initialize stats in `new_game()`**

In the `Ok(GameState { ... })` block (~line 197), add:
```rust
        stats: GameStats::new(
            &players.iter().map(|p| p.id).collect::<Vec<_>>(),
            config.starting_chips,
        ),
```

- [ ] **Step 3: Track Stand/Draw in `apply_player_action`**

In the `TurnAction::Stand` arm (~line 515), before `advance_turn`:
```rust
            if let Some(ps) = state.stats.get_mut(player_id) {
                ps.stands_count += 1;
            }
```

In the `TurnAction::Draw` arm (~line 524), after the draw succeeds (before setting ChoosingDiscard phase):
```rust
            if let Some(ps) = state.stats.get_mut(player_id) {
                ps.draws_count += 1;
            }
```

- [ ] **Step 4: Track tokens_played in `apply_shift_token`**

After `state.token_played_this_turn = true;` (~line 1012):
```rust
    if let Some(ps) = state.stats.get_mut(player_id) {
        ps.tokens_played += 1;
    }
```

Also in the PrimeSabacc early-return path (~line 935), before `return Ok(state)`:
```rust
    if let Some(ps) = state.stats.get_mut(player_id) {
        ps.tokens_played += 1;
    }
```

- [ ] **Step 5: Track tariff/embezzlement losses in `apply_shift_token`**

For `GeneralTariff` (~line 873-879), use a two-pass pattern to avoid borrow conflicts:

```rust
ShiftToken::GeneralTariff => {
    // First pass: apply penalties
    for i in 0..state.players.len() {
        let p = &state.players[i];
        if p.id != player_id && !p.is_eliminated && !immune.contains(&p.id) {
            state.players[i].apply_penalty(1);
        }
    }
    // Second pass: track stats
    for i in 0..state.players.len() {
        let p = &state.players[i];
        if p.id != player_id && !p.is_eliminated && !immune.contains(&p.id) {
            // Player was affected (may now be eliminated but was affected)
        }
    }
    // Simpler: collect affected IDs before applying penalties
}
```

Actually, the simplest approach: track stats **alongside** the penalty. Since `state.players` and `state.stats` are disjoint fields, we can access them both:

```rust
ShiftToken::GeneralTariff => {
    for i in 0..state.players.len() {
        let p = &state.players[i];
        if p.id != player_id && !p.is_eliminated && !immune.contains(&p.id) {
            let pid = p.id;
            state.players[i].apply_penalty(1);
            if let Some(ps) = state.stats.get_mut(pid) {
                ps.chips_lost_to_tariffs += 1;
            }
        }
    }
}
```

Apply the same pattern for `TargetTariff` (track 2) and `Embezzlement` (track 1 per affected player).

- [ ] **Step 6: Track audit losses in `resolve_audits`**

In `resolve_audits`, after each `apply_penalty` call, track stats. Use scoping to drop the player borrow before accessing stats:

```rust
// GeneralAudit
if state.pending_audit.general {
    let source = state.pending_audit.general_source;
    for pid in &state.stood_this_turn {
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

// TargetAudit — same pattern with penalty 3
```

This works because after `player.apply_penalty(2)`, the mutable borrow of `state.players` is released at the end of the `if let` block, so `state.stats.get_mut()` can borrow `state.stats` without conflict.

- [ ] **Step 7: Track round-end stats in `apply_advance_round`**

In the `Reveal` arm of `apply_advance_round`, after `round::apply_results` (~line 663):

```rust
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
            // Record chip snapshots for all players
            state.stats.record_round_chips(&state.players);
```

Note: `record_round_chips` borrows `state.players` immutably and `state.stats` mutably — these are disjoint fields, so no conflict. However, Rust's borrow checker may not allow this through the `&mut self` method call. If it fails, inline the logic:
```rust
            for player in &state.players {
                if let Some(ps) = state.stats.player_stats.iter_mut().find(|s| s.player_id == player.id) {
                    ps.chips_history.push(player.chips);
                }
            }
```

- [ ] **Step 8: Fix borrow issues and verify compilation**

Run: `cargo check -p sabacc-core`

- [ ] **Step 9: Run all tests**

Run: `cargo test -p sabacc-core`
Expected: all 128 tests pass (123 existing + 5 new unit tests from Task 1)

- [ ] **Step 10: Commit**

```
git add crates/sabacc-core/src/game.rs
git commit -m "✨ feat: wire GameStats into GameState and track all events"
```

---

### Task 4: Integration tests for stats tracking

**Files:**
- Create: `crates/sabacc-core/tests/stats_tracking.rs`

- [ ] **Step 1: Write integration tests**

```rust
mod common;
use common::*;
use sabacc_core::game::{self, Action, GameConfig, GamePhase, TokenDistribution};
use sabacc_core::shift_token::ShiftToken;
use sabacc_core::turn::TurnAction;
use rand::rngs::SmallRng;
use rand::SeedableRng;

#[test]
fn stats_track_stands_and_draws() {
    let (mut state, mut rng) = started_game(2, 6, 42);

    // P0 draws
    let pid = state.players[state.current_player_idx].id;
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: pid,
            action: TurnAction::Draw(sabacc_core::turn::DrawSource::SandDeck),
        },
        &mut rng,
    ).unwrap();
    state = game::apply_action(
        state,
        Action::ChooseDiscard {
            player_id: pid,
            choice: sabacc_core::turn::DiscardChoice::DiscardDrawn,
        },
        &mut rng,
    ).unwrap();

    // P1 stands
    let pid1 = state.players[state.current_player_idx].id;
    state = game::apply_action(
        state,
        Action::PlayerAction {
            player_id: pid1,
            action: TurnAction::Stand,
        },
        &mut rng,
    ).unwrap();

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
    };

    let state = game::new_game(config, &mut rng).unwrap();
    let mut state = game::apply_action(state, Action::StartGame, &mut rng).unwrap();

    // Initial history: [6] (starting chips baseline)
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

    // chips_history length = 1 (initial baseline) + rounds_completed
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

    set_hands(&mut state, vec![
        (0, make_hand(sand(3), blood(3))), // Sabacc{3}
        (1, make_hand(sand(1), blood(5))), // NonSabacc{4}
    ]);

    state = fast_forward_to_revelation(state, &mut rng);
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
        2, 6,
        vec![vec![ShiftToken::FreeDraw, ShiftToken::Markdown], vec![]],
        42,
    );

    // P0 plays FreeDraw
    state = game::apply_action(
        state,
        Action::PlayShiftToken { player_id: 0, token: ShiftToken::FreeDraw },
        &mut rng,
    ).unwrap();

    assert_eq!(state.stats.get(0).unwrap().tokens_played, 1);

    // P0 stands, P1 stands (advance to turn 2)
    state = game::apply_action(
        state,
        Action::PlayerAction { player_id: 0, action: TurnAction::Stand },
        &mut rng,
    ).unwrap();
    state = game::apply_action(
        state,
        Action::PlayerAction { player_id: 1, action: TurnAction::Stand },
        &mut rng,
    ).unwrap();

    // Turn 2: P0 plays Markdown
    state = game::apply_action(
        state,
        Action::PlayShiftToken { player_id: 0, token: ShiftToken::Markdown },
        &mut rng,
    ).unwrap();

    assert_eq!(state.stats.get(0).unwrap().tokens_played, 2);
    assert_eq!(state.stats.get(1).unwrap().tokens_played, 0);
}

#[test]
fn stats_tariff_vs_penalty_separation() {
    let (mut state, mut rng) = game_with_tokens(
        2, 6,
        vec![vec![ShiftToken::GeneralTariff], vec![]],
        42,
    );

    // P0 plays GeneralTariff → P1 loses 1 chip to tariff
    state = game::apply_action(
        state,
        Action::PlayShiftToken { player_id: 0, token: ShiftToken::GeneralTariff },
        &mut rng,
    ).unwrap();

    let p1_stats = state.stats.get(1).unwrap();
    assert_eq!(p1_stats.chips_lost_to_tariffs, 1);
    assert_eq!(p1_stats.chips_lost_to_penalties, 0);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test -p sabacc-core`
Expected: all tests pass including 5 new integration tests

- [ ] **Step 3: Commit**

```
git add crates/sabacc-core/tests/stats_tracking.rs
git commit -m "✅ test: add integration tests for GameStats tracking"
```

---

### Task 5: Enrich TUI GameOver screen (data + rendering)

This task combines the overlay data changes and the rendering in one step to avoid compile breakage between commits.

**Files:**
- Modify: `crates/sabacc-cli/src/app.rs:103-119` (StandingEntry, GameOverStats structs)
- Modify: `crates/sabacc-cli/src/app.rs:1556-1595` (check_phase_transitions — populate data)
- Modify: `crates/sabacc-cli/src/widgets/results.rs:237-356` (render_game_over — 2-col layout + Chart)

**Required imports for `results.rs`:**
```rust
use ratatui::widgets::{Chart, Dataset, Axis, GraphType, LegendPosition};
use ratatui::symbols;
```

- [ ] **Step 1: Update `StandingEntry` struct** (~app.rs:103)

Replace existing:
```rust
pub struct StandingEntry {
    pub rank: u8,
    pub player_name: String,
    pub is_human: bool,
    pub final_chips: u8,
    pub elimination_round: Option<u8>,
}
```

- [ ] **Step 2: Add `ChipHistory` struct and update `GameOverStats`** (~app.rs:114)

```rust
pub struct ChipHistory {
    pub player_name: String,
    pub is_human: bool,
    pub data: Vec<(f64, f64)>,
}

pub struct GameOverStats {
    pub rounds_played: u8,
    pub credits_in_pot: u32,
    pub winner_name: String,
    pub human_draws: u16,
    pub human_stands: u16,
    pub human_tokens_played: u8,
    pub human_best_hand: Option<String>,
    pub human_chips_lost_penalties: u16,
    pub human_chips_lost_tariffs: u16,
    pub chip_histories: Vec<ChipHistory>,
}
```

- [ ] **Step 3: Update `check_phase_transitions` to populate new fields** (~app.rs:1556)

Build `StandingEntry` with `final_chips` (from `player.chips`) and `elimination_round` (from `game.elimination_order`).

Build `ChipHistory` for each player: convert `stats.get(pid).chips_history` to `Vec<(f64, f64)>` as `(round_index as f64, chips as f64)`.

Populate human stats fields from `stats.get(human_id)`. Use `HandRank::to_string()` (from Display impl) for `human_best_hand`.

- [ ] **Step 4: Refactor `render_game_over` to 2-column layout with Chart** (~results.rs:237)

Change popup width from 72 to 80 min. Split inner area:
```rust
let [left_col, right_col] = Layout::horizontal([
    Constraint::Percentage(45),
    Constraint::Percentage(55),
]).areas(inner);
```

**Left column** — Standings:
```
★ 1st  Lando    8 chips
  2nd  You      0 (elim. R5)
```
Then human stats:
```
YOUR STATS
Draws: 12 | Stands: 3
Best hand: Sabacc (2)
Tokens played: 2
Chips lost: 4 penalties, 2 tariffs
```
Then footer with `Rounds: N | Pot: X credits`.

**Right column** — Chart widget:
```rust
let datasets: Vec<Dataset> = stats.chip_histories.iter().enumerate().map(|(i, h)| {
    let color = match (h.is_human, i) {
        (true, _) => Color::Rgb(232, 192, 80),
        (_, 0) => Color::Rgb(232, 72, 72),
        (_, 1) => Color::Cyan,
        _ => Color::Green,
    };
    Dataset::default()
        .name(&h.player_name)
        .marker(symbols::Marker::Braille)
        .style(Style::default().fg(color))
        .graph_type(GraphType::Line)
        .data(&h.data)
}).collect();

let max_round = stats.rounds_played as f64;
let max_chips = stats.chip_histories.iter()
    .flat_map(|h| h.data.iter().map(|(_, y)| *y as u8))
    .max().unwrap_or(6) as f64;

let chart = Chart::new(datasets)
    .block(Block::bordered().title("Chip History"))
    .x_axis(Axis::default().title("Round").bounds([0.0, max_round])
        .labels(["0", &format!("{}", stats.rounds_played)]))
    .y_axis(Axis::default().title("Chips").bounds([0.0, max_chips])
        .labels(["0", &format!("{}", max_chips as u8)]))
    .legend_position(Some(LegendPosition::TopLeft));

frame.render_widget(chart, right_col);
```

Footer hint: `[Enter] New game  [q] Quit`.

- [ ] **Step 5: Migrate `short_rank()` in app.rs to use `HandRank::Display`**

Replace `short_rank()` function (~app.rs:1828) to use `rank.to_string()` instead of manual formatting.

- [ ] **Step 6: Verify compilation**

Run: `cargo check -p sabacc-cli`

- [ ] **Step 7: Run all tests**

Run: `cargo test --workspace`

- [ ] **Step 8: Visual test**

Run: `cargo run -p sabacc-cli -- --quick`
Play to GameOver, verify the chart and stats display correctly.

- [ ] **Step 9: Commit**

```
git add crates/sabacc-cli/src/app.rs crates/sabacc-cli/src/widgets/results.rs
git commit -m "✨ feat: enriched GameOver screen with stats and chip history chart"
```

---

### Task 6: Final clippy + cleanup

- [ ] **Step 1: Run clippy on both crates**

Run: `cargo clippy -p sabacc-core -- -D warnings && cargo clippy -p sabacc-cli -- -D warnings`

- [ ] **Step 2: Fix any warnings**

- [ ] **Step 3: Run full test suite**

Run: `cargo test --workspace`

- [ ] **Step 4: Final commit if any cleanup needed**
