// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{definitions::CastlingAvailability, moves::Move, side::Side, zobrist::ZobristHash};
use std::fmt::Display;

/// Represents the state of the board at a given point in time.
/// This includes the half move clock, full move number, side to move,
/// en passant square, castling rights, and the Zobrist hash.
///
/// This is used to restore the state in [`Board`] when un-making a move.
#[derive(Debug, Clone, Copy)]
pub struct BoardState {
    pub half_move_clock: u32,
    pub full_move_number: u32,
    pub side_to_move: Side,
    pub en_passant_square: Option<u8>,
    pub castling_rights: u8,
    pub zobrist_hash: ZobristHash,
    pub next_move: Move,
}

impl Default for BoardState {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardState {
    pub fn new() -> Self {
        BoardState {
            half_move_clock: 0,
            full_move_number: 1,
            side_to_move: Side::White,
            en_passant_square: None,
            castling_rights: CastlingAvailability::NONE,
            zobrist_hash: 0,
            next_move: Move::default(),
        }
    }
}

impl Display for BoardState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "state {{ half_move_clock: {}, full_move_number: {}, side_to_move: {:?}, en_passant_square: {:?}, castling_rights: {:?}, zobrist_hash: {}, next_move: {} }}",
            self.half_move_clock,
            self.full_move_number,
            self.side_to_move,
            self.en_passant_square,
            self.castling_rights,
            self.zobrist_hash,
            self.next_move.to_long_algebraic()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::Move;

    #[test]
    fn default_board_state() {
        let board_state = BoardState::default();
        assert_eq!(board_state.half_move_clock, 0);
        assert_eq!(board_state.full_move_number, 1);
        assert_eq!(board_state.side_to_move, Side::White);
        assert_eq!(board_state.en_passant_square, None);
        assert_eq!(board_state.castling_rights, CastlingAvailability::NONE);
        assert_eq!(board_state.zobrist_hash, 0);
        assert_eq!(board_state.next_move, Move::default());
    }

    #[test]
    fn display_board_state() {
        let board_state = BoardState::new();
        let expected = "state { half_move_clock: 0, full_move_number: 1, side_to_move: White, en_passant_square: None, castling_rights: 0, zobrist_hash: 0, next_move: a1a1 }";
        assert_eq!(board_state.to_string(), expected);
    }
}
