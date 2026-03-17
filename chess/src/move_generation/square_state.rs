use crate::{attacks, bitboard::Bitboard, board::Board, pieces::Piece, side::Side, square::Square};

/// Returns true if the given square is attacked by any piece on the attacking_side.
/// Uses the "super-piece" method: project attacks FROM the target square and check for collisions.
pub fn is_square_attacked_with_occupancy(
    board: &Board,
    square: Square,
    attacking_side: Side,
    occupancy: Bitboard,
) -> bool {
    let sq = square.to_square_index();

    let king_attacks = attacks::king(sq);
    let knight_attacks = attacks::knight(sq);
    let rook_attacks = attacks::rook(sq, occupancy);
    let bishop_attacks = attacks::bishop(sq, occupancy);
    let queen_attacks = rook_attacks | bishop_attacks;
    // Pawn attacks use the opposite side's direction (super-piece method)
    let pawn_attacks = attacks::pawn(sq, attacking_side.opposite());

    (king_attacks & *board.piece_bitboard(Piece::King, attacking_side)) > 0
        || (knight_attacks & *board.piece_bitboard(Piece::Knight, attacking_side)) > 0
        || (rook_attacks & *board.piece_bitboard(Piece::Rook, attacking_side)) > 0
        || (bishop_attacks & *board.piece_bitboard(Piece::Bishop, attacking_side)) > 0
        || (queen_attacks & *board.piece_bitboard(Piece::Queen, attacking_side)) > 0
        || (pawn_attacks & *board.piece_bitboard(Piece::Pawn, attacking_side)) > 0
}

pub fn is_square_attacked(board: &Board, square: Square, attacking_side: Side) -> bool {
    is_square_attacked_with_occupancy(board, square, attacking_side, board.all_pieces())
}

pub fn is_attacked(
    squares: Bitboard,
    attacking_side: Side,
    occupancy: Bitboard,
    board: &Board,
) -> bool {
    for sq in squares.iter() {
        if is_square_attacked_with_occupancy(
            board,
            Square::from_square_index(sq),
            attacking_side,
            occupancy,
        ) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::{
        move_generation::{self, square_state},
        move_list::MoveList,
        moves::MoveType,
    };

    use super::*;

    #[test]
    fn check_is_square_attacked() {
        let board = Board::default_board();
        let us = Side::White;
        let them = us.opposite();
        let black_pieces = board.pieces(them);
        for sq in black_pieces.iter() {
            let square = Square::from_square_index(sq);
            let is_attacked = is_square_attacked(&board, square, us);
            assert!(!is_attacked);
        }

        let mut move_list = MoveList::new();
        move_generation::generate_moves(&board, &mut move_list, MoveType::All);
        let side_to_move = board.side_to_move();
        for mv in move_list.iter().filter(|mv| !mv.is_pawn_two_up()) {
            let to = mv.to();
            let is_attacked =
                is_square_attacked(&board, Square::from_square_index(to), side_to_move);
            assert!(is_attacked, "Square {to} is not attacked by move\n\t{mv}",);
        }

        {
            let board = Board::from_fen("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2").unwrap();
            let king_sq = board.king_square(board.side_to_move());
            assert_eq!(board.side_to_move(), Side::Black);
            assert!(square_state::is_square_attacked(
                &board,
                Square::from_square_index(king_sq),
                board.side_to_move().opposite()
            ));
        }

        {
            let mut board =
                Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                    .unwrap();
            move_generation::generate_moves(&board, &mut move_list, MoveType::All);
            let mv = move_list
                .iter()
                .find(|mv| mv.to_long_algebraic() == "b1c3")
                .unwrap();
            assert!(board.make_move(mv).is_ok());

            let king_sq = board.king_square(Side::White);
            assert_eq!(board.side_to_move(), Side::Black);
            assert!(!square_state::is_square_attacked(
                &board,
                Square::from_square_index(king_sq),
                Side::Black
            ));
        }
    }
}
