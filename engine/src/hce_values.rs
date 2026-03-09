// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{
    definitions::NumberOf,
    pieces::Piece,
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
        S(  26, -101), S(  11,  -41), S(  25,  -23), S(-112,   30), S( -68,   14), S( -18,   11), S(  43,   -3), S( 202, -131),
        S(-128,   12), S( -46,   41), S( -85,   53), S(  36,   35), S( -18,   55), S( -17,   68), S(  15,   55), S( -41,   23),
        S(-143,   25), S(   7,   43), S( -63,   62), S( -79,   74), S( -35,   74), S(  55,   61), S(   5,   63), S( -49,   29),
        S(-100,   10), S( -80,   42), S( -96,   61), S(-141,   74), S(-130,   73), S( -91,   63), S( -96,   53), S(-155,   33),
        S(-108,    0), S( -81,   27), S(-103,   48), S(-135,   65), S(-131,   61), S( -90,   44), S(-103,   33), S(-171,   27),
        S( -55,  -12), S( -18,   10), S( -71,   32), S( -86,   45), S( -76,   44), S( -71,   32), S( -34,   14), S( -79,    7),
        S(  33,  -29), S(  -0,    1), S( -19,   15), S( -53,   26), S( -54,   29), S( -37,   20), S(  12,   -0), S(  10,  -20),
        S(  16,  -71), S(  54,  -47), S(  22,  -19), S( -80,   -1), S( -19,  -19), S( -51,   -3), S(  25,  -34), S(  18,  -71),
    ],
    // Queen
    [
        S( 924, 1422), S( 909, 1444), S( 930, 1463), S( 958, 1446), S( 946, 1448), S( 958, 1446), S(1000, 1391), S( 950, 1424),
        S( 957, 1408), S( 930, 1443), S( 927, 1481), S( 913, 1503), S( 905, 1520), S( 954, 1462), S( 943, 1457), S(1007, 1431),
        S( 970, 1418), S( 962, 1435), S( 959, 1471), S( 957, 1475), S( 966, 1480), S( 993, 1459), S(1007, 1426), S( 999, 1420),
        S( 952, 1440), S( 961, 1451), S( 952, 1461), S( 948, 1483), S( 956, 1482), S( 963, 1471), S( 975, 1469), S( 972, 1447),
        S( 964, 1428), S( 950, 1455), S( 955, 1457), S( 965, 1470), S( 965, 1470), S( 963, 1461), S( 974, 1448), S( 978, 1440),
        S( 960, 1410), S( 969, 1430), S( 971, 1442), S( 966, 1447), S( 973, 1456), S( 980, 1440), S( 987, 1422), S( 982, 1408),
        S( 969, 1399), S( 970, 1405), S( 980, 1410), S( 990, 1416), S( 988, 1422), S( 997, 1384), S(1002, 1351), S(1015, 1324),
        S( 965, 1400), S( 971, 1398), S( 981, 1405), S( 990, 1420), S( 988, 1399), S( 974, 1388), S( 991, 1366), S( 982, 1364),
    ],
    // Rook
    [
        S( 461,  792), S( 443,  802), S( 437,  813), S( 432,  809), S( 444,  803), S( 473,  793), S( 465,  797), S( 497,  785),
        S( 448,  793), S( 441,  807), S( 458,  811), S( 476,  801), S( 459,  803), S( 482,  791), S( 482,  785), S( 512,  771),
        S( 447,  790), S( 477,  789), S( 474,  791), S( 478,  789), S( 511,  773), S( 505,  770), S( 549,  762), S( 511,  760),
        S( 441,  794), S( 457,  790), S( 460,  797), S( 467,  792), S( 474,  777), S( 472,  773), S( 481,  773), S( 476,  766),
        S( 429,  787), S( 430,  790), S( 443,  789), S( 453,  786), S( 455,  783), S( 432,  785), S( 461,  772), S( 445,  769),
        S( 425,  781), S( 435,  778), S( 446,  776), S( 447,  780), S( 455,  774), S( 452,  766), S( 478,  749), S( 454,  752),
        S( 424,  772), S( 436,  776), S( 457,  774), S( 460,  774), S( 464,  766), S( 462,  761), S( 477,  750), S( 441,  758),
        S( 446,  772), S( 453,  772), S( 467,  776), S( 474,  770), S( 479,  764), S( 470,  766), S( 475,  759), S( 449,  758),
    ],
    // Bishop
    [
        S( 308,  431), S( 280,  439), S( 278,  433), S( 229,  445), S( 238,  441), S( 255,  430), S( 295,  429), S( 266,  424),
        S( 312,  418), S( 323,  429), S( 321,  430), S( 313,  432), S( 309,  427), S( 322,  427), S( 297,  436), S( 320,  414),
        S( 335,  436), S( 353,  430), S( 343,  439), S( 346,  431), S( 337,  436), S( 370,  441), S( 356,  433), S( 340,  439),
        S( 322,  433), S( 338,  442), S( 341,  442), S( 355,  458), S( 350,  447), S( 348,  446), S( 334,  440), S( 317,  435),
        S( 328,  429), S( 322,  443), S( 340,  448), S( 359,  450), S( 356,  448), S( 344,  442), S( 336,  437), S( 339,  421),
        S( 330,  430), S( 350,  441), S( 351,  441), S( 353,  446), S( 358,  450), S( 357,  439), S( 354,  432), S( 352,  423),
        S( 347,  434), S( 352,  419), S( 359,  420), S( 347,  432), S( 360,  430), S( 366,  423), S( 372,  427), S( 355,  417),
        S( 335,  418), S( 363,  431), S( 347,  426), S( 336,  428), S( 344,  426), S( 341,  437), S( 357,  414), S( 354,  400),
    ],
    // Knight
    [
        S( 171,  357), S( 189,  407), S( 251,  423), S( 284,  413), S( 325,  417), S( 265,  391), S( 219,  409), S( 220,  335),
        S( 295,  410), S( 311,  425), S( 319,  428), S( 340,  429), S( 334,  421), S( 375,  407), S( 322,  418), S( 333,  393),
        S( 319,  414), S( 342,  424), S( 352,  448), S( 357,  452), S( 374,  443), S( 410,  424), S( 349,  422), S( 353,  406),
        S( 327,  430), S( 336,  439), S( 355,  454), S( 384,  456), S( 353,  460), S( 375,  457), S( 330,  450), S( 359,  424),
        S( 321,  434), S( 331,  429), S( 345,  453), S( 355,  453), S( 363,  458), S( 354,  446), S( 358,  433), S( 334,  431),
        S( 301,  414), S( 321,  421), S( 335,  428), S( 340,  447), S( 356,  444), S( 345,  422), S( 341,  418), S( 328,  419),
        S( 295,  413), S( 308,  419), S( 318,  421), S( 341,  421), S( 341,  421), S( 336,  416), S( 323,  413), S( 322,  428),
        S( 253,  412), S( 313,  404), S( 301,  414), S( 322,  418), S( 326,  420), S( 330,  408), S( 317,  408), S( 286,  411),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 188,  319), S( 186,  327), S( 174,  321), S( 211,  261), S( 173,  269), S( 167,  276), S(  64,  340), S(  74,  329),
        S(  82,  170), S(  81,  192), S( 111,  141), S( 111,  102), S( 113,  103), S( 155,  120), S( 120,  175), S(  97,  154),
        S(  61,  150), S(  70,  157), S(  78,  132), S(  80,  112), S(  99,  116), S( 104,  118), S(  87,  151), S(  88,  123),
        S(  57,  127), S(  65,  144), S(  76,  122), S(  90,  116), S(  91,  117), S(  89,  118), S(  77,  138), S(  78,  109),
        S(  56,  122), S(  67,  137), S(  74,  121), S(  76,  124), S(  90,  128), S(  87,  120), S(  98,  130), S(  85,  105),
        S(  61,  124), S(  69,  140), S(  73,  126), S(  74,  128), S(  85,  138), S( 108,  120), S( 114,  127), S(  81,  103),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-45, 41),
    S(6, 141),
    S(10, 71),
    S(-13, 43),
    S(-15, 14),
    S(-8, 8),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-22, -33),
    S(-1, -29),
    S(-6, -21),
    S(-4, -10),
    S(-17, -4),
    S(-16, -19),
    S(-7, -32),
    S(-19, -45),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-5, -0),
    S(-10, -18),
    S(-22, -13),
    S(-19, -23),
    S(-22, -24),
    S(-16, -10),
    S(-11, -18),
    S(-14, 5),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(21, 73);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-17, -15), S(-31, 8), S(-26, 7), S(-15, 8), S(-17, 15)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(57, -61), //Queen
    S(79, -18), //Rook
    S(50, 31),  //Bishop
    S(46, 4),   //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(29, -47), //Queen
    S(57, -15), //Rook
    S(24, 23),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(48, 41), //Queen
    S(36, 2),  //Rook
    S(0, 0),   //Bishop
    S(21, 16), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-41, -79),
    S(-20, -41),
    S(-7, -18),
    S(-1, -8),
    S(6, 1),
    S(12, 11),
    S(20, 9),
    S(27, 6),
    S(33, -5),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-30, -69),
    S(-18, -48),
    S(-6, -30),
    S(2, -16),
    S(8, -5),
    S(12, 7),
    S(16, 12),
    S(18, 16),
    S(19, 20),
    S(24, 17),
    S(31, 12),
    S(38, 12),
    S(34, 23),
    S(51, -2),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-35, -60),
    S(-22, -40),
    S(-17, -37),
    S(-11, -34),
    S(-12, -25),
    S(-3, -23),
    S(2, -16),
    S(9, -14),
    S(17, -11),
    S(21, -5),
    S(27, -2),
    S(27, 5),
    S(30, 9),
    S(32, 8),
    S(19, 13),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-28, -160),
    S(-25, -171),
    S(-32, -105),
    S(-27, -78),
    S(-25, -63),
    S(-20, -59),
    S(-16, -42),
    S(-14, -29),
    S(-11, -22),
    S(-10, -18),
    S(-8, -12),
    S(-5, -7),
    S(-4, -5),
    S(-4, 0),
    S(-2, 2),
    S(-0, 6),
    S(-1, 14),
    S(1, 12),
    S(10, 9),
    S(24, 0),
    S(26, 3),
    S(82, -30),
    S(72, -25),
    S(98, -49),
    S(225, -99),
    S(189, -103),
    S(81, -36),
    S(41, -34),
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
