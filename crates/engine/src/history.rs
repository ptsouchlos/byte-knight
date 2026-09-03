// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module contains all history tables and consolidates them into a Histories object.

use chess::{bitboard::Bitboard, board::Board, moves::Move, side::Side};

use crate::{
    history::{continuation_history::ContinuationHistory, quiet_history::QuietHistory},
    node::NodeStack,
    score::LargeScoreType,
};

pub mod continuation_history;
pub mod quiet_history;
pub mod threat_bucket;
mod types;
mod util;

/// Holds all history tables for the engine.
/// Credit to the author of [hobbes](https://github.com/kelseyde/hobbes-chess-engine) for this setup (kelseyde)
#[derive(Default)]
pub struct Histories {
    pub quiet_history: QuietHistory,
    pub continuation_history: ContinuationHistory,
}

impl Histories {
    pub(crate) fn get(
        &self,
        board: &Board,
        node_stack: &NodeStack,
        side: Side,
        mv: Move,
        threats: Bitboard,
        ply: usize,
    ) -> LargeScoreType {
        self.quiet_history.get(side, mv, threats)
            + self.continuation_history_score(board, node_stack, &mv, ply)
    }

    pub(crate) fn continuation_history_score(
        &self,
        board: &Board,
        node_stack: &NodeStack,
        mv: &Move,
        ply: usize,
    ) -> i32 {
        let piece = board.piece_type_on_square(mv.from()).unwrap();
        ContinuationHistory::PLIES
            .iter()
            .filter(|&&prev_ply| ply >= prev_ply)
            .filter_map(|&prev_ply| {
                let prev = node_stack[ply - prev_ply];
                let (prev_mv, prev_pc) = (prev.mv?, prev.piece?);
                Some(
                    self.continuation_history
                        .get(prev_mv, prev_pc, *mv, piece, prev_ply as i16),
                )
            })
            .sum()
    }

    pub fn clear(&mut self) {
        self.quiet_history.clear();
        self.continuation_history.clear();
    }
}
