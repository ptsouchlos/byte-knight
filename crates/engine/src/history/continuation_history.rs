// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{moves::Move, pieces::Piece, square::Square};

use crate::{
    history::{types::PieceToHistory, util::gravity},
    score::LargeScoreType,
};

pub struct ContinuationHistory {
    single_ply_entries: PieceToHistory<PieceToHistory<i32>>,
}

impl ContinuationHistory {
    const MAX: i32 = 16384;
    const BONUS_MAX: i32 = Self::MAX / 4;

    pub(crate) fn new() -> Self {
        let single_ply_entries: PieceToHistory<PieceToHistory<i32>> =
            [[[[0; Square::COUNT]; Piece::COUNT]; Square::COUNT]; Piece::COUNT];

        ContinuationHistory { single_ply_entries }
    }

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
        self.single_ply_entries =
            [[[[0; Square::COUNT]; Piece::COUNT]; Square::COUNT]; Piece::COUNT];
    }
}

impl Default for ContinuationHistory {
    fn default() -> Self {
        Self::new()
    }
}
