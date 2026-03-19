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
pub(crate) fn generate_legal_pawn_mobility(
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
pub(crate) fn generate_normal_piece_legal_mobility(
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
pub(crate) fn generate_king_legal_mobility(
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
pub(crate) fn generate_legal_mobility(
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
/// Generate legal tactical moves: captures, en passant, and queen promotions.
///
/// Capture-promotions generate all 4 promotion types (they are captures first).
/// Non-capture promotions generate only the queen promotion variant.
///
/// Must be called with metadata from [`move_generation::metadata::compute`].
pub fn generate_legal_tacticals(
    board: &Board,
    meta: &move_generation::metadata::CheckPinMetadata,
    move_list: &mut MoveList,
) {
    let us = board.side_to_move();
    let their_pieces = board.pieces(us.opposite());
    let king_bb = board.piece_bitboard(Piece::King, us);
    let king_square = board.king_square(us);
    let occupancy = board.all_pieces();
    let en_passant_bb = board
        .en_passant_square()
        .map(Bitboard::from)
        .unwrap_or_default();
    let promotion_rank_bb = Rank::promotion_rank(us).to_bitboard();

    // King captures (castling excluded naturally since castling squares are empty)
    let king_sq = Square::from_square_index(king_square);
    let king_mobility =
        generate_king_legal_mobility(king_sq, board, meta.capture_mask, meta.checkers);
    let king_captures = king_mobility & their_pieces;
    move_generation::enumerate::enumerate_moves(
        &king_captures,
        &king_sq,
        Piece::King,
        board,
        move_list,
        move_generation::enumerate::PromotionFilter::All,
    );

    // Double check: only king moves are legal
    if meta.num_checkers() > 1 {
        return;
    }

    let our_pieces = board.pieces(us);
    let moveable_pieces = our_pieces & !(*king_bb);

    for from_sq in moveable_pieces.iter() {
        let (piece, _) = match board.piece_on_square(from_sq) {
            Some(p) => p,
            None => continue,
        };

        let from_square = Square::from_square_index(from_sq);
        let mobility = generate_legal_mobility(
            piece,
            from_square,
            board,
            meta.pinned,
            meta.capture_mask,
            meta.push_mask,
            meta.orthogonal_pin_rays,
            meta.diagonal_pin_rays,
            meta.checkers,
        );

        if piece == Piece::Pawn {
            // Pawn captures (including en passant and capture-promotions with all 4 types)
            let captures = mobility & (their_pieces | en_passant_bb);
            move_generation::enumerate::enumerate_moves(
                &captures,
                &from_square,
                piece,
                board,
                move_list,
                move_generation::enumerate::PromotionFilter::All,
            );

            // Non-capture queen promotions only (pushes to promotion rank)
            let queen_promo_pushes = mobility & !occupancy & promotion_rank_bb;
            move_generation::enumerate::enumerate_moves(
                &queen_promo_pushes,
                &from_square,
                piece,
                board,
                move_list,
                move_generation::enumerate::PromotionFilter::QueenOnly,
            );
        } else {
            // Non-pawn piece captures
            let captures = mobility & their_pieces;
            move_generation::enumerate::enumerate_moves(
                &captures,
                &from_square,
                piece,
                board,
                move_list,
                move_generation::enumerate::PromotionFilter::All,
            );
        }
    }
}

/// Generate legal quiet moves: non-captures, castling, and underpromotions.
///
/// Non-capture promotion pushes generate only underpromotion types (Rook, Bishop, Knight).
/// Queen promotions are generated by [`generate_legal_tacticals`] instead.
///
/// Must be called with metadata from [`move_generation::metadata::compute`].
pub fn generate_legal_quiets(
    board: &Board,
    meta: &move_generation::metadata::CheckPinMetadata,
    move_list: &mut MoveList,
) {
    let us = board.side_to_move();
    let their_pieces = board.pieces(us.opposite());
    let king_bb = board.piece_bitboard(Piece::King, us);
    let king_square = board.king_square(us);
    let occupancy = board.all_pieces();
    let promotion_rank_bb = Rank::promotion_rank(us).to_bitboard();

    // King quiet moves (non-captures + castling)
    let king_sq = Square::from_square_index(king_square);
    let king_mobility =
        generate_king_legal_mobility(king_sq, board, meta.capture_mask, meta.checkers);
    let king_quiets = king_mobility & !their_pieces;
    move_generation::enumerate::enumerate_moves(
        &king_quiets,
        &king_sq,
        Piece::King,
        board,
        move_list,
        move_generation::enumerate::PromotionFilter::All,
    );

    // Double check: only king moves are legal
    if meta.num_checkers() > 1 {
        return;
    }

    let our_pieces = board.pieces(us);
    let moveable_pieces = our_pieces & !(*king_bb);

    for from_sq in moveable_pieces.iter() {
        let (piece, _) = match board.piece_on_square(from_sq) {
            Some(p) => p,
            None => continue,
        };

        let from_square = Square::from_square_index(from_sq);
        let mobility = generate_legal_mobility(
            piece,
            from_square,
            board,
            meta.pinned,
            meta.capture_mask,
            meta.push_mask,
            meta.orthogonal_pin_rays,
            meta.diagonal_pin_rays,
            meta.checkers,
        );

        if piece == Piece::Pawn {
            // Non-capture, non-promotion pawn pushes
            // Exclude en passant square — it's empty but captured as a tactical
            let en_passant_bb = board
                .en_passant_square()
                .map(Bitboard::from)
                .unwrap_or_default();
            let quiet_pushes = mobility & !occupancy & !promotion_rank_bb & !en_passant_bb;
            move_generation::enumerate::enumerate_moves(
                &quiet_pushes,
                &from_square,
                piece,
                board,
                move_list,
                move_generation::enumerate::PromotionFilter::All,
            );

            // Non-capture underpromotions (pushes to promotion rank, R/B/N only)
            let underpromo_pushes = mobility & !occupancy & promotion_rank_bb;
            move_generation::enumerate::enumerate_moves(
                &underpromo_pushes,
                &from_square,
                piece,
                board,
                move_list,
                move_generation::enumerate::PromotionFilter::UnderOnly,
            );
        } else {
            // Non-pawn quiet moves (mobility excluding enemy pieces)
            let quiets = mobility & !their_pieces;
            move_generation::enumerate::enumerate_moves(
                &quiets,
                &from_square,
                piece,
                board,
                move_list,
                move_generation::enumerate::PromotionFilter::All,
            );
        }
    }
}

/// Generate all legal moves for the current [`Board`] state.
///
/// This is a convenience wrapper that generates tacticals followed by quiets.
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
    let meta = move_generation::metadata::compute(board);
    generate_legal_tacticals(board, &meta, move_list);
    generate_legal_quiets(board, &meta, move_list);
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

    #[test]
    fn staged_generation_equals_full_generation() {
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            "r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2",
            "8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3",
            "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1",
            "r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQkq - 3 2",
        ];

        for fen in &positions {
            let board = Board::from_fen(fen).unwrap();
            let meta = move_generation::metadata::compute(&board);

            let mut all_moves = MoveList::new();
            generate_legal_moves(&board, &mut all_moves);

            let mut staged_moves = MoveList::new();
            generate_legal_tacticals(&board, &meta, &mut staged_moves);
            generate_legal_quiets(&board, &meta, &mut staged_moves);

            assert_eq!(
                all_moves.len(),
                staged_moves.len(),
                "Move count mismatch for position: {fen}\nAll: {}\nStaged: {}",
                all_moves.len(),
                staged_moves.len()
            );

            for mv in all_moves.iter() {
                assert!(
                    staged_moves.iter().any(|sm| *sm == *mv),
                    "Move {} from full gen not found in staged gen for position: {fen}",
                    mv.to_long_algebraic()
                );
            }
        }
    }

    #[test]
    fn tacticals_are_captures_and_queen_promotions() {
        let board =
            Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
        let meta = move_generation::metadata::compute(&board);

        let mut tacticals = MoveList::new();
        generate_legal_tacticals(&board, &meta, &mut tacticals);

        for mv in tacticals.iter() {
            let is_capture = mv.is_capture();
            let is_queen_promo = mv.promotion_piece() == Some(Piece::Queen);
            assert!(
                is_capture || is_queen_promo,
                "Non-tactical move {} found in tacticals",
                mv.to_long_algebraic()
            );
        }
    }

    #[test]
    fn quiets_are_non_captures_and_underpromotions() {
        let board =
            Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
        let meta = move_generation::metadata::compute(&board);

        let mut quiets = MoveList::new();
        generate_legal_quiets(&board, &meta, &mut quiets);

        for mv in quiets.iter() {
            let is_capture = mv.is_capture();
            let is_queen_promo = mv.promotion_piece() == Some(Piece::Queen);
            assert!(
                !is_capture && !is_queen_promo,
                "Tactical move {} found in quiets (capture={}, queen_promo={})",
                mv.to_long_algebraic(),
                is_capture,
                is_queen_promo,
            );
        }
    }

    #[test]
    fn en_passant_is_tactical() {
        let board = Board::from_fen("8/8/8/2k5/3Pp3/8/8/4K3 b - d3 0 1").unwrap();
        let meta = move_generation::metadata::compute(&board);

        let mut tacticals = MoveList::new();
        generate_legal_tacticals(&board, &meta, &mut tacticals);

        let has_ep = tacticals.iter().any(|mv| mv.is_en_passant_capture());
        assert!(has_ep, "En passant capture should be in tacticals");
    }

    #[test]
    fn castling_is_quiet() {
        let board = Board::from_fen("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1").unwrap();
        let meta = move_generation::metadata::compute(&board);

        let mut quiets = MoveList::new();
        generate_legal_quiets(&board, &meta, &mut quiets);

        let has_castle = quiets.iter().any(|mv| mv.is_castle());
        assert!(has_castle, "Castling should be in quiets");

        let mut tacticals = MoveList::new();
        generate_legal_tacticals(&board, &meta, &mut tacticals);

        let castle_in_tacticals = tacticals.iter().any(|mv| mv.is_castle());
        assert!(!castle_in_tacticals, "Castling should NOT be in tacticals");
    }
}
