// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::ops::{Add, Mul};

use chess::{pieces::Piece, side::Side};

use crate::score::Score;

pub trait Eval<Board> {
    fn eval(&self, board: &Board) -> Score;
}

pub trait EvalValues {
    type ReturnScore: Mul<i16, Output = Self::ReturnScore> + Add<Output = Self::ReturnScore>;
    fn psqt(&self, square: u8, piece: Piece, side: Side) -> Self::ReturnScore;
    fn passed_pawn_bonus(&self, square: u8, side: Side) -> Self::ReturnScore;
    fn doubled_pawn_value(&self, square: u8, side: Side) -> Self::ReturnScore;
    fn isolated_pawn_value(&self, square: u8, side: Side) -> Self::ReturnScore;
    fn mobility_value(&self, piece: Piece, count: usize, side: Side) -> Self::ReturnScore;
    // The following terms usually don't need a [`Side`] input, but this is necessary for the [`TracingValues`].
    fn bishop_pair_bonus_value(&self, side: Side) -> Self::ReturnScore;
    fn king_safety_value(&self, piece: Piece, side: Side) -> Self::ReturnScore;
    fn threat_value(&self, piece: Piece, attacked_piece: Piece, side: Side) -> Self::ReturnScore;
}
