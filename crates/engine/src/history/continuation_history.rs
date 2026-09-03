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
    entries: Box<[PieceToHistory<PieceToHistory<i32>>; Self::PLY_COUNT]>,
}

impl ContinuationHistory {
    pub(crate) const PLY_COUNT: usize = 2;
    pub(crate) const PLIES: [usize; Self::PLY_COUNT] = [1, 2];
    const MAX: i32 = 16384;
    const BONUS_MAX: i32 = Self::MAX / 4;

    fn index_from_ply(prev_ply: i16) -> usize {
        (prev_ply & 1 == 0) as usize
    }

    pub(crate) fn get(
        &self,
        prev_mv: Move,
        prev_pc: Piece,
        mv: Move,
        pc: Piece,
        prev_ply: i16,
    ) -> i32 {
        let index = Self::index_from_ply(prev_ply);
        self.entries[index][prev_pc.index()][prev_mv.to().index()][pc.index()][mv.to().index()]
    }

    pub(crate) fn update(
        &mut self,
        prev_mv: Move,
        prev_pc: Piece,
        mv: Move,
        pc: Piece,
        bonus: LargeScoreType,
        prev_ply: i16,
    ) {
        let index = Self::index_from_ply(prev_ply);
        let bonus = bonus.clamp(-Self::BONUS_MAX, Self::BONUS_MAX);
        let entry = &mut self.entries[index][prev_pc.index()][prev_mv.to().index()][pc.index()]
            [mv.to().index()];
        *entry = gravity(*entry, bonus, Self::MAX);
    }

    pub(crate) fn clear(&mut self) {
        self.entries = unsafe { boxed_and_zeroed() };
    }
}

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self {
            entries: unsafe { utils::boxed_and_zeroed() },
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
        let prev_ply = 4i16;

        // Update the score
        cont_hist.update(prev_mv, pc, mv, pc, bonus, prev_ply);
        // Ensure it's non-zero
        let score = cont_hist.get(prev_mv, pc, mv, pc, prev_ply);
        assert!(score > 0);

        // Clear the table
        cont_hist.clear();
        // Now the score should be 0
        let score = cont_hist.get(prev_mv, pc, mv, pc, prev_ply);
        assert!(score == 0);
    }

    #[test]
    fn score_never_exceeds_max() {
        let mut cont_hist = ContinuationHistory::default();
        let prev_mv = Move::new(Square::B2, Square::B4, MoveFlag::DoublePush);
        let mv = Move::new(Square::B4, Square::B5, MoveFlag::Standard);
        let pc = Piece::Pawn;
        let prev_ply = 2i16;

        // Hammer the same cell with maximal bonuses to try to force it past MAX -
        // a saturated entry must never be able to sort above KILLER_BONUS in the move picker
        // once combined with quiet history (see move_picker::tests::combined_history_score_never_exceeds_killer_bonus).
        for _ in 0..10_000 {
            cont_hist.update(prev_mv, pc, mv, pc, i32::MAX, prev_ply);
        }

        let score = cont_hist.get(prev_mv, pc, mv, pc, prev_ply);
        assert!(
            score <= ContinuationHistory::MAX,
            "saturated continuation history entry ({score}) must not exceed MAX ({})",
            ContinuationHistory::MAX
        );

        // Same check in the negative direction.
        for _ in 0..10_000 {
            cont_hist.update(prev_mv, pc, mv, pc, i32::MIN, prev_ply);
        }

        let score = cont_hist.get(prev_mv, pc, mv, pc, prev_ply);
        assert!(
            score >= -ContinuationHistory::MAX,
            "saturated continuation history entry ({score}) must not exceed -MAX ({})",
            -ContinuationHistory::MAX
        );
    }
}
