// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::attacks;
use crate::definitions::RANK_BITBOARDS;
use crate::move_generation;
use crate::move_generation::NORTH;
use crate::move_generation::SOUTH;
use crate::move_generation::enumerate::enumerate_moves;
use crate::move_generation::metadata::CheckPinMetadata;
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

    let attacked_squares_occupancy = occupancy & !king_bb;
    let attacked_squares =
        move_generation::get_attacked_squares(board, them, attacked_squares_occupancy);
    let king_pushes = king_moves_bb & !attacked_squares & !our_pieces & !their_pieces;

    let castling_moves = move_generation::castling::legal_mobility(board, checkers);

    let king_non_checker_attacks = (king_moves_bb & their_pieces & !checkers) & !attacked_squares;

    let mut king_attacks = (king_moves_bb & capture_mask & their_pieces & !attacked_squares)
        | king_non_checker_attacks;

    let k_att = king_attacks;
    for capture_sq in k_att.iter() {
        let modified_occupancy = occupancy & !Bitboard::from_square(capture_sq) & !king_bb;
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
    metadata: &CheckPinMetadata,
) -> Bitboard {
    match piece {
        Piece::Pawn => generate_legal_pawn_mobility(
            board,
            square,
            metadata.pinned,
            metadata.capture_mask,
            metadata.push_mask,
            metadata.orthogonal_pin_rays,
            metadata.diagonal_pin_rays,
            metadata.checkers,
        ),
        Piece::King => {
            generate_king_legal_mobility(square, board, metadata.capture_mask, metadata.checkers)
        }
        _ => generate_normal_piece_legal_mobility(
            piece,
            square,
            board,
            metadata.capture_mask,
            metadata.pinned,
            metadata.push_mask,
            metadata.orthogonal_pin_rays,
            metadata.diagonal_pin_rays,
        ),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MoveFilter {
    All,
    Tacticals,
    Captures,
    Quiets,
}

/// Generate legal moves for the current [`Board`] state.
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
/// use chess::move_generation::legal::MoveFilter;
///
/// let board = Board::default_board();
/// let move_list = move_generation::legal::generate_moves(&board, MoveFilter::All);
/// assert_eq!(20, move_list.len())
/// ```
pub fn generate_moves(board: &Board, move_filter: MoveFilter) -> MoveList {
    let us = board.side_to_move();
    let them = us.opposite();
    let our_pieces = board.pieces(us);
    let their_pieces = board.pieces(them);
    let filter = match move_filter {
        MoveFilter::All => Bitboard::FULL,
        MoveFilter::Captures | MoveFilter::Tacticals => their_pieces,
        MoveFilter::Quiets => !their_pieces,
    };
    let ep_bb = match board.en_passant_square() {
        Some(sq) => Bitboard::from_square(sq),
        None => Bitboard::default(),
    };
    let pawn_filter = match move_filter {
        MoveFilter::All => Bitboard::FULL,
        MoveFilter::Captures => their_pieces | ep_bb,
        MoveFilter::Tacticals => their_pieces | Rank::promotion_rank(us).to_bitboard() | ep_bb,
        MoveFilter::Quiets => !(their_pieces | ep_bb),
    };

    let king_sq_idx = board.king_square(us);
    let king_sq = Square::from_square_index(king_sq_idx);
    let king_bb = Bitboard::from_square(king_sq_idx);

    let mut move_list = MoveList::new();
    let meta = move_generation::metadata::compute(board);

    // King moves first
    let king_moves = generate_king_legal_mobility(
        Square::from_square_index(king_sq_idx),
        board,
        meta.capture_mask,
        meta.checkers,
    ) & filter;

    enumerate_moves(
        &king_moves,
        king_sq,
        Piece::King,
        board,
        move_filter,
        &mut move_list,
    );

    // Return early if in double check since only king moves are legal
    if meta.num_checkers() > 1 {
        return move_list;
    }

    // Proceed with non-king pieces
    let moveable_pieces = our_pieces & !king_bb;

    for from_sq_idx in moveable_pieces.iter() {
        let piece = match board.piece_on_square(from_sq_idx) {
            Some((piece, _)) => piece,
            None => continue,
        };

        let from_sq = Square::from_square_index(from_sq_idx);
        let use_filter = match piece {
            Piece::Pawn => pawn_filter,
            _ => filter,
        };
        let moves = generate_legal_mobility(piece, from_sq, board, &meta) & use_filter;

        enumerate_moves(&moves, from_sq, piece, board, move_filter, &mut move_list);
    }

    move_list
}

pub fn generate_all_moves(board: &Board) -> MoveList {
    generate_moves(board, MoveFilter::All)
}

#[cfg(test)]
mod tests {
    use crate::definitions::Squares;

    use super::*;

    fn generate_moves_for_fen(fen: &str) -> MoveList {
        let board = Board::from_fen(fen).unwrap();
        generate_moves(&board, MoveFilter::All)
    }

    #[test]
    fn en_passant_capture_causes_discovered_check() {
        let move_list = generate_moves_for_fen("8/8/8/8/k2Pp2Q/8/8/3K4 b - d3 0 1");
        for mv in move_list.iter() {
            println!("{mv}");
        }

        assert_eq!(move_list.len(), 6);
    }

    #[test]
    fn king_cannot_move_away_from_slider() {
        let move_list = generate_moves_for_fen("4k3/8/8/8/4R3/8/8/4K3 b - - 0 1");
        assert_eq!(move_list.len(), 4);
    }

    #[test]
    fn king_cannot_slide_away_from_bishop() {
        let move_list = generate_moves_for_fen("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2");
        assert_eq!(move_list.len(), 8);
    }

    #[test]
    fn evade_check_with_en_passant_capture() {
        let move_list = generate_moves_for_fen("8/8/8/2k5/3Pp3/8/8/4K3 b - d3 0 1");

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
            // push-promotion position: white pawn on a7, a8 empty
            "8/P3k3/8/8/8/8/8/4K3 w - - 0 1",
        ];

        for fen in &positions {
            let board = Board::from_fen(fen).unwrap();

            let all_moves = generate_moves(&board, MoveFilter::All);
            let captures = generate_moves(&board, MoveFilter::Captures);
            let quiets = generate_moves(&board, MoveFilter::Quiets);

            let staged_moves = captures
                .iter()
                .chain(quiets.iter())
                .cloned()
                .collect::<Vec<_>>();

            assert_eq!(
                all_moves.len(),
                staged_moves.len(),
                "Move count mismatch for position: {fen}\nAll: {}\nStaged: {}",
                all_moves.len(),
                staged_moves.len()
            );

            for mv in all_moves.iter() {
                assert!(
                    staged_moves.contains(mv),
                    "Move {} from full gen not found in staged gen for position: {fen}",
                    mv.to_long_algebraic()
                );
            }
        }
    }

    /// Validates that every move in `Tacticals` is either a capture or a queen promotion.
    fn validate_tacticals_for_board(board: &Board) {
        let tacticals = move_generation::legal::generate_moves(board, MoveFilter::Tacticals);
        for mv in tacticals.iter() {
            assert!(
                board.captured(mv).is_some()
                    || (mv.is_promotion()
                        && mv.promotion_piece().is_some_and(|pc| pc == Piece::Queen)),
                "Tactical move {} is neither a capture nor a queen promotion",
                mv.to_long_algebraic()
            );
        }
    }

    #[test]
    fn tactical_movegen_includes_queen_promos() {
        // Positions with non-capture queen promotions available for both sides
        let fens = &[
            "6n1/4PP2/8/6k1/K7/4p3/2p5/8 w - - 0 1",
            "6n1/4PP2/8/6k1/K7/4p3/2p5/8 b - - 0 1",
        ];
        for fen in fens {
            let board = Board::from_fen(fen).unwrap();
            let tacticals = generate_moves(&board, MoveFilter::Tacticals);

            // Tacticals should contain queen push-promotions (non-capture promos)
            let queen_push_promos = tacticals
                .iter()
                .filter(|mv| {
                    mv.is_promotion()
                        && board.captured(mv).is_none()
                        && mv.promotion_piece().is_some_and(|pc| pc == Piece::Queen)
                })
                .count();
            assert!(
                queen_push_promos > 0,
                "Position {fen}: tacticals should include queen push-promotions"
            );
            validate_tacticals_for_board(&board);
        }
    }

    #[test]
    fn en_passant_is_in_captures() {
        // En passant available: black pawn on c4, white double-pushed d2-d4
        let board = Board::from_fen("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3").unwrap();

        let captures = generate_moves(&board, MoveFilter::Captures);
        let ep_move = captures.iter().find(|mv| mv.is_en_passant_capture());
        assert!(
            ep_move.is_some(),
            "En passant capture should be included in Captures"
        );

        let tacticals = generate_moves(&board, MoveFilter::Tacticals);
        let ep_move = tacticals.iter().find(|mv| mv.is_en_passant_capture());
        assert!(
            ep_move.is_some(),
            "En passant capture should be included in Tacticals"
        );

        let quiets = generate_moves(&board, MoveFilter::Quiets);
        let ep_move = quiets.iter().find(|mv| mv.is_en_passant_capture());
        assert!(
            ep_move.is_none(),
            "En passant capture should not be in Quiets"
        );
    }

    #[test]
    fn capture_promotions_are_correct() {
        // White pawn on d7, black rook on c8, d8 empty
        // push-promo: d7-d8 (4 types), capture-promo: d7xc8 (4 types)
        let board =
            Board::from_fen("2r5/3P4/8/8/8/8/6k1/4K3 w - - 0 1").unwrap();

        let all = generate_moves(&board, MoveFilter::All);
        let captures = generate_moves(&board, MoveFilter::Captures);
        let quiets = generate_moves(&board, MoveFilter::Quiets);
        let tacticals = generate_moves(&board, MoveFilter::Tacticals);

        // push-promo to d8: 4 promo types in All, 4 in Quiets, 0 in Captures
        let push_promos = |list: &MoveList| {
            list.iter()
                .filter(|mv| mv.is_promotion() && board.captured(mv).is_none())
                .count()
        };
        assert_eq!(push_promos(&all), 4, "All should have 4 push-promos");
        assert_eq!(push_promos(&quiets), 4, "Quiets should have 4 push-promos");
        assert_eq!(push_promos(&captures), 0, "Captures should have 0 push-promos");

        // capture-promo to c8: 4 promo types in All, 4 in Captures, 0 in Quiets
        let cap_promos = |list: &MoveList| {
            list.iter()
                .filter(|mv| mv.is_promotion() && board.captured(mv).is_some())
                .count()
        };
        assert_eq!(cap_promos(&all), 4, "All should have 4 capture-promos");
        assert_eq!(cap_promos(&captures), 4, "Captures should have 4 capture-promos");
        assert_eq!(cap_promos(&quiets), 0, "Quiets should have 0 capture-promos");

        // Tacticals: only queen promos (1 push-promo + 1 capture-promo)
        let tacticals_queen_promos = tacticals
            .iter()
            .filter(|mv| {
                mv.is_promotion()
                    && mv.promotion_piece().is_some_and(|pc| pc == Piece::Queen)
            })
            .count();
        assert_eq!(
            tacticals_queen_promos, 2,
            "Tacticals should have exactly 2 queen promos (1 push + 1 capture)"
        );

        validate_tacticals_for_board(&board);
    }

    #[test]
    fn tacticals_in_check_position() {
        // White king in check from black rook, must evade
        let board =
            Board::from_fen("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1").unwrap();

        let tacticals = generate_moves(&board, MoveFilter::Tacticals);
        let all = generate_moves(&board, MoveFilter::All);

        // Every tactical must be a capture or queen promo
        validate_tacticals_for_board(&board);
        // Tacticals should be a subset of all moves
        for mv in tacticals.iter() {
            assert!(
                all.iter().any(|a| a == mv),
                "Tactical {} not in all moves",
                mv.to_long_algebraic()
            );
        }
    }

    #[test]
    fn tacticals_in_double_check() {
        // Double check: bishop on b5 and rook on e1 both attack black king on e8
        let board = Board::from_fen(
            "4k3/8/8/1B6/8/8/8/4R1K1 b - - 0 1",
        )
        .unwrap();

        let all = generate_moves(&board, MoveFilter::All);
        let tacticals = generate_moves(&board, MoveFilter::Tacticals);

        // In double check, only king moves are legal
        for mv in all.iter() {
            let (piece, _) = board.piece_on_square(mv.from()).unwrap();
            assert_eq!(piece, Piece::King, "Only king moves should be legal in double check");
        }

        validate_tacticals_for_board(&board);
        // Tacticals should only include king captures
        for mv in tacticals.iter() {
            let (piece, _) = board.piece_on_square(mv.from()).unwrap();
            assert_eq!(piece, Piece::King);
        }
    }
}
