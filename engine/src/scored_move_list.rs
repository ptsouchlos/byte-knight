// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{definitions::MAX_MOVE_LIST_SIZE, util::ArrayVecList};

use crate::scored_move::ScoredMove;

pub(crate) type ScoredMoveList = ArrayVecList<ScoredMove, MAX_MOVE_LIST_SIZE>;
