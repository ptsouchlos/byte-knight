use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use chess::board::Board;
use uci_parser::UciSearchOptions;

use crate::defs::MAX_DEPTH;

pub const UCI_OVERHEAD_MS: u64 = 100;
const BEST_MOVE_TIME_BASE: f32 = 1.;
const BEST_MOVE_TIME_FACTOR: f32 = 0.05;
const BEST_MOVE_TIME_MIN_SCALE: f32 = 0.5;

/// Input parameters for the search.
#[derive(Clone, Debug, Copy)]
pub struct SearchLimits {
    pub max_depth: u8,
    pub start_time: Instant,
    pub soft_timeout: Duration,
    pub hard_timeout: Duration,
    pub max_nodes: u64,
}

impl Default for SearchLimits {
    fn default() -> Self {
        SearchLimits {
            max_depth: MAX_DEPTH,
            start_time: Instant::now(),
            soft_timeout: Duration::MAX,
            hard_timeout: Duration::MAX,
            max_nodes: u64::MAX,
        }
    }
}

impl SearchLimits {
    /// Creates a new set of search parameters from the UCI options and the current board.
    pub fn new(uci_options: &UciSearchOptions, board: &Board) -> Self {
        let mut params = Self::default();
        if let Some(depth) = uci_options.depth {
            params.max_depth = depth as u8;
        }

        if let Some(nodes) = uci_options.nodes {
            params.max_nodes = nodes as u64;
        }

        if let Some(time) = uci_options.movetime {
            params.soft_timeout = time;
            params.hard_timeout = time;
        } else {
            let (time, increment) = if board.side_to_move().is_white() {
                (uci_options.wtime, uci_options.winc)
            } else {
                (uci_options.btime, uci_options.binc)
            };

            if let Some(time) = time {
                // TODO: How can we tune these params?
                let inc = increment.unwrap_or(Duration::ZERO);
                let (hard, soft) = SearchLimits::calculate_time_limits(time, inc);
                params.soft_timeout = soft;
                params.hard_timeout = hard;
            }
        }

        params
    }

    pub(crate) fn scaled_soft_limit(&self, best_move_stability: u64) -> Option<Duration> {
        // No time control set (depth/nodes-only search): don't scale.
        if self.soft_timeout.is_zero() || self.soft_timeout == Duration::MAX {
            return None;
        }

        let scaled =
            self.soft_timeout.as_secs_f32() * Self::best_move_stability_scale(best_move_stability);

        Some(Duration::from_secs_f32(scaled))
    }

    fn best_move_stability_scale(best_move_stability: u64) -> f32 {
        (BEST_MOVE_TIME_BASE - BEST_MOVE_TIME_FACTOR * best_move_stability as f32)
            .max(BEST_MOVE_TIME_MIN_SCALE)
    }

    fn calculate_time_limits(time: Duration, inc: Duration) -> (Duration, Duration) {
        let overhead = Duration::from_millis(UCI_OVERHEAD_MS);
        let budget = time.saturating_sub(overhead);
        let soft = budget / 20 + inc / 2;
        let hard = (budget / 5 + inc / 2).min(budget).max(soft);

        (hard, soft)
    }
}

impl Display for SearchLimits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "max depth {} start_time {:?} soft_timeout {:?} hard_timeout {:?}",
            self.max_depth, self.start_time, self.soft_timeout, self.hard_timeout
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_limits_typical() {
        let (hard, soft) = SearchLimits::calculate_time_limits(
            Duration::from_millis(100),
            Duration::from_millis(100),
        );
        assert!(soft > Duration::ZERO);
        assert!(hard > Duration::ZERO);
        assert!(hard >= soft);
    }

    #[test]
    fn time_limits_no_increment() {
        let (hard, soft) =
            SearchLimits::calculate_time_limits(Duration::from_millis(10), Duration::ZERO);
        assert!(hard >= soft);
    }

    #[test]
    fn time_limits_zero_time() {
        // Must not panic; both limits may be zero but that is acceptable.
        let (hard, soft) = SearchLimits::calculate_time_limits(Duration::ZERO, Duration::ZERO);
        let _ = (hard, soft);
    }

    #[test]
    fn stability_scale_initial() {
        assert!((SearchLimits::best_move_stability_scale(0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn stability_scale_mid() {
        assert!((SearchLimits::best_move_stability_scale(5) - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn stability_scale_clamp() {
        // At stability=100 the raw value would be deeply negative; must clamp to floor.
        assert!(SearchLimits::best_move_stability_scale(100) >= BEST_MOVE_TIME_MIN_SCALE);
    }

    #[test]
    fn new_with_time_no_increment_gives_finite_limits() {
        use chess::board::Board;
        use uci_parser::UciSearchOptions;

        let board = Board::default_board();
        let opts = UciSearchOptions {
            wtime: Some(Duration::from_secs(10)),
            ..Default::default()
        };
        // winc intentionally left as None

        let limits = SearchLimits::new(&opts, &board);
        assert_ne!(
            limits.soft_timeout,
            Duration::MAX,
            "soft_timeout should be finite"
        );
        assert_ne!(
            limits.hard_timeout,
            Duration::MAX,
            "hard_timeout should be finite"
        );
    }
}
