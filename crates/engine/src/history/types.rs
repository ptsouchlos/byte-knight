// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::definitions::NumberOf;

/// History table storing values of type `T`, indexed by the 'from' and 'to' squares of a move.
/// Also known as 'butterfly' history.
pub(crate) type FromToHistory<T> = [[T; NumberOf::SQUARES]; NumberOf::SQUARES];

pub(crate) fn default_from_to_history<T: Default + Copy>() -> FromToHistory<T> {
    [[Default::default(); NumberOf::SQUARES]; NumberOf::SQUARES]
}
