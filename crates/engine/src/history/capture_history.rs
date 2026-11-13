// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! Contains capture history table for tracking successful captures.

use chess::{board::Board, moves::Move, pieces::Piece};

use crate::{
    history::{types::PieceToHistory, util},
    score::LargeScoreType,
    tuneable::cap_hist_max,
    utils,
};

/// Capture history table containing [attacker piece][to][victim piece] per side.
pub struct CaptureHistory {
    piece_to_entries: Box<[PieceToHistory<[i16; Piece::COUNT]>; 2]>,
}

impl CaptureHistory {
    /// Entries are stored as `i16`, so the tuneable range of `cap_hist_max` must stay below
    /// `i16::MAX`.
    fn max() -> i32 {
        cap_hist_max()
    }

    fn bonus_max() -> i32 {
        Self::max() / 4
    }

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
        let bonus = bonus.clamp(-Self::bonus_max(), Self::bonus_max());
        let value = &mut self.piece_to_entries[board.side_to_move()][pc][mv.to()][captured_piece];
        *value = util::gravity(*value as i32, bonus, Self::max()) as i16;
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
