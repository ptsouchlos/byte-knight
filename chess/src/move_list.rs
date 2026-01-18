// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use arrayvec::ArrayVec;

use crate::{definitions::MAX_MOVE_LIST_SIZE, moves::Move};

/// A list of moves used in move generation. This is a fixed-size list that can hold up to 218 moves.
/// If more moves are added, the program will panic.
pub struct MoveList {
    moves: ArrayVec<Move, MAX_MOVE_LIST_SIZE>,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    /// Create a new [MoveList] with a capacity of 218 moves.
    /// Note that no default assignment is done for the moves in the list.
    /// The intention is for the items in the [`MoveList`] to be overwritten.
    pub fn new() -> Self {
        MoveList {
            moves: ArrayVec::new(),
        }
    }

    /// Returns the number of moves in the list.
    pub fn len(&self) -> usize {
        self.moves.len()
    }

    /// Returns true if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Push a move to the list. If the list is full, the program will panic.
    /// This is done to avoid the overhead of returning a Result.
    pub fn push(&mut self, mv: Move) {
        self.moves.push(mv);
    }

    /// Get an iterator to the moves in the list.
    pub fn iter(&self) -> impl Iterator<Item = &Move> {
        self.moves.iter()
    }

    /// Get the move at the given index. Returns None if the index is out of bounds.
    pub fn at(&self, index: usize) -> Option<&Move> {
        self.moves.get(index)
    }

    /// Clear the list of moves.
    pub fn clear(&mut self) {
        self.moves.clear();
    }

    pub fn as_slice(&self) -> &[Move] {
        self.moves.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        self.moves.as_mut_slice()
    }
}

#[cfg(test)]
mod tests {
    use crate::square::Square;

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

        let mv = Move::new_king_move(
            &Square::from_square_index(8),
            &Square::from_square_index(16),
            None,
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
            let mv = Move::new_king_move(
                &Square::from_square_index(3_u8),
                &Square::from_square_index(13_u8),
                None,
            );
            move_list.push(mv);
        }
        assert_eq!(move_list.len(), MAX_MOVE_LIST_SIZE);
        assert!(!move_list.is_empty());

        // This will panic
        let mv = Move::new_king_move(
            &Square::from_square_index(0),
            &Square::from_square_index(1),
            None,
        );
        move_list.push(mv);
    }
}
