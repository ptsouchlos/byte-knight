// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{
    definitions::NumberOf,
    pieces::Piece,
    rank::Rank,
    side::Side,
    square::{self},
};

use crate::{
    phased_score::{PhasedScore, S},
    score::ScoreType,
    traits::EvalValues,
};

/// Game phase increment for each piece
/// Ordered to match the indexing of [`Piece`]
/// King, Queen, Rook, Bishop, Knight, Pawn
pub const GAME_PHASE_INC: [ScoreType; 6] = [0, 4, 2, 1, 1, 0];

/// Maximum game phase
pub const GAME_PHASE_MAX: i32 = 24;

/// Piece-Square Tables, ordered by the ordinality of the pieces. See ['pieces::Piece']
#[rustfmt::skip]
pub const PSQTS : [[PhasedScore; NumberOf::SQUARES]; NumberOf::PIECE_TYPES] = [
    // King
    [
        S(  32,  -89), S(  30,  -38), S(  34,  -22), S( -88,   23), S( -53,   11), S( -17,   10), S(  42,   -8), S( 194, -123),
        S(-104,   14), S( -38,   44), S( -80,   56), S(  33,   36), S( -17,   57), S(  -8,   69), S(  12,   57), S( -38,   25),
        S(-128,   29), S(  -3,   50), S( -66,   68), S( -87,   80), S( -38,   82), S(  51,   69), S(  -0,   70), S( -32,   34),
        S( -79,   12), S( -64,   45), S( -91,   65), S(-138,   78), S(-122,   78), S( -82,   71), S( -82,   60), S(-133,   38),
        S( -81,   -1), S( -61,   24), S( -85,   47), S(-120,   62), S(-117,   60), S( -72,   44), S( -81,   32), S(-140,   25),
        S( -38,  -13), S(  -3,    4), S( -57,   27), S( -80,   40), S( -69,   38), S( -59,   27), S( -24,    8), S( -49,   -1),
        S(  23,  -29), S( -22,   -2), S( -30,    7), S( -54,   14), S( -56,   20), S( -42,   14), S( -14,   -2), S(  13,  -30),
        S(  -2,  -67), S(   1,  -36), S( -13,  -21), S( -80,  -11), S( -27,  -28), S( -61,   -4), S( -15,  -28), S(   7,  -73),
    ],
    // Queen
    [
        S( 916, 1401), S( 909, 1417), S( 930, 1434), S( 956, 1417), S( 951, 1419), S( 960, 1419), S(1002, 1364), S( 943, 1402),
        S( 957, 1391), S( 937, 1423), S( 933, 1456), S( 919, 1476), S( 912, 1494), S( 962, 1443), S( 970, 1425), S(1009, 1413),
        S( 961, 1404), S( 958, 1418), S( 957, 1450), S( 956, 1455), S( 961, 1462), S( 991, 1442), S( 999, 1416), S( 984, 1413),
        S( 947, 1420), S( 956, 1430), S( 952, 1440), S( 948, 1462), S( 955, 1461), S( 960, 1450), S( 969, 1449), S( 966, 1427),
        S( 955, 1406), S( 946, 1431), S( 953, 1432), S( 960, 1447), S( 962, 1444), S( 958, 1439), S( 969, 1424), S( 971, 1419),
        S( 953, 1388), S( 959, 1410), S( 963, 1418), S( 956, 1422), S( 965, 1429), S( 968, 1420), S( 978, 1402), S( 972, 1394),
        S( 960, 1375), S( 963, 1381), S( 967, 1388), S( 975, 1395), S( 973, 1400), S( 983, 1369), S( 988, 1346), S( 992, 1323),
        S( 950, 1383), S( 959, 1378), S( 963, 1390), S( 966, 1400), S( 971, 1382), S( 960, 1372), S( 967, 1357), S( 969, 1349),
    ],
    // Rook
    [
        S( 456,  781), S( 451,  794), S( 451,  795), S( 443,  792), S( 454,  781), S( 474,  789), S( 457,  793), S( 450,  784),
        S( 440,  793), S( 451,  807), S( 462,  804), S( 484,  788), S( 461,  790), S( 479,  801), S( 453,  795), S( 462,  778),
        S( 439,  783), S( 470,  788), S( 469,  783), S( 466,  779), S( 498,  763), S( 497,  773), S( 527,  768), S( 473,  764),
        S( 437,  783), S( 453,  785), S( 458,  785), S( 462,  781), S( 466,  765), S( 464,  775), S( 462,  778), S( 450,  767),
        S( 430,  772), S( 431,  783), S( 446,  774), S( 449,  774), S( 453,  767), S( 431,  783), S( 453,  770), S( 437,  760),
        S( 426,  763), S( 434,  769), S( 444,  762), S( 439,  767), S( 452,  756), S( 447,  763), S( 473,  744), S( 452,  740),
        S( 427,  755), S( 435,  766), S( 450,  762), S( 450,  762), S( 457,  751), S( 456,  756), S( 466,  745), S( 434,  744),
        S( 439,  762), S( 446,  763), S( 453,  766), S( 456,  760), S( 465,  751), S( 457,  761), S( 452,  757), S( 444,  745),
    ],
    // Bishop
    [
        S( 306,  433), S( 284,  437), S( 280,  430), S( 233,  441), S( 244,  438), S( 248,  427), S( 295,  427), S( 278,  420),
        S( 321,  416), S( 325,  429), S( 325,  428), S( 312,  430), S( 308,  425), S( 324,  425), S( 298,  433), S( 305,  419),
        S( 333,  433), S( 349,  427), S( 340,  436), S( 343,  427), S( 341,  430), S( 367,  438), S( 352,  432), S( 335,  436),
        S( 319,  430), S( 336,  438), S( 341,  436), S( 359,  453), S( 350,  442), S( 347,  442), S( 330,  436), S( 320,  433),
        S( 329,  426), S( 321,  439), S( 338,  445), S( 359,  445), S( 357,  444), S( 341,  437), S( 338,  432), S( 339,  418),
        S( 325,  426), S( 350,  437), S( 350,  437), S( 348,  441), S( 354,  445), S( 355,  435), S( 355,  427), S( 351,  418),
        S( 343,  431), S( 348,  415), S( 357,  416), S( 343,  427), S( 354,  426), S( 363,  420), S( 370,  421), S( 356,  412),
        S( 334,  417), S( 356,  435), S( 336,  422), S( 332,  426), S( 341,  423), S( 335,  433), S( 347,  416), S( 366,  394),
    ],
    // Knight
    [
        S( 179,  354), S( 205,  402), S( 252,  419), S( 281,  411), S( 316,  414), S( 260,  386), S( 224,  399), S( 222,  336),
        S( 296,  410), S( 315,  423), S( 318,  422), S( 333,  423), S( 328,  414), S( 363,  406), S( 320,  418), S( 329,  392),
        S( 323,  413), S( 337,  421), S( 346,  445), S( 354,  449), S( 365,  442), S( 399,  424), S( 345,  420), S( 347,  405),
        S( 330,  428), S( 336,  436), S( 354,  450), S( 378,  453), S( 351,  458), S( 374,  453), S( 333,  447), S( 360,  423),
        S( 322,  430), S( 332,  428), S( 343,  449), S( 353,  449), S( 360,  453), S( 351,  443), S( 361,  430), S( 335,  429),
        S( 302,  412), S( 322,  418), S( 330,  426), S( 339,  443), S( 355,  440), S( 340,  420), S( 343,  415), S( 327,  418),
        S( 297,  411), S( 309,  418), S( 319,  419), S( 337,  418), S( 338,  417), S( 336,  415), S( 331,  409), S( 328,  422),
        S( 264,  417), S( 302,  404), S( 300,  414), S( 318,  418), S( 325,  418), S( 329,  407), S( 306,  411), S( 301,  416),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 179,  308), S( 178,  309), S( 145,  313), S( 169,  277), S( 133,  295), S( 158,  298), S( 138,  340), S( 124,  328),
        S(  85,  178), S(  76,  198), S( 104,  155), S( 104,  127), S( 102,  133), S( 120,  156), S(  98,  198), S(  62,  185),
        S(  67,  149), S(  72,  151), S(  80,  127), S(  78,  110), S(  98,  111), S(  87,  118), S(  82,  144), S(  63,  129),
        S(  63,  128), S(  66,  138), S(  76,  117), S(  90,  111), S(  90,  112), S(  86,  112), S(  83,  126), S(  63,  112),
        S(  59,  123), S(  68,  132), S(  73,  117), S(  74,  119), S(  84,  123), S(  77,  116), S(  91,  121), S(  63,  109),
        S(  59,  125), S(  67,  134), S(  67,  123), S(  60,  126), S(  73,  134), S(  80,  121), S(  94,  123), S(  52,  113),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-14, 68),
    S(7, 133),
    S(12, 67),
    S(-11, 40),
    S(-13, 14),
    S(-7, 9),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-17, -36),
    S(2, -28),
    S(-3, -20),
    S(-5, -8),
    S(-12, -2),
    S(-6, -16),
    S(1, -23),
    S(-6, -41),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-6, -1),
    S(-8, -18),
    S(-18, -13),
    S(-16, -22),
    S(-20, -24),
    S(-7, -13),
    S(-10, -17),
    S(-0, -2),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(20, 67);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-14, -11), S(-20, 8), S(-23, 6), S(-11, 9), S(-13, 6)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(82, -42), //Queen
    S(90, 4),   //Rook
    S(64, 50),  //Bishop
    S(63, 26),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(55, -24), //Queen
    S(69, 14),  //Rook
    S(33, 36),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(73, 56), //Queen
    S(55, 27), //Rook
    S(0, 0),   //Bishop
    S(23, 23), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-45, -90),
    S(-24, -53),
    S(-12, -31),
    S(-6, -22),
    S(1, -13),
    S(7, -4),
    S(16, -5),
    S(22, -9),
    S(28, -21),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-31, -79),
    S(-20, -58),
    S(-9, -41),
    S(-2, -27),
    S(4, -16),
    S(8, -5),
    S(11, -1),
    S(13, 3),
    S(14, 6),
    S(18, 3),
    S(25, -2),
    S(29, -1),
    S(26, 8),
    S(39, -17),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-37, -89),
    S(-29, -66),
    S(-25, -63),
    S(-21, -58),
    S(-24, -50),
    S(-18, -46),
    S(-17, -39),
    S(-14, -38),
    S(-11, -35),
    S(-9, -30),
    S(-7, -28),
    S(-9, -23),
    S(-6, -21),
    S(-7, -22),
    S(-16, -22),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-37, -253),
    S(-37, -212),
    S(-47, -141),
    S(-44, -118),
    S(-42, -101),
    S(-38, -96),
    S(-35, -80),
    S(-34, -67),
    S(-31, -59),
    S(-29, -56),
    S(-28, -49),
    S(-25, -43),
    S(-24, -42),
    S(-24, -36),
    S(-22, -35),
    S(-20, -32),
    S(-20, -24),
    S(-18, -26),
    S(-10, -29),
    S(4, -39),
    S(8, -39),
    S(59, -71),
    S(52, -67),
    S(70, -88),
    S(204, -142),
    S(215, -172),
    S(156, -115),
    S(91, -104),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(29, 26);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(32, 8),
    S(29, 1),
    S(24, 12),
    S(29, 11),
    S(30, 17),
    S(46, -1),
    S(60, -4),
    S(84, -2),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(2, 49),
    S(7, 19),
    S(7, 19),
    S(17, 10),
    S(7, 14),
    S(22, -2),
    S(28, 2),
    S(7, 33),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(65, -21), S(52, -9), S(39, -6), S(30, 7)],
    // Left adjacent
    [S(43, -10), S(21, -6), S(16, -3), S(14, 6)],
    // Right adjacent
    [S(40, -20), S(32, -8), S(22, -0), S(14, 8)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(100, 224), S(-42, 117), S(-4, 24), S(9, 4)],
    // Left adjacent
    [S(-19, 205), S(-70, 99), S(-26, 25), S(-3, 4)],
    // Right adjacent
    [S(-49, 231), S(-54, 85), S(-22, 22), S(-3, 3)],
];

// Bonus for Rook on 7th Rank. Bonus scaled based on Rook's File. Separate
// bonus if enemy king is in Rank 8. Flipped for black
pub const ROOK_RANK_BONUS: [[PhasedScore; NumberOf::FILES]; 2] = [
    [
        S(9, -19),
        S(20, -21),
        S(19, -23),
        S(20, -17),
        S(25, -23),
        S(13, -24),
        S(16, -19),
        S(-17, -9),
    ],
    [
        S(-5, 16),
        S(-17, 18),
        S(-13, 19),
        S(-24, 18),
        S(-19, 16),
        S(-14, 15),
        S(-2, 28),
        S(-4, 38),
    ],
];

const RANK_1: u8 = 1;

#[derive(Debug, Clone, Copy, Default)]
pub struct ByteKnightValues {}

impl EvalValues for ByteKnightValues {
    type ReturnScore = PhasedScore;

    fn psqt(&self, square: u8, piece: Piece, side: Side) -> Self::ReturnScore {
        PSQTS[piece as usize][square::flip_if(side == Side::White, square) as usize]
    }

    fn passed_pawn_bonus(&self, square: u8, side: Side) -> Self::ReturnScore {
        let (_file, rank) = square::from_square(square::flip_if(side == Side::White, square));
        PASSED_PAWN_BONUS[(rank - RANK_1) as usize]
    }

    fn doubled_pawn_value(&self, square: u8, side: Side) -> Self::ReturnScore {
        let (file, _rank) = square::from_square(square::flip_if(side == Side::White, square));
        DOUBLED_PAWN_VALUES[file as usize]
    }

    fn isolated_pawn_value(&self, square: u8, side: Side) -> Self::ReturnScore {
        let (file, _rank) = square::from_square(square::flip_if(side == Side::White, square));
        ISOLATED_PAWN_VALUES[file as usize]
    }

    fn mobility_value(&self, piece: Piece, count: usize, _side: Side) -> Self::ReturnScore {
        match piece {
            Piece::Knight => KNIGHT_MOBILITY[count],
            Piece::Bishop => BISHOP_MOBILITY[count],
            Piece::Rook => ROOK_MOBILITY[count],
            Piece::Queen => QUEEN_MOBILITY[count],
            _ => S(0, 0),
        }
    }

    fn bishop_pair_bonus_value(&self, _side: Side) -> Self::ReturnScore {
        BISHOP_PAIR_BONUS
    }

    fn king_safety_value(&self, piece: Piece, _side: Side) -> Self::ReturnScore {
        assert!(piece != Piece::King);
        KING_SAFETY[piece as usize - 1]
    }

    fn threat_value(&self, piece: Piece, attacked_piece: Piece, _side: Side) -> Self::ReturnScore {
        match piece {
            Piece::Pawn => PAWN_THREAT[attacked_piece as usize],
            Piece::Knight => KNIGHT_THREAT[attacked_piece as usize],
            Piece::Bishop => BISHOP_THREAT[attacked_piece as usize],
            _ => S(0, 0),
        }
    }

    fn tempo_bonus(&self, _side: Side) -> Self::ReturnScore {
        TEMPO_BONUS
    }

    fn open_file_bonus(&self, square: u8, _side: Side) -> Self::ReturnScore {
        let (file, _rank) = square::from_square(square);
        ROOK_OPEN_FILE_BONUS[file as usize]
    }

    fn semi_open_file_bonus(&self, square: u8, _side: Side) -> Self::ReturnScore {
        let (file, _rank) = square::from_square(square);
        ROOK_SEMI_OPEN_FILE_BONUS[file as usize]
    }

    fn pawn_shield_value(
        &self,
        file_index: usize,
        rank_index: usize,
        _side: Side,
    ) -> Self::ReturnScore {
        PAWN_SHIELD[file_index][rank_index]
    }

    fn pawn_storm_value(
        &self,
        file_index: usize,
        rank_index: usize,
        _side: Side,
    ) -> Self::ReturnScore {
        PAWN_STORM[file_index][rank_index]
    }

    fn rook_rank_bonus(
        &self,
        rook_square: u8,
        enemy_king_square: u8,
        side: Side,
    ) -> Self::ReturnScore {
        let (rook_file, rook_rank) = square::from_square(rook_square);
        let bonus_rank = match side {
            Side::White => Rank::R7,
            Side::Black => Rank::R2,
        };
        if rook_rank != bonus_rank.as_number() {
            return Default::default();
        }

        let king_rank = Rank::of(enemy_king_square);
        let king_rank_num = king_rank.as_number();

        let is_king_ahead = match side {
            Side::White => king_rank_num > rook_rank,
            Side::Black => king_rank_num < rook_rank,
        };

        ROOK_RANK_BONUS[is_king_ahead as usize][rook_file as usize]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mobility() {
        let values = ByteKnightValues::default();
        let score = values.mobility_value(Piece::Pawn, 3, Side::White);
        assert_eq!(score, S(0, 0));
    }
}
