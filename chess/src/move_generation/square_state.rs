use crate::{attacks, bitboard::Bitboard, board::Board, side::Side, square::Square};

pub fn is_square_attacked(board: &Board, square: Square, attacking_side: Side) -> bool {
    !attacks::all_attackers_of(
        square.to_square_index(),
        board,
        attacking_side,
        board.all_pieces(),
    )
    .is_empty()
}

pub fn is_attacked(
    squares: Bitboard,
    attacking_side: Side,
    occupancy: Bitboard,
    board: &Board,
) -> bool {
    for sq in squares.iter() {
        if !attacks::all_attackers_of(sq, board, attacking_side, occupancy).is_empty() {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use crate::{
        move_generation::{self, MoveFilter, square_state},
        move_list::MoveList,
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
        move_generation::generate_moves(&board, &mut move_list, MoveFilter::All);
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
            move_generation::generate_moves(&board, &mut move_list, MoveFilter::All);
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
