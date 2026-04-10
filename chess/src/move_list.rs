// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{definitions::MAX_MOVE_LIST_SIZE, moves::Move, util::ArrayVecList};

/// A list of moves used in move generation. This is a fixed-size list that can hold up to 218 moves.
/// If more moves are added, the program will panic.
pub type MoveList = ArrayVecList<Move, MAX_MOVE_LIST_SIZE>;

#[cfg(test)]
mod tests {
    use crate::{definitions::MAX_MOVE_LIST_SIZE, moves::MoveFlag, square::Square};

    use super::*;

    #[test]
    fn default() {
        let move_list: MoveList = Default::default();
        assert_eq!(move_list.len(), 0);
        assert!(move_list.is_empty());
    }

    #[test]
    fn push() {
        let mut move_list = MoveList::new();
        assert_eq!(move_list.len(), 0);
        assert!(move_list.is_empty());

        let mv = Move::new(
            Square::from_square_index(8),
            Square::from_square_index(16),
            MoveFlag::Standard,
        );
        move_list.push(mv);
        assert_eq!(move_list.len(), 1);
        assert!(!move_list.is_empty());

        move_list.push(mv);
        assert_eq!(move_list.len(), 2);
    }

    #[test]
    #[should_panic]
    fn push_with_overflow() {
        let mut move_list = MoveList::new();
        assert_eq!(move_list.len(), 0);
        assert!(move_list.is_empty());

        for _ in 0..MAX_MOVE_LIST_SIZE {
            let mv = Move::new(
                Square::from_square_index(3_u8),
                Square::from_square_index(13_u8),
                MoveFlag::Standard,
            );
            move_list.push(mv);
        }
        assert_eq!(move_list.len(), MAX_MOVE_LIST_SIZE);
        assert!(!move_list.is_empty());

        // This will panic
        let mv = Move::new(
            Square::from_square_index(0),
            Square::from_square_index(1),
            MoveFlag::Standard,
        );
        move_list.push(mv);
    }
}
