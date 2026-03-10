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
        S(  25,  -94), S(  22,  -37), S(  31,  -21), S(-102,   29), S( -62,   13), S( -21,   13), S(  38,   -4), S( 197, -127),
        S(-123,   16), S( -51,   47), S( -84,   54), S(  34,   35), S( -20,   56), S( -13,   70), S(  -1,   61), S( -55,   27),
        S(-146,   30), S(  -5,   47), S( -65,   63), S( -81,   74), S( -35,   77), S(  57,   64), S( -11,   69), S( -52,   32),
        S(-103,   14), S( -81,   44), S( -96,   60), S(-135,   72), S(-122,   71), S( -88,   63), S( -94,   55), S(-158,   35),
        S(-106,    2), S( -78,   26), S(-100,   46), S(-127,   61), S(-124,   58), S( -86,   41), S(-101,   32), S(-167,   26),
        S( -56,  -10), S( -15,    9), S( -69,   29), S( -81,   41), S( -72,   40), S( -68,   29), S( -32,   11), S( -74,    4),
        S(  32,  -29), S(  -2,   -1), S( -18,   13), S( -51,   23), S( -53,   26), S( -36,   17), S(  10,   -4), S(  10,  -24),
        S(  18,  -70), S(  52,  -47), S(  16,  -20), S( -79,   -2), S( -25,  -20), S( -48,   -5), S(  21,  -37), S(  19,  -73),
    ],
    // Queen
    [
        S( 924, 1416), S( 911, 1436), S( 931, 1454), S( 959, 1435), S( 951, 1437), S( 960, 1441), S( 995, 1387), S( 943, 1423),
        S( 964, 1411), S( 940, 1444), S( 937, 1477), S( 924, 1496), S( 915, 1516), S( 960, 1465), S( 955, 1456), S(1009, 1433),
        S( 971, 1424), S( 967, 1438), S( 963, 1472), S( 961, 1476), S( 961, 1485), S( 998, 1462), S(1007, 1432), S( 999, 1423),
        S( 955, 1442), S( 962, 1452), S( 955, 1463), S( 951, 1483), S( 960, 1483), S( 964, 1471), S( 975, 1468), S( 970, 1448),
        S( 961, 1426), S( 952, 1452), S( 956, 1453), S( 964, 1470), S( 967, 1466), S( 961, 1459), S( 972, 1443), S( 976, 1437),
        S( 959, 1407), S( 965, 1426), S( 969, 1438), S( 961, 1444), S( 970, 1452), S( 974, 1439), S( 983, 1419), S( 979, 1405),
        S( 968, 1391), S( 968, 1399), S( 974, 1406), S( 984, 1412), S( 982, 1417), S( 993, 1382), S( 998, 1349), S(1013, 1318),
        S( 961, 1396), S( 971, 1391), S( 978, 1400), S( 979, 1416), S( 984, 1395), S( 970, 1384), S( 988, 1362), S( 980, 1360),
    ],
    // Rook
    [
        S( 460,  793), S( 444,  802), S( 438,  811), S( 434,  807), S( 444,  800), S( 469,  795), S( 458,  799), S( 490,  786),
        S( 453,  796), S( 447,  809), S( 461,  813), S( 479,  802), S( 460,  804), S( 485,  794), S( 481,  790), S( 511,  777),
        S( 451,  794), S( 479,  792), S( 477,  793), S( 481,  790), S( 514,  776), S( 508,  775), S( 549,  768), S( 512,  765),
        S( 445,  797), S( 459,  792), S( 462,  799), S( 471,  794), S( 478,  780), S( 473,  778), S( 480,  779), S( 475,  773),
        S( 432,  788), S( 435,  790), S( 448,  789), S( 457,  786), S( 459,  783), S( 434,  787), S( 461,  774), S( 443,  773),
        S( 426,  780), S( 437,  777), S( 447,  775), S( 448,  778), S( 456,  772), S( 450,  766), S( 476,  748), S( 452,  753),
        S( 425,  770), S( 437,  774), S( 457,  772), S( 460,  772), S( 464,  764), S( 459,  759), S( 473,  750), S( 440,  757),
        S( 438,  767), S( 450,  768), S( 463,  773), S( 470,  767), S( 475,  761), S( 462,  762), S( 470,  755), S( 439,  752),
    ],
    // Bishop
    [
        S( 308,  435), S( 283,  440), S( 277,  434), S( 230,  445), S( 240,  441), S( 250,  431), S( 291,  432), S( 274,  425),
        S( 322,  421), S( 326,  431), S( 326,  430), S( 313,  433), S( 308,  428), S( 321,  429), S( 296,  438), S( 315,  420),
        S( 337,  437), S( 353,  430), S( 343,  439), S( 346,  431), S( 337,  435), S( 373,  441), S( 358,  434), S( 343,  438),
        S( 323,  434), S( 337,  441), S( 343,  441), S( 358,  457), S( 352,  447), S( 350,  446), S( 331,  440), S( 321,  436),
        S( 329,  429), S( 323,  442), S( 338,  448), S( 360,  450), S( 358,  448), S( 342,  441), S( 337,  436), S( 339,  421),
        S( 328,  429), S( 350,  440), S( 351,  440), S( 349,  445), S( 355,  449), S( 356,  439), S( 355,  430), S( 352,  421),
        S( 345,  434), S( 349,  417), S( 358,  419), S( 345,  431), S( 357,  428), S( 365,  422), S( 367,  426), S( 355,  415),
        S( 335,  420), S( 361,  435), S( 343,  426), S( 338,  428), S( 345,  426), S( 337,  436), S( 357,  415), S( 358,  401),
    ],
    // Knight
    [
        S( 180,  353), S( 198,  405), S( 252,  423), S( 283,  413), S( 324,  414), S( 263,  388), S( 220,  405), S( 227,  331),
        S( 299,  414), S( 315,  427), S( 317,  427), S( 334,  428), S( 330,  417), S( 366,  409), S( 319,  422), S( 333,  395),
        S( 324,  418), S( 339,  425), S( 347,  450), S( 353,  454), S( 366,  446), S( 404,  427), S( 347,  424), S( 352,  407),
        S( 330,  432), S( 337,  439), S( 355,  455), S( 381,  458), S( 349,  463), S( 373,  459), S( 330,  452), S( 363,  426),
        S( 322,  434), S( 334,  431), S( 344,  453), S( 355,  452), S( 362,  458), S( 353,  447), S( 360,  434), S( 335,  431),
        S( 302,  413), S( 322,  420), S( 332,  428), S( 341,  446), S( 356,  443), S( 342,  421), S( 342,  416), S( 328,  418),
        S( 297,  412), S( 310,  420), S( 319,  420), S( 339,  420), S( 340,  419), S( 336,  415), S( 325,  412), S( 323,  425),
        S( 259,  415), S( 309,  403), S( 305,  415), S( 324,  419), S( 327,  420), S( 330,  407), S( 313,  407), S( 294,  415),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 189,  319), S( 186,  328), S( 174,  322), S( 209,  261), S( 173,  268), S( 164,  279), S(  65,  339), S(  77,  330),
        S(  89,  169), S(  85,  189), S( 115,  139), S( 116,  101), S( 118,  101), S( 158,  119), S( 123,  173), S( 104,  153),
        S(  66,  149), S(  74,  154), S(  82,  131), S(  83,  112), S( 101,  115), S( 106,  118), S(  89,  149), S(  92,  124),
        S(  63,  127), S(  68,  142), S(  79,  122), S(  93,  115), S(  94,  117), S(  91,  118), S(  80,  137), S(  83,  110),
        S(  60,  122), S(  72,  135), S(  78,  121), S(  78,  123), S(  93,  127), S(  90,  120), S( 101,  128), S(  88,  106),
        S(  63,  124), S(  72,  137), S(  75,  125), S(  76,  126), S(  87,  137), S( 109,  120), S( 114,  125), S(  83,  104),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-44, 44),
    S(3, 140),
    S(10, 70),
    S(-14, 42),
    S(-15, 14),
    S(-8, 7),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-20, -27),
    S(2, -26),
    S(-6, -20),
    S(-3, -8),
    S(-14, -3),
    S(-13, -18),
    S(-3, -29),
    S(-14, -41),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-5, -2),
    S(-9, -18),
    S(-21, -14),
    S(-18, -22),
    S(-21, -24),
    S(-15, -11),
    S(-11, -17),
    S(-15, 3),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(20, 71);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-17, -11), S(-30, 10), S(-25, 7), S(-15, 10), S(-17, 15)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(85, -41), //Queen
    S(96, 8),   //Rook
    S(67, 49),  //Bishop
    S(66, 26),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(58, -25), //Queen
    S(72, 15),  //Rook
    S(33, 36),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(77, 58), //Queen
    S(56, 29), //Rook
    S(0, 0),   //Bishop
    S(24, 24), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-42, -76),
    S(-20, -39),
    S(-8, -17),
    S(-1, -7),
    S(6, 2),
    S(12, 12),
    S(20, 11),
    S(27, 7),
    S(33, -5),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-28, -67),
    S(-16, -47),
    S(-4, -29),
    S(3, -14),
    S(10, -3),
    S(13, 9),
    S(17, 14),
    S(18, 18),
    S(19, 22),
    S(24, 18),
    S(30, 14),
    S(35, 13),
    S(31, 22),
    S(48, -3),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-33, -60),
    S(-22, -39),
    S(-17, -37),
    S(-11, -33),
    S(-11, -24),
    S(-3, -20),
    S(1, -13),
    S(8, -11),
    S(15, -7),
    S(20, -1),
    S(25, 1),
    S(24, 8),
    S(27, 12),
    S(29, 10),
    S(16, 13),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-29, -215),
    S(-31, -169),
    S(-39, -98),
    S(-34, -75),
    S(-31, -60),
    S(-26, -57),
    S(-22, -41),
    S(-20, -28),
    S(-17, -20),
    S(-15, -16),
    S(-14, -9),
    S(-11, -3),
    S(-9, -1),
    S(-10, 5),
    S(-8, 6),
    S(-5, 9),
    S(-5, 16),
    S(-3, 14),
    S(7, 9),
    S(20, -1),
    S(25, -1),
    S(79, -35),
    S(72, -33),
    S(96, -58),
    S(230, -113),
    S(225, -135),
    S(128, -70),
    S(72, -62),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(30, 26);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
    S(1, 1),
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
