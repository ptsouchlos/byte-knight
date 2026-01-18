// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::move_order::MoveOrder;
use chess::moves::Move;

/// Iterator type that yields moves in a sorted order based on their move orders.
/// The moves are sorted in-place, meaning that the original array of moves is modified.
/// The sorting is done in a way that the highest scoring moves are returned first.
/// The iterator is stateful and keeps track of the current index in the moves array.
/// The iterator is designed to be used in a loop, where each call to `next()`.
pub(crate) struct InplaceIncrementalSort<'s> {
    moves: &'s mut [Move],
    move_order: &'s mut [MoveOrder],
    current_index: usize,
}

impl<'s> InplaceIncrementalSort<'s> {
    /// Creates a new `InplaceIncrementalSort` iterator.
    ///
    /// # Arguments
    ///
    /// * `moves` - A mutable slice of moves to be sorted.
    /// * `order` - A mutable slice of move orders corresponding to the moves.
    ///
    /// # Panics
    ///
    /// Panics if the lengths of `moves` and `order` are not equal.
    ///
    pub(crate) fn new(moves: &'s mut [Move], order: &'s mut [MoveOrder]) -> Self {
        debug_assert!(
            moves.len() == order.len(),
            "Moves and move orders must have the same length"
        );

        Self {
            moves,
            move_order: order,
            current_index: 0,
        }
    }

    pub(crate) fn has_next(&self) -> bool {
        self.current_index < self.moves.len()
    }

    pub(crate) fn next(&mut self) -> Option<Move> {
        if !self.has_next() {
            return None;
        }

        // Find the index of the maximum score among remaining elements
        let mut max_idx = self.current_index;
        for i in self.current_index..self.move_order.len() {
            // This comparison ensures we get the HIGHEST scores first
            // MoveOrdering sorts in descending order
            if self.move_order[i] < self.move_order[max_idx] {
                max_idx = i;
            }
        }

        // Swap the max element with the current position
        if max_idx != self.current_index {
            self.move_order.swap(self.current_index, max_idx);
            self.moves.swap(self.current_index, max_idx);
        }

        // Move the index forward and return the current max item
        let current = self.current_index;
        self.current_index += 1;

        // Take the item at the current index
        Some(self.moves[current])
    }
}

impl Iterator for InplaceIncrementalSort<'_> {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.moves.len() - self.current_index;
        (remaining, Some(remaining))
    }

    fn count(self) -> usize {
        self.moves.len() - self.current_index
    }
}

#[cfg(test)]
mod tests {

    use chess::{
        board::Board,
        move_generation::MoveGenerator,
        move_list::MoveList,
        moves::{Move, MoveType},
        pieces::Piece,
    };

    use crate::{inplace_incremental_sort::InplaceIncrementalSort, move_order::MoveOrder};

    #[test]
    fn test_incremental_sort() {
        let move_gen = MoveGenerator::new();
        let mut move_list = MoveList::new();
        let board = Board::default_board();

        // generate moves for starting position (20)
        move_gen.generate_moves(&board, &mut move_list, MoveType::All);

        let mut order = vec![
            MoveOrder::Quiet(14),
            MoveOrder::Capture(Piece::Bishop, Piece::Rook),
            MoveOrder::Capture(Piece::Queen, Piece::Pawn),
            MoveOrder::TtMove,
            MoveOrder::Quiet(15),
            MoveOrder::Quiet(16),
            MoveOrder::Quiet(17),
            MoveOrder::Quiet(100),
            MoveOrder::Capture(Piece::Pawn, Piece::Queen),
            MoveOrder::Capture(Piece::Knight, Piece::Bishop),
            MoveOrder::Quiet(4),
            MoveOrder::Quiet(24),
            MoveOrder::Quiet(43),
            MoveOrder::Quiet(987),
            MoveOrder::Quiet(354),
            MoveOrder::Quiet(64),
            MoveOrder::Quiet(79),
            MoveOrder::Quiet(484),
            MoveOrder::Quiet(769),
            MoveOrder::Quiet(13),
        ];

        let print_test_moves = |msg: &str, mvs: &[Move], order: &[MoveOrder]| {
            println!("{msg}:");
            for i in 0..mvs.len() {
                println!(
                    "test_moves[{i}]: {} {:?}",
                    mvs[i].to_long_algebraic(),
                    order[i]
                );
            }
        };

        print_test_moves("before sorting", move_list.as_slice(), &order);

        let incremental_sort = InplaceIncrementalSort::new(move_list.as_mut_slice(), &mut order);

        // check if has next
        assert!(incremental_sort.has_next());

        let expected_moves = vec![
            "g1h3", "g1f3", "b1c3", "c2c4", "c2c3", "e2e4", "h2h3", "g2g4", "f2f3", "b2b4", "g2g3",
            "f2f4", "e2e3", "d2d4", "b2b3", "a2a4", "a2a3", "b1a3", "h2h4", "d2d3",
        ];

        // loop through the iterator
        for (i, mv) in incremental_sort.enumerate() {
            println!("{}: {}", i, mv.to_long_algebraic());
            assert_eq!(mv.to_long_algebraic(), expected_moves[i]);
        }

        print_test_moves("after sorting", move_list.as_slice(), &order);
    }
}
