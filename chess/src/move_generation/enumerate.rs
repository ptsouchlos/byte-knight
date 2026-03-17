use crate::{
    bitboard::Bitboard,
    board::Board,
    move_list::MoveList,
    moves::{Move, MoveDescriptor, PromotionDescriptor},
    pieces::Piece,
    rank::Rank,
    square::{self, Square},
};

/// Enumerate all moves in a given bitboard and add them to the given [`MoveList`]
#[allow(clippy::panic)]
pub(crate) fn enumerate_moves(
    bitboard: &Bitboard,
    from: &Square,
    piece: Piece,
    board: &Board,
    move_list: &mut MoveList,
) {
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
            // we have to add 4 moves for each promotion type
            for promotion_type in [
                PromotionDescriptor::Queen,
                PromotionDescriptor::Rook,
                PromotionDescriptor::Bishop,
                PromotionDescriptor::Knight,
            ] {
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
