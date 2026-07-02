// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{
    attacks,
    bitboard::Bitboard,
    board::Board,
    move_generation::{NORTH, SOUTH},
    pieces::Piece,
    rays,
    side::Side,
};

/// Precomputed check and pin metadata for the current position.
///
/// This is computed once per position and shared across staged move generation
/// (tacticals and quiets). All fields are needed by the legal move generators
/// to enforce pin, check evasion, and capture/push mask constraints.
#[derive(Clone, Debug)]
pub struct CheckPinMetadata {
    /// Bitboard of enemy pieces currently giving check to the king.
    pub checkers: Bitboard,
    /// Bitboard of squares that can be captured (filtered when in check).
    pub capture_mask: Bitboard,
    /// Bitboard of squares that can be pushed to (ray between checker and king for slider checks).
    pub push_mask: Bitboard,
    /// Bitboard of our pieces that are pinned to the king.
    pub pinned: Bitboard,
    /// Bitboard of orthogonal pin rays (rook/queen pins along ranks/files).
    pub orthogonal_pin_rays: Bitboard,
    /// Bitboard of diagonal pin rays (bishop/queen pins along diagonals).
    pub diagonal_pin_rays: Bitboard,
    /// All occupied squares (both sides). Cached here so the per-piece legal
    /// generators don't recompute it on every call.
    pub occupancy: Bitboard,
    /// Bitboard of the side-to-move's pieces.
    pub our_pieces: Bitboard,
    /// Bitboard of the enemy's pieces.
    pub their_pieces: Bitboard,
}

impl CheckPinMetadata {
    /// Returns true if the side to move is in check.
    pub fn in_check(&self) -> bool {
        !self.checkers.is_empty()
    }

    /// Returns the number of pieces giving check.
    pub fn num_checkers(&self) -> u32 {
        self.checkers.number_of_occupied_squares()
    }
}

/// Compute check and pin metadata for the current position.
///
/// Uses the "super-piece" method: projects attacks from the king square
/// to find checkers and pinners in a single pass over enemy sliding pieces.
pub fn compute(board: &Board) -> CheckPinMetadata {
    let us = board.side_to_move();
    let them = us.opposite();
    let occupancy = board.all_pieces();
    let empty = !occupancy;
    let their_pieces = board.pieces(them);
    let our_pieces = board.pieces(us);
    let enemy_or_empty = their_pieces | empty;
    let king_sq = board.king_square(us);

    let mut pinned = Bitboard::default();
    let mut capture_mask = enemy_or_empty & !(board.piece_bitboard(Piece::King, them));
    let mut orthogonal_pin_rays = Bitboard::default();
    let mut diagonal_pin_rays = Bitboard::default();

    // Super-piece method: project attacks from king square with opposite side semantics
    let mut checkers = board.piece_bitboard(Piece::Knight, them) & attacks::knight(king_sq)
        | board.piece_bitboard(Piece::Pawn, them) & attacks::pawn(king_sq, us);

    let enemy_sliding_attacks = attacks::rook(king_sq, Bitboard::default())
        & (board.piece_bitboard(Piece::Rook, them) | board.piece_bitboard(Piece::Queen, them))
        | attacks::bishop(king_sq, Bitboard::default())
            & (board.piece_bitboard(Piece::Bishop, them)
                | board.piece_bitboard(Piece::Queen, them));

    for next_attacker_sq in enemy_sliding_attacks {
        let attacker_bb = Bitboard::from(next_attacker_sq);

        let ray = rays::between(king_sq, next_attacker_sq);

        let king_file = king_sq.file();
        let king_rank = king_sq.rank();
        let attacker_file = next_attacker_sq.file();
        let attacker_rank = next_attacker_sq.rank();
        let is_orthogonal = king_file == attacker_file || king_rank == attacker_rank;
        let is_diagonal = (king_sq.inner() as i16 - next_attacker_sq.inner() as i16).abs() % 9 == 0
            || (king_sq.inner() as i16 - next_attacker_sq.inner() as i16).abs() % 7 == 0;

        match (ray & occupancy).number_of_occupied_squares() {
            0 => {
                checkers |= Bitboard::from(next_attacker_sq);
            }
            1 => {
                let overlap = ray & our_pieces;
                if overlap.number_of_occupied_squares() == 1 {
                    pinned |= ray & our_pieces;
                    if is_orthogonal {
                        orthogonal_pin_rays |= ray | attacker_bb;
                    } else if is_diagonal {
                        diagonal_pin_rays |= ray | attacker_bb;
                    }
                }
            }
            _ => {}
        }
    }

    let mut push_mask = Bitboard::filled();

    let checkers_count = checkers.number_of_occupied_squares();
    if checkers_count >= 1 {
        let is_single_check = checkers_count == 1;

        capture_mask = checkers & !(board.piece_bitboard(Piece::King, them));

        if is_single_check {
            // We've already established that the checkers bitboard has exactly 1 occuppied square.
            let checker = checkers.lsb().unwrap();

            let ray = rays::between(king_sq, checker);

            if let Some((piece, side)) = board.piece_on_square(checker) {
                debug_assert!(side == them);
                let is_slider = piece.is_slider();
                if is_slider {
                    push_mask = ray;
                } else {
                    push_mask = Bitboard::default();
                }
            }
        }
    }

    let en_passant_bb = board
        .en_passant_square()
        .map(Bitboard::from)
        .unwrap_or_default();
    match board.side_to_move() {
        Side::White => {
            let left = en_passant_bb >> SOUTH;
            if left & checkers != 0 {
                capture_mask |= en_passant_bb;
            }
        }
        Side::Black => {
            let right = en_passant_bb << NORTH;
            if right & checkers != 0 {
                capture_mask |= en_passant_bb;
            }
        }
    }

    CheckPinMetadata {
        checkers,
        capture_mask,
        push_mask,
        pinned,
        orthogonal_pin_rays,
        diagonal_pin_rays,
        occupancy,
        our_pieces,
        their_pieces,
    }
}
