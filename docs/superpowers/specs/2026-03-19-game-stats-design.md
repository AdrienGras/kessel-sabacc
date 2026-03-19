# Game Stats — Design Spec

## Context

The game currently shows a basic GameOver screen with standings (rank, name, elimination status) and minimal stats (rounds played, pot, winner name). The user wants richer end-of-game statistics including per-player tracking during the game and a chip history line chart.

## Requirements

- Track per-player stats during the game (draws, stands, tokens played, chips lost, rounds won, best hand)
- Track chip history per round for each player (enables line chart)
- Display enriched standings at GameOver (final chips, elimination round)
- Display human player stats summary
- Display a Ratatui `Chart` widget with one line per player showing chip evolution over rounds
- No persistent storage — stats exist only for the duration of one game

## Architecture

### Core: `stats.rs`

New module `sabacc-core/src/stats.rs` with two structs:

```rust
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
    /// Chips (reserve) at each round boundary.
    /// Index 0 = game start (starting_chips), index 1 = after round 1, etc.
    pub chips_history: Vec<u8>,
}

pub struct GameStats {
    pub player_stats: Vec<PlayerStats>,
}
```

`GameStats` is added as `pub stats: GameStats` field on `GameState`.

### Initialization

`GameStats::new(players, starting_chips)` is called in `new_game()`. It creates one `PlayerStats` per player with zeroed counters and `chips_history` initialized to `vec![starting_chips]` (round 0 baseline). This ensures the chart always has a starting data point.

### `best_hand` comparison

Compare via `HandRank::strength_key()` — lower key means stronger hand. Update `best_hand` when the new rank has a strictly lower `strength_key` than the current one (or when `best_hand` is `None`).

### Stat collection points in `game.rs`

| Event | Stat updated |
|-------|-------------|
| `new_game()` | `GameStats::new()` with one `PlayerStats` per player, `chips_history = vec![starting_chips]` |
| `PlayerAction(Stand)` | `stands_count += 1` |
| `PlayerAction(Draw(_))` | `draws_count += 1` |
| `PlayShiftToken` | `tokens_played += 1` |
| `GeneralTariff/TargetTariff/Embezzlement` (victim) | `chips_lost_to_tariffs += amount` |
| `GeneralAudit/TargetAudit` resolved (victim) | `chips_lost_to_tariffs += amount` |
| `Reveal → RoundEnd` (apply_advance_round) | `rounds_played += 1`, `rounds_won` for winners, `chips_lost_to_penalties` for losers, `best_hand` updated via `strength_key()`, `chips_history.push(player.chips)` for all players |

### TUI: GameOver screen layout

```
╭─ GAME OVER ─ Winner: {name} ───────────────────────────────╮
│                                                              │
│  STANDINGS                           CHIP HISTORY            │
│  ★ 1st  Lando    8 chips            ┌──────────────────┐    │
│    2nd  You      0 (elim. R5)       │ Ratatui Chart    │    │
│    3rd  Han      0 (elim. R3)       │ Line per player  │    │
│                                      │ X: round number  │    │
│  YOUR STATS                          │ Y: chips         │    │
│  Draws: 12 | Stands: 3              │ Legend: names     │    │
│  Best hand: Sabacc{2}               └──────────────────┘    │
│  Tokens played: 2                                            │
│  Chips lost: 4 penalties, 2 tariffs                          │
│                                                              │
│                    [Enter] New game  [q] Quit                │
╰──────────────────────────────────────────────────────────────╯
```

Minimum popup width: 80 columns to accommodate the two-column layout with chart.

- Left column: standings + human player stats
- Right column: `Chart` widget with `Dataset` per player
- X axis: rounds (0..N), Y axis: chips (0..max_chips)
- Colors: Sand amber (`#E8C050`) for human, Blood red (`#E84848`) for bot 1, Cyan for bot 2, Green for bot 3
- Marker: `Marker::Braille` for smooth lines (requires UTF-8 terminal, which is already assumed by the TUI)
- Legend positioned top-left

### Overlay data structure changes

```rust
// Enhanced StandingEntry (app.rs)
pub struct StandingEntry {
    pub rank: u8,
    pub player_name: String,
    pub is_human: bool,
    pub final_chips: u8,
    pub elimination_round: Option<u8>,  // None if not eliminated
}

// Enhanced GameOverStats (app.rs)
pub struct GameOverStats {
    pub rounds_played: u8,
    pub credits_in_pot: u32,
    pub winner_name: String,
    pub human_stats: PlayerStats,          // cloned from core stats
    pub chip_histories: Vec<ChipHistory>,  // for chart
}

pub struct ChipHistory {
    pub player_name: String,
    pub is_human: bool,
    pub data: Vec<(f64, f64)>,  // (round, chips) for Chart Dataset
}
```

Conversion from `PlayerStats.chips_history: Vec<u8>` to `ChipHistory.data: Vec<(f64, f64)>` happens in `check_phase_transitions` when building `GameOverStats`.

### `Display` for `HandRank`

Add `impl fmt::Display for HandRank` in `hand.rs`. The format should match the existing `rank_str` logic in the TUI (e.g., "Pure Sabacc", "Sabacc (2)", "Non-Sabacc (diff 3)"). The existing `rank_str` field in `RoundResultEntry` should be migrated to use `HandRank::fmt()` for consistency.

## Files modified

| File | Change |
|------|--------|
| `sabacc-core/src/stats.rs` | New: `PlayerStats`, `GameStats` with `new()` and update methods |
| `sabacc-core/src/lib.rs` | Add `pub mod stats` |
| `sabacc-core/src/game.rs` | Add `stats: GameStats` to `GameState`, update in action handlers |
| `sabacc-core/src/hand.rs` | Add `Display` impl for `HandRank` |
| `sabacc-cli/src/app.rs` | Enrich `StandingEntry` and `GameOverStats`, populate from core stats |
| `sabacc-cli/src/widgets/results.rs` | Refactor `render_game_over`: 2-column layout, add Chart widget |

## Testing

Unit tests in `stats.rs`:
- Draws and stands counts increment correctly across multiple turns
- `chips_history` has length `rounds + 1` (including round 0 baseline) after N rounds
- `best_hand` updates only when a stronger hand is found (lower `strength_key`)
- Tariff vs penalty chip losses are tracked separately
- `tokens_played` increments on token play

Integration test:
- Run a full bot game, verify `GameStats` has non-zero totals and correct `chips_history` length

Verification:
- `cargo test -p sabacc-core && cargo clippy -p sabacc-core -- -D warnings`
- Visual: `cargo run -p sabacc-cli -- --quick` and play to GameOver
