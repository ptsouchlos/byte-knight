use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use chess::board::Board;
use uci_parser::UciSearchOptions;

use crate::defs::MAX_DEPTH;

pub const UCI_OVERHEAD_MS: u64 = 50;
const BEST_MOVE_TIME_BASE: f32 = 1.5;
const BEST_MOVE_TIME_FACTOR: f32 = 0.1;

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

            // Validate we have valide time and increment
            if let (Some(time), Some(inc)) = (time, increment) {
                // TODO: How can we tune these params?
                let (hard, soft) = SearchLimits::calculate_time_limits(time, inc);
                params.soft_timeout = soft;
                params.hard_timeout = hard;
            }
        }

        params
    }

    pub(crate) fn scaled_soft_limit(&self, best_move_stability: u64) -> Option<Duration> {
        if self.soft_timeout.is_zero() {
            return None;
        }

        // No time control set (depth/nodes-only search): don't scale.
        if self.soft_timeout == Duration::MAX {
            return Some(Duration::MAX);
        }

        let scaled =
            self.soft_timeout.as_secs_f32() * Self::best_move_stability_scale(best_move_stability);

        Some(Duration::from_secs_f32(scaled))
    }

    fn best_move_stability_scale(best_move_stability: u64) -> f32 {
        BEST_MOVE_TIME_BASE - BEST_MOVE_TIME_FACTOR * best_move_stability as f32
    }

    fn calculate_time_limits(time: Duration, inc: Duration) -> (Duration, Duration) {
        let max_time =
            Duration::from_millis((time.as_millis() as u64).saturating_sub(UCI_OVERHEAD_MS));
        let hard = max_time / 20 - inc / 2;
        let soft = hard.as_secs_f64() * 0.6;

        (hard, Duration::from_secs_f64(soft))
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
