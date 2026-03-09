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
        S(  25, -100), S(  10,  -40), S(  24,  -22), S(-111,   30), S( -67,   14), S( -18,   11), S(  42,   -3), S( 201, -130),
        S(-127,   12), S( -46,   41), S( -85,   53), S(  35,   35), S( -18,   54), S( -17,   68), S(  14,   55), S( -41,   23),
        S(-142,   25), S(   7,   43), S( -63,   62), S( -79,   73), S( -35,   73), S(  54,   61), S(   4,   62), S( -49,   29),
        S( -99,   10), S( -80,   42), S( -95,   61), S(-140,   74), S(-129,   72), S( -90,   62), S( -96,   53), S(-154,   33),
        S(-108,    0), S( -81,   27), S(-103,   48), S(-134,   65), S(-130,   61), S( -90,   43), S(-103,   33), S(-170,   27),
        S( -55,  -12), S( -18,   10), S( -71,   32), S( -85,   45), S( -76,   44), S( -71,   32), S( -34,   14), S( -78,    7),
        S(  33,  -29), S(  -1,    1), S( -19,   15), S( -52,   25), S( -54,   29), S( -38,   20), S(  11,    0), S(   9,  -19),
        S(  16,  -70), S(  53,  -46), S(  21,  -19), S( -80,   -1), S( -20,  -19), S( -51,   -2), S(  25,  -34), S(  18,  -70),
    ],
    // Queen
    [
        S( 923, 1416), S( 908, 1438), S( 928, 1457), S( 956, 1440), S( 945, 1442), S( 956, 1440), S( 998, 1386), S( 948, 1418),
        S( 955, 1402), S( 928, 1437), S( 925, 1475), S( 912, 1496), S( 904, 1514), S( 953, 1456), S( 941, 1451), S(1005, 1425),
        S( 968, 1412), S( 960, 1429), S( 957, 1465), S( 956, 1469), S( 964, 1474), S( 991, 1453), S(1005, 1420), S( 997, 1414),
        S( 950, 1434), S( 959, 1445), S( 950, 1455), S( 946, 1477), S( 954, 1476), S( 961, 1465), S( 973, 1462), S( 970, 1441),
        S( 962, 1422), S( 948, 1449), S( 953, 1451), S( 963, 1463), S( 963, 1464), S( 961, 1455), S( 972, 1442), S( 976, 1434),
        S( 958, 1404), S( 967, 1424), S( 969, 1436), S( 964, 1441), S( 971, 1450), S( 978, 1434), S( 985, 1416), S( 980, 1402),
        S( 968, 1393), S( 968, 1400), S( 978, 1404), S( 988, 1410), S( 986, 1416), S( 995, 1379), S(1000, 1346), S(1013, 1319),
        S( 963, 1395), S( 969, 1393), S( 979, 1400), S( 987, 1414), S( 986, 1394), S( 972, 1382), S( 989, 1361), S( 980, 1358),
    ],
    // Rook
    [
        S( 460,  790), S( 442,  800), S( 436,  811), S( 431,  807), S( 443,  801), S( 472,  791), S( 464,  795), S( 495,  783),
        S( 447,  791), S( 440,  805), S( 457,  809), S( 474,  800), S( 458,  801), S( 480,  789), S( 481,  783), S( 510,  769),
        S( 446,  788), S( 475,  788), S( 472,  789), S( 477,  787), S( 509,  771), S( 503,  768), S( 547,  760), S( 510,  758),
        S( 440,  792), S( 456,  788), S( 459,  795), S( 465,  790), S( 473,  775), S( 471,  771), S( 480,  772), S( 475,  765),
        S( 428,  785), S( 429,  788), S( 442,  787), S( 452,  785), S( 454,  781), S( 432,  784), S( 459,  770), S( 444,  768),
        S( 425,  779), S( 434,  777), S( 445,  774), S( 446,  778), S( 454,  773), S( 451,  764), S( 476,  747), S( 453,  750),
        S( 423,  770), S( 435,  774), S( 456,  772), S( 459,  772), S( 463,  765), S( 460,  759), S( 475,  749), S( 440,  757),
        S( 445,  770), S( 451,  770), S( 465,  774), S( 473,  769), S( 478,  763), S( 468,  764), S( 473,  757), S( 448,  756),
    ],
    // Bishop
    [
        S( 307,  430), S( 280,  438), S( 278,  432), S( 230,  444), S( 238,  440), S( 255,  429), S( 294,  429), S( 266,  423),
        S( 312,  417), S( 323,  428), S( 320,  429), S( 312,  431), S( 309,  427), S( 321,  426), S( 297,  435), S( 319,  413),
        S( 334,  435), S( 352,  429), S( 342,  438), S( 345,  430), S( 336,  435), S( 369,  440), S( 355,  432), S( 339,  438),
        S( 321,  432), S( 337,  441), S( 340,  441), S( 354,  457), S( 349,  446), S( 347,  445), S( 333,  439), S( 317,  434),
        S( 328,  429), S( 321,  442), S( 339,  447), S( 358,  449), S( 355,  447), S( 343,  441), S( 335,  436), S( 338,  420),
        S( 329,  429), S( 349,  440), S( 350,  440), S( 352,  445), S( 357,  449), S( 356,  438), S( 353,  431), S( 351,  422),
        S( 346,  433), S( 351,  418), S( 358,  419), S( 346,  431), S( 359,  429), S( 365,  422), S( 370,  426), S( 354,  416),
        S( 334,  417), S( 362,  430), S( 346,  425), S( 335,  427), S( 343,  425), S( 340,  436), S( 356,  413), S( 352,  400),
    ],
    // Knight
    [
        S( 172,  357), S( 190,  406), S( 251,  422), S( 284,  413), S( 324,  416), S( 265,  391), S( 220,  408), S( 221,  335),
        S( 294,  409), S( 311,  424), S( 319,  428), S( 339,  428), S( 333,  420), S( 374,  406), S( 322,  417), S( 333,  393),
        S( 319,  414), S( 341,  424), S( 351,  447), S( 356,  451), S( 373,  442), S( 408,  423), S( 348,  422), S( 352,  405),
        S( 327,  430), S( 335,  438), S( 354,  453), S( 383,  455), S( 352,  459), S( 374,  456), S( 329,  449), S( 359,  424),
        S( 321,  433), S( 331,  429), S( 344,  452), S( 354,  452), S( 362,  457), S( 353,  445), S( 357,  432), S( 333,  430),
        S( 301,  414), S( 321,  421), S( 335,  428), S( 339,  446), S( 355,  443), S( 344,  421), S( 341,  417), S( 328,  419),
        S( 294,  412), S( 308,  419), S( 318,  421), S( 340,  421), S( 340,  420), S( 335,  415), S( 323,  413), S( 322,  427),
        S( 253,  412), S( 312,  404), S( 300,  413), S( 321,  417), S( 326,  420), S( 329,  408), S( 317,  408), S( 286,  410),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 187,  319), S( 186,  327), S( 173,  321), S( 211,  261), S( 173,  269), S( 167,  276), S(  65,  340), S(  74,  328),
        S(  81,  169), S(  80,  190), S( 109,  140), S( 110,  101), S( 112,  102), S( 154,  119), S( 118,  174), S(  96,  153),
        S(  60,  148), S(  69,  155), S(  78,  131), S(  79,  111), S(  98,  114), S( 102,  116), S(  86,  149), S(  87,  122),
        S(  56,  126), S(  64,  142), S(  75,  120), S(  89,  115), S(  91,  116), S(  88,  117), S(  76,  137), S(  77,  108),
        S(  55,  121), S(  67,  136), S(  74,  120), S(  75,  123), S(  89,  126), S(  86,  118), S(  97,  129), S(  84,  104),
        S(  60,  123), S(  69,  138), S(  72,  125), S(  73,  127), S(  85,  136), S( 107,  119), S( 113,  125), S(  80,  102),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-46, 38),
    S(6, 140),
    S(10, 71),
    S(-13, 43),
    S(-15, 14),
    S(-8, 8),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-22, -32),
    S(-1, -29),
    S(-6, -21),
    S(-4, -10),
    S(-17, -4),
    S(-16, -19),
    S(-7, -32),
    S(-18, -44),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-5, -0),
    S(-9, -18),
    S(-22, -13),
    S(-19, -22),
    S(-21, -24),
    S(-16, -10),
    S(-10, -18),
    S(-13, 5),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(21, 72);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-17, -15), S(-30, 8), S(-26, 7), S(-15, 8), S(-17, 15)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(56, -60), //Queen
    S(78, -17), //Rook
    S(50, 31),  //Bishop
    S(46, 4),   //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(28, -47), //Queen
    S(56, -14), //Rook
    S(24, 23),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(47, 40), //Queen
    S(36, 2),  //Rook
    S(0, 0),   //Bishop
    S(21, 16), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-43, -82),
    S(-22, -44),
    S(-10, -22),
    S(-3, -12),
    S(3, -3),
    S(9, 7),
    S(17, 5),
    S(24, 2),
    S(30, -8),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-32, -72),
    S(-20, -51),
    S(-8, -34),
    S(-1, -19),
    S(6, -8),
    S(9, 4),
    S(13, 8),
    S(15, 12),
    S(16, 16),
    S(21, 13),
    S(28, 9),
    S(35, 8),
    S(31, 19),
    S(48, -6),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-38, -66),
    S(-25, -46),
    S(-21, -43),
    S(-15, -40),
    S(-15, -32),
    S(-6, -29),
    S(-2, -22),
    S(6, -21),
    S(13, -17),
    S(18, -12),
    S(23, -9),
    S(23, -2),
    S(26, 2),
    S(28, 2),
    S(15, 6),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-33, -208),
    S(-32, -179),
    S(-39, -114),
    S(-35, -87),
    S(-32, -72),
    S(-27, -68),
    S(-23, -51),
    S(-22, -39),
    S(-19, -31),
    S(-17, -27),
    S(-15, -22),
    S(-12, -16),
    S(-11, -14),
    S(-11, -9),
    S(-10, -8),
    S(-7, -4),
    S(-8, 4),
    S(-6, 3),
    S(3, -0),
    S(16, -10),
    S(19, -6),
    S(74, -39),
    S(64, -34),
    S(90, -59),
    S(223, -112),
    S(214, -131),
    S(109, -67),
    S(60, -58),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(1, 2);

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
