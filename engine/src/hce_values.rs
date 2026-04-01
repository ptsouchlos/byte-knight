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
        S(  32,  -90), S(  31,  -37), S(  35,  -22), S( -90,   24), S( -55,   12), S( -17,   11), S(  42,   -8), S( 194, -124),
        S(-106,   14), S( -37,   46), S( -79,   57), S(  35,   38), S( -16,   59), S(  -7,   71), S(  12,   59), S( -40,   26),
        S(-128,   30), S(   0,   51), S( -62,   70), S( -85,   81), S( -34,   84), S(  57,   71), S(   4,   72), S( -31,   35),
        S( -78,   13), S( -62,   46), S( -89,   67), S(-137,   80), S(-119,   80), S( -79,   73), S( -79,   62), S(-134,   39),
        S( -82,   -0), S( -61,   25), S( -85,   49), S(-120,   64), S(-117,   62), S( -72,   45), S( -82,   34), S(-143,   26),
        S( -40,  -13), S(  -3,    5), S( -58,   28), S( -81,   42), S( -70,   40), S( -60,   29), S( -26,   10), S( -53,    0),
        S(  22,  -30), S( -24,   -2), S( -32,    7), S( -56,   14), S( -58,   20), S( -44,   14), S( -15,   -2), S(  12,  -31),
        S(  -2,  -75), S(   1,  -42), S( -13,  -26), S( -81,  -17), S( -28,  -33), S( -62,   -9), S( -15,  -33), S(   7,  -79),
    ],
    // Queen
    [
        S( 922, 1410), S( 915, 1426), S( 936, 1443), S( 962, 1426), S( 957, 1428), S( 967, 1428), S(1009, 1373), S( 949, 1411),
        S( 964, 1399), S( 943, 1432), S( 939, 1465), S( 925, 1486), S( 918, 1504), S( 969, 1452), S( 977, 1433), S(1017, 1421),
        S( 968, 1412), S( 964, 1427), S( 963, 1459), S( 963, 1464), S( 968, 1471), S( 999, 1450), S(1007, 1423), S( 991, 1420),
        S( 954, 1428), S( 962, 1440), S( 958, 1449), S( 954, 1471), S( 961, 1470), S( 967, 1459), S( 975, 1458), S( 972, 1436),
        S( 961, 1414), S( 952, 1440), S( 959, 1441), S( 966, 1457), S( 969, 1453), S( 964, 1448), S( 975, 1433), S( 977, 1427),
        S( 959, 1396), S( 965, 1419), S( 970, 1427), S( 962, 1431), S( 971, 1439), S( 975, 1429), S( 985, 1410), S( 979, 1402),
        S( 966, 1384), S( 969, 1389), S( 973, 1397), S( 981, 1404), S( 980, 1409), S( 990, 1377), S( 995, 1354), S( 999, 1330),
        S( 956, 1391), S( 965, 1386), S( 969, 1399), S( 972, 1409), S( 977, 1390), S( 966, 1380), S( 974, 1365), S( 976, 1355),
    ],
    // Rook
    [
        S( 457,  786), S( 452,  799), S( 452,  800), S( 444,  797), S( 455,  785), S( 474,  794), S( 457,  798), S( 450,  789),
        S( 448,  790), S( 448,  809), S( 466,  804), S( 478,  793), S( 460,  790), S( 484,  794), S( 471,  793), S( 466,  784),
        S( 441,  787), S( 472,  791), S( 472,  786), S( 468,  782), S( 501,  765), S( 500,  775), S( 531,  772), S( 475,  767),
        S( 439,  786), S( 456,  789), S( 461,  789), S( 465,  784), S( 469,  767), S( 467,  778), S( 465,  782), S( 452,  771),
        S( 432,  775), S( 433,  786), S( 448,  778), S( 451,  777), S( 455,  770), S( 433,  786), S( 455,  774), S( 439,  763),
        S( 428,  766), S( 436,  773), S( 446,  766), S( 441,  771), S( 454,  760), S( 449,  766), S( 476,  747), S( 454,  743),
        S( 428,  758), S( 437,  769), S( 452,  766), S( 452,  766), S( 459,  754), S( 459,  759), S( 469,  748), S( 436,  746),
        S( 441,  764), S( 449,  766), S( 456,  769), S( 459,  762), S( 467,  753), S( 460,  764), S( 454,  760), S( 447,  748),
    ],
    // Bishop
    [
        S( 307,  435), S( 285,  439), S( 280,  432), S( 232,  442), S( 243,  439), S( 247,  429), S( 295,  429), S( 278,  422),
        S( 323,  418), S( 327,  431), S( 326,  429), S( 312,  432), S( 309,  426), S( 325,  427), S( 299,  435), S( 306,  421),
        S( 335,  435), S( 351,  429), S( 342,  437), S( 345,  428), S( 343,  431), S( 369,  440), S( 354,  434), S( 337,  438),
        S( 320,  432), S( 338,  440), S( 343,  438), S( 361,  455), S( 352,  444), S( 349,  444), S( 332,  438), S( 322,  434),
        S( 330,  427), S( 323,  441), S( 339,  447), S( 361,  447), S( 359,  446), S( 342,  439), S( 339,  434), S( 341,  420),
        S( 327,  428), S( 352,  439), S( 352,  438), S( 350,  443), S( 356,  447), S( 357,  437), S( 357,  428), S( 353,  420),
        S( 345,  433), S( 350,  416), S( 359,  418), S( 345,  429), S( 356,  428), S( 365,  422), S( 372,  423), S( 358,  413),
        S( 336,  418), S( 358,  436), S( 337,  424), S( 334,  427), S( 342,  425), S( 337,  435), S( 349,  418), S( 368,  395),
    ],
    // Knight
    [
        S( 177,  354), S( 204,  403), S( 252,  420), S( 282,  412), S( 317,  415), S( 260,  387), S( 223,  400), S( 221,  335),
        S( 296,  411), S( 316,  424), S( 319,  424), S( 335,  424), S( 330,  415), S( 365,  407), S( 321,  419), S( 330,  392),
        S( 324,  414), S( 338,  422), S( 348,  447), S( 356,  450), S( 367,  443), S( 402,  425), S( 347,  422), S( 349,  406),
        S( 331,  429), S( 338,  437), S( 356,  452), S( 381,  455), S( 353,  459), S( 376,  455), S( 335,  449), S( 362,  424),
        S( 323,  432), S( 334,  429), S( 345,  451), S( 355,  450), S( 362,  455), S( 353,  445), S( 363,  432), S( 337,  430),
        S( 302,  413), S( 323,  419), S( 332,  427), S( 341,  444), S( 357,  442), S( 342,  421), S( 344,  416), S( 328,  419),
        S( 298,  412), S( 310,  419), S( 320,  420), S( 338,  419), S( 339,  418), S( 337,  416), S( 332,  410), S( 329,  424),
        S( 264,  418), S( 303,  405), S( 301,  415), S( 319,  420), S( 326,  419), S( 330,  408), S( 306,  412), S( 302,  417),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 179,  308), S( 178,  310), S( 144,  315), S( 169,  278), S( 133,  296), S( 160,  298), S( 140,  341), S( 126,  328),
        S(  86,  182), S(  77,  203), S( 107,  159), S( 106,  129), S( 104,  136), S( 123,  159), S( 100,  202), S(  64,  189),
        S(  69,  152), S(  74,  154), S(  82,  130), S(  80,  112), S( 100,  113), S(  89,  121), S(  83,  147), S(  65,  132),
        S(  65,  130), S(  68,  141), S(  78,  120), S(  92,  113), S(  92,  114), S(  88,  115), S(  85,  129), S(  65,  114),
        S(  60,  126), S(  70,  135), S(  75,  119), S(  76,  122), S(  86,  125), S(  79,  118), S(  93,  123), S(  65,  111),
        S(  61,  128), S(  68,  137), S(  68,  126), S(  61,  129), S(  74,  137), S(  81,  123), S(  96,  126), S(  53,  115),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-9, 74),
    S(7, 136),
    S(13, 69),
    S(-11, 41),
    S(-13, 14),
    S(-7, 9),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-18, -37),
    S(2, -29),
    S(-3, -21),
    S(-5, -9),
    S(-12, -2),
    S(-6, -16),
    S(1, -24),
    S(-6, -42),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-6, -1),
    S(-8, -18),
    S(-19, -13),
    S(-17, -23),
    S(-21, -24),
    S(-7, -14),
    S(-10, -18),
    S(-0, -2),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(21, 68);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-14, -11), S(-20, 8), S(-23, 6), S(-12, 8), S(-13, 6)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(84, -43), //Queen
    S(92, 5),   //Rook
    S(66, 51),  //Bishop
    S(64, 27),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(56, -25), //Queen
    S(71, 14),  //Rook
    S(33, 36),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(75, 57), //Queen
    S(56, 28), //Rook
    S(0, 0),   //Bishop
    S(23, 24), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-40, -85),
    S(-18, -47),
    S(-6, -25),
    S(0, -15),
    S(7, -6),
    S(13, 4),
    S(22, 2),
    S(29, -1),
    S(35, -14),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-25, -74),
    S(-15, -52),
    S(-3, -35),
    S(4, -20),
    S(10, -9),
    S(13, 3),
    S(17, 7),
    S(19, 10),
    S(20, 14),
    S(24, 11),
    S(32, 6),
    S(35, 6),
    S(32, 16),
    S(45, -10),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-30, -79),
    S(-21, -54),
    S(-17, -51),
    S(-14, -47),
    S(-16, -39),
    S(-10, -35),
    S(-9, -28),
    S(-7, -26),
    S(-3, -23),
    S(-2, -18),
    S(1, -16),
    S(-1, -11),
    S(2, -9),
    S(1, -11),
    S(-8, -10),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-23, -246),
    S(-23, -201),
    S(-33, -125),
    S(-30, -100),
    S(-29, -83),
    S(-24, -78),
    S(-21, -61),
    S(-20, -48),
    S(-18, -39),
    S(-16, -36),
    S(-14, -30),
    S(-12, -23),
    S(-10, -22),
    S(-11, -16),
    S(-8, -15),
    S(-6, -12),
    S(-6, -4),
    S(-4, -6),
    S(5, -9),
    S(19, -20),
    S(23, -19),
    S(75, -52),
    S(67, -48),
    S(85, -69),
    S(218, -123),
    S(226, -153),
    S(160, -99),
    S(93, -92),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(30, 27);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(32, 9),
    S(29, 2),
    S(25, 12),
    S(30, 11),
    S(31, 17),
    S(47, -0),
    S(60, -4),
    S(86, -2),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(2, 50),
    S(7, 19),
    S(7, 19),
    S(18, 10),
    S(7, 14),
    S(23, -2),
    S(28, 2),
    S(7, 34),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(66, -21), S(53, -10), S(40, -7), S(31, 7)],
    // Left adjacent
    [S(44, -10), S(22, -6), S(16, -3), S(14, 6)],
    // Right adjacent
    [S(40, -20), S(32, -9), S(23, -0), S(15, 8)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(106, 227), S(-43, 120), S(-4, 25), S(9, 4)],
    // Left adjacent
    [S(-3, 205), S(-72, 102), S(-27, 26), S(-3, 4)],
    // Right adjacent
    [S(-27, 229), S(-56, 87), S(-23, 22), S(-3, 3)],
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
