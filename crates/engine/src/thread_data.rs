// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module defines a thread data structure that holds information and data that is used in search.
//! Credit to the Hobbes author for this original setup which has been adapted for use in byte-knight.

use std::time::{Duration, Instant};

use chess::{board::Board, moves::Move, square::Square};
use uci_parser::UciSearchOptions;

use crate::{
    history::Histories,
    killers_table::KillerMovesTable,
    node::NodeStack,
    score::{Score, ScoreType},
    search::limits::{NODE_TM_MIN_DEPTH, SearchLimits, SoftLimitInputs},
    ttable::TranspositionTable,
    tuneable::score_stability_threshold,
    utils::boxed_and_zeroed,
};

/// Number of nodes searched between wall-clock polls for the hard time limit.
/// Node and depth limits are still enforced exactly on every check.
const NODES_BETWEEN_TIME_CHECKS: u64 = 2048;

pub struct ThreadData {
    pub(crate) transposition_table: TranspositionTable,
    pub(crate) histories: Histories,
    pub(crate) killers_table: KillerMovesTable,
    pub(crate) bestmove_stability: u64,
    pub(crate) eval_stability: u64,
    pub(crate) prev_best_move: Option<Move>,
    pub(crate) start_time: Instant,
    pub(crate) limits: SearchLimits,
    pub(crate) depth: i32,
    pub(crate) seldepth: ScoreType,
    pub(crate) nodes: u64,
    nodes_until_time_check: u64,
    stopped: bool,
    pub(crate) stack: NodeStack,
    pub(crate) avg_root_score: Option<Score>,
    pub(crate) root_move_nodes: Box<[[u64; Square::COUNT]; Square::COUNT]>,
}

pub enum LimitType {
    Soft,
    Hard,
}

impl Default for ThreadData {
    fn default() -> Self {
        ThreadData {
            transposition_table: TranspositionTable::default(),
            histories: Histories::default(),
            killers_table: KillerMovesTable::default(),
            bestmove_stability: 0,
            eval_stability: 0,
            prev_best_move: None,
            start_time: Instant::now(),
            limits: SearchLimits::default(),
            depth: 1,
            seldepth: 0,
            nodes: 0,
            nodes_until_time_check: 0,
            stopped: false,
            stack: NodeStack::default(),
            avg_root_score: None,
            root_move_nodes: unsafe { boxed_and_zeroed() },
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
        self.eval_stability = 0;
        self.prev_best_move = None;
        self.stack = NodeStack::default();
        self.avg_root_score = None;
        self.root_move_nodes = unsafe { boxed_and_zeroed() };
    }

    pub fn reset_start_time(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn time(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn clear(&mut self) {
        self.transposition_table.clear();
        self.histories.clear();
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

    /// Update score stability based on the new score from the current root search score.
    pub fn update_eval_stability(&mut self, score: Score) {
        let avg = match self.avg_root_score {
            None => score,
            Some(avg) => {
                let new_avg = ((avg.0 as i32 + score.0 as i32) / 2) as ScoreType;
                Score::new(new_avg)
            }
        };

        // Update the average score with the new score.
        self.avg_root_score = Some(avg);

        if (score.0 as i32 - avg.0 as i32).abs() < score_stability_threshold() {
            self.eval_stability += 1;
        } else {
            self.eval_stability = 0;
        }
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

        let inputs = SoftLimitInputs {
            best_move_stability: self.bestmove_stability_for_scaling(),
            eval_stability: self.eval_stability_for_scaling(),
            best_move_node_fraction: self.node_tm_for_scaling(),
        };
        if let Some(soft_time) = self.limits.scaled_soft_limit(inputs)
            && self.start_time.elapsed() >= soft_time
        {
            return true;
        }

        // Stop once the next iteration would exceed max_depth. The current
        // iteration at `depth == max_depth` must still run.
        self.depth > self.limits.max_depth as i32
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

    fn eval_stability_for_scaling(&self) -> Option<u64> {
        self.avg_root_score.map(|_| self.eval_stability)
    }

    fn node_tm_for_scaling(&self) -> Option<f32> {
        if let Some(best) = self.prev_best_move
            && (self.depth - 1) >= NODE_TM_MIN_DEPTH
            && self.nodes > 0
        {
            let best_nodes = self.root_move_nodes[best.from().index()][best.to().index()];
            (best_nodes > 0).then(|| best_nodes as f32 / self.nodes as f32)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use chess::moves::MoveFlag;

    use super::*;

    /// Feed `scores` through [`ThreadData::update_eval_stability`] and return
    /// the counter after each update.
    fn feed(td: &mut ThreadData, scores: &[ScoreType]) -> Vec<u64> {
        scores
            .iter()
            .map(|&s| {
                td.update_eval_stability(Score::new(s));
                td.eval_stability
            })
            .collect()
    }

    #[test]
    fn eval_stability_counts_up_on_stable_scores() {
        let mut td = ThreadData::default();
        assert_eq!(feed(&mut td, &[30, 32, 31, 30]), vec![1, 2, 3, 4]);
    }

    #[test]
    fn eval_stability_resets_on_jump() {
        let mut td = ThreadData::default();
        assert_eq!(feed(&mut td, &[30, 32, 120]), vec![1, 2, 0]);
    }

    #[test]
    fn eval_stability_compares_after_average_update() {
        // From avg 0: score 22 gives new avg 11, diff 11 < 12, so stable.
        let mut td = ThreadData::default();
        assert_eq!(feed(&mut td, &[0, 22]), vec![1, 2]);

        // Score 23 gives new avg 11, diff 12, so it resets. Comparing against
        // the old average would reset both cases.
        let mut td = ThreadData::default();
        assert_eq!(feed(&mut td, &[0, 23]), vec![1, 0]);
    }

    #[test]
    fn eval_stability_handles_near_mate_scores() {
        let mut td = ThreadData::default();
        assert_eq!(feed(&mut td, &[31_000, -31_000]), vec![1, 0]);
        assert_eq!(td.avg_root_score, Some(Score::new(0)));
    }

    #[test]
    fn eval_stability_for_scaling_gates_on_average() {
        let mut td = ThreadData::default();
        assert_eq!(td.eval_stability_for_scaling(), None);

        // A reset counter still scales (at the maximum), it doesn't switch off.
        feed(&mut td, &[30, 120]);
        assert_eq!(td.eval_stability_for_scaling(), Some(0));
    }

    /// A [`ThreadData`] that has just completed `completed_depth` with `best`
    /// as its best move, with `best_nodes` of `total_nodes` spent on it.
    fn td_after_iteration(
        best: Move,
        completed_depth: i32,
        best_nodes: u64,
        total_nodes: u64,
    ) -> ThreadData {
        let mut td = ThreadData {
            prev_best_move: Some(best),
            depth: completed_depth + 1,
            nodes: total_nodes,
            ..Default::default()
        };
        td.root_move_nodes[best.from().index()][best.to().index()] = best_nodes;
        td
    }

    #[test]
    fn node_tm_for_scaling_returns_fraction() {
        let best = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);
        let td = td_after_iteration(best, NODE_TM_MIN_DEPTH, 750, 1000);
        assert_eq!(td.node_tm_for_scaling(), Some(0.75));
    }

    #[test]
    fn node_tm_for_scaling_gates_on_depth() {
        let best = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);
        let td = td_after_iteration(best, NODE_TM_MIN_DEPTH - 1, 750, 1000);
        assert_eq!(td.node_tm_for_scaling(), None);
    }

    #[test]
    fn node_tm_for_scaling_gates_on_best_move() {
        let td = ThreadData {
            depth: NODE_TM_MIN_DEPTH + 1,
            nodes: 1000,
            ..Default::default()
        };
        assert_eq!(td.node_tm_for_scaling(), None);
    }

    #[test]
    fn node_tm_for_scaling_gates_on_zero_nodes() {
        let best = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);
        // Best move never searched at the root.
        let td = td_after_iteration(best, NODE_TM_MIN_DEPTH, 0, 1000);
        assert_eq!(td.node_tm_for_scaling(), None);
        // No nodes searched at all.
        let td = td_after_iteration(best, NODE_TM_MIN_DEPTH, 0, 0);
        assert_eq!(td.node_tm_for_scaling(), None);
    }

    #[test]
    fn reset_clears_root_move_nodes() {
        let best = Move::new(Square::E2, Square::E4, MoveFlag::DoublePush);
        let mut td = td_after_iteration(best, NODE_TM_MIN_DEPTH, 750, 1000);
        td.reset();
        assert!(td.root_move_nodes.iter().flatten().all(|&n| n == 0));
    }

    #[test]
    fn reset_clears_eval_stability() {
        let mut td = ThreadData::default();
        feed(&mut td, &[30, 32, 31]);
        td.reset();
        assert_eq!(td.eval_stability, 0);
        assert_eq!(td.avg_root_score, None);
    }
}
