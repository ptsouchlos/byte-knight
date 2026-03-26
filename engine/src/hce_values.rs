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
        S(  32,  -87), S(  30,  -36), S(  34,  -21), S( -87,   24), S( -52,   12), S( -17,   11), S(  42,   -8), S( 194, -121),
        S(-103,   14), S( -37,   45), S( -79,   56), S(  33,   37), S( -17,   58), S(  -8,   69), S(  12,   57), S( -37,   25),
        S(-126,   30), S(  -2,   51), S( -64,   69), S( -85,   80), S( -36,   82), S(  53,   70), S(   1,   71), S( -31,   34),
        S( -78,   13), S( -62,   46), S( -88,   66), S(-135,   79), S(-118,   78), S( -79,   71), S( -79,   61), S(-132,   38),
        S( -81,    0), S( -60,   25), S( -84,   48), S(-118,   63), S(-115,   61), S( -71,   45), S( -81,   33), S(-140,   26),
        S( -40,  -12), S(  -5,    6), S( -58,   28), S( -80,   41), S( -69,   39), S( -60,   29), S( -27,   10), S( -52,    1),
        S(  20,  -28), S( -24,   -2), S( -32,    8), S( -56,   14), S( -58,   20), S( -44,   14), S( -16,   -1), S(  10,  -29),
        S(  -3,  -72), S(  -0,  -40), S( -14,  -25), S( -80,  -16), S( -28,  -31), S( -62,   -8), S( -16,  -31), S(   5,  -76),
    ],
    // Queen
    [
        S( 914, 1398), S( 907, 1413), S( 928, 1430), S( 953, 1414), S( 948, 1415), S( 958, 1415), S( 999, 1361), S( 940, 1399),
        S( 954, 1387), S( 935, 1419), S( 930, 1451), S( 917, 1471), S( 910, 1489), S( 959, 1439), S( 967, 1420), S(1006, 1408),
        S( 959, 1400), S( 955, 1414), S( 954, 1445), S( 953, 1450), S( 958, 1457), S( 988, 1437), S( 996, 1411), S( 981, 1408),
        S( 945, 1416), S( 953, 1426), S( 949, 1435), S( 945, 1457), S( 952, 1456), S( 957, 1445), S( 966, 1444), S( 963, 1423),
        S( 952, 1402), S( 944, 1427), S( 950, 1428), S( 957, 1443), S( 959, 1439), S( 955, 1434), S( 966, 1420), S( 968, 1414),
        S( 950, 1384), S( 956, 1406), S( 960, 1414), S( 953, 1418), S( 962, 1425), S( 965, 1416), S( 975, 1398), S( 969, 1390),
        S( 957, 1372), S( 959, 1378), S( 964, 1385), S( 971, 1392), S( 970, 1396), S( 980, 1366), S( 985, 1343), S( 989, 1320),
        S( 947, 1380), S( 956, 1375), S( 960, 1387), S( 963, 1396), S( 968, 1379), S( 957, 1369), S( 964, 1354), S( 966, 1345),
    ],
    // Rook
    [
        S( 455,  781), S( 449,  793), S( 449,  794), S( 441,  791), S( 452,  780), S( 471,  788), S( 454,  793), S( 448,  784),
        S( 443,  789), S( 443,  807), S( 460,  802), S( 472,  791), S( 454,  789), S( 477,  793), S( 465,  792), S( 460,  783),
        S( 439,  782), S( 469,  786), S( 469,  781), S( 465,  777), S( 497,  761), S( 496,  770), S( 526,  767), S( 472,  763),
        S( 437,  781), S( 453,  784), S( 458,  784), S( 462,  779), S( 466,  763), S( 464,  773), S( 462,  777), S( 450,  766),
        S( 430,  770), S( 431,  781), S( 446,  773), S( 448,  772), S( 453,  765), S( 431,  781), S( 452,  769), S( 437,  759),
        S( 426,  762), S( 433,  768), S( 443,  761), S( 439,  766), S( 452,  755), S( 447,  761), S( 472,  743), S( 451,  739),
        S( 426,  754), S( 435,  765), S( 449,  761), S( 449,  761), S( 456,  750), S( 456,  755), S( 466,  744), S( 434,  742),
        S( 439,  760), S( 446,  761), S( 453,  764), S( 456,  758), S( 464,  749), S( 457,  759), S( 451,  756), S( 444,  744),
    ],
    // Bishop
    [
        S( 306,  433), S( 284,  436), S( 279,  430), S( 233,  440), S( 244,  437), S( 248,  427), S( 295,  427), S( 278,  420),
        S( 321,  416), S( 325,  428), S( 325,  427), S( 311,  430), S( 307,  424), S( 323,  424), S( 297,  432), S( 304,  419),
        S( 333,  433), S( 348,  426), S( 339,  435), S( 342,  426), S( 341,  429), S( 366,  437), S( 351,  431), S( 334,  435),
        S( 319,  429), S( 335,  437), S( 341,  435), S( 358,  452), S( 350,  441), S( 346,  441), S( 330,  435), S( 320,  432),
        S( 328,  425), S( 321,  438), S( 337,  444), S( 358,  444), S( 356,  443), S( 340,  436), S( 337,  432), S( 338,  417),
        S( 325,  425), S( 349,  436), S( 349,  436), S( 347,  440), S( 353,  444), S( 354,  434), S( 354,  426), S( 350,  418),
        S( 342,  431), S( 347,  414), S( 356,  416), S( 342,  427), S( 353,  426), S( 362,  420), S( 369,  420), S( 355,  411),
        S( 334,  416), S( 355,  434), S( 335,  422), S( 331,  425), S( 340,  423), S( 334,  433), S( 346,  415), S( 365,  393),
    ],
    // Knight
    [
        S( 179,  355), S( 206,  402), S( 252,  419), S( 281,  411), S( 315,  413), S( 260,  386), S( 224,  399), S( 223,  336),
        S( 296,  410), S( 315,  422), S( 317,  422), S( 333,  422), S( 328,  413), S( 362,  406), S( 319,  417), S( 329,  392),
        S( 322,  413), S( 336,  421), S( 345,  444), S( 353,  448), S( 364,  441), S( 398,  423), S( 344,  420), S( 347,  404),
        S( 329,  427), S( 336,  435), S( 353,  449), S( 377,  452), S( 350,  456), S( 373,  452), S( 333,  446), S( 359,  422),
        S( 321,  430), S( 332,  427), S( 343,  448), S( 353,  448), S( 359,  452), S( 351,  442), S( 360,  429), S( 335,  428),
        S( 301,  411), S( 321,  417), S( 330,  425), S( 339,  442), S( 354,  439), S( 339,  419), S( 342,  414), S( 326,  418),
        S( 297,  410), S( 309,  418), S( 318,  419), S( 336,  418), S( 337,  417), S( 335,  414), S( 330,  409), S( 327,  422),
        S( 264,  416), S( 302,  404), S( 300,  414), S( 318,  418), S( 324,  418), S( 329,  407), S( 305,  411), S( 301,  416),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 179,  307), S( 177,  309), S( 145,  313), S( 169,  277), S( 134,  295), S( 158,  298), S( 138,  339), S( 124,  327),
        S(  84,  176), S(  75,  196), S( 104,  154), S( 103,  125), S( 101,  132), S( 119,  154), S(  97,  196), S(  62,  183),
        S(  67,  148), S(  72,  149), S(  79,  126), S(  78,  109), S(  97,  110), S(  86,  117), S(  81,  142), S(  63,  128),
        S(  63,  126), S(  66,  137), S(  75,  116), S(  89,  110), S(  89,  111), S(  85,  111), S(  82,  125), S(  63,  111),
        S(  58,  122), S(  67,  131), S(  72,  116), S(  73,  118), S(  83,  122), S(  76,  115), S(  90,  120), S(  63,  108),
        S(  59,  124), S(  66,  133), S(  66,  122), S(  59,  126), S(  72,  133), S(  79,  120), S(  93,  122), S(  51,  112),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-15, 65),
    S(7, 132),
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
    S(-20, -23),
    S(-7, -13),
    S(-9, -17),
    S(-0, -2),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(20, 66);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-14, -11), S(-20, 7), S(-23, 6), S(-11, 8), S(-13, 6)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(82, -42), //Queen
    S(89, 5),   //Rook
    S(64, 50),  //Bishop
    S(62, 26),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(54, -24), //Queen
    S(69, 14),  //Rook
    S(32, 35),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(73, 55), //Queen
    S(54, 27), //Rook
    S(0, 0),   //Bishop
    S(23, 23), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-47, -93),
    S(-26, -57),
    S(-14, -35),
    S(-8, -25),
    S(-2, -16),
    S(5, -7),
    S(13, -9),
    S(20, -12),
    S(25, -24),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-33, -82),
    S(-22, -61),
    S(-11, -44),
    S(-5, -30),
    S(2, -19),
    S(5, -8),
    S(8, -4),
    S(10, -0),
    S(11, 3),
    S(15, 0),
    S(23, -5),
    S(26, -5),
    S(23, 5),
    S(36, -20),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-40, -94),
    S(-32, -71),
    S(-28, -68),
    S(-25, -63),
    S(-27, -56),
    S(-21, -52),
    S(-20, -45),
    S(-18, -44),
    S(-15, -40),
    S(-13, -36),
    S(-10, -34),
    S(-12, -29),
    S(-9, -27),
    S(-10, -29),
    S(-19, -28),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-43, -257),
    S(-43, -219),
    S(-53, -149),
    S(-50, -126),
    S(-48, -110),
    S(-44, -105),
    S(-41, -88),
    S(-40, -76),
    S(-37, -68),
    S(-36, -64),
    S(-34, -58),
    S(-32, -52),
    S(-30, -51),
    S(-31, -45),
    S(-29, -44),
    S(-26, -41),
    S(-27, -33),
    S(-25, -35),
    S(-16, -38),
    S(-2, -48),
    S(1, -47),
    S(52, -79),
    S(45, -76),
    S(63, -97),
    S(196, -151),
    S(209, -182),
    S(153, -125),
    S(88, -113),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(29, 26);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(31, 9),
    S(28, 2),
    S(24, 12),
    S(29, 11),
    S(30, 17),
    S(45, -0),
    S(59, -4),
    S(83, -2),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(2, 49),
    S(7, 19),
    S(7, 18),
    S(17, 10),
    S(7, 14),
    S(22, -2),
    S(27, 2),
    S(7, 33),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(64, -20), S(51, -9), S(39, -6), S(30, 6)],
    // Left adjacent
    [S(42, -9), S(21, -6), S(15, -3), S(13, 6)],
    // Right adjacent
    [S(39, -19), S(31, -8), S(22, -0), S(14, 8)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(99, 222), S(-41, 116), S(-4, 24), S(9, 4)],
    // Left adjacent
    [S(-20, 204), S(-70, 99), S(-26, 25), S(-3, 4)],
    // Right adjacent
    [S(-50, 229), S(-54, 85), S(-22, 22), S(-3, 3)],
];

// Bonus for Rook on 7th Rank (relative to side). Flipped for black.
pub const ROOK_ON_7TH_BONUS: PhasedScore = S(3, -4);

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

    fn rook_on_7th_bonus(&self, _side: Side) -> Self::ReturnScore {
        ROOK_ON_7TH_BONUS
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
