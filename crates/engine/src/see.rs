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
pub(crate) fn piece_value(piece: Piece) -> i32 {
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

pub(crate) fn move_gain(board: &Board, mv: &Move) -> i32 {
    if mv.is_en_passant_capture() {
        return piece_value(Piece::Pawn);
    }
    board.captured(mv).map_or(0, piece_value)
}

fn move_value(board: &Board, mv: Move) -> i32 {
    let mut balance = move_gain(board, &mv);
    if let Some(promo_piece) = mv.promotion_piece() {
        // The pawn is spent to produce the promoted piece, so net gain is
        // `captured + (promo - pawn)`.
        balance += piece_value(promo_piece) - piece_value(Piece::Pawn);
    }
    balance
}

/// Returns `true` if the static exchange on `mv` gains at least `threshold` centipawns
/// for the side to move. Castle and quiet moves are treated as no capture and return threshold <= 0.
///
/// # Arguments
/// - `board`: The current board state.
/// - `mv`: The capturing move to evalulate.
/// - `threshold`: The SEE threshold value to compare against.
///
/// # Returns
/// True if the current move >= the threshold, false otherwise.
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

    // After the capture, our own piece sits on `to` and is what the opponent
    // will recapture. For promotions the pawn has already become the promoted
    // piece, so that's the one at risk.
    let next_victim = mv.promotion_piece().unwrap_or(attacker_piece);

    // Initial balance: what we gain on this move minus the threshold.
    let mut balance = move_value(board, mv) - threshold;
    if balance < 0 {
        return false; // Even winning the victim uncontested doesn't meet the threshold.
    }

    // Subtract the value of the piece we risk losing to a recapture.
    balance -= piece_value(next_victim);
    if balance >= 0 {
        // We're up enough that even losing our piece back doesn't hurt.
        return true;
    }

    // Seed occupancy: remove the capturing piece so sliders behind it are exposed.
    let mut occ = board.all_pieces() ^ Bitboard::from(from);

    // For en-passant the captured pawn is not on `to`; remove it explicitly so that
    // sliders behind it can be discovered in the swap loop.
    if mv.is_en_passant_capture() {
        let ep_pawn_sq = attacks::ep_capture_square(to, attacker_side)
            .unwrap_or_else(|| unreachable!("EP capture square invalid for move {mv:?}"));
        occ ^= Bitboard::from(ep_pawn_sq);
    } else {
        // Remove to as that's the target square
        occ ^= Bitboard::from(to);
    }

    // Opponent gets to recapture first.
    let mut side_to_move = attacker_side.opposite();

    loop {
        // All pieces of the side to move that still attack `to` with the current occupancy.
        let attackers = attacks::all_attackers_of(to, board, side_to_move, occ) & occ;
        if attackers.is_empty() {
            // No more attackers
            break;
        }

        // Find and remove the least-valuable attacker from occupancy.
        let least_valueable_attacker = match pop_lva(attackers, board, side_to_move, &mut occ) {
            Some(p) => p,
            None => break,
        };

        // A king cannot legally move to a square attacked by the opponent.  If the king
        // is the only remaining attacker and the opponent still has defenders, the side
        // forfeits further captures.
        if least_valueable_attacker == Piece::King {
            let opp_defenders =
                attacks::all_attackers_of(to, board, side_to_move.opposite(), occ) & occ;
            if !opp_defenders.is_empty() {
                break;
            }
        }

        // Negamax balance update: flip perspective and charge the attacker's value.
        side_to_move = side_to_move.opposite();
        balance = -balance - 1 - piece_value(least_valueable_attacker);
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
        pieces::Piece,
    };

    use crate::see::piece_value;

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

    /// PxP — equal pawn trade, no defenders.
    /// White pawn takes Black pawn; pawn stays on d5, Black can't recapture.
    #[test]
    fn pawn_takes_pawn_no_defender() {
        let fen = "8/8/8/3p4/4P3/8/8/K6k w - - 0 1";
        assert_see(fen, "e4d5", 0, true);
        assert_see(fen, "e4d5", piece_value(Piece::Pawn), true);
        assert_see(fen, "e4d5", piece_value(Piece::Pawn) + 1, false);
    }

    /// PxQ — undefended queen.
    /// White pawn takes queen; pawn stays alive, Black can't recapture
    #[test]
    fn pawn_takes_queen_no_defender() {
        let fen = "8/8/8/3q4/4P3/8/8/K6k w - - 0 1";
        assert_see(fen, "e4d5", 0, true);
        assert_see(fen, "e4d5", piece_value(Piece::Queen), true);
        assert_see(fen, "e4d5", piece_value(Piece::Queen) + 1, false);
    }

    /// NxP with a pawn defender — classic losing exchange.
    /// SEE < 0
    #[test]
    fn knight_takes_pawn_pawn_defends() {
        // White knight on f3, Black pawn on e5, Black pawn on d6 defends e5.
        let fen = "8/8/3p4/4p3/8/5N2/8/K6k w - - 0 1";
        // This is a losing capture because black recaptures the knight
        assert_see(fen, "f3e5", 0, false);
    }

    /// QxP where the pawn is defended by another pawn.
    /// SEE < 0
    #[test]
    fn queen_takes_pawn_pawn_defends() {
        // White queen on e1, Black pawn on e5, Black pawn on d6 defends e5.
        let fen = "8/8/3p4/4p3/8/8/8/K3Q2k w - - 0 1";
        assert_see(fen, "e1e5", 0, false);
    }

    /// Rook-behind-rook x-ray on the a-file.
    ///
    /// White Ra2 (attacker) captures Black Pa5; Black Ra8 recaptures;
    /// White Ra1 (x-ray behind Ra2, now exposed) recaptures.
    /// Net SEE = pawn − rook + rook = pawn value.
    #[test]
    fn rook_xray_orthogonal() {
        let fen = "r7/8/8/p7/8/8/R7/R5K1 w - - 0 1";
        assert_see(fen, "a2a5", 0, true);
        assert_see(fen, "a2a5", piece_value(Piece::Pawn), true);
        assert_see(fen, "a2a5", piece_value(Piece::Pawn) + 1, false);
    }

    /// Bishop-behind-queen diagonal x-ray.
    ///
    /// White Qc3 takes Black Pe5 (100); Black Bf6 recaptures (−900);
    /// White Bb2 (x-ray on b2-e5 diagonal) recaptures (+330); Black no more.
    /// Net SEE = pawn − queen + bishop - bishop < 0.
    #[test]
    fn bishop_xray_diagonal() {
        // White queen on c3, White bishop on b2 (x-ray along b2-e5 diagonal).
        // Black pawn on e5, Black bishop on f6 defends e5.
        let fen = "8/8/5b2/4p3/8/2Q5/1B6/7K w - - 0 1";
        assert_see(fen, "c3e5", 0, false);
    }

    /// The opponent king cannot legally recapture on a square still defended by us.
    ///
    /// White Qc3 takes Black Pe5 (100 gain). Black King on f4 is the only Black attacker
    /// of e5, but White Bishop on h2 covers e5 diagonally — the king legality break fires.
    /// SEE = pawn value (pawn gained for free, opponent king can't step in due to two checkers)
    #[test]
    fn king_cannot_recapture_defended_square() {
        let fen = "8/8/8/4p3/5k2/2Q5/7B/K7 w - - 0 1";
        assert_see(fen, "c3e5", 0, true);
        assert_see(fen, "c3e5", piece_value(Piece::Pawn), true);
        assert_see(fen, "c3e5", piece_value(Piece::Pawn) + 1, false);
    }

    /// En-passant capture: the captured pawn is not on the target square.
    /// SEE = pawn value (White pawn gains a pawn; Black has nothing to recapture).
    #[test]
    fn en_passant_captures_pawn() {
        // White pawn on e5, Black pawn on d5 (just double-pushed → d6 is EP square).
        let fen = "8/8/8/3pP3/8/8/8/K6k w - d6 0 1";
        assert_see(fen, "e5d6", 0, true);
        assert_see(fen, "e5d6", piece_value(Piece::Pawn), true);
        assert_see(fen, "e5d6", piece_value(Piece::Pawn) + 1, false);
    }

    /// Pawn captures rook and promotes to queen; nobody can recapture.
    /// SEE = Rook + promo bonus to queen = rook value + queen value - pawn value
    #[test]
    fn pawn_captures_rook_promotes_queen() {
        let fen = "r7/1P6/8/8/8/8/8/K6k w - - 0 1";
        let expected_value =
            piece_value(Piece::Rook) + piece_value(Piece::Queen) - piece_value(Piece::Pawn);
        assert_see(fen, "b7a8q", 0, true);
        assert_see(fen, "b7a8q", expected_value, true);
        assert_see(fen, "b7a8q", expected_value + 1, false);
    }

    /// Pawn captures rook, promotes to queen; Black queen recaptures on a8.
    /// SEE = rook + queen promo - pawn - queen
    #[test]
    fn pawn_captures_rook_promotes_queen_opponent_recaptures() {
        // White Pb7 captures Black Ra8 and promotes. Black Qa1 recaptures along the a-file.
        // White king on h2 (not in check from either Black piece).
        let expected_value = piece_value(Piece::Rook) + piece_value(Piece::Queen)
            - piece_value(Piece::Pawn)
            - piece_value(Piece::Queen);
        let fen = "r7/1P6/8/8/8/8/7K/q7 w - - 0 1";
        assert_see(fen, "b7a8q", 0, true);
        assert_see(fen, "b7a8q", expected_value, true);
        assert_see(fen, "b7a8q", expected_value + 1, false);
    }

    /// Castling involves no capture: SEE = 0.
    #[test]
    fn castling_no_capture() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        assert_see(fen, "e1g1", 0, true); // 0 >= 0
        assert_see(fen, "e1g1", 1, false); // 0 < 1
    }
}
