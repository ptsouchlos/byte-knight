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
        S(  38,  -91), S(  38,  -37), S(  44,  -21), S( -80,   25), S( -45,   12), S(  -8,   12), S(  52,   -7), S( 202, -126),
        S( -99,   15), S( -29,   48), S( -73,   59), S(  44,   39), S(  -8,   61), S(   1,   73), S(  23,   61), S( -29,   27),
        S(-122,   31), S(   8,   53), S( -56,   72), S( -79,   84), S( -27,   87), S(  66,   74), S(  12,   75), S( -22,   36),
        S( -72,   14), S( -55,   48), S( -83,   69), S(-132,   83), S(-114,   83), S( -73,   75), S( -73,   64), S(-129,   40),
        S( -76,    0), S( -54,   27), S( -79,   51), S(-115,   67), S(-112,   64), S( -65,   47), S( -75,   35), S(-138,   28),
        S( -32,  -12), S(   5,    6), S( -51,   30), S( -75,   44), S( -63,   41), S( -53,   31), S( -18,   11), S( -45,    1),
        S(  32,  -30), S( -16,   -2), S( -24,    8), S( -49,   15), S( -51,   22), S( -37,   15), S(  -7,   -1), S(  21,  -31),
        S(   7,  -76), S(  10,  -42), S(  -5,  -26), S( -75,  -16), S( -20,  -33), S( -55,   -8), S(  -7,  -33), S(  16,  -80),
    ],
    // Queen
    [
        S( 928, 1420), S( 921, 1436), S( 943, 1454), S( 969, 1437), S( 965, 1438), S( 975, 1438), S(1018, 1381), S( 957, 1421),
        S( 971, 1408), S( 950, 1442), S( 946, 1476), S( 932, 1497), S( 925, 1516), S( 977, 1463), S( 985, 1444), S(1026, 1431),
        S( 976, 1422), S( 972, 1437), S( 971, 1470), S( 970, 1475), S( 975, 1482), S(1007, 1461), S(1016, 1433), S(1000, 1431),
        S( 961, 1439), S( 970, 1450), S( 965, 1459), S( 962, 1483), S( 969, 1482), S( 975, 1470), S( 983, 1469), S( 980, 1446),
        S( 969, 1424), S( 960, 1451), S( 966, 1452), S( 974, 1468), S( 977, 1464), S( 972, 1458), S( 983, 1443), S( 985, 1437),
        S( 967, 1406), S( 973, 1429), S( 978, 1437), S( 970, 1442), S( 979, 1449), S( 983, 1439), S( 993, 1420), S( 987, 1412),
        S( 974, 1393), S( 977, 1399), S( 981, 1406), S( 989, 1413), S( 988, 1419), S( 998, 1386), S(1003, 1362), S(1008, 1338),
        S( 964, 1401), S( 973, 1396), S( 977, 1408), S( 980, 1418), S( 986, 1400), S( 974, 1389), S( 982, 1373), S( 984, 1363),
    ],
    // Rook
    [
        S( 461,  790), S( 455,  804), S( 455,  804), S( 447,  801), S( 458,  790), S( 478,  798), S( 460,  803), S( 453,  793),
        S( 451,  795), S( 451,  813), S( 470,  808), S( 482,  797), S( 463,  795), S( 488,  799), S( 474,  798), S( 469,  788),
        S( 444,  791), S( 476,  796), S( 476,  790), S( 472,  787), S( 506,  769), S( 504,  779), S( 536,  776), S( 479,  771),
        S( 442,  791), S( 459,  794), S( 464,  793), S( 468,  789), S( 473,  771), S( 471,  782), S( 468,  786), S( 456,  775),
        S( 435,  779), S( 435,  791), S( 452,  782), S( 454,  781), S( 459,  774), S( 435,  790), S( 458,  778), S( 442,  767),
        S( 431,  770), S( 438,  777), S( 449,  770), S( 444,  775), S( 458,  763), S( 452,  770), S( 480,  750), S( 457,  746),
        S( 431,  762), S( 440,  773), S( 455,  769), S( 455,  770), S( 462,  758), S( 462,  763), S( 472,  752), S( 439,  750),
        S( 444,  768), S( 452,  770), S( 459,  773), S( 463,  766), S( 471,  757), S( 463,  767), S( 457,  764), S( 450,  751),
    ],
    // Bishop
    [
        S( 308,  437), S( 285,  441), S( 280,  434), S( 231,  445), S( 242,  442), S( 246,  431), S( 296,  431), S( 278,  424),
        S( 324,  419), S( 328,  433), S( 328,  431), S( 313,  435), S( 310,  428), S( 326,  429), S( 299,  437), S( 306,  423),
        S( 336,  437), S( 353,  431), S( 344,  440), S( 346,  430), S( 345,  434), S( 371,  442), S( 356,  436), S( 338,  440),
        S( 322,  434), S( 339,  442), S( 345,  440), S( 363,  458), S( 354,  447), S( 351,  447), S( 333,  440), S( 323,  436),
        S( 332,  429), S( 324,  443), S( 341,  449), S( 363,  449), S( 361,  448), S( 344,  441), S( 341,  436), S( 342,  421),
        S( 328,  430), S( 354,  441), S( 354,  441), S( 352,  445), S( 358,  450), S( 359,  439), S( 359,  430), S( 355,  422),
        S( 346,  435), S( 352,  418), S( 361,  419), S( 347,  431), S( 358,  430), S( 367,  424), S( 375,  424), S( 360,  415),
        S( 337,  420), S( 360,  439), S( 339,  426), S( 335,  429), S( 344,  427), S( 338,  438), S( 350,  419), S( 370,  396),
    ],
    // Knight
    [
        S( 175,  354), S( 203,  404), S( 253,  422), S( 283,  413), S( 319,  416), S( 260,  387), S( 223,  401), S( 221,  335),
        S( 298,  412), S( 319,  426), S( 321,  425), S( 337,  425), S( 332,  416), S( 369,  408), S( 323,  420), S( 333,  393),
        S( 326,  415), S( 341,  424), S( 351,  449), S( 359,  452), S( 371,  445), S( 407,  426), S( 350,  423), S( 352,  407),
        S( 334,  431), S( 341,  439), S( 359,  454), S( 384,  457), S( 356,  461), S( 380,  457), S( 338,  451), S( 366,  426),
        S( 325,  433), S( 336,  431), S( 348,  453), S( 358,  452), S( 365,  457), S( 356,  447), S( 366,  433), S( 340,  432),
        S( 304,  414), S( 325,  420), S( 334,  429), S( 344,  446), S( 360,  444), S( 344,  422), S( 347,  417), S( 331,  421),
        S( 299,  413), S( 312,  421), S( 322,  422), S( 341,  421), S( 342,  420), S( 340,  417), S( 335,  411), S( 332,  426),
        S( 265,  419), S( 305,  406), S( 303,  416), S( 322,  421), S( 328,  421), S( 333,  410), S( 308,  414), S( 304,  419),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 182,  310), S( 180,  312), S( 146,  316), S( 171,  278), S( 134,  297), S( 159,  300), S( 139,  344), S( 124,  331),
        S(  89,  186), S(  79,  208), S( 110,  163), S( 109,  132), S( 107,  139), S( 126,  163), S( 103,  207), S(  65,  194),
        S(  71,  156), S(  76,  158), S(  84,  133), S(  82,  115), S( 102,  116), S(  91,  124), S(  86,  150), S(  66,  135),
        S(  66,  134), S(  70,  144), S(  80,  123), S(  95,  116), S(  94,  117), S(  90,  117), S(  87,  132), S(  66,  117),
        S(  62,  129), S(  71,  139), S(  76,  122), S(  77,  125), S(  88,  129), S(  81,  121), S(  95,  127), S(  66,  114),
        S(  62,  131), S(  70,  141), S(  70,  129), S(  63,  133), S(  76,  141), S(  83,  126), S(  98,  129), S(  54,  118),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-8, 83),
    S(7, 140),
    S(13, 70),
    S(-11, 42),
    S(-13, 14),
    S(-7, 10),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-18, -38),
    S(2, -29),
    S(-4, -21),
    S(-5, -9),
    S(-13, -3),
    S(-7, -17),
    S(1, -24),
    S(-6, -43),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-7, -1),
    S(-8, -19),
    S(-19, -14),
    S(-17, -23),
    S(-21, -25),
    S(-7, -14),
    S(-10, -18),
    S(-0, -2),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(21, 70);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-15, -12), S(-21, 8), S(-24, 6), S(-12, 9), S(-13, 6)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(86, -44), //Queen
    S(94, 5),   //Rook
    S(67, 52),  //Bishop
    S(66, 28),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(58, -26), //Queen
    S(73, 15),  //Rook
    S(34, 37),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(77, 58), //Queen
    S(57, 28), //Rook
    S(0, 0),   //Bishop
    S(24, 24), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-35, -78),
    S(-13, -39),
    S(-1, -16),
    S(6, -6),
    S(13, 3),
    S(19, 13),
    S(28, 11),
    S(35, 8),
    S(41, -5),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-19, -67),
    S(-8, -45),
    S(4, -27),
    S(11, -12),
    S(17, -1),
    S(21, 11),
    S(24, 16),
    S(26, 19),
    S(27, 23),
    S(32, 20),
    S(39, 15),
    S(43, 15),
    S(40, 25),
    S(53, -1),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-22, -65),
    S(-14, -41),
    S(-9, -37),
    S(-6, -33),
    S(-9, -24),
    S(-2, -20),
    S(-1, -13),
    S(1, -12),
    S(5, -8),
    S(6, -3),
    S(9, -1),
    S(7, 4),
    S(10, 6),
    S(9, 4),
    S(0, 5),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-5, -250),
    S(-7, -183),
    S(-17, -103),
    S(-14, -78),
    S(-12, -60),
    S(-8, -55),
    S(-5, -37),
    S(-4, -24),
    S(-1, -15),
    S(1, -12),
    S(3, -5),
    S(5, 1),
    S(7, 2),
    S(6, 8),
    S(8, 9),
    S(11, 13),
    S(10, 21),
    S(12, 19),
    S(22, 15),
    S(36, 5),
    S(40, 6),
    S(93, -28),
    S(86, -24),
    S(104, -46),
    S(239, -101),
    S(248, -131),
    S(189, -81),
    S(116, -71),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(30, 27);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(33, 9),
    S(30, 2),
    S(25, 12),
    S(30, 12),
    S(32, 18),
    S(48, -0),
    S(62, -4),
    S(88, -2),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(2, 52),
    S(7, 20),
    S(7, 19),
    S(18, 10),
    S(7, 15),
    S(23, -2),
    S(29, 2),
    S(8, 35),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(68, -21), S(54, -10), S(41, -7), S(32, 7)],
    // Left adjacent
    [S(41, -21), S(33, -9), S(23, -0), S(15, 8)],
    // Right adjacent
    [S(45, -10), S(22, -6), S(16, -3), S(14, 6)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(68, -21), S(54, -10), S(41, -7), S(32, 7)],
    // Left adjacent
    [S(41, -21), S(33, -9), S(23, -0), S(15, 8)],
    // Right adjacent
    [S(45, -10), S(22, -6), S(16, -3), S(14, 6)],
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
