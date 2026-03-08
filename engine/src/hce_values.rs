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
        S(  20, -104), S(  -0,  -42), S(  16,  -23), S(-128,   31), S( -82,   13), S( -23,   11), S(  40,   -3), S( 193, -131),
        S(-129,   12), S( -48,   42), S( -91,   55), S(  32,   36), S( -21,   56), S( -22,   70), S(  19,   57), S( -48,   24),
        S(-151,   26), S(   5,   44), S( -71,   64), S( -85,   75), S( -38,   75), S(  52,   62), S(   5,   64), S( -57,   31),
        S(-103,   10), S( -80,   44), S( -96,   62), S(-144,   76), S(-129,   74), S( -90,   63), S( -98,   55), S(-162,   34),
        S(-109,   -0), S( -78,   27), S( -99,   49), S(-130,   66), S(-122,   62), S( -82,   44), S( -99,   33), S(-176,   27),
        S( -53,  -13), S(  -9,   10), S( -63,   32), S( -74,   46), S( -63,   44), S( -63,   32), S( -23,   13), S( -74,    5),
        S(  41,  -32), S(  11,   -1), S(  -9,   15), S( -48,   27), S( -46,   30), S( -28,   20), S(  24,   -2), S(  21,  -23),
        S(  26,  -75), S(  55,  -48), S(  16,  -21), S( -92,   -1), S( -24,  -25), S( -62,   -2), S(  27,  -34), S(  24,  -72),
    ],
    // Queen
    [
        S( 912, 1455), S( 922, 1462), S( 951, 1479), S( 983, 1461), S( 965, 1463), S( 974, 1461), S(1004, 1413), S( 943, 1449),
        S( 964, 1423), S( 939, 1464), S( 939, 1503), S( 927, 1523), S( 915, 1542), S( 964, 1483), S( 946, 1479), S(1005, 1447),
        S( 974, 1436), S( 970, 1457), S( 969, 1496), S( 971, 1495), S( 974, 1501), S( 999, 1478), S(1007, 1443), S( 992, 1432),
        S( 952, 1464), S( 961, 1481), S( 958, 1489), S( 951, 1514), S( 960, 1510), S( 966, 1493), S( 970, 1488), S( 967, 1459),
        S( 959, 1453), S( 950, 1480), S( 951, 1487), S( 962, 1502), S( 961, 1501), S( 958, 1486), S( 972, 1463), S( 970, 1453),
        S( 952, 1432), S( 963, 1452), S( 961, 1475), S( 956, 1477), S( 963, 1486), S( 968, 1465), S( 980, 1436), S( 970, 1423),
        S( 952, 1425), S( 960, 1431), S( 971, 1432), S( 973, 1449), S( 972, 1453), S( 983, 1407), S( 989, 1369), S( 999, 1336),
        S( 948, 1426), S( 938, 1430), S( 948, 1441), S( 966, 1435), S( 957, 1436), S( 941, 1419), S( 962, 1390), S( 956, 1392),
    ],
    // Rook
    [
        S( 473,  807), S( 456,  817), S( 459,  828), S( 455,  826), S( 467,  817), S( 492,  804), S( 469,  810), S( 506,  799),
        S( 464,  804), S( 456,  818), S( 473,  824), S( 490,  816), S( 468,  817), S( 491,  802), S( 483,  796), S( 516,  783),
        S( 460,  799), S( 485,  800), S( 486,  801), S( 487,  799), S( 521,  783), S( 502,  780), S( 540,  773), S( 505,  771),
        S( 444,  803), S( 461,  800), S( 466,  808), S( 475,  803), S( 480,  786), S( 471,  780), S( 471,  781), S( 471,  773),
        S( 422,  797), S( 426,  801), S( 439,  801), S( 453,  799), S( 456,  792), S( 421,  794), S( 445,  779), S( 434,  776),
        S( 413,  792), S( 425,  790), S( 438,  788), S( 435,  793), S( 442,  787), S( 430,  778), S( 462,  756), S( 440,  760),
        S( 409,  785), S( 425,  789), S( 443,  788), S( 440,  789), S( 444,  779), S( 441,  773), S( 459,  762), S( 424,  771),
        S( 431,  781), S( 433,  789), S( 446,  798), S( 451,  795), S( 456,  787), S( 442,  779), S( 455,  777), S( 431,  770),
    ],
    // Bishop
    [
        S( 313,  437), S( 296,  448), S( 297,  444), S( 246,  458), S( 247,  454), S( 262,  444), S( 306,  442), S( 269,  432),
        S( 322,  425), S( 340,  447), S( 339,  449), S( 327,  451), S( 322,  446), S( 335,  445), S( 302,  455), S( 320,  423),
        S( 345,  451), S( 367,  448), S( 359,  462), S( 363,  452), S( 351,  457), S( 381,  460), S( 363,  451), S( 341,  452),
        S( 330,  449), S( 342,  468), S( 353,  464), S( 362,  482), S( 358,  470), S( 358,  468), S( 338,  464), S( 320,  448),
        S( 325,  446), S( 329,  465), S( 344,  472), S( 359,  473), S( 358,  472), S( 354,  465), S( 343,  459), S( 328,  434),
        S( 330,  442), S( 346,  455), S( 349,  462), S( 351,  466), S( 357,  471), S( 352,  462), S( 353,  444), S( 353,  431),
        S( 339,  437), S( 346,  432), S( 358,  434), S( 338,  449), S( 349,  451), S( 360,  439), S( 367,  440), S( 349,  414),
        S( 318,  411), S( 341,  436), S( 327,  413), S( 319,  437), S( 324,  431), S( 323,  434), S( 344,  419), S( 337,  393),
    ],
    // Knight
    [
        S( 153,  335), S( 197,  403), S( 264,  428), S( 298,  418), S( 339,  422), S( 270,  396), S( 222,  405), S( 211,  311),
        S( 299,  405), S( 322,  429), S( 333,  444), S( 354,  444), S( 346,  435), S( 391,  421), S( 325,  423), S( 333,  389),
        S( 329,  417), S( 361,  440), S( 381,  461), S( 383,  468), S( 403,  456), S( 440,  436), S( 367,  438), S( 354,  409),
        S( 328,  434), S( 345,  458), S( 373,  471), S( 400,  474), S( 366,  481), S( 395,  474), S( 337,  468), S( 358,  427),
        S( 315,  434), S( 337,  446), S( 354,  472), S( 356,  473), S( 368,  477), S( 355,  465), S( 355,  447), S( 325,  425),
        S( 296,  415), S( 322,  439), S( 340,  450), S( 344,  466), S( 355,  465), S( 343,  444), S( 343,  432), S( 316,  416),
        S( 279,  405), S( 292,  423), S( 312,  436), S( 327,  438), S( 327,  436), S( 330,  430), S( 313,  411), S( 311,  417),
        S( 230,  395), S( 289,  380), S( 276,  416), S( 292,  415), S( 298,  419), S( 315,  404), S( 293,  388), S( 263,  388),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 187,  320), S( 190,  329), S( 184,  320), S( 225,  262), S( 184,  268), S( 176,  272), S(  65,  341), S(  68,  329),
        S(  85,  173), S(  96,  193), S( 128,  140), S( 128,  102), S( 132,  103), S( 177,  119), S( 138,  177), S( 105,  156),
        S(  61,  151), S(  80,  159), S(  91,  133), S(  91,  114), S( 112,  118), S( 113,  119), S( 100,  153), S(  91,  125),
        S(  50,  131), S(  73,  146), S(  76,  124), S(  94,  119), S(  94,  120), S(  88,  121), S(  84,  141), S(  76,  111),
        S(  46,  125), S(  67,  141), S(  73,  123), S(  71,  129), S(  86,  132), S(  81,  123), S( 102,  134), S(  82,  106),
        S(  46,  129), S(  67,  144), S(  67,  131), S(  54,  128), S(  76,  141), S( 100,  124), S( 113,  130), S(  73,  107),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-22, 42),
    S(10, 142),
    S(12, 71),
    S(-13, 43),
    S(-14, 15),
    S(-7, 9),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-18, -33),
    S(4, -29),
    S(-7, -20),
    S(-3, -9),
    S(-15, -4),
    S(-14, -19),
    S(-7, -33),
    S(-16, -45),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-2, -0),
    S(-12, -19),
    S(-23, -14),
    S(-22, -24),
    S(-24, -26),
    S(-17, -10),
    S(-14, -18),
    S(-13, 6),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(23, 76);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-21, -16), S(-41, 8), S(-34, 5), S(-16, 9), S(-16, 16)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(54, -64), //Queen
    S(75, -18), //Rook
    S(47, 34),  //Bishop
    S(39, 5),   //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(29, -49), //Queen
    S(57, -16), //Rook
    S(28, 28),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(48, 41), //Queen
    S(42, 3),  //Rook
    S(0, 0),   //Bishop
    S(24, 16), //Knight
    S(0, 0),   //Pawn
];

// Mobility bonuses for knights, ordered by the number of moves available (0-8)
pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(0, 0),
    S(1, 1),
    S(2, 2),
    S(3, 3),
    S(4, 4),
    S(5, 5),
    S(6, 6),
    S(7, 7),
    S(8, 8),
];

// Mobility bonuses for bishops, ordered by the number of moves available (0-13)
pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(0, 0),
    S(1, 1),
    S(2, 2),
    S(3, 3),
    S(4, 4),
    S(5, 5),
    S(6, 6),
    S(7, 7),
    S(8, 8),
    S(9, 9),
    S(10, 10),
    S(11, 11),
    S(12, 12),
    S(13, 13),
];

// Mobility bonuses for rooks, ordered by the number of moves available (0-14)
pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(0, 0),
    S(1, 1),
    S(2, 2),
    S(3, 3),
    S(4, 4),
    S(5, 5),
    S(6, 6),
    S(7, 7),
    S(8, 8),
    S(9, 9),
    S(10, 10),
    S(11, 11),
    S(12, 12),
    S(13, 13),
    S(14, 14),
];

// Mobility bonuses for queens, ordered by the number of moves available (0-27)
pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(0, 0),
    S(1, 1),
    S(2, 2),
    S(3, 3),
    S(4, 4),
    S(5, 5),
    S(6, 6),
    S(7, 7),
    S(8, 8),
    S(9, 9),
    S(10, 10),
    S(11, 11),
    S(12, 12),
    S(13, 13),
    S(14, 14),
    S(15, 15),
    S(16, 16),
    S(17, 17),
    S(18, 18),
    S(19, 19),
    S(20, 20),
    S(21, 21),
    S(22, 22),
    S(23, 23),
    S(24, 24),
    S(25, 25),
    S(26, 26),
    S(27, 27),
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
