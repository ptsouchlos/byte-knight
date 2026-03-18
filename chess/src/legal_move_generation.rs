// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::attacks;
use crate::definitions::RANK_BITBOARDS;
use crate::move_generation;
use crate::move_generation::NORTH;
use crate::move_generation::SOUTH;
use crate::move_list::MoveList;
use crate::rays;
use crate::square;
use crate::{
    bitboard::Bitboard, board::Board, pieces::Piece, rank::Rank, side::Side, square::Square,
};

/// Calculate the en passant bitboard for the current position.
/// This will return a bitboard with the en passant square set if it is a valid move.
///
/// # Arguments
/// - from - The square the pawn is moving from
/// - board - The current board state
/// - push_mask - The push mask for the king. See [`calculate_check_and_pin_metadata`] for more.
/// - checkers - The squares that are attacking the king. See [`calculate_checkers`] for more.
///
/// # Returns
/// A [`Bitboard`] with the en passant square set if it is a valid move, otherwise an empty bitboard.
fn calculate_en_passant_bitboard(
    from: u8,
    board: &Board,
    push_mask: Bitboard,
    checkers: Bitboard,
) -> Bitboard {
    let en_passant_sq = board.en_passant_square();

    match en_passant_sq {
        Some(sq) => {
            let en_passant_bb = Bitboard::from_square(sq);

            let mut occupancy = board.all_pieces();
            occupancy &= !(Bitboard::from_square(from));
            let captured_sq = match board.side_to_move() {
                Side::White => sq - SOUTH as u8,
                Side::Black => sq + NORTH as u8,
            };
            occupancy &= !(Bitboard::from_square(captured_sq));
            let mut discovered_checkers = move_generation::calculate_checkers(board, occupancy);
            let king_sq = board.king_square(board.side_to_move());
            let (_, king_rank) = square::from_square(king_sq);
            discovered_checkers &= RANK_BITBOARDS[king_rank as usize];

            let is_discovered_check = discovered_checkers.number_of_occupied_squares() > 0
                && checkers.number_of_occupied_squares() == 0;
            let ep_is_blocker = en_passant_bb.intersects(push_mask) && !is_discovered_check;

            if !is_discovered_check || ep_is_blocker {
                en_passant_bb
            } else {
                Bitboard::default()
            }
        }
        None => Bitboard::default(),
    }
}

/// Generate the legal pawn moves from the given square with the given board state.
///
/// # Arguments
///
/// - board - The current board state
/// - square - The square to generate moves for
/// - pinned_pieces - The pinned pieces on the board
/// - capture_mask - The mask of squares that can be captured. Will be all squares if king is not in check.
/// - push_mask - The mask of squares that can be pushed to. Will be all squares if king is not in check.
/// - orthogonal_pin_rays - The rays of orthogonal pins
/// - diagonal_pin_rays - The rays of diagonal pins
/// - checkers - The squares that are attacking the king
///
/// # Returns
/// A [`Bitboard`] with the legal moves for the pawn.
///
/// These moves need to be enumerated to get the actual moves. See [`move_generation::enumerate_moves`]
#[allow(clippy::too_many_arguments)]
fn generate_legal_pawn_mobility(
    board: &Board,
    square: Square,
    pinned_pieces: Bitboard,
    capture_mask: Bitboard,
    push_mask: Bitboard,
    orthogonal_pin_rays: Bitboard,
    diagonal_pin_rays: Bitboard,
    checkers: Bitboard,
) -> Bitboard {
    let is_pinned = pinned_pieces.intersects(Bitboard::from_square(square.to_square_index()));
    let us = board.side_to_move();
    let their_pieces = board.pieces(us.opposite());
    let direction = match us {
        Side::White => NORTH as u8,
        Side::Black => SOUTH as u8,
    };
    let from_square = square.to_square_index();
    let to_square = match us {
        Side::White => {
            let (result, did_overflow) = from_square.overflowing_add(direction);
            match did_overflow {
                true => None,
                false => Some(result),
            }
        }
        Side::Black => {
            let (result, did_overflow) = from_square.overflowing_sub(direction);
            match did_overflow {
                true => None,
                false => Some(result),
            }
        }
    };

    let mut pushes: Bitboard = match to_square {
        Some(to) => Bitboard::from_square(to),
        None => Bitboard::default(),
    };

    let occupancy = board.all_pieces();
    let is_unobstructed = pushes & !occupancy == Bitboard::default();

    let can_double_push = match us {
        Side::White => square::is_square_on_rank(from_square, Rank::R2 as u8),
        Side::Black => square::is_square_on_rank(from_square, Rank::R7 as u8),
    };

    if can_double_push && !is_unobstructed {
        let double_push_sq = match us {
            Side::White => {
                let (result, did_overflow) = from_square.overflowing_add(2 * NORTH as u8);
                match did_overflow {
                    true => None,
                    false => Some(result),
                }
            }
            Side::Black => {
                let (result, did_overflow) = from_square.overflowing_sub(2 * SOUTH as u8);
                match did_overflow {
                    true => None,
                    false => Some(result),
                }
            }
        };

        if let Some(to) = double_push_sq {
            let bb = Bitboard::from_square(to);
            pushes |= bb;
        }
    }

    let en_passant_bb: Bitboard =
        calculate_en_passant_bitboard(from_square, board, push_mask, checkers);

    let hv_pin_ray_mask = if is_pinned {
        orthogonal_pin_rays
    } else {
        Bitboard::FULL
    };

    let diag_pin_ray_mask = if is_pinned {
        diagonal_pin_rays
    } else {
        Bitboard::FULL
    };

    let legal_pushes = (pushes & !occupancy) & hv_pin_ray_mask;
    let attacks = attacks::pawn(square.to_square_index(), us)
        & (their_pieces | en_passant_bb)
        & diag_pin_ray_mask;

    (legal_pushes | attacks) & (capture_mask | push_mask)
}

/// Generate the legal moves for a normal piece (not a pawn or king) from the given square.
/// This function will take into account pinned pieces and generate the legal moves for the piece.
///
/// # Arguments
///
/// - piece - The piece to generate moves for
/// - square - The square to generate moves for
/// - board - The current board state
/// - capture_mask - The mask of squares that can be captured. Will be all squares if king is not in check.
/// - pinned_mask - The mask of squares that are pinned
/// - push_mask - The mask of squares that can be pushed to. Will be all squares if king is not in check.
/// - orthogonal_pin_rays - The rays of orthogonal pins
/// - diagonal_pin_rays - The rays of diagonal pins
///
/// # Returns
///
/// A [`Bitboard`] with the legal moves for the piece.
///
/// These moves need to be enumerated to get the actual moves. See [`move_generation::enumerate_moves`]
#[allow(clippy::too_many_arguments)]
fn generate_normal_piece_legal_mobility(
    piece: Piece,
    square: Square,
    board: &Board,
    capture_mask: Bitboard,
    pinned_mask: Bitboard,
    push_mask: Bitboard,
    orthogonal_pin_rays: Bitboard,
    diagonal_pin_rays: Bitboard,
) -> Bitboard {
    let is_pinned = pinned_mask.intersects(Bitboard::from_square(square.to_square_index()));
    let us = board.side_to_move();
    let their_pieces = board.pieces(us.opposite());
    let from_square = square.to_square_index();
    let occupancy = board.all_pieces();
    let pin_rays = orthogonal_pin_rays | diagonal_pin_rays;

    assert!(!piece.is_king() && !piece.is_pawn());

    let piece_attacks = attacks::for_piece_on_square(piece, from_square, occupancy, us);

    let our_pieces = board.pieces(us);
    let empty = !(their_pieces | our_pieces);

    let pin_ray_mask = if is_pinned {
        let king_sq = board.king_square(us);

        let pinners = their_pieces & pin_rays;
        let piece_bb = Bitboard::from_square(square.to_square_index());
        let mut true_ray_mask = Bitboard::default();

        for pinner_sq in pinners.iter() {
            let ray = rays::between(pinner_sq, king_sq);

            if ray.intersects(piece_bb) {
                true_ray_mask |= ray | Bitboard::from_square(pinner_sq);
            }
        }

        true_ray_mask
    } else {
        Bitboard::FULL
    };

    ((piece_attacks & capture_mask & their_pieces) | (piece_attacks & empty & push_mask))
        & pin_ray_mask
}

/// Generate legal moves for the king
///
/// # Arguments
///
/// - `square` - The square index of the king
/// - `board` - The board state
/// - `capture_mask` - The mask of squares that can be captured
/// - `checkers` - The mask of squares that are checking the king
///
/// # Returns
///
/// A [`Bitboard`] of legal moves for the king
fn generate_king_legal_mobility(
    square: Square,
    board: &Board,
    capture_mask: Bitboard,
    checkers: Bitboard,
) -> Bitboard {
    let us = board.side_to_move();
    let them = us.opposite();
    let our_pieces = board.pieces(us);
    let their_pieces = board.pieces(them);
    let occupancy = our_pieces | their_pieces;

    let king_bb = board.piece_bitboard(Piece::King, us);

    let king_moves_bb = attacks::king(square.to_square_index());

    let attacked_squares_occupancy = occupancy & !*king_bb;
    let attacked_squares =
        move_generation::get_attacked_squares(board, them, attacked_squares_occupancy);
    let king_pushes = king_moves_bb & !attacked_squares & !our_pieces & !their_pieces;

    let castling_moves = move_generation::castling::legal_mobility(board, checkers);

    let king_non_checker_attacks = (king_moves_bb & their_pieces & !checkers) & !attacked_squares;

    let mut king_attacks = (king_moves_bb & capture_mask & their_pieces & !attacked_squares)
        | king_non_checker_attacks;

    let k_att = king_attacks;
    for capture_sq in k_att.iter() {
        let modified_occupancy = occupancy & !Bitboard::from_square(capture_sq) & !*king_bb;
        let is_invalid_capture =
            !attacks::all_attackers_of(capture_sq, board, them, modified_occupancy).is_empty();
        if is_invalid_capture {
            king_attacks &= !Bitboard::from_square(capture_sq);
        }
    }

    king_pushes | king_attacks | castling_moves
}

/// Generate legal moves for the given piece. This is a delegating function
/// that calls the appropriate function to generate legal moves for the piece.
///
/// # Arguments
///
/// - `piece` - The piece to generate legal moves for
/// - `square` - The square index of the piece
/// - `board` - The board state
/// - `pinned_mask` - The mask of pinned pieces
/// - `capture_mask` - The mask of squares that can be captured
/// - `push_mask` - The mask of squares that can be pushed to
/// - `orthogonal_pin_rays` - The mask of orthogonal pin rays
/// - `diagonal_pin_rays` - The mask of diagonal pin rays
/// - `checkers` - The mask of squares that are checking the king
///
/// # Returns
///
/// A [`Bitboard`] of legal moves for the piece that can them be enumerated.
#[allow(clippy::too_many_arguments)]
fn generate_legal_mobility(
    piece: Piece,
    square: Square,
    board: &Board,
    pinned_mask: Bitboard,
    capture_mask: Bitboard,
    push_mask: Bitboard,
    orthogonal_pin_rays: Bitboard,
    diagonal_pin_rays: Bitboard,
    checkers: Bitboard,
) -> Bitboard {
    match piece {
        Piece::Pawn => generate_legal_pawn_mobility(
            board,
            square,
            pinned_mask,
            capture_mask,
            push_mask,
            orthogonal_pin_rays,
            diagonal_pin_rays,
            checkers,
        ),
        Piece::King => generate_king_legal_mobility(square, board, capture_mask, checkers),
        _ => generate_normal_piece_legal_mobility(
            piece,
            square,
            board,
            capture_mask,
            pinned_mask,
            push_mask,
            orthogonal_pin_rays,
            diagonal_pin_rays,
        ),
    }
}

/// Generate all legal moves for the current [`Board`] state.
///
/// # Arguments
///
/// - `board` - The current board state
/// - `move_list` - The list of moves to append to
///
/// # Examples
///
/// ```
/// use chess::board::Board;
/// use chess::move_list::MoveList;
/// use chess::move_generation;
///
/// let board = Board::default_board();
/// let mut move_list = MoveList::new();
/// move_generation::generate_legal_moves(&board, &mut move_list);
/// assert_eq!(20, move_list.len())
/// ```
pub fn generate_legal_moves(board: &Board, move_list: &mut MoveList) {
    let us = board.side_to_move();
    let our_pieces = board.pieces(us);

    let king_bb = board.piece_bitboard(Piece::King, us);
    let king_square = board.king_square(us);

    let (checkers, capture_mask, push_mask, pinned, orthogonal_pin_rays, diagonal_pin_rays) =
        move_generation::calculate_check_and_pin_metadata(board);

    let king_sq = Square::from_square_index(king_square);
    let king_moves = generate_king_legal_mobility(king_sq, board, capture_mask, checkers);

    move_generation::enumerate::enumerate_moves(
        &king_moves,
        &king_sq,
        Piece::King,
        board,
        move_list,
    );

    let num_checkers = checkers.as_number().count_ones();
    if num_checkers > 1 {
        return;
    }

    let moveable_pieces = our_pieces & !(*king_bb);
    for from_sq in moveable_pieces.iter() {
        let piece = match board.piece_on_square(from_sq) {
            Some((piece, _)) => piece,
            None => continue,
        };

        let from_square = Square::from_square_index(from_sq);
        let moves = generate_legal_mobility(
            piece,
            from_square,
            board,
            pinned,
            capture_mask,
            push_mask,
            orthogonal_pin_rays,
            diagonal_pin_rays,
            checkers,
        );

        move_generation::enumerate::enumerate_moves(&moves, &from_square, piece, board, move_list);
    }
}

#[cfg(test)]
mod tests {
    use crate::definitions::Squares;

    use super::*;

    #[test]
    fn en_passant_capture_causes_discovered_check() {
        let board = Board::from_fen("8/8/8/8/k2Pp2Q/8/8/3K4 b - d3 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_legal_moves(&board, &mut move_list);

        for mv in move_list.iter() {
            println!("{mv}");
        }

        assert_eq!(move_list.len(), 6);
    }

    #[test]
    fn king_cannot_move_away_from_slider() {
        let board = Board::from_fen("4k3/8/8/8/4R3/8/8/4K3 b - - 0 1").unwrap();

        let mut move_list = MoveList::new();
        generate_legal_moves(&board, &mut move_list);
        assert_eq!(move_list.len(), 4);
    }

    #[test]
    fn king_cannot_slide_away_from_bishop() {
        let board = Board::from_fen("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2").unwrap();

        let mut move_list = MoveList::new();
        generate_legal_moves(&board, &mut move_list);
        assert_eq!(move_list.len(), 8);
    }

    #[test]
    fn evade_check_with_en_passant_capture() {
        let board = Board::from_fen("8/8/8/2k5/3Pp3/8/8/4K3 b - d3 0 1").unwrap();
        let mut move_list = MoveList::new();
        generate_legal_moves(&board, &mut move_list);

        for mv in move_list.iter() {
            println!("{mv}");
        }

        assert_eq!(move_list.len(), 9);
    }

    #[test]
    fn rays_between_verification() {
        let ray = rays::between(Squares::A1, Squares::H8);

        let expected = Bitboard::from_square(Squares::B2)
            | Bitboard::from_square(Squares::C3)
            | Bitboard::from_square(Squares::D4)
            | Bitboard::from_square(Squares::E5)
            | Bitboard::from_square(Squares::F6)
            | Bitboard::from_square(Squares::G7);
        println!("{ray}");
        assert_eq!(ray, expected);

        let ray = rays::between(Squares::H1, Squares::A8);

        let expected = Bitboard::from_square(Squares::G2)
            | Bitboard::from_square(Squares::F3)
            | Bitboard::from_square(Squares::E4)
            | Bitboard::from_square(Squares::D5)
            | Bitboard::from_square(Squares::C6)
            | Bitboard::from_square(Squares::B7);
        assert_eq!(ray, expected);

        let ray = rays::between(Squares::A1, Squares::C2);
        assert!(ray == Bitboard::default());
    }
}
