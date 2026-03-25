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
        S(  32,  -89), S(  31,  -37), S(  35,  -21), S( -89,   24), S( -54,   12), S( -17,   11), S(  42,   -8), S( 194, -124),
        S(-105,   14), S( -37,   46), S( -79,   57), S(  34,   37), S( -16,   59), S(  -7,   70), S(  13,   58), S( -39,   26),
        S(-128,   30), S(  -1,   51), S( -63,   69), S( -85,   81), S( -35,   83), S(  55,   70), S(   3,   71), S( -31,   34),
        S( -79,   13), S( -62,   46), S( -89,   66), S(-136,   80), S(-119,   79), S( -80,   72), S( -79,   61), S(-134,   38),
        S( -82,   -0), S( -61,   25), S( -85,   48), S(-120,   64), S(-117,   61), S( -72,   45), S( -82,   33), S(-143,   26),
        S( -40,  -12), S(  -4,    5), S( -58,   28), S( -81,   41), S( -70,   39), S( -60,   29), S( -26,   10), S( -53,    1),
        S(  22,  -29), S( -24,   -2), S( -32,    7), S( -56,   14), S( -58,   20), S( -44,   14), S( -15,   -2), S(  11,  -30),
        S(  -1,  -73), S(   1,  -41), S( -13,  -25), S( -80,  -16), S( -27,  -32), S( -61,   -8), S( -15,  -32), S(   7,  -78),
    ],
    // Queen
    [
        S( 919, 1406), S( 912, 1421), S( 933, 1439), S( 959, 1422), S( 954, 1424), S( 964, 1424), S(1006, 1369), S( 946, 1407),
        S( 961, 1395), S( 940, 1427), S( 936, 1460), S( 922, 1481), S( 915, 1499), S( 966, 1448), S( 973, 1429), S(1013, 1417),
        S( 965, 1408), S( 961, 1422), S( 960, 1455), S( 959, 1459), S( 964, 1466), S( 995, 1446), S(1003, 1419), S( 988, 1417),
        S( 951, 1424), S( 959, 1435), S( 955, 1444), S( 951, 1466), S( 958, 1465), S( 964, 1454), S( 972, 1453), S( 969, 1432),
        S( 958, 1410), S( 950, 1436), S( 956, 1436), S( 963, 1452), S( 966, 1448), S( 961, 1443), S( 972, 1429), S( 974, 1423),
        S( 956, 1392), S( 962, 1414), S( 967, 1423), S( 959, 1427), S( 968, 1434), S( 972, 1424), S( 982, 1406), S( 975, 1398),
        S( 963, 1380), S( 966, 1385), S( 970, 1393), S( 978, 1400), S( 977, 1405), S( 986, 1374), S( 991, 1350), S( 995, 1327),
        S( 953, 1388), S( 962, 1382), S( 966, 1395), S( 969, 1404), S( 974, 1386), S( 963, 1376), S( 970, 1361), S( 972, 1352),
    ],
    // Rook
    [
        S( 457,  784), S( 449,  799), S( 451,  798), S( 443,  795), S( 454,  783), S( 473,  792), S( 460,  792), S( 449,  787),
        S( 447,  789), S( 445,  808), S( 465,  802), S( 477,  791), S( 459,  789), S( 482,  793), S( 461,  793), S( 465,  782),
        S( 441,  785), S( 468,  790), S( 471,  784), S( 467,  781), S( 500,  764), S( 499,  774), S( 528,  774), S( 474,  766),
        S( 438,  785), S( 457,  787), S( 460,  787), S( 464,  783), S( 468,  766), S( 466,  776), S( 463,  781), S( 451,  769),
        S( 431,  773), S( 432,  785), S( 448,  776), S( 450,  775), S( 455,  768), S( 432,  784), S( 456,  772), S( 439,  762),
        S( 428,  765), S( 434,  775), S( 445,  764), S( 441,  769), S( 453,  758), S( 449,  764), S( 469,  747), S( 453,  742),
        S( 428,  757), S( 431,  767), S( 451,  764), S( 451,  764), S( 458,  753), S( 458,  758), S( 466,  750), S( 435,  745),
        S( 441,  763), S( 452,  758), S( 455,  768), S( 458,  761), S( 466,  752), S( 459,  762), S( 448,  761), S( 446,  746),
    ],
    // Bishop
    [
        S( 307,  434), S( 284,  438), S( 280,  431), S( 233,  441), S( 243,  438), S( 247,  428), S( 295,  428), S( 278,  421),
        S( 322,  417), S( 326,  430), S( 326,  428), S( 312,  432), S( 308,  425), S( 324,  426), S( 298,  434), S( 305,  420),
        S( 334,  434), S( 350,  428), S( 341,  437), S( 344,  428), S( 342,  431), S( 368,  439), S( 353,  433), S( 336,  437),
        S( 320,  431), S( 337,  439), S( 342,  437), S( 360,  454), S( 351,  443), S( 348,  443), S( 331,  437), S( 321,  433),
        S( 329,  427), S( 322,  440), S( 339,  446), S( 360,  446), S( 358,  445), S( 341,  438), S( 338,  433), S( 340,  419),
        S( 326,  427), S( 351,  438), S( 351,  437), S( 349,  442), S( 355,  446), S( 356,  436), S( 356,  428), S( 352,  419),
        S( 344,  432), S( 349,  416), S( 358,  417), S( 344,  428), S( 355,  427), S( 364,  421), S( 371,  422), S( 357,  413),
        S( 335,  418), S( 357,  435), S( 337,  423), S( 333,  427), S( 342,  424), S( 336,  434), S( 348,  417), S( 367,  394),
    ],
    // Knight
    [
        S( 178,  354), S( 204,  403), S( 252,  420), S( 281,  411), S( 316,  414), S( 260,  386), S( 223,  399), S( 222,  336),
        S( 296,  411), S( 316,  423), S( 318,  423), S( 334,  423), S( 329,  414), S( 364,  407), S( 320,  419), S( 330,  392),
        S( 323,  414), S( 337,  422), S( 347,  446), S( 355,  449), S( 366,  443), S( 401,  424), S( 346,  421), S( 348,  405),
        S( 331,  428), S( 337,  437), S( 355,  451), S( 379,  454), S( 352,  458), S( 375,  454), S( 334,  448), S( 361,  424),
        S( 322,  431), S( 333,  429), S( 344,  450), S( 354,  449), S( 361,  454), S( 352,  444), S( 362,  431), S( 336,  429),
        S( 302,  412), S( 322,  418), S( 331,  427), S( 340,  443), S( 356,  441), S( 341,  420), S( 344,  415), S( 328,  419),
        S( 297,  411), S( 310,  419), S( 320,  420), S( 337,  419), S( 339,  418), S( 336,  415), S( 331,  410), S( 329,  424),
        S( 264,  417), S( 302,  405), S( 301,  414), S( 319,  419), S( 326,  419), S( 330,  408), S( 306,  412), S( 301,  417),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 180,  308), S( 178,  310), S( 145,  314), S( 169,  277), S( 133,  295), S( 158,  298), S( 138,  341), S( 124,  328),
        S(  85,  180), S(  77,  200), S( 106,  157), S( 105,  128), S( 103,  134), S( 121,  158), S(  99,  200), S(  63,  187),
        S(  68,  151), S(  73,  152), S(  81,  128), S(  79,  111), S(  98,  112), S(  88,  119), S(  82,  145), S(  64,  131),
        S(  64,  129), S(  67,  139), S(  77,  119), S(  91,  112), S(  91,  113), S(  87,  113), S(  84,  128), S(  64,  113),
        S(  59,  124), S(  69,  134), S(  74,  118), S(  75,  121), S(  85,  124), S(  78,  117), S(  91,  122), S(  64,  110),
        S(  60,  126), S(  67,  136), S(  68,  124), S(  61,  128), S(  74,  136), S(  81,  122), S(  95,  124), S(  52,  114),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-12, 71),
    S(7, 135),
    S(13, 68),
    S(-11, 40),
    S(-13, 14),
    S(-7, 9),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-18, -36),
    S(2, -28),
    S(-3, -20),
    S(-5, -9),
    S(-12, -2),
    S(-6, -16),
    S(1, -23),
    S(-5, -41),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-6, -1),
    S(-8, -18),
    S(-19, -13),
    S(-17, -23),
    S(-21, -24),
    S(-7, -14),
    S(-10, -17),
    S(-0, -2),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(21, 67);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-14, -11), S(-20, 8), S(-23, 6), S(-11, 8), S(-13, 6)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(83, -42), //Queen
    S(91, 5),   //Rook
    S(65, 51),  //Bishop
    S(64, 27),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(55, -25), //Queen
    S(70, 14),  //Rook
    S(33, 36),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(74, 57), //Queen
    S(55, 28), //Rook
    S(0, 0),   //Bishop
    S(23, 24), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-42, -88),
    S(-21, -50),
    S(-9, -28),
    S(-3, -18),
    S(4, -9),
    S(10, 0),
    S(19, -1),
    S(25, -5),
    S(31, -17),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-28, -76),
    S(-18, -55),
    S(-6, -38),
    S(1, -24),
    S(7, -13),
    S(10, -1),
    S(14, 3),
    S(16, 7),
    S(17, 10),
    S(21, 7),
    S(28, 2),
    S(32, 2),
    S(29, 12),
    S(42, -13),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-34, -84),
    S(-25, -60),
    S(-21, -57),
    S(-18, -52),
    S(-20, -45),
    S(-14, -41),
    S(-13, -34),
    S(-11, -32),
    S(-8, -29),
    S(-6, -24),
    S(-3, -22),
    S(-5, -17),
    S(-2, -15),
    S(-3, -17),
    S(-12, -16),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-29, -249),
    S(-30, -206),
    S(-40, -132),
    S(-37, -109),
    S(-35, -92),
    S(-31, -87),
    S(-28, -70),
    S(-27, -58),
    S(-24, -49),
    S(-23, -46),
    S(-21, -40),
    S(-19, -34),
    S(-17, -32),
    S(-17, -26),
    S(-15, -25),
    S(-13, -22),
    S(-13, -14),
    S(-11, -16),
    S(-2, -20),
    S(11, -30),
    S(15, -29),
    S(67, -61),
    S(60, -57),
    S(78, -79),
    S(211, -133),
    S(221, -162),
    S(159, -106),
    S(93, -97),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(29, 26);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(32, 9),
    S(29, 2),
    S(24, 12),
    S(29, 11),
    S(31, 17),
    S(46, -0),
    S(60, -4),
    S(85, -2),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(2, 50),
    S(7, 19),
    S(7, 19),
    S(17, 10),
    S(7, 14),
    S(22, -2),
    S(28, 1),
    S(7, 34),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(65, -21), S(52, -10), S(39, -6), S(30, 7)],
    // Left adjacent
    [S(43, -10), S(21, -6), S(16, -3), S(14, 6)],
    // Right adjacent
    [S(40, -20), S(32, -9), S(23, -0), S(14, 8)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(102, 226), S(-42, 119), S(-4, 25), S(9, 4)],
    // Left adjacent
    [S(-15, 207), S(-71, 101), S(-26, 25), S(-3, 4)],
    // Right adjacent
    [S(-43, 232), S(-55, 87), S(-23, 22), S(-3, 3)],
];

// Bonus for Rook on 7th Rank. Bonus scaled based on Rook's File. Separate
// bonus if enemy king is in Rank 8. Flipped for black
pub const ROOK_RANK_BONUS: [[PhasedScore; NumberOf::FILES]; 2] = [
    [
        S(0, -3),
        S(0, -11),
        S(-2, 3),
        S(-5, 2),
        S(1, -0),
        S(-3, -7),
        S(17, -10),
        S(-12, 11),
    ],
    [
        S(12, -1),
        S(-0, 9),
        S(12, -7),
        S(-4, 1),
        S(3, -6),
        S(2, -4),
        S(5, 17),
        S(-8, 15),
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
