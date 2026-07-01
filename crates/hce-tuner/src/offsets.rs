// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use chess::{definitions::NumberOf, pieces::Piece, side::Side, square};

pub(crate) struct Offsets;

macro_rules! offsets {
    ($($name:ident : $size:expr),* $(,)?) => {
        offsets!(@acc 0usize; $($name : $size),*);
    };
    (@acc $acc:expr; $name:ident : $size:expr $(, $rest_name:ident : $rest_size:expr)*) => {
        pub const $name: usize = $acc;
        offsets!(@acc $acc + $size; $($rest_name : $rest_size),*);
    };
    (@acc $acc:expr;) => {
        pub const PARAMETER_COUNT: usize = $acc;
    };
}

impl Offsets {
    offsets!(
        PSQT:          64 * NumberOf::PIECE_TYPES,
        PASSED_PAWN:   NumberOf::PASSED_PAWN_RANKS,
        DOUBLED_PAWN:  NumberOf::FILES,
        ISOLATED_PAWN: NumberOf::FILES,
        BISHOP_PAIR:   1,
        KING_SAFETY:   NumberOf::PIECE_TYPES - 1,
        PAWN_THREAT:   NumberOf::PIECE_TYPES,
        KNIGHT_THREAT: NumberOf::PIECE_TYPES,
        BISHOP_THREAT: NumberOf::PIECE_TYPES,
        KNIGHT_MOBILITY: NumberOf::KNIGHT_MOVES +1,
        BISHOP_MOBILITY: NumberOf::BISHOP_MOVES +1,
        ROOK_MOBILITY: NumberOf::ROOK_MOVES +1,
        QUEEN_MOBILITY: NumberOf::QUEEN_MOVES +1,
        TEMPO_BONUS: 1,
        ROOK_OPEN_FILE_BONUS: NumberOf::FILES,
        ROOK_SEMI_OPEN_FILE_BONUS: NumberOf::FILES,
        PAWN_SHIELD: NumberOf::KING_FLANK_FILES * NumberOf::PAWN_SHIELD_RANKS,
        PAWN_STORM:  NumberOf::KING_FLANK_FILES * NumberOf::PAWN_STORM_RANKS,
    );

    pub(crate) fn offset_for_piece_and_square(square: usize, piece: Piece, side: Side) -> usize {
        Self::PSQT
            + (piece as usize * NumberOf::SQUARES)
            + square::flip_if(side == Side::White, square as u8) as usize
    }

    pub(crate) fn offset_for_passed_pawn(square: usize, side: Side) -> usize {
        let (_file, rank) = square::from_square(square::flip_if(side == Side::White, square as u8));
        // Note: File and rank are 0 based
        Self::PASSED_PAWN + (rank - 1) as usize
    }

    pub(crate) fn offset_for_doubled_pawn(square: usize, side: Side) -> usize {
        let (file, _rank) = square::from_square(square::flip_if(side == Side::White, square as u8));
        Self::DOUBLED_PAWN + file as usize
    }

    pub(crate) fn offset_for_isolated_pawn(square: usize, side: Side) -> usize {
        let (file, _rank) = square::from_square(square::flip_if(side == Side::White, square as u8));
        Self::ISOLATED_PAWN + file as usize
    }

    pub(crate) fn offset_for_mobility(piece: Piece, mobility: usize) -> usize {
        assert!(
            matches!(
                piece,
                Piece::Knight | Piece::Bishop | Piece::Rook | Piece::Queen
            ),
            "Mobility is only defined for Knight, Bishop, Rook, and Queen"
        );

        let base_offset = match piece {
            Piece::Knight => Self::KNIGHT_MOBILITY,
            Piece::Bishop => Self::BISHOP_MOBILITY,
            Piece::Rook => Self::ROOK_MOBILITY,
            Piece::Queen => Self::QUEEN_MOBILITY,
            _ => unreachable!(),
        };

        base_offset + mobility
    }

    pub(crate) fn offset_for_bishop_pair() -> usize {
        Self::BISHOP_PAIR
    }

    pub(crate) fn offset_for_king_safety(piece: Piece) -> usize {
        assert_ne!(
            piece,
            Piece::King,
            "Cannot check safety if attacker if King"
        );
        Self::KING_SAFETY + piece as usize - 1
    }

    pub(crate) fn offset_for_threat(piece: Piece, attacked_piece: Piece) -> usize {
        assert_ne!(
            piece,
            Piece::King,
            "Cannot check safety if attacker if King"
        );
        assert_ne!(
            attacked_piece,
            Piece::King,
            "Cannot check safety if attacked piece is King"
        );
        let base_offset = match piece {
            Piece::Pawn => Self::PAWN_THREAT,
            Piece::Knight => Self::KNIGHT_THREAT,
            Piece::Bishop => Self::BISHOP_THREAT,
            _ => unreachable!(),
        };
        base_offset + attacked_piece as usize
    }

    pub(crate) fn offset_for_tempo_bonus() -> usize {
        Self::TEMPO_BONUS
    }

    pub(crate) fn offset_for_rook_open_file(square: u8) -> usize {
        let (file, _rank) = square::from_square(square);
        Self::ROOK_OPEN_FILE_BONUS + file as usize
    }

    pub(crate) fn offset_for_rook_semi_open_file(square: u8) -> usize {
        let (file, _rank) = square::from_square(square);
        Self::ROOK_SEMI_OPEN_FILE_BONUS + file as usize
    }

    pub(crate) fn offset_for_pawn_shield(file_index: usize, rank_index: usize) -> usize {
        Self::PAWN_SHIELD + file_index * NumberOf::PAWN_SHIELD_RANKS + rank_index
    }

    pub(crate) fn offset_for_pawn_storm(file_index: usize, rank_index: usize) -> usize {
        Self::PAWN_STORM + file_index * NumberOf::PAWN_STORM_RANKS + rank_index
    }
}

pub const PARAMETER_COUNT: usize = Offsets::PARAMETER_COUNT;

#[cfg(test)]
mod tests {
    use chess::{file::File, rank::Rank, square::Square};

    use super::*;

    #[test]
    fn offset_calculation() {
        // verify that offset calculation is correct
        let sq = 33;
        let piece = Piece::Pawn;
        let offset = Offsets::offset_for_piece_and_square(sq, piece, Side::Black);
        assert_eq!(353, offset);
        let offset = Offsets::offset_for_piece_and_square(sq, piece, Side::White);
        assert_eq!(345, offset);
    }

    #[test]
    fn offset_pawn_calculation() {
        let file = File::C;
        let rank = Rank::R2;

        let sq = Square::from_file_rank(file.to_char(), rank.inner()).unwrap();
        let offset = Offsets::offset_for_passed_pawn(sq.inner() as usize, Side::Black);
        assert_eq!(Offsets::PASSED_PAWN, offset);
        let offset = Offsets::offset_for_passed_pawn(sq.inner() as usize, Side::White);
        assert_eq!(389, offset);

        let doubled_offset = Offsets::offset_for_doubled_pawn(sq.inner() as usize, Side::White);
        assert_eq!(Offsets::DOUBLED_PAWN + file as usize, doubled_offset);

        let double_offset_2 = Offsets::offset_for_doubled_pawn(sq.inner() as usize, Side::Black);
        assert_eq!(Offsets::DOUBLED_PAWN + file as usize, double_offset_2);

        let isolated_offset = Offsets::offset_for_isolated_pawn(sq.inner() as usize, Side::White);
        assert_eq!(Offsets::ISOLATED_PAWN + file as usize, isolated_offset);
        let isolated_offset_2 = Offsets::offset_for_isolated_pawn(sq.inner() as usize, Side::Black);
        assert_eq!(Offsets::ISOLATED_PAWN + file as usize, isolated_offset_2);

        let bishop_pair_offset = Offsets::offset_for_bishop_pair();
        assert_eq!(Offsets::BISHOP_PAIR, bishop_pair_offset);

        for piece in Piece::iter().filter(|&p| p != Piece::King) {
            let king_offset = Offsets::offset_for_king_safety(piece);
            assert!(king_offset >= Offsets::KING_SAFETY);
            assert!(king_offset < PARAMETER_COUNT);
        }
    }

    #[test]
    fn offsets_for_threats() {
        let offset = Offsets::offset_for_threat(Piece::Pawn, Piece::Queen);
        assert_eq!(offset, Offsets::PAWN_THREAT + Piece::Queen as usize);

        let offset = Offsets::offset_for_threat(Piece::Knight, Piece::Rook);
        assert_eq!(offset, Offsets::KNIGHT_THREAT + Piece::Rook as usize);

        let offset = Offsets::offset_for_threat(Piece::Bishop, Piece::Knight);
        assert_eq!(offset, Offsets::BISHOP_THREAT + Piece::Knight as usize);
    }
}
