// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{moves::Move, pieces::Piece};

use crate::{
    history::{types::PieceToHistory, util::gravity},
    score::LargeScoreType,
    utils::{self, boxed_and_zeroed},
};

pub struct ContinuationHistory {
    single_ply_entries: Box<PieceToHistory<PieceToHistory<i32>>>,
}

impl ContinuationHistory {
    const MAX: i32 = 16384;
    const BONUS_MAX: i32 = Self::MAX / 4;

    pub(crate) fn get(&self, prev_mv: Move, prev_pc: Piece, mv: Move, pc: Piece) -> i32 {
        self.single_ply_entries[prev_pc.index()][prev_mv.to().index()][pc.index()][mv.to().index()]
    }

    pub(crate) fn update(
        &mut self,
        prev_mv: Move,
        prev_pc: Piece,
        mv: Move,
        pc: Piece,
        bonus: LargeScoreType,
    ) {
        let bonus = bonus.clamp(-Self::BONUS_MAX, Self::BONUS_MAX);
        let entry = &mut self.single_ply_entries[prev_pc.index()][prev_mv.to().index()][pc.index()]
            [mv.to().index()];
        *entry = gravity(*entry, bonus, Self::MAX);
    }

    pub(crate) fn clear(&mut self) {
        self.single_ply_entries = unsafe { boxed_and_zeroed() };
    }
}

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self {
            single_ply_entries: unsafe { utils::boxed_and_zeroed() },
        }
    }
}

#[cfg(test)]
mod tests {
    use chess::{
        moves::{Move, MoveFlag},
        pieces::Piece,
        square::Square,
    };

    use crate::history::continuation_history::ContinuationHistory;

    #[test]
    fn clear_table() {
        // This test is mostly here to validate that we don't overflow the stack when clearing the table.
        let mut cont_hist = ContinuationHistory::default();
        let prev_mv = Move::new(Square::B2, Square::B4, MoveFlag::DoublePush);
        let mv = Move::new(Square::B4, Square::B5, MoveFlag::Standard);
        let bonus = 300;
        let pc = Piece::Pawn;
        // Update the score
        cont_hist.update(prev_mv, pc, mv, pc, bonus);
        // Ensure it's non-zero
        let score = cont_hist.get(prev_mv, pc, mv, pc);
        assert!(score > 0);

        // Clear the table
        cont_hist.clear();
        // Now the score should be 0
        let score = cont_hist.get(prev_mv, pc, mv, pc);
        assert!(score == 0);
    }
}
