// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::score::LargeScoreType;
use chess::definitions::MAX_MOVE_LIST_SIZE;
use chess::move_list::MoveList;
use chess::moves::Move;

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct ScoredMove {
    pub(crate) mv: Move,
    pub(crate) score: LargeScoreType,
}
