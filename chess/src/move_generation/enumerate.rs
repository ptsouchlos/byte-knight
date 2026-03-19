// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module defines functionality for enumerating moves from a given bitboard.

use crate::{
    bitboard::Bitboard,
    board::Board,
    move_list::MoveList,
    moves::{Move, MoveDescriptor, PromotionDescriptor},
    pieces::Piece,
    rank::Rank,
    square::{self, Square},
};

/// Controls which promotion types are generated during move enumeration.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PromotionFilter {
    /// Generate all 4 promotion types (Queen, Rook, Bishop, Knight).
    All,
    /// Generate only queen promotions. Used for tactical move generation.
    QueenOnly,
    /// Generate only underpromotions (Rook, Bishop, Knight). Used for quiet move generation.
    UnderOnly,
}

/// Enumerate all moves in a given bitboard and add them to the given [`MoveList`]
///
/// # Arguments
/// - `bitboard`: The [`Bitboard`] to enumerate moves for.
/// - `from`: The from square the moves originate from.
/// - `piece`: The `piece` that is moving.
/// - `board`: The current [`Board`].
/// - `move_list`: The [`MoveList`] to push enumerated moves into.
/// - `promotion_filter`: Controls which promotion types are generated.
#[allow(clippy::panic)]
pub(crate) fn enumerate_moves(
    bitboard: &Bitboard,
    from: &Square,
    piece: Piece,
    board: &Board,
    move_list: &mut MoveList,
    promotion_filter: PromotionFilter,
) {
    // Stop if the bitboard is empty.
    if bitboard.as_number() == 0 {
        return;
    }

    let us = board.side_to_move();
    let them = us.opposite();
    let enemy_pieces = board.pieces(them);
    let promotion_rank = Rank::promotion_rank(us);
    for to_square in bitboard.iter() {
        let (file, rank) = square::from_square(to_square);
        let (from_file, _) = square::from_square(from.to_square_index());

        let en_passant = match board.en_passant_square() {
            Some(en_passant_square) => en_passant_square == to_square && piece == Piece::Pawn,
            None => false,
        };

        let is_capture: bool = enemy_pieces.is_square_occupied(to_square) || en_passant;
        // 2 rows = 16 squares
        let is_double_move =
            piece == Piece::Pawn && (to_square as i8 - from.to_square_index() as i8).abs() == 16;
        let is_promotion =
            piece == Piece::Pawn && square::is_square_on_rank(to_square, promotion_rank as u8);

        if is_double_move && en_passant {
            panic!("Double move and en passant should not happen");
        }

        // a castle is the only time a king can move 2 squares
        let is_castle = piece == Piece::King && from_file.abs_diff(file) == 2;

        let mut move_desc = MoveDescriptor::None;
        if is_double_move {
            move_desc = MoveDescriptor::PawnTwoUp;
        } else if en_passant {
            move_desc = MoveDescriptor::EnPassantCapture;
        } else if is_castle {
            move_desc = MoveDescriptor::Castle;
        }

        let capture_piece = if is_capture && !en_passant {
            Some(board.piece_on_square(to_square).unwrap().0)
        } else if en_passant {
            Some(Piece::Pawn)
        } else {
            None
        };

        let to_square = square::to_square_object(file, rank);
        if is_promotion {
            let promotion_types: &[PromotionDescriptor] = match promotion_filter {
                PromotionFilter::All => &[
                    PromotionDescriptor::Queen,
                    PromotionDescriptor::Rook,
                    PromotionDescriptor::Bishop,
                    PromotionDescriptor::Knight,
                ],
                PromotionFilter::QueenOnly => &[PromotionDescriptor::Queen],
                PromotionFilter::UnderOnly => &[
                    PromotionDescriptor::Rook,
                    PromotionDescriptor::Bishop,
                    PromotionDescriptor::Knight,
                ],
            };
            for promotion_type in promotion_types {
                let mv = Move::new(
                    from,
                    &to_square,
                    move_desc,
                    piece,
                    capture_piece,
                    Some(promotion_type.to_piece()),
                );
                move_list.push(mv);
            }
        } else if is_castle {
            let mv = Move::new_castle(from, &to_square);
            move_list.push(mv);
        } else {
            let mv = Move::new(from, &to_square, move_desc, piece, capture_piece, None);
            move_list.push(mv);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{board::Board, move_list::MoveList, pieces::Piece};

    #[test]
    fn enumerate_queen_promotion_only() {
        // White pawn on e7, no captures — push to e8
        let board = Board::from_fen("4k3/4P3/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let pawn_sq = Square::from_file_rank('e', 6).unwrap(); // e7
        let bb = Bitboard::from_square(60); // e8

        let mut all = MoveList::new();
        enumerate_moves(
            &bb,
            &pawn_sq,
            Piece::Pawn,
            &board,
            &mut all,
            PromotionFilter::All,
        );
        assert_eq!(all.len(), 4); // Q, R, B, N

        let mut queen_only = MoveList::new();
        enumerate_moves(
            &bb,
            &pawn_sq,
            Piece::Pawn,
            &board,
            &mut queen_only,
            PromotionFilter::QueenOnly,
        );
        assert_eq!(queen_only.len(), 1);
        assert!(
            queen_only
                .iter()
                .all(|mv| mv.promotion_piece() == Some(Piece::Queen))
        );

        let mut under_only = MoveList::new();
        enumerate_moves(
            &bb,
            &pawn_sq,
            Piece::Pawn,
            &board,
            &mut under_only,
            PromotionFilter::UnderOnly,
        );
        assert_eq!(under_only.len(), 3);
        assert!(
            under_only
                .iter()
                .all(|mv| mv.promotion_piece() != Some(Piece::Queen))
        );
    }
}
