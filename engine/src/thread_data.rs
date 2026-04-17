// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module defines a thread data structure that holds information and data that is used in search.
//! Credit to the Hobbes author for this original setup which has been adapted for use in byte-knight.

use chess::{board::Board, moves::Move};
use uci_parser::UciSearchOptions;

use crate::{score::ScoreType, search::limits::SearchLimits};

pub struct ThreadData {
    pub(crate) bestmove_stability: u64,
    pub(crate) prev_best_move: Option<Move>,
    pub(crate) limits: SearchLimits,
    pub(crate) depth: i32,
    pub(crate) seldepth: ScoreType,
    pub(crate) nodes: u64,
}

pub enum LimitType {
    Soft,
    Hard,
}

impl ThreadData {
    pub fn new(uci_options: &UciSearchOptions, board: &Board) -> Self {
        Self::from_limits(SearchLimits::new(uci_options, board))
    }

    /// Create [`ThreadData`] from pre-built [`SearchLimits`].
    pub fn from_limits(limits: SearchLimits) -> Self {
        ThreadData {
            bestmove_stability: 0,
            prev_best_move: None,
            limits,
            depth: 1,
            seldepth: 0,
            nodes: 0,
        }
    }

    pub fn reset(&mut self) {
        self.depth = 1;
        self.nodes = 0;
        self.seldepth = 0;
        self.bestmove_stability = 0;
        self.prev_best_move = None;
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

    pub fn should_stop(&self, limit_type: LimitType) -> bool {
        match limit_type {
            LimitType::Soft => self.soft_limit_reached(),
            LimitType::Hard => self.hard_limit_reached(),
        }
    }

    /// Check if the soft limit has been reached.
    fn soft_limit_reached(&self) -> bool {
        let best_move_stability = self.bestmove_stability_for_scaling();
        if let Some(soft_time) = self.limits.scaled_soft_limit(best_move_stability)
            && self.limits.start_time.elapsed() >= soft_time
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
    /// This includes time and nodes. The max-depth boundary is handled by the
    /// iterative-deepening loop via the soft limit; checking it here would
    /// cause in-tree stop checks to abort the final iteration mid-search.
    fn hard_limit_reached(&self) -> bool {
        if self.limits.start_time.elapsed() >= self.limits.hard_timeout {
            return true;
        }

        if self.nodes >= self.limits.max_nodes {
            return true;
        }

        if self.depth > self.limits.max_depth as i32 {
            return true;
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
