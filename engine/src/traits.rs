// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{pieces::Piece, side::Side};

use crate::score::Score;

pub trait Eval<Board> {
    fn eval(&self, board: &Board) -> Score;
}

pub trait EvalValues {
    type ReturnScore;
    fn psqt(&self, square: u8, piece: Piece, side: Side) -> Self::ReturnScore;
    fn passed_pawn_bonus(&self, square: u8, side: Side) -> Self::ReturnScore;
    fn doubled_pawn_value(&self, square: u8, side: Side) -> Self::ReturnScore;
    fn isolated_pawn_value(&self, square: u8, side: Side) -> Self::ReturnScore;
}
