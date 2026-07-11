// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module defines a thread data structure that holds information and data that is used in search.
//! Credit to the Hobbes author for this original setup which has been adapted for use in byte-knight.

use std::time::{Duration, Instant};

use chess::{board::Board, moves::Move};
use uci_parser::UciSearchOptions;

use crate::{
    history_table::HistoryTable, killers_table::KillerMovesTable, node::NodeStack,
    score::ScoreType, search::limits::SearchLimits, ttable::TranspositionTable,
};

/// Number of nodes searched between wall-clock polls for the hard time limit.
/// Node and depth limits are still enforced exactly on every check.
const NODES_BETWEEN_TIME_CHECKS: u64 = 2048;

pub struct ThreadData {
    pub(crate) transposition_table: TranspositionTable,
    pub(crate) history_table: HistoryTable,
    pub(crate) killers_table: KillerMovesTable,
    pub(crate) bestmove_stability: u64,
    pub(crate) prev_best_move: Option<Move>,
    pub(crate) start_time: Instant,
    pub(crate) limits: SearchLimits,
    pub(crate) depth: i32,
    pub(crate) seldepth: ScoreType,
    pub(crate) nodes: u64,
    nodes_until_time_check: u64,
    stopped: bool,
    pub(crate) stack: NodeStack,
}

pub enum LimitType {
    Soft,
    Hard,
}

impl Default for ThreadData {
    fn default() -> Self {
        ThreadData {
            transposition_table: TranspositionTable::default(),
            history_table: HistoryTable::default(),
            killers_table: KillerMovesTable::default(),
            bestmove_stability: 0,
            prev_best_move: None,
            start_time: Instant::now(),
            limits: SearchLimits::default(),
            depth: 1,
            seldepth: 0,
            nodes: 0,
            nodes_until_time_check: 0,
            stopped: false,
            stack: NodeStack::default(),
        }
    }
}

impl ThreadData {
    pub fn new(uci_options: &UciSearchOptions, board: &Board) -> Self {
        Self::from_limits(SearchLimits::new(uci_options, board))
    }

    /// Create [`ThreadData`] from pre-built [`SearchLimits`].
    pub fn from_limits(limits: SearchLimits) -> Self {
        ThreadData {
            limits,
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        self.depth = 1;
        self.nodes = 0;
        self.nodes_until_time_check = 0;
        self.stopped = false;
        self.seldepth = 0;
        self.bestmove_stability = 0;
        self.prev_best_move = None;
        self.stack = NodeStack::default();
    }

    pub fn reset_start_time(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn time(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn clear(&mut self) {
        self.transposition_table.clear();
        self.history_table.clear();
        self.killers_table.clear();
    }

    /// Update best-move stability based on the new root best move. If the new
    /// best move matches the previous iteration's best move, increment the
    /// stability counter; otherwise reset it to zero.
    pub fn update_bestmove_stability(&mut self, new_best: Option<Move>) {
        match (self.prev_best_move, new_best) {
            (Some(prev), Some(new)) if prev == new => {
                self.bestmove_stability += 1;
            }
            _ => {
                self.bestmove_stability = 0;
            }
        }
        self.prev_best_move = new_best;
    }

    /// Returns true if a hard limit already stopped the search. Unlike
    /// [`Self::should_stop`], this never polls the clock.
    pub fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// Check if the current search should stop for the given [`LimitType`].
    pub fn should_stop(&mut self, limit_type: LimitType) -> bool {
        match limit_type {
            LimitType::Soft => self.soft_limit_reached(),
            LimitType::Hard => self.hard_limit_reached(),
        }
    }

    /// Check if the soft limit has been reached.
    fn soft_limit_reached(&self) -> bool {
        // A hard stop already triggered mid-iteration; don't start another one.
        if self.stopped {
            return true;
        }

        let best_move_stability = self.bestmove_stability_for_scaling();
        if let Some(soft_time) = self.limits.scaled_soft_limit(best_move_stability)
            && self.start_time.elapsed() >= soft_time
        {
            return true;
        }

        // Stop once the next iteration would exceed max_depth. The current
        // iteration at `depth == max_depth` must still run.
        if self.depth > self.limits.max_depth as i32 {
            return true;
        }

        false
    }

    /// Check if the hard limit has been reached.
    /// This includes time and nodes.
    fn hard_limit_reached(&mut self) -> bool {
        // Something previously stopped the search.
        // This flag is reset in [`ThreadData::reset`]
        if self.stopped {
            return true;
        }

        // Have we exceeded the max nodes
        if self.nodes >= self.limits.max_nodes {
            self.stopped = true;
            return true;
        }

        if self.depth > self.limits.max_depth as i32 {
            return true;
        }

        // Reading the wall clock on every node is expensive, so only poll it
        // once the node count crosses the next check threshold.
        if self.nodes >= self.nodes_until_time_check {
            // Update for the next time check
            self.nodes_until_time_check = self.nodes + NODES_BETWEEN_TIME_CHECKS;
            if self.start_time.elapsed() >= self.limits.hard_timeout {
                self.stopped = true;
                return true;
            }
        }

        false
    }

    fn bestmove_stability_for_scaling(&self) -> Option<u64> {
        // Only return a value if the previous move has a value.
        if self.prev_best_move.is_some() {
            Some(self.bestmove_stability)
        } else {
            None
        }
    }
}
