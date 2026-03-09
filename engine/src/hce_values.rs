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
        S(  26, -102), S(  12,  -41), S(  27,  -23), S(-114,   30), S( -69,   14), S( -18,   11), S(  43,   -3), S( 200, -132),
        S(-128,   12), S( -45,   41), S( -86,   54), S(  37,   35), S( -17,   55), S( -17,   69), S(  16,   56), S( -41,   23),
        S(-143,   25), S(   8,   44), S( -64,   63), S( -80,   74), S( -35,   75), S(  56,   61), S(   5,   63), S( -49,   30),
        S(-100,   10), S( -80,   43), S( -96,   62), S(-142,   75), S(-131,   73), S( -91,   63), S( -97,   54), S(-156,   33),
        S(-109,   -0), S( -82,   27), S(-104,   48), S(-136,   66), S(-132,   61), S( -91,   44), S(-104,   33), S(-172,   27),
        S( -55,  -12), S( -17,   10), S( -71,   32), S( -86,   46), S( -76,   45), S( -71,   32), S( -34,   14), S( -79,    7),
        S(  34,  -30), S(   1,    0), S( -19,   15), S( -53,   26), S( -54,   29), S( -37,   20), S(  13,   -0), S(  11,  -20),
        S(  17,  -72), S(  55,  -48), S(  22,  -20), S( -80,   -1), S( -19,  -20), S( -51,   -3), S(  26,  -35), S(  19,  -72),
    ],
    // Queen
    [
        S( 925, 1429), S( 909, 1450), S( 930, 1471), S( 959, 1453), S( 947, 1455), S( 958, 1454), S(1001, 1398), S( 950, 1431),
        S( 958, 1414), S( 930, 1450), S( 927, 1488), S( 913, 1511), S( 905, 1528), S( 955, 1469), S( 944, 1464), S(1009, 1438),
        S( 971, 1424), S( 963, 1442), S( 960, 1478), S( 959, 1482), S( 967, 1487), S( 995, 1466), S(1009, 1433), S(1000, 1427),
        S( 953, 1447), S( 962, 1458), S( 953, 1468), S( 948, 1490), S( 957, 1489), S( 964, 1478), S( 977, 1476), S( 973, 1454),
        S( 965, 1434), S( 951, 1462), S( 956, 1464), S( 966, 1477), S( 966, 1477), S( 964, 1468), S( 975, 1455), S( 979, 1447),
        S( 961, 1416), S( 970, 1437), S( 972, 1448), S( 967, 1454), S( 974, 1463), S( 981, 1446), S( 988, 1429), S( 983, 1414),
        S( 971, 1405), S( 971, 1412), S( 981, 1416), S( 991, 1422), S( 990, 1428), S( 999, 1390), S(1003, 1357), S(1017, 1330),
        S( 966, 1407), S( 972, 1404), S( 982, 1412), S( 991, 1426), S( 990, 1406), S( 975, 1394), S( 992, 1372), S( 983, 1370),
    ],
    // Rook
    [
        S( 462,  794), S( 443,  804), S( 438,  815), S( 433,  811), S( 445,  805), S( 474,  795), S( 466,  799), S( 498,  786),
        S( 449,  795), S( 442,  809), S( 459,  813), S( 477,  803), S( 460,  805), S( 483,  792), S( 484,  787), S( 514,  773),
        S( 447,  792), S( 478,  791), S( 475,  793), S( 479,  790), S( 512,  775), S( 506,  771), S( 551,  764), S( 513,  761),
        S( 442,  795), S( 458,  792), S( 461,  799), S( 468,  794), S( 475,  779), S( 473,  775), S( 483,  775), S( 477,  768),
        S( 430,  789), S( 431,  792), S( 444,  791), S( 454,  788), S( 456,  785), S( 433,  787), S( 462,  773), S( 446,  771),
        S( 426,  783), S( 435,  780), S( 447,  778), S( 448,  782), S( 456,  776), S( 453,  767), S( 479,  750), S( 455,  753),
        S( 425,  774), S( 437,  777), S( 458,  775), S( 461,  776), S( 465,  768), S( 463,  762), S( 478,  752), S( 442,  760),
        S( 447,  774), S( 454,  773), S( 468,  778), S( 476,  772), S( 480,  766), S( 471,  767), S( 476,  760), S( 450,  759),
    ],
    // Bishop
    [
        S( 308,  432), S( 280,  440), S( 278,  433), S( 229,  446), S( 238,  442), S( 255,  431), S( 295,  430), S( 267,  425),
        S( 313,  419), S( 324,  430), S( 322,  431), S( 313,  433), S( 310,  428), S( 322,  427), S( 298,  437), S( 320,  415),
        S( 336,  437), S( 354,  431), S( 344,  440), S( 347,  432), S( 338,  437), S( 371,  442), S( 357,  434), S( 341,  440),
        S( 322,  434), S( 338,  443), S( 342,  443), S( 356,  459), S( 351,  448), S( 349,  447), S( 334,  441), S( 318,  436),
        S( 329,  430), S( 323,  444), S( 341,  449), S( 360,  451), S( 357,  449), S( 345,  443), S( 336,  438), S( 340,  422),
        S( 331,  431), S( 351,  442), S( 352,  442), S( 354,  447), S( 359,  451), S( 358,  440), S( 355,  433), S( 353,  424),
        S( 348,  435), S( 353,  420), S( 361,  420), S( 348,  433), S( 361,  431), S( 367,  424), S( 373,  428), S( 356,  417),
        S( 335,  418), S( 364,  432), S( 348,  427), S( 337,  429), S( 345,  427), S( 342,  438), S( 358,  415), S( 355,  401),
    ],
    // Knight
    [
        S( 170,  357), S( 188,  407), S( 251,  423), S( 284,  414), S( 325,  417), S( 265,  391), S( 219,  409), S( 220,  334),
        S( 295,  410), S( 311,  425), S( 320,  429), S( 340,  429), S( 335,  421), S( 376,  407), S( 323,  418), S( 334,  393),
        S( 320,  415), S( 342,  425), S( 353,  449), S( 358,  453), S( 375,  444), S( 411,  424), S( 350,  423), S( 353,  406),
        S( 328,  431), S( 336,  439), S( 356,  455), S( 385,  457), S( 354,  461), S( 376,  458), S( 331,  451), S( 360,  425),
        S( 322,  434), S( 332,  430), S( 345,  454), S( 356,  453), S( 363,  459), S( 355,  447), S( 359,  433), S( 335,  432),
        S( 302,  415), S( 322,  422), S( 336,  429), S( 341,  448), S( 357,  445), S( 345,  422), S( 342,  418), S( 329,  420),
        S( 295,  413), S( 309,  420), S( 319,  422), S( 341,  422), S( 342,  421), S( 336,  416), S( 324,  414), S( 323,  429),
        S( 253,  412), S( 313,  405), S( 301,  414), S( 322,  418), S( 327,  421), S( 331,  408), S( 317,  409), S( 286,  411),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 188,  320), S( 187,  328), S( 174,  322), S( 212,  261), S( 173,  269), S( 168,  276), S(  63,  341), S(  73,  329),
        S(  83,  172), S(  81,  194), S( 112,  143), S( 113,  103), S( 114,  104), S( 157,  121), S( 121,  177), S(  98,  156),
        S(  61,  151), S(  71,  158), S(  79,  133), S(  81,  114), S( 100,  117), S( 105,  119), S(  88,  153), S(  89,  125),
        S(  58,  129), S(  65,  145), S(  77,  123), S(  91,  117), S(  92,  118), S(  90,  119), S(  78,  140), S(  78,  110),
        S(  56,  123), S(  68,  139), S(  75,  122), S(  76,  125), S(  91,  129), S(  88,  121), S(  99,  132), S(  86,  106),
        S(  62,  125), S(  70,  141), S(  74,  128), S(  75,  130), S(  86,  139), S( 109,  121), S( 115,  128), S(  82,  104),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-44, 45),
    S(6, 143),
    S(11, 72),
    S(-13, 43),
    S(-15, 14),
    S(-8, 8),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-22, -33),
    S(-1, -30),
    S(-6, -21),
    S(-4, -10),
    S(-17, -4),
    S(-16, -20),
    S(-8, -32),
    S(-19, -45),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-5, -0),
    S(-10, -18),
    S(-22, -13),
    S(-19, -23),
    S(-22, -24),
    S(-16, -11),
    S(-11, -18),
    S(-14, 6),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(22, 74);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-17, -15), S(-31, 8), S(-26, 7), S(-15, 8), S(-17, 15)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(58, -62), //Queen
    S(79, -18), //Rook
    S(51, 31),  //Bishop
    S(47, 4),   //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(29, -48), //Queen
    S(58, -15), //Rook
    S(25, 23),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(48, 41), //Queen
    S(37, 2),  //Rook
    S(0, 0),   //Bishop
    S(22, 16), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-38, -76),
    S(-17, -37),
    S(-5, -14),
    S(2, -4),
    S(9, 5),
    S(15, 15),
    S(23, 13),
    S(30, 10),
    S(36, -0),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-28, -66),
    S(-15, -45),
    S(-3, -27),
    S(4, -12),
    S(11, -1),
    S(14, 11),
    S(18, 16),
    S(20, 20),
    S(22, 24),
    S(27, 21),
    S(34, 16),
    S(41, 16),
    S(37, 27),
    S(54, 1),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-32, -54),
    S(-18, -34),
    S(-14, -31),
    S(-8, -27),
    S(-8, -19),
    S(1, -16),
    S(5, -9),
    S(13, -8),
    S(20, -4),
    S(25, 1),
    S(31, 4),
    S(31, 12),
    S(34, 15),
    S(35, 15),
    S(23, 20),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-25, -106),
    S(-18, -162),
    S(-25, -96),
    S(-21, -69),
    S(-18, -53),
    S(-13, -49),
    S(-9, -32),
    S(-8, -19),
    S(-4, -12),
    S(-3, -8),
    S(-1, -2),
    S(2, 3),
    S(3, 5),
    S(3, 11),
    S(5, 12),
    S(7, 16),
    S(6, 25),
    S(8, 23),
    S(17, 20),
    S(31, 10),
    S(33, 14),
    S(90, -20),
    S(79, -14),
    S(105, -39),
    S(202, -69),
    S(144, -62),
    S(57, -8),
    S(27, -13),
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
