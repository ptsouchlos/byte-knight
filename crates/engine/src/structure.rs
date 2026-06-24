use chess::{bitboard::Bitboard, bitboard_helpers, board::Board, pieces::Piece, side::Side};

/// Calculate the kings pawn shield and storm for the given side.
///
/// # Arguments
/// - board: The [`Board`] to analyze.
/// - side: The [`Side`] to find the pawn shield and storm for.
///
/// # Returns
/// (pawn shield (friendly), pawn storm (enemy)) pair of [`Bitboard`]s.
pub(crate) fn king_pawn_shield_and_storm(board: &Board, side: Side) -> (Bitboard, Bitboard) {
    let king_sq = board.king_square(side);
    let our_pawns = board.piece_bitboard(Piece::Pawn, side);
    let their_pawns = board.piece_bitboard(Piece::Pawn, side.opposite());

    let king_sq_bb = Bitboard::from_square(king_sq);
    let king_sq_adjacent_bb =
        king_sq_bb | bitboard_helpers::west(king_sq_bb) | bitboard_helpers::east(king_sq_bb);

    let king_files_bb = match side {
        Side::White => bitboard_helpers::north_fill(king_sq_adjacent_bb),
        Side::Black => bitboard_helpers::south_fill(king_sq_adjacent_bb),
    };

    // This fill let's us filter out all but the closest friendly pawns.
    let friendly_pawn_fill = match side {
        Side::White => bitboard_helpers::north_fill(bitboard_helpers::north(our_pawns)),
        Side::Black => bitboard_helpers::south_fill(bitboard_helpers::south(our_pawns)),
    };

    // Pawn shield: Closest pawns to our king on it's file and adjacent ones.
    let friendly_on_file = (our_pawns & !friendly_pawn_fill) & king_files_bb;

    // This fill lets us filter out all but the closest enemy pawns.
    let enemy_pawn_fill = match side {
        Side::White => bitboard_helpers::north_fill(bitboard_helpers::north(their_pawns)),
        Side::Black => bitboard_helpers::south_fill(bitboard_helpers::south(their_pawns)),
    };

    // Pawn Storm: Closest enemy pawns on our king's file and adjacent ones.
    let enemy_on_file = (their_pawns & !enemy_pawn_fill) & king_files_bb;

    (friendly_on_file, enemy_on_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_detect_closest_pawns_on_file() {
        let overlapped_pawns = Board::from_fen("4k3/5pp1/6p1/8/8/5PP1/6P1/6K1 w - - 0 1").unwrap();
        let no_overlapped_pawns = Board::from_fen("4k3/5p2/6p1/8/8/5P2/6P1/6K1 w - - 0 1").unwrap();

        let (pawn_shield_w, pawn_storm_w) =
            king_pawn_shield_and_storm(&overlapped_pawns, Side::White);
        let (pawn_shield_b, pawn_storm_b) =
            king_pawn_shield_and_storm(&overlapped_pawns, Side::Black);

        let (pawn_shield_no_overlap_w, pawn_storm_no_overlap_w) =
            king_pawn_shield_and_storm(&no_overlapped_pawns, Side::White);
        let (pawn_shield_no_overlap_b, pawn_storm_no_overlap_b) =
            king_pawn_shield_and_storm(&no_overlapped_pawns, Side::Black);

        assert_eq!(pawn_shield_w, pawn_shield_no_overlap_w);
        assert_eq!(pawn_storm_w, pawn_storm_no_overlap_w);
        assert_eq!(pawn_shield_b, pawn_shield_no_overlap_b);
        assert_eq!(pawn_storm_b, pawn_storm_no_overlap_b);
    }
}
