use crate::hand::HandRank;
use crate::player::Player;
use crate::PlayerId;

/// Per-player statistics tracked during the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerStats {
    /// The player this stats entry belongs to.
    pub player_id: PlayerId,
    /// Number of rounds won.
    pub rounds_won: u8,
    /// Number of rounds played.
    pub rounds_played: u8,
    /// Total draw actions taken.
    pub draws_count: u16,
    /// Total stand actions taken.
    pub stands_count: u16,
    /// Number of shift tokens played.
    pub tokens_played: u8,
    /// Chips lost to round-end penalties.
    pub chips_lost_to_penalties: u16,
    /// Chips lost to tariff shift tokens.
    pub chips_lost_to_tariffs: u16,
    /// Best hand achieved during the game (lowest strength_key).
    pub best_hand: Option<HandRank>,
    /// Chips at each round boundary. Index 0 = game start, index N = after round N.
    pub chips_history: Vec<u8>,
}

impl PlayerStats {
    /// Create a new `PlayerStats` with starting chip baseline.
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
    /// Per-player statistics.
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
        self.player_stats
            .iter_mut()
            .find(|s| s.player_id == player_id)
    }

    /// Get stats for a player.
    pub fn get(&self, player_id: PlayerId) -> Option<&PlayerStats> {
        self.player_stats
            .iter()
            .find(|s| s.player_id == player_id)
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
