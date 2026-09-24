// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! Contains capture history table for tracking successful captures.

use chess::{board::Board, moves::Move, pieces::Piece};

use crate::{
    history::{types::PieceToHistory, util},
    score::LargeScoreType,
    tuneable::{capture_history_offset, capture_history_scale},
    utils,
};

/// Computes the capture-history bonus/malus magnitude for a given depth.
pub(crate) fn calculate_bonus_for_depth(depth: i16) -> i16 {
    let mult = capture_history_scale() as i16;
    let offset = capture_history_offset() as i16;
    depth.saturating_mul(mult).saturating_sub(offset)
}

/// Capture history table containing [attacker piece][to][victim piece] per side.
pub struct CaptureHistory {
    piece_to_entries: Box<[PieceToHistory<[i16; Piece::COUNT]>; 2]>,
}

impl CaptureHistory {
    const MAX: i32 = 16384;
    const BONUS_MAX: i32 = Self::MAX / 4;

    pub(crate) fn get(&self, board: &Board, mv: Move, pc: Piece, captured_piece: Piece) -> i16 {
        self.piece_to_entries[board.side_to_move()][pc][mv.to()][captured_piece]
    }

    pub(crate) fn update(
        &mut self,
        board: &Board,
        mv: Move,
        pc: Piece,
        captured_piece: Piece,
        bonus: LargeScoreType,
    ) {
        let bonus = bonus.clamp(-Self::BONUS_MAX, Self::BONUS_MAX);
        let value = &mut self.piece_to_entries[board.side_to_move()][pc][mv.to()][captured_piece];
        *value = util::gravity(*value as i32, bonus, Self::MAX) as i16;
    }

    pub(crate) fn clear(&mut self) {
        self.piece_to_entries = unsafe { utils::boxed_and_zeroed() }
    }
}

impl Default for CaptureHistory {
    fn default() -> Self {
        Self {
            piece_to_entries: unsafe { utils::boxed_and_zeroed() },
        }
    }
}
