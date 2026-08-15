// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! A cache for pawn structure evaluations, indexed by a hash of the pawn positions.
//! This cache is designed to speed up the evaluation of pawn structures by storing
//! previously computed scores for pawn structures that have already been seen.

use std::cell::Cell;

use chess::{bitboard::Bitboard, side::Side};

use crate::{phased_score::PhasedScore, utils};
const SIZE: usize = 1 << 15; // 32768 entries

/// Entry type for the pawn cache.
/// Each entry stores the bitboard for white/black pawns to ensure
/// that we can verify the correctness of the cached score in case of hash collisions.
/// The pawn structure bitboards are not stored because they're too big and unnecessary.
#[derive(Debug, Clone, Copy, Default)]
struct Entry {
    white_pawns: Bitboard,
    black_pawns: Bitboard,
    score: [PhasedScore; Side::COUNT],
}

/// A simple pawn cache implementation that uses a fixed-size array of `Cell<Entry>` to store pawn structure evaluations.
/// Box<Cell<Entry>> is used to allow interior mutability while keeping the overall structure of the cache simple and efficient.
/// This lets us update entries in place without needing &mut references to the cache.
pub(crate) struct PawnCache {
    table: Box<[Cell<Entry>; SIZE]>,
}

impl PawnCache {
    /// Create a new pawn cache. All entries are default initialized.
    pub fn new() -> Self {
        Self {
            table: vec![Cell::default(); SIZE]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
        }
    }

    /// Helper to get the index in the cache for a given pawn hash.
    fn get_index(&self, pawn_hash: u64) -> usize {
        utils::fast_range_64(pawn_hash, self.table.len() as u64) as usize
    }

    /// Probe the cache for a given pawn hash and structure.
    /// # Arguments
    ///
    /// - `pawn_hash`: The zobrist hash of the pawn structure, used to index into the cache.
    /// - `white_bb`: The bitboard representing the positions of the white pawns.
    /// - `black_bb`: The bitboard representing the positions of the black pawns.
    ///
    /// # Returns
    /// Optional array of [`PhasedScore`] for both sides if the entry is found and the bitboards match.
    pub fn probe(
        &self,
        pawn_hash: u64,
        white_bb: Bitboard,
        black_bb: Bitboard,
    ) -> Option<[PhasedScore; Side::COUNT]> {
        let index = self.get_index(pawn_hash);
        if let Some(cell) = self.table.get(index) {
            let entry = cell.get();
            if entry.white_pawns == white_bb && entry.black_pawns == black_bb {
                return Some(entry.score);
            }
        }
        None
    }

    /// Store a pawn structure evaluation entry into the cache.
    pub fn store(
        &self,
        pawn_hash: u64,
        white_bb: Bitboard,
        black_bb: Bitboard,
        score: [PhasedScore; Side::COUNT],
    ) {
        let index = self.get_index(pawn_hash);
        self.table[index].set(Entry {
            white_pawns: white_bb,
            black_pawns: black_bb,
            score,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::bitboard::Bitboard;

    #[test]
    fn test_pawn_cache() {
        let cache = PawnCache::new();
        let white_pawns = Bitboard::from_square(12) | Bitboard::from_square(28);
        let black_pawns = Bitboard::from_square(52) | Bitboard::from_square(60);
        let score = [PhasedScore::new(100, 50), PhasedScore::new(-100, -50)];

        cache.store(12345, white_pawns, black_pawns, score);
        let retrieved_score = cache.probe(12345, white_pawns, black_pawns);
        assert_eq!(retrieved_score, Some(score));

        // Test that probing with different bitboards returns None
        // This simulates a hash collision where the pawn hash is the same but the actual pawn positions differ.
        //  In a real chess engine, this could happen due to the nature of Zobrist hashing, so it's important to ensure that the cache correctly handles such cases.
        let different_white_pawns = Bitboard::from_square(13) | Bitboard::from_square(29);
        let different_black_pawns = Bitboard::from_square(53) | Bitboard::from_square(61);
        let retrieved_score_collision =
            cache.probe(12345, different_white_pawns, different_black_pawns);
        assert_eq!(retrieved_score_collision, None);
    }
}
