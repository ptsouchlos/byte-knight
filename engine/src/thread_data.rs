// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module defines a thread data structure that holds information and data that is used in search.

use std::time::Instant;

use chess::board::Board;
use uci_parser::UciSearchOptions;

use crate::{score::ScoreType, search::limits::SearchLimits};

pub struct ThreadData {
    bestmove_stability: u64,
    limits: SearchLimits,
    start_time: Instant,
    depth: i32,
    seldepth: i32,
    nodes: u64,
}

pub enum LimitType {
    Soft,
    Hard,
}

impl ThreadData {
    pub fn new(uci_options: &UciSearchOptions, board: &Board) -> Self {
        ThreadData {
            bestmove_stability: 0,
            limits: SearchLimits::new(uci_options, board),
            start_time: Instant::now(),
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
    }

    pub fn should_stop(&self, depth: ScoreType, limit_type: LimitType) -> bool {
        if depth <= 1 {
            return false;
        }

        match limit_type {
            LimitType::Soft => self.soft_limit_reached(),
            LimitType::Hard => self.hard_limit_reached(),
        }
    }

    /// Check if the soft limit has been reached.
    /// This includes
    fn soft_limit_reached(&self) -> bool {
        let best_move_stability = self.bestmove_stability;
        if let Some(soft_time) = self.limits.scaled_soft_limit(best_move_stability) {
            if self.start_time.elapsed() >= soft_time {
                return true;
            }
        }

        // Always check for depth limits
        if self.depth >= self.limits.max_depth as i32 {
            return true;
        }

        false
    }

    /// Check if the hard limit has been reached.
    /// This includes time, nodes and depth.
    fn hard_limit_reached(&self) -> bool {
        if self.start_time.elapsed() >= self.limits.hard_timeout {
            return true;
        }

        if self.nodes >= self.limits.max_nodes {
            return true;
        }

        // Always check for depth limits
        if self.depth >= self.limits.max_depth as i32 {
            return true;
        }

        false
    }
}
