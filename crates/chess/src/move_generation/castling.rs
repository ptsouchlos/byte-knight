// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{
    bitboard::Bitboard, board::Board, move_generation, pieces::Piece, side::Side, square::Square,
};

// Credit to Hobbes author for CastleSafety and CastleTravel masks.
/// Squares that must not be attacked when the king castles
pub struct CastleSafety;

impl CastleSafety {
    pub const WQS: Bitboard = Bitboard::new(0x000000000000001C);
    pub const WKS: Bitboard = Bitboard::new(0x0000000000000070);
    pub const BQS: Bitboard = Bitboard::new(0x1C00000000000000);
    pub const BKS: Bitboard = Bitboard::new(0x7000000000000000);
}

/// Squares that must be unoccupied when the king castles
pub struct CastleTravel;

impl CastleTravel {
    pub const WKS: Bitboard = Bitboard::new(0x0000000000000060);
    pub const WQS: Bitboard = Bitboard::new(0x000000000000000E);
    pub const BKS: Bitboard = Bitboard::new(0x6000000000000000);
    pub const BQS: Bitboard = Bitboard::new(0x0E00000000000000);
}

/// Starting square for the Rook.
///
/// # Arguments
/// - `side`: The side to get the starting square for.
/// - `kingside`: True for kingside rook, false otherwise.
///
/// # Returns
/// Square index
pub fn rook_from(side: Side, kingside: bool) -> Square {
    match (side, kingside) {
        (Side::White, true) => Square::H1,
        (Side::White, false) => Square::A1,
        (Side::Black, true) => Square::H8,
        (Side::Black, false) => Square::A8,
    }
}

/// Starting square for the King.
///
/// # Arguments
/// - `side`: The side to get the starting square for.
///
/// # Returns
/// Square index
pub fn king_from(side: Side) -> Square {
    match side {
        Side::White => Square::E1,
        Side::Black => Square::E8,
    }
}

/// Generate legal castling moves for the king.
///
/// # Arguments
///
/// - square - The square the king is on
/// - board - The current board state
/// - attacked_squares - The squares that are attacked by the opponent
/// - checkers - The squares that are checking the king
///
/// # Returns
///
/// A [`Bitboard`] with the legal castling moves for the king.
pub(crate) fn legal_mobility(board: &Board, checkers: Bitboard) -> Bitboard {
    /*
     * For castling, the king and rook must not have moved.
     * The squares between the king and rook must be empty.
     * The squares the king moves through must not be under attack (including start and end).
     * The king must not be in check.
     * The king must not move through check.
     * The king must not end up in check.
     *
     * FIDE Laws of Chess:
     * 3.8.2.1 The right to castle has been lost:
     *     3.8.2.1.1 if the king has already moved, or
     *     3.8.2.1.2 with a rook that has already moved.
     *
     * 3.8.2.2 Castling is prevented temporarily:
     *     3.8.2.2.1 if the square on which the king stands, or the square which it must cross, or the square which it is to occupy, is attacked by one or more of the opponent's pieces, or
     *     3.8.2.2.2 if there is any piece between the king and the rook with which castling is to be effected.
     */

    let in_check = checkers.number_of_occupied_squares() > 0;
    if in_check {
        return Bitboard::default();
    }

    let us = board.side_to_move();
    let them = us.opposite();
    let occ = board.all_pieces();
    let mut castling_moves = Bitboard::default();
    let king_side_castle = board.can_castle_kingside(us);
    let queen_side_castle = board.can_castle_queenside(us);
    let king_sq = board.king_square(us);

    let is_square_in_place = king_sq == king_from(us);
    if !is_square_in_place {
        return Bitboard::default();
    }

    if king_side_castle {
        let travel_mask = if us == Side::White {
            CastleTravel::WKS
        } else {
            CastleTravel::BKS
        };
        let safety_mask = if us == Side::White {
            CastleSafety::WKS
        } else {
            CastleSafety::BKS
        };

        let rook_in_place = board
            .piece_on_square(rook_from(us, true))
            .is_some_and(|(piece, side)| piece == Piece::Rook && side == us);
        if (occ & travel_mask).is_empty()
            && !move_generation::square_state::is_attacked(safety_mask, them, occ, board)
            && rook_in_place
        {
            castling_moves |= Bitboard::from_square(king_sq.inner() + 2);
        }
    }
    if queen_side_castle {
        let travel_mask = if us == Side::White {
            CastleTravel::WQS
        } else {
            CastleTravel::BQS
        };
        let safety_mask = if us == Side::White {
            CastleSafety::WQS
        } else {
            CastleSafety::BQS
        };

        let rook_in_place = board
            .piece_on_square(rook_from(us, false))
            .is_some_and(|(piece, side)| piece == Piece::Rook && side == us);

        if (occ & travel_mask).is_empty()
            && !move_generation::square_state::is_attacked(safety_mask, them, occ, board)
            && rook_in_place
        {
            castling_moves |= Bitboard::from_square(king_sq.inner() - 2);
        }
    }
    castling_moves
}

#[cfg(test)]
mod tests {
    use crate::move_generation::castling::{CastleSafety, CastleTravel};

    #[test]
    fn castle_safety() {
        println!(
            "{}\n{}\n{}\n{}\n",
            CastleSafety::WKS,
            CastleSafety::WQS,
            CastleSafety::BKS,
            CastleSafety::BQS
        );
    }

    #[test]
    fn castle_travel() {
        println!(
            "{}\n{}\n{}\n{}\n",
            CastleTravel::WKS,
            CastleTravel::WQS,
            CastleTravel::BKS,
            CastleTravel::BQS
        );
    }
}
