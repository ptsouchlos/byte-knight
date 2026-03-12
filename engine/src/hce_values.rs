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
        S(  28,  -96), S(  28,  -40), S(  33,  -23), S(-101,   27), S( -62,   13), S( -20,   13), S(  39,   -3), S( 194, -124),
        S(-117,   13), S( -48,   45), S( -82,   53), S(  33,   35), S( -23,   56), S( -16,   70), S(  -6,   62), S( -58,   28),
        S(-144,   27), S(  -2,   46), S( -63,   62), S( -84,   74), S( -40,   77), S(  50,   64), S( -16,   70), S( -59,   34),
        S( -98,   11), S( -78,   42), S( -94,   59), S(-138,   72), S(-125,   71), S( -92,   63), S( -97,   55), S(-160,   36),
        S(-101,    1), S( -76,   26), S( -99,   46), S(-129,   61), S(-126,   58), S( -87,   41), S(-103,   32), S(-167,   27),
        S( -52,  -10), S( -11,    8), S( -67,   29), S( -81,   41), S( -72,   40), S( -67,   29), S( -32,   11), S( -72,    4),
        S(  38,  -29), S(   1,   -0), S( -14,   13), S( -49,   23), S( -50,   26), S( -34,   17), S(  11,   -2), S(  11,  -22),
        S(  20,  -69), S(  51,  -46), S(  13,  -21), S( -81,   -1), S( -28,  -20), S( -50,   -4), S(  18,  -35), S(  19,  -70),
    ],
    // Queen
    [
        S( 923, 1412), S( 914, 1429), S( 934, 1448), S( 964, 1427), S( 956, 1429), S( 964, 1433), S( 998, 1380), S( 939, 1421),
        S( 962, 1403), S( 941, 1437), S( 936, 1471), S( 923, 1490), S( 912, 1511), S( 956, 1460), S( 953, 1450), S( 998, 1433),
        S( 970, 1415), S( 965, 1432), S( 963, 1465), S( 960, 1470), S( 961, 1477), S( 998, 1454), S(1005, 1426), S( 996, 1416),
        S( 955, 1433), S( 962, 1446), S( 955, 1455), S( 952, 1476), S( 960, 1476), S( 964, 1463), S( 975, 1460), S( 971, 1440),
        S( 961, 1419), S( 953, 1445), S( 956, 1446), S( 964, 1463), S( 967, 1458), S( 963, 1451), S( 974, 1436), S( 975, 1431),
        S( 959, 1401), S( 965, 1421), S( 969, 1431), S( 961, 1437), S( 971, 1444), S( 974, 1434), S( 984, 1414), S( 978, 1400),
        S( 966, 1388), S( 969, 1393), S( 974, 1399), S( 983, 1407), S( 981, 1413), S( 991, 1379), S( 997, 1348), S(1003, 1325),
        S( 958, 1394), S( 967, 1389), S( 972, 1399), S( 974, 1412), S( 978, 1394), S( 967, 1382), S( 984, 1361), S( 977, 1358),
    ],
    // Rook
    [
        S( 454,  790), S( 448,  802), S( 448,  803), S( 444,  798), S( 448,  789), S( 469,  797), S( 446,  803), S( 431,  798),
        S( 447,  792), S( 447,  810), S( 464,  806), S( 479,  794), S( 453,  794), S( 478,  797), S( 456,  799), S( 444,  792),
        S( 441,  789), S( 474,  793), S( 474,  787), S( 471,  783), S( 501,  768), S( 503,  776), S( 529,  774), S( 466,  772),
        S( 440,  789), S( 458,  791), S( 461,  791), S( 467,  786), S( 468,  770), S( 470,  779), S( 464,  783), S( 446,  774),
        S( 432,  777), S( 434,  788), S( 449,  780), S( 453,  779), S( 455,  772), S( 433,  787), S( 452,  776), S( 431,  767),
        S( 429,  769), S( 437,  775), S( 447,  768), S( 444,  771), S( 453,  761), S( 451,  766), S( 473,  748), S( 449,  744),
        S( 429,  760), S( 438,  771), S( 453,  767), S( 454,  767), S( 458,  756), S( 459,  761), S( 458,  753), S( 427,  749),
        S( 443,  766), S( 451,  767), S( 458,  770), S( 463,  763), S( 469,  753), S( 462,  764), S( 457,  758), S( 438,  750),
    ],
    // Bishop
    [
        S( 309,  434), S( 285,  439), S( 280,  433), S( 231,  443), S( 240,  440), S( 249,  431), S( 290,  431), S( 278,  424),
        S( 322,  418), S( 327,  430), S( 326,  429), S( 313,  431), S( 307,  427), S( 318,  429), S( 294,  438), S( 311,  419),
        S( 336,  435), S( 351,  430), S( 343,  438), S( 344,  430), S( 337,  434), S( 370,  440), S( 355,  434), S( 339,  438),
        S( 322,  432), S( 337,  441), S( 342,  440), S( 357,  456), S( 351,  446), S( 349,  445), S( 331,  439), S( 321,  435),
        S( 329,  428), S( 322,  441), S( 338,  448), S( 359,  449), S( 358,  447), S( 342,  440), S( 338,  436), S( 340,  420),
        S( 327,  429), S( 350,  440), S( 351,  440), S( 349,  445), S( 355,  449), S( 356,  438), S( 356,  430), S( 352,  420),
        S( 343,  434), S( 349,  417), S( 358,  419), S( 344,  430), S( 356,  428), S( 365,  422), S( 368,  426), S( 355,  415),
        S( 335,  420), S( 359,  435), S( 338,  424), S( 333,  428), S( 341,  426), S( 336,  435), S( 356,  414), S( 360,  399),
    ],
    // Knight
    [
        S( 183,  352), S( 202,  404), S( 255,  421), S( 284,  412), S( 322,  414), S( 262,  388), S( 225,  404), S( 229,  332),
        S( 298,  411), S( 315,  426), S( 318,  426), S( 333,  427), S( 332,  415), S( 363,  409), S( 317,  421), S( 326,  395),
        S( 323,  415), S( 338,  424), S( 347,  449), S( 353,  452), S( 365,  445), S( 403,  426), S( 344,  424), S( 352,  406),
        S( 330,  430), S( 337,  439), S( 354,  454), S( 380,  457), S( 349,  462), S( 373,  458), S( 331,  451), S( 364,  424),
        S( 322,  433), S( 333,  430), S( 344,  452), S( 355,  452), S( 361,  457), S( 353,  446), S( 361,  433), S( 335,  431),
        S( 301,  414), S( 322,  420), S( 332,  428), S( 341,  445), S( 356,  443), S( 341,  421), S( 344,  416), S( 328,  418),
        S( 295,  413), S( 310,  420), S( 319,  421), S( 338,  419), S( 339,  419), S( 335,  416), S( 327,  411), S( 324,  425),
        S( 261,  417), S( 305,  404), S( 301,  415), S( 319,  419), S( 325,  420), S( 331,  408), S( 311,  408), S( 296,  415),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 193,  331), S( 184,  328), S( 172,  322), S( 217,  258), S( 176,  265), S( 166,  273), S(  60,  337), S(  84,  331),
        S(  97,  174), S(  90,  188), S( 120,  140), S( 124,  100), S( 126,  100), S( 168,  115), S( 128,  171), S( 118,  152),
        S(  72,  153), S(  79,  154), S(  84,  132), S(  88,  112), S( 105,  114), S( 115,  115), S(  97,  147), S( 105,  122),
        S(  67,  131), S(  72,  142), S(  80,  122), S(  96,  115), S(  96,  116), S(  98,  116), S(  88,  135), S(  94,  108),
        S(  63,  127), S(  74,  135), S(  77,  122), S(  80,  123), S(  94,  126), S(  95,  118), S( 108,  127), S(  97,  105),
        S(  64,  129), S(  74,  137), S(  73,  127), S(  77,  124), S(  87,  135), S( 114,  117), S( 121,  123), S(  91,  103),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-31, 44),
    S(4, 140),
    S(10, 70),
    S(-14, 42),
    S(-16, 14),
    S(-10, 8),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-25, -35),
    S(-3, -28),
    S(-7, -22),
    S(-7, -9),
    S(-18, -4),
    S(-20, -17),
    S(-12, -28),
    S(-29, -41),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-3, -2),
    S(-9, -17),
    S(-19, -14),
    S(-18, -22),
    S(-21, -23),
    S(-15, -10),
    S(-8, -18),
    S(-10, 3),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(20, 70);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-16, -12), S(-24, 8), S(-25, 7), S(-14, 10), S(-17, 15)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(84, -41), //Queen
    S(93, 9),   //Rook
    S(66, 50),  //Bishop
    S(64, 26),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(57, -25), //Queen
    S(72, 14),  //Rook
    S(33, 36),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(76, 57), //Queen
    S(55, 28), //Rook
    S(0, 0),   //Bishop
    S(24, 24), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-42, -79),
    S(-21, -42),
    S(-9, -20),
    S(-3, -10),
    S(4, -1),
    S(10, 9),
    S(18, 7),
    S(25, 3),
    S(31, -8),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-29, -69),
    S(-17, -49),
    S(-6, -32),
    S(1, -17),
    S(8, -6),
    S(11, 6),
    S(14, 10),
    S(16, 14),
    S(16, 18),
    S(21, 15),
    S(27, 10),
    S(32, 10),
    S(29, 19),
    S(43, -6),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-39, -68),
    S(-28, -46),
    S(-25, -43),
    S(-21, -39),
    S(-22, -32),
    S(-17, -27),
    S(-15, -21),
    S(-12, -19),
    S(-9, -15),
    S(-7, -11),
    S(-5, -9),
    S(-7, -3),
    S(-5, -1),
    S(-6, -4),
    S(-17, -1),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-27, -234),
    S(-29, -183),
    S(-39, -110),
    S(-35, -86),
    S(-33, -72),
    S(-29, -67),
    S(-25, -51),
    S(-24, -39),
    S(-21, -31),
    S(-19, -27),
    S(-17, -20),
    S(-15, -14),
    S(-13, -13),
    S(-14, -7),
    S(-12, -6),
    S(-9, -3),
    S(-10, 5),
    S(-8, 3),
    S(2, -2),
    S(15, -11),
    S(20, -11),
    S(73, -45),
    S(64, -41),
    S(88, -66),
    S(223, -121),
    S(226, -149),
    S(148, -94),
    S(89, -82),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(30, 26);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(34, 8),
    S(30, 2),
    S(25, 12),
    S(27, 13),
    S(34, 17),
    S(41, 2),
    S(66, -4),
    S(110, -9),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(5, 49),
    S(7, 19),
    S(6, 19),
    S(17, 8),
    S(13, 11),
    S(17, 1),
    S(36, -0),
    S(35, 24),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(1, 1), S(1, 1), S(1, 1), S(1, 1)],
    // Left adjacent
    [S(1, 1), S(1, 1), S(1, 1), S(1, 1)],
    // Right adjacent
    [S(1, 1), S(1, 1), S(1, 1), S(1, 1)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(1, 1), S(1, 1), S(1, 1), S(1, 1)],
    // Left adjacent
    [S(1, 1), S(1, 1), S(1, 1), S(1, 1)],
    // Right adjacent
    [S(1, 1), S(1, 1), S(1, 1), S(1, 1)],
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
