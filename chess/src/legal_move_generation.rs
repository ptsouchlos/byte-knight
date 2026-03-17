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
    bitboard::Bitboard, bitboard_helpers, board::Board, definitions::Squares, pieces::Piece,
    rank::Rank, side::Side, square::Square,
};

/// Calculates checkers, pinned pieces, capture mask, push mask and pin rays for the current position.
///
/// # Arguments
///
/// - board - The current board state
///
/// # Returns
///
/// A tuple containing:
/// - A [`Bitboard`] representing the squares that are checking the king
/// - A [`Bitboard`] representing the squares can be attacked
/// - A [`Bitboard`] representing the squares that can be pushed to
/// - A [`Bitboard`] representing the squares that are pinned
/// - A [`Bitboard`] representing the orthogonal pin rays
/// - A [`Bitboard`] representing the diagonal pin rays
///
fn calculate_check_and_pin_metadata(
    board: &Board,
) -> (Bitboard, Bitboard, Bitboard, Bitboard, Bitboard, Bitboard) {
    let us = board.side_to_move();
    let them = us.opposite();
    let occupancy = board.all_pieces();
    let empty = !occupancy;
    let their_pieces = board.pieces(them);
    let our_pieces = board.pieces(us);
    let enemy_or_empty = their_pieces | empty;
    let king_sq = board.king_square(us);

    let mut pinned = Bitboard::default();
    let mut capture_mask = enemy_or_empty & !(*board.piece_bitboard(Piece::King, them));
    let mut orthogonal_pin_rays = Bitboard::default();
    let mut diagonal_pin_rays = Bitboard::default();

    // Super-piece method: project attacks from king square with opposite side semantics
    let mut checkers = *board.piece_bitboard(Piece::Knight, them) & attacks::knight(king_sq)
        | *board.piece_bitboard(Piece::Pawn, them) & attacks::pawn(king_sq, us);

    let enemy_sliding_attacks = attacks::rook(king_sq, Bitboard::default())
        & (*board.piece_bitboard(Piece::Rook, them) | *board.piece_bitboard(Piece::Queen, them))
        | attacks::bishop(king_sq, Bitboard::default())
            & (*board.piece_bitboard(Piece::Bishop, them)
                | *board.piece_bitboard(Piece::Queen, them));

    for next_attacker_sq in enemy_sliding_attacks.iter() {
        let attacker_bb = Bitboard::from_square(next_attacker_sq);

        let ray = rays::between(king_sq, next_attacker_sq);

        let (king_file, king_rank) = square::from_square(king_sq);
        let (attacker_file, attacker_rank) = square::from_square(next_attacker_sq);
        let is_orthogonal = king_file == attacker_file || king_rank == attacker_rank;
        let is_diagonal = (king_sq as i16 - next_attacker_sq as i16).abs() % 9 == 0
            || (king_sq as i16 - next_attacker_sq as i16).abs() % 7 == 0;

        match (ray & occupancy).number_of_occupied_squares() {
            0 => {
                checkers |= Bitboard::from_square(next_attacker_sq);
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

    let mut push_mask = Bitboard::from(u64::MAX);

    if checkers.number_of_occupied_squares() >= 1 {
        let is_single_check = checkers.number_of_occupied_squares() == 1;

        capture_mask = checkers & !(*board.piece_bitboard(Piece::King, them));

        if is_single_check {
            let mut checkers_clone = checkers;
            let checker = bitboard_helpers::next_bit(&mut checkers_clone) as u8;

            let ray = rays::between(king_sq, checker as u8);

            if let Some((piece, side)) = board.piece_on_square(checker as u8) {
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

    (
        checkers,
        capture_mask,
        push_mask,
        pinned,
        orthogonal_pin_rays,
        diagonal_pin_rays,
    )
}

/// Calculate 'checkers' and 'pinned' bitboard masks for the current position.
///
/// # Arguments
/// - board - The current board state
/// - occupancy - The occupancy bitboard
///
/// # Returns
///
/// A [`Bitboard`] representing the squares that are checking the king.
fn calculate_checkers(board: &Board, occupancy: Bitboard) -> Bitboard {
    let us = board.side_to_move();
    let king_bb = board.piece_bitboard(Piece::King, us);
    let king_square = board.king_square(us);
    let kingless_occupancy = occupancy & !(*king_bb);

    attacks::all_attackers_of(king_square, board, us.opposite(), kingless_occupancy)
}

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
    push_mask: &Bitboard,
    checkers: &Bitboard,
) -> Bitboard {
    let en_passant_sq = board.en_passant_square();

    match en_passant_sq {
        Some(sq) => {
            let en_passant_bb = en_passant_sq.map(Bitboard::from).unwrap_or_default();

            let mut occupancy = board.all_pieces();
            occupancy &= !(Bitboard::from_square(from));
            let captured_sq = match board.side_to_move() {
                Side::White => sq - SOUTH as u8,
                Side::Black => sq + NORTH as u8,
            };
            occupancy &= !(Bitboard::from_square(captured_sq));
            let mut discovered_checkers = calculate_checkers(board, occupancy);
            let king_sq = board.king_square(board.side_to_move());
            let (_, king_rank) = square::from_square(king_sq);
            discovered_checkers &= RANK_BITBOARDS[king_rank as usize];

            let is_discovered_check = discovered_checkers.number_of_occupied_squares() > 0
                && checkers.number_of_occupied_squares() == 0;
            let ep_is_blocker = en_passant_bb.intersects(*push_mask) && !is_discovered_check;

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
    square: &Square,
    pinned_pieces: &Bitboard,
    capture_mask: &Bitboard,
    push_mask: &Bitboard,
    orthogonal_pin_rays: &Bitboard,
    diagonal_pin_rays: &Bitboard,
    checkers: &Bitboard,
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
        *orthogonal_pin_rays
    } else {
        Bitboard::from(u64::MAX)
    };

    let diag_pin_ray_mask = if is_pinned {
        *diagonal_pin_rays
    } else {
        Bitboard::from(u64::MAX)
    };

    let legal_pushes = (pushes & !occupancy) & hv_pin_ray_mask;
    let attacks = attacks::pawn(square.to_square_index(), us)
        & (their_pieces | en_passant_bb)
        & diag_pin_ray_mask;

    (legal_pushes | attacks) & (*capture_mask | *push_mask)
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
    square: &Square,
    board: &Board,
    capture_mask: &Bitboard,
    pinned_mask: &Bitboard,
    push_mask: &Bitboard,
    orthogonal_pin_rays: &Bitboard,
    diagonal_pin_rays: &Bitboard,
) -> Bitboard {
    let is_pinned = pinned_mask.intersects(Bitboard::from_square(square.to_square_index()));
    let us = board.side_to_move();
    let their_pieces = board.pieces(us.opposite());
    let from_square = square.to_square_index();
    let occupancy = board.all_pieces();
    let pin_rays = *orthogonal_pin_rays | *diagonal_pin_rays;

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
        Bitboard::from(u64::MAX)
    };

    ((piece_attacks & *capture_mask & their_pieces) | (piece_attacks & empty & *push_mask))
        & pin_ray_mask
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
fn generate_legal_castling_mobility(
    square: &Square,
    board: &Board,
    attacked_squares: &Bitboard,
    checkers: &Bitboard,
) -> Bitboard {
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
    let occupancy = board.all_pieces();
    let mut castling_moves = Bitboard::default();
    let king_side_castle = board.can_castle_kingside(us);
    let queen_side_castle = board.can_castle_queenside(us);

    let king_sq = match us {
        Side::White => Squares::E1,
        Side::Black => Squares::E8,
    };

    let king_in_place = king_sq == square.to_square_index();
    if !king_in_place {
        return Bitboard::default();
    }

    if king_side_castle {
        let king_side_rook = match us {
            Side::White => Squares::H1,
            Side::Black => Squares::H8,
        };
        let maybe_rook = board.piece_on_square(king_side_rook);
        let rook_in_place = match maybe_rook {
            Some((Piece::Rook, side)) => side == us,
            _ => false,
        };

        let king_side_empty = match us {
            Side::White => Bitboard::from_square(Squares::F1) | Bitboard::from_square(Squares::G1),
            Side::Black => Bitboard::from_square(Squares::F8) | Bitboard::from_square(Squares::G8),
        };

        let king_side_target_sq = match us {
            Side::White => Squares::G1,
            Side::Black => Squares::G8,
        };

        let is_king_ray_empty = king_side_empty & occupancy == Bitboard::default();
        let is_king_ray_attacked = king_side_empty & *attacked_squares != Bitboard::default();
        if is_king_ray_empty && !is_king_ray_attacked && rook_in_place && king_in_place {
            castling_moves |= Bitboard::from_square(king_side_target_sq);
        }
    }

    if queen_side_castle {
        let queen_side_rook = match us {
            Side::White => Squares::A1,
            Side::Black => Squares::A8,
        };
        let maybe_rook = board.piece_on_square(queen_side_rook);
        let rook_in_place = match maybe_rook {
            Some((Piece::Rook, side)) => side == us,
            _ => false,
        };

        let queen_side_no_attack = match us {
            Side::White => Bitboard::from_square(Squares::C1) | Bitboard::from_square(Squares::D1),
            Side::Black => Bitboard::from_square(Squares::C8) | Bitboard::from_square(Squares::D8),
        };
        let queen_side_empty = match us {
            Side::White => queen_side_no_attack | Bitboard::from_square(Squares::B1),
            Side::Black => queen_side_no_attack | Bitboard::from_square(Squares::B8),
        };

        let queen_side_target_sq = match us {
            Side::White => Squares::C1,
            Side::Black => Squares::C8,
        };

        let is_king_ray_empty = queen_side_empty & occupancy == Bitboard::default();
        let is_king_ray_attacked = queen_side_no_attack & *attacked_squares != Bitboard::default();
        if is_king_ray_empty && !is_king_ray_attacked && rook_in_place && king_in_place {
            castling_moves |= Bitboard::from_square(queen_side_target_sq);
        }
    }
    castling_moves
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
    square: &Square,
    board: &Board,
    capture_mask: &Bitboard,
    checkers: &Bitboard,
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
        move_generation::get_attacked_squares(board, them, &attacked_squares_occupancy);
    let king_pushes = king_moves_bb & !attacked_squares & !our_pieces & !their_pieces;

    let castling_moves =
        generate_legal_castling_mobility(square, board, &attacked_squares, checkers);

    let king_non_checker_attacks = (king_moves_bb & their_pieces & !*checkers) & !attacked_squares;

    let mut king_attacks = (king_moves_bb & *capture_mask & their_pieces & !attacked_squares)
        | king_non_checker_attacks;

    let k_att = king_attacks;
    for capture_sq in k_att.iter() {
        let modified_occupancy = occupancy & !Bitboard::from_square(capture_sq) & !*king_bb;
        let is_invalid_capture = move_generation::is_square_attacked_with_occupancy(
            board,
            &Square::from_square_index(capture_sq),
            them,
            modified_occupancy,
        );
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
    square: &Square,
    board: &Board,
    pinned_mask: &Bitboard,
    capture_mask: &Bitboard,
    push_mask: &Bitboard,
    orthogonal_pin_rays: &Bitboard,
    diagonal_pin_rays: &Bitboard,
    checkers: &Bitboard,
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
        calculate_check_and_pin_metadata(board);

    let king_sq = Square::from_square_index(king_square);
    let king_moves = generate_king_legal_mobility(&king_sq, board, &capture_mask, &checkers);

    move_generation::enumerate_moves(&king_moves, &king_sq, Piece::King, board, move_list);

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
            &from_square,
            board,
            &pinned,
            &capture_mask,
            &push_mask,
            &orthogonal_pin_rays,
            &diagonal_pin_rays,
            &checkers,
        );

        move_generation::enumerate_moves(&moves, &from_square, piece, board, move_list);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_pinned_pieces() {
        let board =
            Board::from_fen("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2")
                .unwrap();
        let occupancy = board.all_pieces();
        let (_, _, _, pinned, _, _) = calculate_check_and_pin_metadata(&board);
        let checkers = calculate_checkers(&board, occupancy);
        assert_eq!(checkers, 0);
        assert_eq!(pinned, Bitboard::from_square(Squares::D7));
    }

    #[test]
    fn calculate_pinned_pieces_2() {
        let board = Board::from_fen("8/8/8/8/k2Pp2Q/8/8/3K4 b - d3 0 1").unwrap();
        let occupancy = board.all_pieces();
        let (_, _, _, pinned, _, _) = calculate_check_and_pin_metadata(&board);
        let checkers = calculate_checkers(&board, occupancy);
        assert_eq!(checkers, 0);
        assert_eq!(pinned, Bitboard::default());
    }

    #[test]
    fn calculate_pinned_pieces_3() {
        let board =
            Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQKR2 b Q - 2 8").unwrap();

        let occupancy = board.all_pieces();
        let (_, _, _, pinned, orthogonal_rays, diagonal_rays) =
            calculate_check_and_pin_metadata(&board);
        let pin_rays = orthogonal_rays | diagonal_rays;
        let checkers = calculate_checkers(&board, occupancy);
        assert_eq!(checkers, 0);
        assert_eq!(pinned, 0);
        assert_eq!(pin_rays, 0);
    }

    #[test]
    fn calculate_pins() {
        let board =
            Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nPB5/B1P1P3/5N2/q2P1KPP/b2Q1R2 w kq - 0 3")
                .unwrap();
        let (_, _, _, pinned_pieces, horizontal_pin_rays, diagonal_pin_rays) =
            calculate_check_and_pin_metadata(&board);

        assert_eq!(pinned_pieces.number_of_occupied_squares(), 2);
        println!("horizontal pin rays:\n{horizontal_pin_rays}");
        println!("diagonal pin rays:\n{diagonal_pin_rays}");

        assert!(pinned_pieces.intersects(Bitboard::from_square(Squares::C5)));
        assert!(pinned_pieces.intersects(Bitboard::from_square(Squares::D2)));
    }

    #[test]
    fn check_pinned_and_capture_mask() {
        let board =
            Board::from_fen("rnQq1k1r/pp2bppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R b KQ - 0 8").unwrap();
        let (checkers, capture_mask, push_mask, pinned, orthogonal_rays, diagonal_rays) =
            calculate_check_and_pin_metadata(&board);
        println!("checkers:\n{checkers}");
        println!("check mask:\n{capture_mask}");
        println!("push mask:\n{push_mask}");
        println!("pinned:\n{pinned}");
        println!("orthogonal rays:\n{orthogonal_rays}");
        println!("diagonal rays:\n{diagonal_rays}");

        assert_eq!(checkers, 0);
        assert_eq!(pinned, Bitboard::from_square(Squares::D8));
        println!("capture mask:\n{capture_mask}");
        println!("push mask:\n{push_mask}");
    }

    #[test]
    fn check_pinned_and_capture_mask_2() {
        let board = Board::from_fen("4B1r1/2q2p2/QP4k1/3P2p1/7B/8/6K1/7R b - - 3 59").unwrap();
        let (checkers, capture_mask, push_mask, pinned, orthogonal_rays, diagonal_rays) =
            calculate_check_and_pin_metadata(&board);
        println!("checkers:\n{checkers}");
        println!("check mask:\n{capture_mask}");
        println!("push mask:\n{push_mask}");
        println!("pinned:\n{pinned}");
        println!("orthogonal rays:\n{orthogonal_rays}");
        println!("diagonal rays:\n{diagonal_rays}");

        assert_eq!(checkers, 0);
        assert_eq!(pinned, Bitboard::from_square(Squares::F7));
        assert_eq!(orthogonal_rays, 0);
        assert!(diagonal_rays > 0);
    }

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
