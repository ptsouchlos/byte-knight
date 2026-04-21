// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{
    attacks, bitboard::Bitboard, bitboard_helpers, board::Board, moves::Move, pieces::Piece,
    side::Side,
};

use crate::tuneable::{
    see_value_bishop, see_value_knight, see_value_pawn, see_value_queen, see_value_rook,
};

#[inline(always)]
fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => see_value_pawn(),
        Piece::Bishop => see_value_bishop(),
        Piece::Knight => see_value_knight(),
        Piece::Rook => see_value_rook(),
        Piece::Queen => see_value_queen(),
        Piece::King => 0,
    }
}

/// Scans `attackers` for the least-valuable piece belonging to `side` on the board.
/// Pops the found square from `occ` and returns the piece type.
///
/// `attackers` must be pre-filtered to `& occ` before passing in so that
/// already-captured pieces are excluded.
fn pop_lva(attackers: Bitboard, board: &Board, side: Side, occ: &mut Bitboard) -> Option<Piece> {
    for piece in [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ] {
        // Are there any of these piece in the attackers?
        let mut candidates = board.piece_bitboard(piece, side) & attackers;
        if !candidates.is_empty() {
            // If yes, remove the square of the LVA from the occupancy
            let sq = bitboard_helpers::next_bit(&mut candidates) as u8;
            *occ ^= Bitboard::from_square(sq);
            return Some(piece);
        }
    }
    None
}

/// Returns `true` if the static exchange on `mv` gains at least `threshold` centipawns
/// for the side to move.
///
/// Uses the standard threshold-SEE algorithm: the swap loop exits as soon as the result
/// can be determined, avoiding a full-depth simulation when possible.
///
/// # Edge cases
/// - **Castle moves**: no capture occurs; returns `threshold <= 0`.
/// - **Quiet moves**: SEE = 0; returns `threshold <= 0`.
/// - **En-passant captures**: the captured pawn's square is removed from the occupancy
///   bitboard before the swap loop, so sliders behind it are correctly discovered.
/// - **Promotion captures**: the promoted piece is used as the attacker value (not the
///   pawn), and the promotion bonus is credited on the first ply.
pub fn see(board: &Board, mv: Move, threshold: i32) -> bool {
    // Castles don't involve any captures
    if mv.is_castle() {
        return threshold <= 0;
    }

    let from = mv.from();
    let to = mv.to();

    // Determine the side and piece making the first capture.
    let (attacker_piece, attacker_side) = match board.piece_on_square(from) {
        Some(p) => p,
        None => return false,
    };

    // Determine the captured piece.  `board.captured` handles en-passant (→ Some(Pawn))
    // and castles (→ None) correctly.
    let victim = match board.captured(&mv) {
        Some(v) => v,
        None => {
            // Quiet move: no exchange.
            return threshold <= 0;
        }
    };

    // For promotions the pawn becomes the promoted piece on `to`.  The opponent would
    // capture the promoted piece, and we also gain the promotion bonus on this ply.
    let (next_victim_val, promo_bonus) = if let Some(promo) = mv.flag().promotion_piece() {
        let promo_val = piece_value(promo);
        (promo_val, promo_val - piece_value(Piece::Pawn))
    } else {
        (piece_value(attacker_piece), 0)
    };

    // Initial balance: what we gain on this move minus the threshold.
    let mut balance = piece_value(victim) + promo_bonus - threshold;
    if balance < 0 {
        return false; // Even winning the victim uncontested doesn't meet the threshold.
    }

    // Subtract the value of the piece we risk losing to a recapture.
    balance -= next_victim_val;
    if balance >= 0 {
        return true; // We're up enough that even losing our piece back doesn't hurt.
    }

    // Seed occupancy: remove the capturing piece so sliders behind it are exposed.
    let mut occ = board.all_pieces() ^ Bitboard::from_square(from);

    // For en-passant the captured pawn is not on `to`; remove it explicitly so that
    // sliders behind it can be discovered in the swap loop.
    if mv.is_en_passant_capture() {
        let ep_pawn_sq = attacks::ep_capture_square(to, attacker_side);
        occ ^= Bitboard::from_square(ep_pawn_sq);
    }

    // Opponent gets to recapture first.
    let mut side_to_move = attacker_side.opposite();

    loop {
        // All pieces of `side` that still attack `to` with the current occupancy.
        let attackers = attacks::all_attackers_of(to, board, side_to_move, occ) & occ;
        if attackers.is_empty() {
            break;
        }

        // Find and remove the least-valuable attacker from occupancy.
        let lva = match pop_lva(attackers, board, side_to_move, &mut occ) {
            Some(p) => p,
            None => break,
        };

        // A king cannot legally move to a square attacked by the opponent.  If the king
        // is the only remaining attacker and the opponent still has defenders, the side
        // forfeits further captures.
        if lva == Piece::King {
            let opp_defenders =
                attacks::all_attackers_of(to, board, side_to_move.opposite(), occ) & occ;
            if !opp_defenders.is_empty() {
                break;
            }
        }

        // Negamax balance update: flip perspective and charge the attacker's value.
        side_to_move = side_to_move.opposite();
        balance = -balance - 1 - piece_value(lva);
        if balance >= 0 {
            // The side that just captured (before the flip) is demonstrably ahead;
            // the current `side` can stop.
            break;
        }
    }

    // The side that ran out of good attackers loses.  We win if the final `side`
    // is NOT the original attacker's side.
    side_to_move != attacker_side
}

#[cfg(test)]
mod tests {
    use chess::{
        board::Board,
        move_generation::{legal::generate_moves_with_metadata, metadata, move_filter::MoveFilter},
    };

    use super::see;

    /// Parse `fen`, find the legal move whose long-algebraic notation is `uci_move`,
    /// and assert that `see(board, mv, threshold) == expected`.
    #[allow(clippy::panic)]
    fn assert_see(fen: &str, uci_move: &str, threshold: i32, expected: bool) {
        let board = Board::from_fen(fen).unwrap();
        let meta = metadata::compute(&board);
        let all = generate_moves_with_metadata(&board, MoveFilter::All, &meta);
        let mv = *all
            .iter()
            .find(|m| m.to_long_algebraic() == uci_move)
            .unwrap_or_else(|| panic!("move {uci_move} not found in {fen}"));

        let result = see(&board, mv, threshold);
        assert_eq!(
            result, expected,
            "SEE({uci_move}, threshold={threshold}) in {fen}: got {result}, expected {expected}"
        );
    }

    // ─── Trivial single-capture tests ───────────────────────────────────────────

    /// PxP — equal pawn trade, no defenders.
    /// SEE = 100 (White pawn takes Black pawn; pawn stays on d5, Black can't recapture).
    #[test]
    fn pawn_takes_pawn_no_defender() {
        let fen = "8/8/8/3p4/4P3/8/8/K6k w - - 0 1";
        assert_see(fen, "e4d5", 0, true);
        assert_see(fen, "e4d5", 100, true); // SEE 100 >= 100
        assert_see(fen, "e4d5", 101, false); // SEE 100 < 101
    }

    /// PxQ — undefended queen.
    /// SEE = 900 (White pawn takes queen; pawn stays alive, Black can't recapture).
    #[test]
    fn pawn_takes_queen_no_defender() {
        let fen = "8/8/8/3q4/4P3/8/8/K6k w - - 0 1";
        assert_see(fen, "e4d5", 0, true);
        assert_see(fen, "e4d5", 900, true); // SEE 900 >= 900
        assert_see(fen, "e4d5", 901, false); // SEE 900 < 901
    }

    // ─── Losing-exchange tests ───────────────────────────────────────────────────

    /// NxP with a pawn defender — classic losing exchange.
    /// SEE = 100 − 320 = −220 < 0.
    #[test]
    fn knight_takes_pawn_pawn_defends() {
        // White knight on f3, Black pawn on e5, Black pawn on d6 defends e5.
        let fen = "8/8/3p4/4p3/8/5N2/8/K6k w - - 0 1";
        assert_see(fen, "f3e5", 0, false);
    }

    /// QxP where the pawn is defended by another pawn.
    /// SEE = 100 − 900 = −800 < 0.
    #[test]
    fn queen_takes_pawn_pawn_defends() {
        // White queen on e1, Black pawn on e5, Black pawn on d6 defends e5.
        let fen = "8/8/3p4/4p3/8/8/8/K3Q2k w - - 0 1";
        assert_see(fen, "e1e5", 0, false);
    }

    // ─── X-ray tests ────────────────────────────────────────────────────────────

    /// Rook-behind-rook x-ray on the a-file.
    ///
    /// White Ra2 (attacker) captures Black Pa5; Black Ra8 recaptures;
    /// White Ra1 (x-ray behind Ra2, now exposed) recaptures.
    /// Net SEE = 100 (pawn) − 500 (Ra2) + 500 (Ra1 takes Ra8) = 100.
    #[test]
    fn rook_xray_orthogonal() {
        // Ra2 is the attacker; Ra1 is the x-ray piece behind it on the a-file.
        let fen = "r7/8/8/p7/8/8/R7/R5K1 w - - 0 1";
        assert_see(fen, "a2a5", 0, true);
        assert_see(fen, "a2a5", 100, true); // SEE 100 >= 100
        assert_see(fen, "a2a5", 101, false); // SEE 100 < 101
    }

    /// Bishop-behind-queen diagonal x-ray.
    ///
    /// White Qc3 takes Black Pe5 (100); Black Bf6 recaptures (−900);
    /// White Bb2 (x-ray on b2-e5 diagonal) recaptures (+330); Black no more.
    /// Net SEE = 100 − 900 + 330 − 330 = −800 < 0.
    #[test]
    fn bishop_xray_diagonal() {
        // White queen on c3, White bishop on b2 (x-ray along b2-e5 diagonal).
        // Black pawn on e5, Black bishop on f6 defends e5.
        let fen = "8/8/5b2/4p3/8/2Q5/1B6/7K w - - 0 1";
        assert_see(fen, "c3e5", 0, false);
    }

    // ─── King legality test ──────────────────────────────────────────────────────

    /// The opponent king cannot legally recapture on a square still defended by us.
    ///
    /// White Qc3 takes Black Pe5 (100 gain). Black King on f4 is the only Black attacker
    /// of e5, but White Bishop on h2 covers e5 diagonally — the king legality break fires.
    /// SEE = 100 (pawn gained for free, opponent king can't step in).
    #[test]
    fn king_cannot_recapture_defended_square() {
        let fen = "8/8/8/4p3/5k2/2Q5/7B/K7 w - - 0 1";
        assert_see(fen, "c3e5", 0, true);
        assert_see(fen, "c3e5", 100, true);
        assert_see(fen, "c3e5", 101, false); // victim (100) - threshold (101) < 0: first check
    }

    // ─── En-passant test ─────────────────────────────────────────────────────────

    /// En-passant capture: the captured pawn is not on the target square.
    /// SEE = 100 (White pawn gains a pawn; Black has nothing to recapture).
    #[test]
    fn en_passant_captures_pawn() {
        // White pawn on e5, Black pawn on d5 (just double-pushed → d6 is EP square).
        let fen = "8/8/8/3pP3/8/8/8/K6k w - d6 0 1";
        assert_see(fen, "e5d6", 0, true);
        assert_see(fen, "e5d6", 100, true);
        assert_see(fen, "e5d6", 101, false);
    }

    // ─── Promotion-capture tests ─────────────────────────────────────────────────

    /// Pawn captures rook and promotes to queen; nobody can recapture.
    /// SEE = Rook (500) + promo bonus Queen−Pawn (800) = 1300.
    #[test]
    fn pawn_captures_rook_promotes_queen() {
        let fen = "r7/1P6/8/8/8/8/8/K6k w - - 0 1";
        assert_see(fen, "b7a8q", 0, true);
        assert_see(fen, "b7a8q", 1300, true);
        assert_see(fen, "b7a8q", 1301, false);
    }

    /// Pawn captures rook, promotes to queen; Black queen recaptures on a8.
    /// SEE = 500 + 800 − 900 = 400.
    #[test]
    fn pawn_captures_rook_promotes_queen_opponent_recaptures() {
        // White Pb7 captures Black Ra8 and promotes. Black Qa1 recaptures along the a-file.
        // White king on h2 (not in check from either Black piece).
        let fen = "r7/1P6/8/8/8/8/7K/q7 w - - 0 1";
        assert_see(fen, "b7a8q", 0, true);
        assert_see(fen, "b7a8q", 400, true); // SEE 400 >= 400
        assert_see(fen, "b7a8q", 401, false); // SEE 400 < 401
    }

    // ─── Castle smoke test ───────────────────────────────────────────────────────

    /// Castling involves no capture: SEE = 0.
    #[test]
    fn castling_no_capture() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        assert_see(fen, "e1g1", 0, true); // 0 >= 0
        assert_see(fen, "e1g1", 1, false); // 0 < 1
    }
}
