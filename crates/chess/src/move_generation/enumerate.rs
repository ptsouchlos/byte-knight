// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module defines functionality for enumerating moves from a given bitboard.

use crate::{
    bitboard::Bitboard,
    board::Board,
    move_generation::MoveFilter,
    move_list::MoveList,
    moves::{Move, MoveFlag},
    pieces::Piece,
    rank::Rank,
    square::{self, Square},
};

/// Enumerate all moves in a given bitboard and add them to the given [`MoveList`]
///
/// # Arguments
/// - `bitboard`: The [`Bitboard`] to enumerate moves for.
/// - `from`: The from square the moves originate from.
/// - `piece`: The `piece` that is moving.
/// - `board`: The current [`Board`].
/// - `move_filter`: The current move filter. Used to handling pawn promotion enumeration.
/// - `move_list`: The [`MoveList`] to push enumerated moves into.
/// - `promotion_filter`: Controls which promotion types are generated.
#[allow(clippy::panic)]
pub(crate) fn enumerate_moves(
    bitboard: &Bitboard,
    from: Square,
    piece: Piece,
    board: &Board,
    move_filter: MoveFilter,
    move_list: &mut MoveList,
) {
    // Stop if the bitboard is empty.
    if bitboard.as_number() == 0 {
        return;
    }

    let from_sq_idx = from.inner();
    let (from_file, _from_rank) = square::from_square(from_sq_idx);

    let us = board.side_to_move();

    let promotion_rank = Rank::promotion_rank(us);
    for to_square in bitboard.iter() {
        let (file, rank) = square::from_square(to_square);

        let is_en_passant = match board.en_passant_square() {
            Some(en_passant_square) => en_passant_square == to_square && piece == Piece::Pawn,
            None => false,
        };

        // 2 rows = 16 squares
        let is_double_move =
            piece == Piece::Pawn && (to_square as i8 - from_sq_idx as i8).abs() == 16;
        let is_promotion =
            piece == Piece::Pawn && square::is_square_on_rank(to_square, promotion_rank as u8);

        if is_double_move && is_en_passant {
            panic!("Double move and en passant should not happen");
        }

        // A castle is the only time a king can move 2 squares
        let is_castle = piece == Piece::King && from_file.abs_diff(file) == 2;

        let to_square = square::to_square_object(file, rank);
        // Promotions are not quiet moves
        if is_promotion && move_filter != MoveFilter::Quiets {
            let flags: &[MoveFlag] = &[
                MoveFlag::PromotionQueen,
                MoveFlag::PromotionRook,
                MoveFlag::PromotionBishop,
                MoveFlag::PromotionKnight,
            ];
            for flg in flags {
                let mv = Move::new(from, to_square, *flg);
                move_list.push(mv);
            }
        } else if is_castle {
            let mv = Move::new_castle(from, to_square);
            move_list.push(mv);
        } else {
            let move_desc = if is_en_passant {
                MoveFlag::EnPassant
            } else if is_double_move {
                MoveFlag::DoublePush
            } else {
                MoveFlag::Standard
            };
            let mv = Move::new(from, to_square, move_desc);
            move_list.push(mv);
        }
    }
}
