// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::attacks;
use crate::definitions::RANK_BITBOARDS;
use crate::move_generation;
use crate::move_generation::emit_pawn_targets;
use crate::move_generation::enumerate::enumerate_moves;
use crate::move_generation::metadata::CheckPinMetadata;
use crate::move_generation::move_filter::MoveFilter;
use crate::move_list::MoveList;
use crate::moves::{Move, MoveFlag};
use crate::rays;
use crate::{bitboard::Bitboard, board::Board, pieces::Piece, rank::Rank, square::Square};

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
            let en_passant_bb = Bitboard::from(sq);

            let mut occupancy = board.all_pieces();
            occupancy &= !(Bitboard::from_square(from));

            let captured_sq = sq.backward_unchecked(board.side_to_move());

            occupancy &= !(Bitboard::from(captured_sq));
            let mut discovered_checkers = move_generation::calculate_checkers(board, occupancy);
            let king_sq = board.king_square(board.side_to_move());
            let king_rank = king_sq.rank();
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

/// Generate all legal pawn moves setwise and push them into the given [`MoveList`].
///
/// Pawns are advanced together with directional shifts (mirroring the pseudo-legal [`move_generation::get_pawn_moves`])
/// and the legality masks are applied to the shifted target sets. This works because every mask the per-pawn path applied
/// is shared across pawns: check evasion uses `capture_mask | push_mask`, and pinned pawns are restricted by the union of
/// pin rays. En passant stays per-pawn since its discovered-check test depends on the capturing pawn's square.
///
/// # Arguments
/// - `board`: The current board to generate legal pawn moves for.
/// - `move_filter`: [`MoveFilter`] for the mvoes to be generated.
/// - `metadata`: Metadata of the current position structure (pins, checkers and so on).
/// - `move_list`: The [`MoveList`] to push moves into.
fn generate_legal_pawn_moves(
    board: &Board,
    move_filter: MoveFilter,
    metadata: &CheckPinMetadata,
    move_list: &mut MoveList,
) {
    let us = board.side_to_move();
    let them = us.opposite();
    let pawns = board.piece_bitboard(Piece::Pawn, us);
    if pawns.is_empty() {
        return;
    }

    let enemies = board.pieces(them);
    let empty = !metadata.occupancy;
    let promotion_rank = Rank::promotion_rank(us);
    // Check-evasion mask; all squares when the king is not in check.
    let evasions = metadata.capture_mask | metadata.push_mask;

    // Non-promotion pushes, promotion pushes, and captures are gated independently
    // so each `MoveFilter` selects exactly the same set the per-pawn generator did:
    // Quiets = non-promo pushes; Captures = all captures/EP; Tacticals = promo
    // pushes + all captures; All = everything.
    let want_quiet_pushes = matches!(move_filter, MoveFilter::All | MoveFilter::Quiets);
    let want_promo_pushes = matches!(move_filter, MoveFilter::All | MoveFilter::Tacticals);
    let want_captures = matches!(
        move_filter,
        MoveFilter::All | MoveFilter::Captures | MoveFilter::Tacticals
    );

    let (push, capture_left, capture_right, third_rank) = move_generation::pawn_shifts(us);
    // Inverse rank delta of one forward push (used to recover a move's origin).
    let back = -us.forward_delta();

    // A pinned pawn may only move along the union of pin rays: pushes along
    // orthogonal rays, captures along diagonal rays. The pin masks apply to
    // destination squares only.
    let unpinned = pawns & !metadata.pinned;
    let pinned = pawns & metadata.pinned;

    if want_quiet_pushes || want_promo_pushes {
        let single_unpinned = push(unpinned) & empty;
        let single_pinned = push(pinned) & empty;
        let single = (single_unpinned | (single_pinned & metadata.orthogonal_pin_rays)) & evasions;
        emit_pawn_targets(
            move_list,
            single,
            (0, back),
            MoveFlag::Standard,
            promotion_rank,
            want_promo_pushes,
            want_quiet_pushes,
        );

        // A double push exists only where the intermediate single-push square is
        // empty and the pawn started on its home rank. Pin and evasion masks
        // apply to the destination only, not the intermediate square.
        if want_quiet_pushes {
            let double = (push(single_unpinned & third_rank) & empty
                | (push(single_pinned & third_rank) & empty & metadata.orthogonal_pin_rays))
                & evasions;
            emit_pawn_targets(
                move_list,
                double,
                (0, 2 * back),
                MoveFlag::DoublePush,
                promotion_rank,
                false,
                true,
            );
        }
    }

    // Captures (including capture-promotions) and en passant.
    if want_captures {
        let left = (capture_left(unpinned) | (capture_left(pinned) & metadata.diagonal_pin_rays))
            & enemies
            & evasions;
        emit_pawn_targets(
            move_list,
            left,
            (1, back),
            MoveFlag::Standard,
            promotion_rank,
            true,
            true,
        );
        let right = (capture_right(unpinned)
            | (capture_right(pinned) & metadata.diagonal_pin_rays))
            & enemies
            & evasions;
        emit_pawn_targets(
            move_list,
            right,
            (-1, back),
            MoveFlag::Standard,
            promotion_rank,
            true,
            true,
        );

        // En passant is rare; legality (discovered check along the king's rank)
        // depends on the capturing pawn's square, so handle the at most two
        // attackers individually.
        if let Some(ep_square) = board.en_passant_square() {
            // Our pawns that attack the EP square are exactly those a `them` pawn
            // on the EP square would attack.
            let ep_attackers = pawns & attacks::pawn(ep_square, them);
            for from in ep_attackers {
                let pin_mask = if metadata.pinned.intersects(Bitboard::from(from)) {
                    metadata.diagonal_pin_rays
                } else {
                    Bitboard::filled()
                };
                let ep_bb = calculate_en_passant_bitboard(
                    from.inner(),
                    board,
                    metadata.push_mask,
                    metadata.checkers,
                ) & pin_mask
                    & evasions;
                if !ep_bb.is_empty() {
                    move_list.push(Move::new(from, ep_square, MoveFlag::EnPassant));
                }
            }
        }
    }
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
fn generate_normal_piece_legal_mobility(
    piece: Piece,
    square: Square,
    board: &Board,
    meta: &CheckPinMetadata,
) -> Bitboard {
    let is_pinned = meta
        .pinned
        .intersects(Bitboard::from_square(square.inner()));
    let us = board.side_to_move();
    let their_pieces = board.pieces(us.opposite());
    let occupancy = meta.occupancy;
    let pin_rays = meta.orthogonal_pin_rays | meta.diagonal_pin_rays;

    assert!(!piece.is_king() && !piece.is_pawn());

    let piece_attacks = attacks::for_piece_on_square(piece, square, occupancy, us);

    let empty = !occupancy;

    let pin_ray_mask = if is_pinned {
        let king_sq = board.king_square(us);

        let pinners = their_pieces & pin_rays;
        let piece_bb = Bitboard::from_square(square.inner());
        let mut true_ray_mask = Bitboard::default();

        for pinner in pinners.iter() {
            let pinner_sq = Square::from_square_index(pinner);
            let ray = rays::between(pinner_sq, king_sq);

            if ray.intersects(piece_bb) {
                true_ray_mask |= ray | Bitboard::from(pinner_sq);
            }
        }

        true_ray_mask
    } else {
        Bitboard::filled()
    };

    ((piece_attacks & meta.capture_mask & their_pieces) | (piece_attacks & empty & meta.push_mask))
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
    meta: &CheckPinMetadata,
) -> Bitboard {
    let us = board.side_to_move();
    let them = us.opposite();
    let our_pieces = board.pieces(us);
    let their_pieces = board.pieces(them);
    let occupancy = meta.occupancy;

    let king_bb = board.piece_bitboard(Piece::King, us);

    let king_moves_bb = attacks::king(square);

    let attacked_squares_occupancy = occupancy & !king_bb;
    let attacked_squares =
        move_generation::get_attacked_squares(board, them, attacked_squares_occupancy);
    let king_pushes = king_moves_bb & !attacked_squares & !our_pieces & !their_pieces;

    let castling_moves = move_generation::castling::legal_mobility(board, meta.checkers);

    let king_non_checker_attacks =
        (king_moves_bb & their_pieces & !meta.checkers) & !attacked_squares;

    let mut king_attacks = (king_moves_bb & meta.capture_mask & their_pieces & !attacked_squares)
        | king_non_checker_attacks;

    let k_att = king_attacks;
    for capture_sq in k_att {
        let modified_occupancy = occupancy & !Bitboard::from(capture_sq) & !king_bb;
        let is_invalid_capture =
            !attacks::all_attackers_of(capture_sq, board, them, modified_occupancy).is_empty();
        if is_invalid_capture {
            king_attacks &= !Bitboard::from(capture_sq);
        }
    }

    king_pushes | king_attacks | castling_moves
}

/// Generate legal moves for the current [`Board`] state using pre-computed metadata.
///
/// This is the core implementation. It accepts pre-computed [`CheckPinMetadata`] to
/// avoid recomputing it when generating tacticals and quiets in separate stages.
///
/// # Arguments
///
/// - `board` - The current board state
/// - `move_filter` - Which moves to generate
/// - `meta` - Pre-computed check/pin metadata for this position
pub fn generate_moves_with_metadata(
    board: &Board,
    move_filter: MoveFilter,
    meta: &CheckPinMetadata,
) -> MoveList {
    let us = board.side_to_move();
    let them = us.opposite();
    let our_pieces = board.pieces(us);
    let their_pieces = board.pieces(them);
    let filter = match move_filter {
        MoveFilter::All => Bitboard::filled(),
        MoveFilter::Captures | MoveFilter::Tacticals => their_pieces,
        MoveFilter::Quiets => !their_pieces,
    };

    let king_sq = board.king_square(us);
    let king_bb = king_sq.as_bitboard();

    let mut move_list = MoveList::new();

    // King moves first
    let king_moves = generate_king_legal_mobility(king_sq, board, meta) & filter;

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

    // All pawn moves are generated setwise; the function does its own filtering.
    generate_legal_pawn_moves(board, move_filter, meta, &mut move_list);

    // The remaining pieces (knights and sliders) are generated per-square.
    let pawns = board.piece_bitboard(Piece::Pawn, us);
    let moveable_pieces = our_pieces & !king_bb & !pawns;

    for from_sq in moveable_pieces {
        // The square is one of our pieces, so the side is already known; only the
        // piece type is needed here.
        let piece = match board.piece_type_on_square(from_sq) {
            Some(piece) => piece,
            None => continue,
        };

        let moves = generate_normal_piece_legal_mobility(piece, from_sq, board, meta) & filter;

        enumerate_moves(&moves, from_sq, piece, board, move_filter, &mut move_list);
    }

    move_list
}

/// Generate legal moves for the current [`Board`] state.
///
/// This is a convenience wrapper that computes check/pin metadata and delegates
/// to [`generate_moves_with_metadata`].
///
/// # Arguments
///
/// - `board` - The current board state
/// - `move_filter` - Which moves to generate
///
/// # Examples
///
/// ```
/// use chess::board::Board;
/// use chess::move_list::MoveList;
/// use chess::move_generation;
/// use chess::move_generation::move_filter::MoveFilter;
///
/// let board = Board::default_board();
/// let move_list = move_generation::legal::generate_moves(&board, MoveFilter::All);
/// assert_eq!(20, move_list.len())
/// ```
pub fn generate_moves(board: &Board, move_filter: MoveFilter) -> MoveList {
    let meta = move_generation::metadata::compute(board);
    generate_moves_with_metadata(board, move_filter, &meta)
}

pub fn generate_all_moves(board: &Board) -> MoveList {
    generate_moves(board, MoveFilter::All)
}

#[cfg(test)]
mod tests {
    use crate::moves::MoveFlag;

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
        let ray = rays::between(Square::A1, Square::H8);

        let expected = Bitboard::from(Square::B2)
            | Bitboard::from(Square::C3)
            | Bitboard::from(Square::D4)
            | Bitboard::from(Square::E5)
            | Bitboard::from(Square::F6)
            | Bitboard::from(Square::G7);
        println!("{ray}");
        assert_eq!(ray, expected);

        let ray = rays::between(Square::H1, Square::A8);

        let expected = Bitboard::from(Square::G2)
            | Bitboard::from(Square::F3)
            | Bitboard::from(Square::E4)
            | Bitboard::from(Square::D5)
            | Bitboard::from(Square::C6)
            | Bitboard::from(Square::B7);
        assert_eq!(ray, expected);

        let ray = rays::between(Square::A1, Square::C2);
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
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        ];

        for fen in &positions {
            let board = Board::from_fen(fen).unwrap();

            let all_moves = generate_moves(&board, MoveFilter::All);
            let captures = generate_moves(&board, MoveFilter::Tacticals);
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
                        && mv.promotion_piece().is_some_and(|pc| matches!(
                            pc,
                            Piece::Queen | Piece::Rook | Piece::Bishop | Piece::Knight
                        ))),
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
            "8/P3k3/8/8/8/8/8/4K3 w - - 0 1",
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
        let board = Board::from_fen("2r5/3P4/8/8/8/8/6k1/4K3 w - - 0 1").unwrap();
        println!("{}\n{}", board.to_fen(), board);
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
        assert_eq!(
            push_promos(&tacticals),
            4,
            "Tacticals should have 4 push-promos"
        );
        assert_eq!(
            push_promos(&captures),
            0,
            "Captures should have 0 push-promos"
        );

        // capture-promo to c8: 4 promo types in All, 4 in Captures, 0 in Quiets
        let cap_promos = |list: &MoveList| {
            list.iter()
                .filter(|mv| mv.is_promotion() && board.captured(mv).is_some())
                .count()
        };
        assert_eq!(cap_promos(&all), 4, "All should have 4 capture-promos");
        assert_eq!(
            cap_promos(&captures),
            4,
            "Captures should have 4 capture-promos"
        );
        assert_eq!(
            cap_promos(&quiets),
            0,
            "Quiets should have 0 capture-promos"
        );

        // Tacticals: only queen promos (1 push-promo + 1 capture-promo)
        let tacticals_queen_promos = tacticals
            .iter()
            .filter(|mv| {
                mv.is_promotion() && mv.promotion_piece().is_some_and(|pc| pc == Piece::Queen)
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
        let board = Board::from_fen("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1").unwrap();

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
        let board = Board::from_fen("4k3/8/8/1B6/8/8/8/4R1K1 b - - 0 1").unwrap();

        let all = generate_moves(&board, MoveFilter::All);
        let tacticals = generate_moves(&board, MoveFilter::Tacticals);

        // In double check, only king moves are legal
        for mv in all.iter() {
            let (piece, _) = board.piece_on_square(mv.from()).unwrap();
            assert_eq!(
                piece,
                Piece::King,
                "Only king moves should be legal in double check"
            );
        }

        validate_tacticals_for_board(&board);
        // Tacticals should only include king captures
        for mv in tacticals.iter() {
            let (piece, _) = board.piece_on_square(mv.from()).unwrap();
            assert_eq!(piece, Piece::King);
        }
    }

    #[test]
    fn no_quiets_with_tactical_movegen() {
        let fen = "3r4/8/3p4/3QP3/8/8/8/4K1k1 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();
        println!("{}", board);
        let moves = move_generation::legal::generate_moves(&board, MoveFilter::Tacticals);
        for mv in moves.iter() {
            let is_capture = board.captured(mv).is_some();
            let is_queen_promo = mv.flag() == MoveFlag::PromotionQueen;
            assert!(is_capture || is_queen_promo);
            println!("{}", mv.to_long_algebraic());
        }
    }
}
