// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use core::panic;

use arrayvec::ArrayVec;
use chess::{definitions::MAX_MOVES, moves::Move};

/// Represents the [Principle Variation](https://www.chessprogramming.org/Principal_Variation) during a search.
#[derive(Debug, Clone)]
pub struct PrincipleVariation {
    data: ArrayVec<Move, MAX_MOVES>,
}

impl Default for PrincipleVariation {
    fn default() -> Self {
        Self::new()
    }
}

impl PrincipleVariation {
    /// Create a new, empty ['PrincipleVariation'].
    pub(crate) fn new() -> Self {
        Self {
            data: ArrayVec::new(),
        }
    }

    /// Push a move to the ['PrincipleVariation'].
    ///
    /// # Arguments:
    ///
    /// - m: The new move to add to the principle variation.
    ///
    /// # Returns:
    ///
    /// Empty Result<> on success or an error if the underlying ArrayVec
    /// is full before trying to push.
    #[allow(clippy::panic)]
    pub(crate) fn push(&mut self, m: Move) {
        self.data.try_push(m).unwrap_or_else(|err| {
            panic!(
                "Error extending PV of size {} when adding {}\n {err}",
                self.data.len(),
                m
            );
        })
    }

    /// Extend the current [PrincipleVariation] with the given move and another principle variation.
    ///
    /// This will clear the current PV and append the given move and [PrincipleVariation].
    ///
    /// # Arguments:
    ///
    /// - m: The new move to add to the principle variation.
    /// - pv: The principle variation to append to the current variation.
    #[inline(always)]
    #[allow(clippy::panic)]
    pub(crate) fn extend(&mut self, m: Move, pv: &Self) {
        self.clear();
        self.push(m);
        self.data
            .try_extend_from_slice(pv.data.as_slice())
            .unwrap_or_else(|err| {
                panic!(
                    "Error extending PV of size {} when adding {} and {:?}\n  {err}",
                    self.data.len(),
                    m,
                    pv
                );
            })
    }

    /// Clear the principle variation.
    pub(crate) fn clear(&mut self) {
        self.data.clear();
    }

    /// Returns an iterator to the underlying data of the [PrincipleVariation].
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Move> {
        self.data.iter()
    }
}

#[cfg(test)]
mod tests {
    use chess::{definitions::Squares, moves::MoveDescriptor, pieces::Piece, square::Square};

    use super::*;

    fn make_moves() -> (Move, Move) {
        let move1 = Move::new(
            &Square::from_square_index(Squares::E2),
            &Square::from_square_index(Squares::E4),
            MoveDescriptor::PawnTwoUp,
            Piece::Pawn,
            None,
            None,
        );

        let move2 = Move::new(
            &Square::from_square_index(Squares::E7),
            &Square::from_square_index(Squares::E5),
            MoveDescriptor::PawnTwoUp,
            Piece::Pawn,
            None,
            None,
        );

        (move1, move2)
    }

    #[test]
    fn add_moves_to_principle_variation() {
        let mut pv = PrincipleVariation::new();

        let (move1, move2) = make_moves();

        // Push moves to the principle variation
        pv.push(move1);
        pv.push(move2);

        // Check that the moves are in the principle variation
        assert_eq!(pv.data.len(), 2);
    }

    #[test]
    fn extend_principle_variation() {
        let mut pv = PrincipleVariation::new();

        let (move1, move2) = make_moves();

        // Push the first move to the principle variation
        pv.push(move1);

        // Create a new principle variation to extend from
        let mut pv2 = PrincipleVariation::new();
        pv2.push(move2);

        // Extend the original principle variation with the new one
        pv.extend(move1, &pv2);

        // Check that the moves are in the principle variation
        assert_eq!(pv.data.len(), 1 + pv2.data.len());
    }

    #[test]
    #[should_panic(expected = "Error extending PV")]
    fn extending_or_pushing_move_past_max_size_panics() {
        let (move1, move2) = make_moves();
        let mut pv = PrincipleVariation::new();

        // Fill the principle variation to its maximum size
        for _ in 0..MAX_MOVES {
            pv.push(move1);
        }

        // Attempt to push another move, which should fail
        pv.push(move2);

        // reset
        pv = PrincipleVariation::new();
        let mut pv2 = PrincipleVariation::new();

        // Fill the principle variation to its maximum size
        for _ in 0..MAX_MOVES - 10 {
            pv.push(move1);
        }

        for _ in 0..10 {
            pv2.push(move2);
        }

        // Attempt to extend the principle variation with another one that would exceed the max size
        pv.extend(move2, &pv2);
    }
}
