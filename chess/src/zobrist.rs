// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::board::Board;

pub mod keys;
pub mod values;

/// A Zobrist hash value.
pub type ZobristHash = u64;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Hashes {
    board: u64,
    pawn: u64,
}

impl Hashes {
    pub fn new(board: &Board) -> Self {
        Hashes {
            board: keys::get_hash(board),
            pawn: keys::get_pawn_hash(board),
        }
    }

    pub fn update_hash(&mut self, hash: u64) {
        self.board ^= hash;
    }

    pub fn board_hash(&self) -> u64 {
        self.board
    }

    #[allow(dead_code)]
    pub fn pawn_hash(&self) -> u64 {
        self.pawn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn test_hash_consistency() {
        let board = Board::default_board();
        let hash1 = Hashes::new(&board);
        let hash2 = Hashes::new(&board);
        assert_eq!(hash1.board, hash2.board);
        assert_eq!(hash1.pawn, hash2.pawn);
    }
}
