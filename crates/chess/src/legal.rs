// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module contains move legality utilities including checks for "pseudo-legal" moves
//! as well as full move legality. Pseudo-legal checks involve making sure the move is consistent
//! with the current state of the board (i.e. move piece matches piece at from position, valid
//! move flag and so on). Pseudo-legal checks don't verify king safety after the move is made.
//!
//! This module also includes a full legal move check without making the move on the board.
//! This check leverages the [`CheckPinMetadata`] to verify that a move is legal and will leave
//! the king in a safe state after the move is made. This includes things like double check,
//! king moves not being attacked, x-ray detection of king checks, pinned piece checks and EP
//! disovered check.
//!

use crate::{
    attacks,
    bitboard::Bitboard,
    board::Board,
    definitions::RANK_BITBOARDS,
    file::File,
    move_generation::{self, NORTH, SOUTH, metadata::CheckPinMetadata, square_state},
    moves::{Move, MoveFlag},
    pieces::Piece,
    rank::Rank,
    rays,
    side::Side,
    square::{self, Square},
};

/// Checks if a given [`Move`] on the given [`Board`].
///
/// Verifies:
/// 1. The from-square contains a piece of the side to move.
/// 2. The to-square does not contain a friendly piece.
/// 3. The move flag is consistent with the piece type.
/// 4. The to-square is reachable (considering occupancy for sliders).
/// 5. Special-move preconditions (castling path/rights/attacks, EP square,
///    double-push start rank, promotion rank).
///
/// Does not verify that the king is left safe — use [`is_legal`] for that.
///
/// # Arguments
/// - `board`: The current position
/// - `mv`: The move to check
///
/// # Returns
/// True if the move is pseudo-legal, false otherwise.
pub fn is_pseudo_legal(board: &Board, mv: &Move) -> bool {
    if mv.is_null_move() {
        return false;
    }

    let us = board.side_to_move();
    let them = us.opposite();
    let from = mv.from();
    let to = mv.to();
    let flag = mv.flag();

    let (piece, side) = match board.piece_on_square(from) {
        Some(ps) => ps,
        None => return false,
    };
    if side != us {
        return false;
    }

    if let Some((_, to_side)) = board.piece_on_square(to)
        && to_side == us
    {
        return false;
    }

    if flag.is_promotion() && piece != Piece::Pawn {
        return false;
    }

    let occupancy = board.all_pieces();

    if mv.flag().validate(piece).is_err() {
        return false;
    }

    match piece {
        Piece::Pawn => is_pseudo_legal_pawn(board, us, them, from, to, flag, occupancy),
        Piece::King => is_pseudo_legal_king(board, us, from, to, flag, occupancy),
        _ => {
            if matches!(
                flag,
                MoveFlag::CastleK | MoveFlag::CastleQ | MoveFlag::EnPassant | MoveFlag::DoublePush
            ) {
                return false;
            }
            attacks::for_piece_on_square(piece, from, occupancy, us).is_square_occupied(to)
        }
    }
}

/// Validates that the given pawn move is pseudo-legal.
fn is_pseudo_legal_pawn(
    board: &Board,
    us: Side,
    them: Side,
    from: u8,
    to: u8,
    flag: MoveFlag,
    occupancy: Bitboard,
) -> bool {
    let start_rank = Rank::pawn_start_rank(us);
    let promo_rank = Rank::promotion_rank(us);
    let push_delta = match us {
        Side::White => NORTH as i8,
        Side::Black => -(SOUTH as i8),
    };

    let from_rank = Rank::of(from);
    let to_rank = Rank::of(to);

    if flag.is_promotion() && to_rank != promo_rank {
        return false;
    }
    if !flag.is_promotion() && to_rank == promo_rank {
        return false;
    }

    match flag {
        MoveFlag::DoublePush => {
            if from_rank != start_rank {
                return false;
            }
            let intermediate = (from as i8 + push_delta) as u8;
            let dest = (from as i8 + 2 * push_delta) as u8;
            dest == to
                && !occupancy.is_square_occupied(intermediate)
                && !occupancy.is_square_occupied(to)
        }
        MoveFlag::EnPassant => {
            if board.en_passant_square() != Some(to) {
                return false;
            }
            if !attacks::pawn(from, us).is_square_occupied(to) {
                return false;
            }
            // The enemy pawn being captured must actually exist.
            let captured_sq = match us {
                // EP capture is 1 row back towards starting rank
                Side::White => to - 8,
                Side::Black => to + 8,
            };
            // Check that the captured EP piece is in fact their pawn.
            board
                .piece_on_square(captured_sq)
                .is_some_and(|(p, s)| p == Piece::Pawn && s == them)
        }
        MoveFlag::CastleK | MoveFlag::CastleQ => {
            // Invalid pawn move
            false
        }
        _ => {
            let file_diff = (File::of(from) as u8).abs_diff(File::of(to) as u8);
            if file_diff == 0 {
                let dest = (from as i8 + push_delta) as u8;
                dest == to && !occupancy.is_square_occupied(to)
            } else if file_diff == 1 {
                attacks::pawn(from, us).is_square_occupied(to)
                    && board.piece_on_square(to).is_some_and(|(_, s)| s == them)
            } else {
                false
            }
        }
    }
}

fn is_pseudo_legal_king(
    board: &Board,
    us: Side,
    from: u8,
    to: u8,
    flag: MoveFlag,
    occupancy: Bitboard,
) -> bool {
    match flag {
        MoveFlag::CastleK => {
            let (king_sq, rook_sq, f_sq, g_sq) = match us {
                Side::White => (Square::E1, Square::H1, Square::F1, Square::G1),
                Side::Black => (Square::E8, Square::H8, Square::F8, Square::G8),
            };
            if from != king_sq.inner() || to != g_sq.inner() {
                return false;
            }
            if !board.can_castle_kingside(us) {
                return false;
            }
            board
                .piece_on_square(rook_sq.inner())
                .is_some_and(|(p, s)| p == Piece::Rook && s == us)
                && !occupancy.is_square_occupied(f_sq.inner())
                && !occupancy.is_square_occupied(g_sq.inner())
                && !square_state::is_square_attacked(
                    board,
                    Square::from_square_index(king_sq.inner()),
                    us.opposite(),
                )
                && !square_state::is_square_attacked(
                    board,
                    Square::from_square_index(f_sq.inner()),
                    us.opposite(),
                )
                && !square_state::is_square_attacked(
                    board,
                    Square::from_square_index(g_sq.inner()),
                    us.opposite(),
                )
        }
        MoveFlag::CastleQ => {
            let (king_sq, rook_sq, d_sq, c_sq, b_sq) = match us {
                Side::White => (Square::E1, Square::A1, Square::D1, Square::C1, Square::B1),
                Side::Black => (Square::E8, Square::A8, Square::D8, Square::C8, Square::B8),
            };
            if from != king_sq.inner() || to != c_sq.inner() {
                return false;
            }
            if !board.can_castle_queenside(us) {
                return false;
            }
            board
                .piece_on_square(rook_sq.inner())
                .is_some_and(|(p, s)| p == Piece::Rook && s == us)
                && !occupancy.is_square_occupied(d_sq.inner())
                && !occupancy.is_square_occupied(c_sq.inner())
                && !occupancy.is_square_occupied(b_sq.inner())
                && !square_state::is_square_attacked(
                    board,
                    Square::from_square_index(king_sq.inner()),
                    us.opposite(),
                )
                && !square_state::is_square_attacked(
                    board,
                    Square::from_square_index(d_sq.inner()),
                    us.opposite(),
                )
                && !square_state::is_square_attacked(
                    board,
                    Square::from_square_index(c_sq.inner()),
                    us.opposite(),
                )
        }
        _ => {
            if matches!(flag, MoveFlag::DoublePush | MoveFlag::EnPassant) {
                return false;
            }
            attacks::king(from).is_square_occupied(to)
        }
    }
}

/// Returns `true` if `mv` is fully legal given precomputed check/pin `meta`.
///
/// Assumes the move has already passed [`is_pseudo_legal`]. Validates king
/// safety without cloning the board or executing make/unmake.
pub fn is_legal_with_metadata(board: &Board, mv: &Move, meta: &CheckPinMetadata) -> bool {
    let us = board.side_to_move();
    let them = us.opposite();
    let from = mv.from();
    let to = mv.to();
    let king_sq = board.king_square(us);
    let occupancy = board.all_pieces();

    let piece = match board.piece_on_square(from) {
        Some((p, _)) => p,
        None => return false,
    };

    // Double check - only king moves are legal.
    if meta.num_checkers() >= 2 {
        return piece == Piece::King
            && !mv.is_castle()
            && king_destination_safe(board, to, them, occupancy);
    }

    // Check king moves
    if piece == Piece::King {
        if mv.is_castle() {
            // Castling transit/destination attacks are already verified in is_pseudo_legal.
            return true;
        }
        return king_destination_safe(board, to, them, occupancy);
    }

    // En passant (must be tested before the generic check-evasion path because
    // even an unpinned pawn can expose the king via horizontal discovered check).
    if mv.is_en_passant_capture() {
        return ep_legal(board, mv, meta, king_sq, us, them, occupancy);
    }

    // Single check evasion: non-king piece must capture the checker or block the ray.
    if meta.in_check() {
        let to_bb = Bitboard::from_square(to);
        if !to_bb.intersects(meta.capture_mask | meta.push_mask) {
            return false;
        }
        // A pinned piece that evades check must still stay on its pin ray.
        if Bitboard::from_square(from).intersects(meta.pinned) {
            return pinned_move_is_on_ray(meta, from, to, king_sq, board.pieces(them));
        }
        return true;
    }

    // Pinned piece: destination must stay on the pin ray.
    if Bitboard::from_square(from).intersects(meta.pinned) {
        return pinned_move_is_on_ray(meta, from, to, king_sq, board.pieces(them));
    }

    // Unpinned non-king non-EP: pseudo-legality already confirmed reachability.
    true
}

/// Returns `true` if `mv` is fully legal in `board`.
///
/// Combines [`is_pseudo_legal`] with a make-free king-safety check via
/// [`is_legal_with_metadata`]. Computes [`CheckPinMetadata`] internally.
pub fn is_legal(board: &Board, mv: &Move) -> bool {
    if !is_pseudo_legal(board, mv) {
        return false;
    }
    let meta = move_generation::metadata::compute(board);
    is_legal_with_metadata(board, mv, &meta)
}

// ─── Private helpers ─────────────────────────────────────────────────────────

/// Returns `true` if the king can safely move to `to`.
///
/// Removes the king from occupancy so x-ray attacks through the king are
/// detected. For captures, also removes the captured piece.
fn king_destination_safe(board: &Board, to: u8, them: Side, occupancy: Bitboard) -> bool {
    let us = board.side_to_move();
    let king_bb = board.piece_bitboard(Piece::King, us);
    // Remove king so sliders attacking through it are detected.
    let occ = occupancy & !king_bb;
    // Remove any captured piece so X-ray defense behind it is detected.
    let occ = occ & !Bitboard::from_square(to);
    attacks::all_attackers_of(to, board, them, occ).is_empty()
}

/// Returns `true` if an en-passant capture is legal.
///
/// Handles both check-evasion semantics and the horizontal discovered-check
/// case (removing both pawns from the king's rank can reveal a rook/queen).
fn ep_legal(
    board: &Board,
    mv: &Move,
    meta: &CheckPinMetadata,
    king_sq: u8,
    us: Side,
    them: Side,
    occupancy: Bitboard,
) -> bool {
    let from = mv.from();
    let to = mv.to();
    // The pawn being captured sits one rank behind the EP square.
    let captured_sq = match us {
        Side::White => to - 8,
        Side::Black => to + 8,
    };
    let captured_bb = Bitboard::from_square(captured_sq);
    let to_bb = Bitboard::from_square(to);

    // In check: EP must capture the checking pawn or land on the push-mask ray.
    if meta.in_check()
        && !captured_bb.intersects(meta.checkers)
        && !to_bb.intersects(meta.push_mask)
    {
        return false;
    }

    // Horizontal discovered check: remove both pawns and test king's rank.
    let modified_occ =
        occupancy & !Bitboard::from_square(from) & !Bitboard::from_square(captured_sq);
    let (_, king_rank) = square::from_square(king_sq);
    let rank_bb = RANK_BITBOARDS[king_rank as usize];
    let rank_attackers = attacks::rook(king_sq, modified_occ)
        & rank_bb
        & (board.piece_bitboard(Piece::Rook, them) | board.piece_bitboard(Piece::Queen, them));

    rank_attackers.is_empty()
}

/// Checks if the move of a pinned piece is along the pinned ray.
///
/// # Arguments
/// - `meta`: Pre-computed [`CheckPinMetadata`] for the current position.
/// - `from`: From square for the move.
/// - `to`: To square for the move.
/// - `king_sq`: Current square of the king for the side to move.
/// - `their_pieces`: Enemy piece bitboard.
///
/// # Returns
/// True if the pinned move is on the ray, false otherwise.
fn pinned_move_is_on_ray(
    meta: &CheckPinMetadata,
    from: u8,
    to: u8,
    king_sq: u8,
    their_pieces: Bitboard,
) -> bool {
    let all_pin_rays = meta.orthogonal_pin_rays | meta.diagonal_pin_rays;
    // Iterate enemy pieces that lie on any pin ray to find the pinner of `from`.
    for pinner_sq in (their_pieces & all_pin_rays).iter() {
        let ray = rays::between(pinner_sq, king_sq);
        if ray.is_square_occupied(from) {
            // `from` is between the king and this pinner — this is our pin ray.
            let full_ray = ray | Bitboard::from_square(pinner_sq);
            return full_ray.is_square_occupied(to);
        }
    }
    // Pinned flag was set but no matching ray found — should not happen with
    // correct metadata, but treat as illegal to be safe.
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        board::Board,
        move_generation::{self, legal::generate_all_moves, metadata},
    };

    #[allow(clippy::panic)]
    fn assert_all_legal_are_pseudo_legal(fen_str: &str) {
        let board = Board::from_fen(fen_str).unwrap_or_else(|e| panic!("bad FEN: {e}"));
        let moves = generate_all_moves(&board);
        for mv in moves.iter() {
            assert!(
                is_pseudo_legal(&board, mv),
                "Legal move {} should be pseudo-legal in {}",
                mv.to_long_algebraic(),
                fen_str,
            );
        }
    }

    #[test]
    fn startpos_all_legal_are_pseudo_legal() {
        assert_all_legal_are_pseudo_legal(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        );
    }

    #[test]
    fn complex_middlegame_pseudo_legal() {
        assert_all_legal_are_pseudo_legal(
            "rnb2k1r/pp4pp/2p3q1/3p1n2/3Pp2B/1B2N3/PPP2PPP/RN1Q1RK1 b - - 3 14",
        );
    }

    #[test]
    fn position_with_en_passant_pseudo_legal() {
        assert_all_legal_are_pseudo_legal(
            "rnbqkbnr/pppp1ppp/8/4pP2/8/8/PPPPP1PP/RNBQKBNR w KQkq e6 0 3",
        );
    }

    #[test]
    fn position_with_castling_pseudo_legal() {
        assert_all_legal_are_pseudo_legal("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
    }

    #[test]
    fn position_with_promotions_pseudo_legal() {
        assert_all_legal_are_pseudo_legal("8/P5k1/8/8/8/8/1K4p1/8 w - - 0 1");
    }

    #[test]
    fn rejects_garbage_knight_move() {
        let board =
            Board::from_fen("rnb2k1r/pp4pp/2p3q1/3p1n2/3Pp3/1B2N3/PPP2PPK/RN1Q1R2 b - - 0 15")
                .unwrap();
        // f5→f1 is not a valid knight destination.
        let garbage = Move::new(
            Square::from_square_index(37),
            Square::from_square_index(5),
            MoveFlag::Standard,
        );
        assert!(!is_pseudo_legal(&board, &garbage));
    }

    #[test]
    fn rejects_null_move() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        assert!(!is_pseudo_legal(&board, &Move::default()));
    }

    #[test]
    fn rejects_wrong_side_piece() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let wrong = Move::new(
            Square::from_square_index(48),
            Square::from_square_index(40),
            MoveFlag::Standard,
        );
        assert!(!is_pseudo_legal(&board, &wrong));
    }

    #[test]
    fn rejects_pawn_push_to_occupied() {
        let board =
            Board::from_fen("rnbqkbnr/pppppppp/8/8/8/4p3/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        let blocked = Move::new(Square::E2, Square::E3, MoveFlag::Standard);
        assert!(!is_pseudo_legal(&board, &blocked));
    }

    #[test]
    fn rejects_castle_through_check() {
        let board = Board::from_fen("3rk3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
        let castle_q = Move::new(Square::E1, Square::C1, MoveFlag::CastleQ);
        assert!(!is_pseudo_legal(&board, &castle_q));
    }

    // GGN crash regression test
    //
    // Each test below corresponds to an illegal move that was played in a
    // tournament game due to a TT hash collision. The old `check_move_preconditions`
    // had no movement-rule (reachability) validation, so these moves were
    // incorrectly accepted and executed, corrupting the board state.

    #[test]
    fn reject_illegal_move_f3_to_e5() {
        // Recreate error state (illegal move) from OpenBench error
        // A Black pawn on f3 cannot reach e5.
        // There is an enemy piece (White queen) on e5, so the old code treated this
        // as a valid capture with a Standard flag and accepted it.
        // White queen on e5, Black pawn on f3, kings to keep position valid.
        let board = Board::from_fen("8/8/8/4Q3/8/5p2/8/4K1k1 b - - 0 1").unwrap();
        let f3 = Square::from_square_index(21); // f3
        let e5 = Square::from_square_index(36); // e5
        let mv = Move::new(f3, e5, MoveFlag::Standard);
        assert!(!is_pseudo_legal(&board, &mv));
    }

    #[test]
    fn reject_illegal_move_e4_to_e5_blocked() {
        // Round 151: White makes an illegal move e4e5.
        // Black just played 18...e5, occupying e5. White tries to push e4→e5 into
        // the occupied square. The old code never validated destination occupancy
        // for pawn pushes (only captures were checked) and accepted the move.
        let board = Board::from_fen("8/8/8/4p3/4P3/8/8/4K1k1 w - - 0 1").unwrap();
        let e4 = Square::from_square_index(28); // e4
        let e5 = Square::from_square_index(36); // e5
        let mv = Move::new(e4, e5, MoveFlag::Standard);
        assert!(!is_pseudo_legal(&board, &mv));
    }

    // ── Make-free legality equivalence tests ─────────────────────────────────
    //
    // For each position, generate every pseudo-legal move, then assert that
    // is_legal_with_metadata() agrees with the reference clone+make check.
    #[allow(clippy::panic)]
    fn assert_make_free_agrees_with_make(fen_str: &str) {
        let board = Board::from_fen(fen_str).unwrap_or_else(|e| panic!("bad FEN: {e}"));
        let meta = metadata::compute(&board);
        // Generate a superset: all legal moves from the chess crate.
        let legal_moves = generate_all_moves(&board);
        // Verify each legal move is accepted by the make-free check.
        for mv in legal_moves.iter() {
            assert!(
                is_legal_with_metadata(&board, mv, &meta),
                "Legal move {} was rejected by make-free check in {}",
                mv.to_long_algebraic(),
                fen_str,
            );
        }
        // Also verify pseudo-legal-but-illegal moves are rejected (e.g. pinned piece moves).
        // We do this by checking that is_legal() == move_generation::is_legal() for all
        // pseudo-legal moves.
        // Build the full pseudo-legal list via the pseudo-legal move gen.
        use crate::move_generation::move_filter::MoveFilter;
        let mut pseudo_list = crate::move_list::MoveList::new();
        move_generation::generate_moves(&board, &mut pseudo_list, MoveFilter::All);
        for mv in pseudo_list.iter() {
            let make_free =
                is_pseudo_legal(&board, mv) && is_legal_with_metadata(&board, mv, &meta);
            let reference = move_generation::is_legal(&board, mv);
            assert_eq!(
                make_free,
                reference,
                "Disagreement on {} in {}: make-free={} reference={}",
                mv.to_long_algebraic(),
                fen_str,
                make_free,
                reference,
            );
        }
    }

    #[test]
    fn make_free_agrees_startpos() {
        assert_make_free_agrees_with_make(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        );
    }

    #[test]
    fn make_free_agrees_middlegame() {
        assert_make_free_agrees_with_make(
            "rnb2k1r/pp4pp/2p3q1/3p1n2/3Pp2B/1B2N3/PPP2PPP/RN1Q1RK1 b - - 3 14",
        );
    }

    #[test]
    fn make_free_agrees_with_castling() {
        assert_make_free_agrees_with_make("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1");
    }

    #[test]
    fn make_free_agrees_with_promotions() {
        assert_make_free_agrees_with_make("8/P5k1/8/8/8/8/1K4p1/8 w - - 0 1");
    }

    #[test]
    fn make_free_double_check_only_king_moves() {
        // Double check: Nc6+ Bb5+ — only king moves are legal.
        assert_make_free_agrees_with_make(
            "r1bqkb1r/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 0 1",
        );
    }

    #[test]
    fn make_free_pinned_piece() {
        // Bishop on d3 is pinned to the king on e4 by a rook on a6 (diagonal).
        // Use a position where a piece is clearly pinned.
        assert_make_free_agrees_with_make(
            "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2",
        );
    }

    #[test]
    fn make_free_ep_discovered_check_rejected() {
        // EP discovered check: removing both pawns exposes king to queen.
        // FEN parsing now strips obviously-illegal EP targets, so set the EP
        // square directly to test the move-generation legality path.
        let mut board = Board::from_fen("8/8/8/8/k2Pp2Q/8/8/3K4 b - - 0 1").unwrap();
        board.set_en_passant_square(Some(Square::D3.inner()));
        let meta = metadata::compute(&board);
        // e4→d3 EP
        let ep_mv = Move::new(Square::E4, Square::D3, MoveFlag::EnPassant);
        assert!(is_pseudo_legal(&board, &ep_mv), "EP should be pseudo-legal");
        assert!(
            !is_legal_with_metadata(&board, &ep_mv, &meta),
            "EP should be illegal (horizontal discovered check)"
        );
    }

    #[test]
    fn make_free_ep_resolves_check() {
        // Black pawn on e4, white pawn just double-pushed to d4 (ep sq = d3),
        // white rook on d1 is checking the king on d8. The EP capture takes the
        // checking pawn.
        assert_make_free_agrees_with_make("3k4/8/8/8/3Pp3/8/8/3RK3 b - d3 0 1");
    }

    #[test]
    fn make_free_king_capture_defended_piece() {
        // King tries to capture a defended piece — should be rejected.
        assert_make_free_agrees_with_make("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1");
    }

    #[test]
    fn make_free_agrees_complex_kiwipete() {
        // KiwiPete — classic tricky position with many edge cases.
        assert_make_free_agrees_with_make(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        );
    }

    #[test]
    fn make_free_pinned_piece_in_single_check() {
        // White king d1, rook d3 pinned on d-file by Black rook d8, bishop f3
        // gives check. Rd3xf3 captures the checker but leaves king exposed to
        // Rd8 along the d-file — must be rejected.
        assert_make_free_agrees_with_make("3r3k/8/8/8/8/3R1b2/8/3K4 w - - 0 1");
    }

    #[test]
    fn reject_pinned_piece_captures_checker() {
        // Same position as above — specific regression test for the move Rd3xf3.
        let board = Board::from_fen("3r3k/8/8/8/8/3R1b2/8/3K4 w - - 0 1").unwrap();
        let meta = metadata::compute(&board);
        let d3 = Square::D3;
        let f3 = Square::F3;
        let mv = Move::new(d3, f3, MoveFlag::Standard);
        assert!(
            is_pseudo_legal(&board, &mv),
            "Rd3xf3 should be pseudo-legal (rook can reach f3)"
        );
        assert!(
            !is_legal_with_metadata(&board, &mv, &meta),
            "Rd3xf3 should be illegal (rook is pinned on d-file)"
        );
    }
}
