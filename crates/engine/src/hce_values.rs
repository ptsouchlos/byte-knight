// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{
    definitions::NumberOf,
    pieces::Piece,
    side::Side,
    square::{self, Square},
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
pub const PSQTS : [[PhasedScore; Square::COUNT]; Piece::COUNT] = [
    // King
    [
        S(  22,  -89), S(  30,  -29), S(  30,  -20), S( -93,   26), S( -35,   10), S(  -9,   15), S(  57,   -6), S( 182, -123),
        S( -79,   10), S( -30,   46), S( -82,   56), S(  32,   38), S( -27,   58), S( -13,   69), S(  11,   57), S( -38,   21),
        S( -93,   27), S(   3,   50), S( -51,   68), S( -83,   78), S( -49,   82), S(  27,   73), S( -10,   73), S( -29,   36),
        S( -78,   14), S( -41,   44), S( -91,   67), S(-130,   79), S(-112,   77), S( -81,   72), S( -93,   64), S(-140,   39),
        S( -78,    0), S( -56,   26), S( -81,   49), S(-115,   64), S(-115,   62), S( -76,   46), S( -77,   33), S(-136,   25),
        S( -31,  -12), S(  -4,    6), S( -55,   29), S( -82,   43), S( -72,   41), S( -60,   30), S( -27,   10), S( -48,   -1),
        S(  25,  -29), S( -16,   -4), S( -30,    7), S( -62,   17), S( -61,   22), S( -47,   15), S( -15,   -2), S(   9,  -31),
        S(  -6,  -74), S(   1,  -42), S( -17,  -27), S( -82,  -16), S( -33,  -34), S( -69,   -9), S( -18,  -33), S(   2,  -80),
    ],
    // Queen
    [
        S( 912, 1411), S( 922, 1412), S( 945, 1431), S( 961, 1420), S( 974, 1412), S( 981, 1409), S(1004, 1371), S( 968, 1391),
        S( 959, 1393), S( 943, 1420), S( 939, 1456), S( 928, 1475), S( 922, 1489), S( 968, 1444), S( 975, 1425), S(1005, 1416),
        S( 966, 1396), S( 959, 1418), S( 963, 1454), S( 960, 1458), S( 970, 1463), S(1002, 1435), S(1010, 1415), S( 985, 1416),
        S( 952, 1416), S( 961, 1429), S( 955, 1442), S( 953, 1464), S( 966, 1460), S( 966, 1452), S( 977, 1445), S( 971, 1428),
        S( 962, 1400), S( 953, 1423), S( 960, 1427), S( 966, 1448), S( 970, 1441), S( 962, 1443), S( 975, 1427), S( 976, 1413),
        S( 955, 1383), S( 965, 1401), S( 970, 1415), S( 963, 1417), S( 971, 1427), S( 974, 1421), S( 984, 1397), S( 981, 1380),
        S( 967, 1371), S( 968, 1376), S( 974, 1380), S( 981, 1386), S( 981, 1393), S( 990, 1360), S( 994, 1341), S( 990, 1323),
        S( 957, 1369), S( 965, 1364), S( 969, 1376), S( 972, 1393), S( 978, 1373), S( 965, 1358), S( 981, 1335), S( 974, 1340),
    ],
    // Rook
    [
        S( 471,  778), S( 472,  787), S( 454,  795), S( 462,  787), S( 483,  773), S( 482,  786), S( 469,  791), S( 461,  780),
        S( 448,  787), S( 446,  804), S( 465,  797), S( 488,  785), S( 470,  783), S( 492,  785), S( 466,  789), S( 462,  782),
        S( 448,  783), S( 473,  786), S( 473,  780), S( 475,  777), S( 507,  759), S( 509,  768), S( 530,  766), S( 470,  766),
        S( 434,  783), S( 449,  786), S( 458,  785), S( 467,  779), S( 472,  762), S( 472,  773), S( 466,  776), S( 450,  767),
        S( 430,  771), S( 428,  782), S( 449,  773), S( 451,  773), S( 457,  763), S( 433,  782), S( 459,  768), S( 436,  756),
        S( 424,  763), S( 429,  769), S( 441,  763), S( 437,  767), S( 451,  755), S( 447,  762), S( 468,  743), S( 448,  738),
        S( 424,  755), S( 430,  765), S( 447,  760), S( 446,  762), S( 455,  750), S( 455,  755), S( 458,  742), S( 424,  743),
        S( 438,  760), S( 444,  761), S( 452,  763), S( 456,  759), S( 464,  748), S( 456,  759), S( 445,  754), S( 443,  743),
    ],
    // Bishop
    [
        S( 307,  430), S( 299,  436), S( 274,  434), S( 237,  442), S( 245,  436), S( 233,  432), S( 314,  425), S( 281,  423),
        S( 321,  415), S( 325,  430), S( 323,  428), S( 311,  430), S( 307,  426), S( 322,  425), S( 296,  436), S( 313,  417),
        S( 333,  432), S( 348,  429), S( 338,  438), S( 341,  430), S( 348,  431), S( 371,  439), S( 356,  431), S( 335,  436),
        S( 323,  429), S( 338,  437), S( 343,  438), S( 362,  453), S( 352,  445), S( 353,  440), S( 332,  438), S( 322,  432),
        S( 332,  424), S( 324,  438), S( 339,  445), S( 360,  446), S( 360,  445), S( 342,  440), S( 340,  432), S( 340,  417),
        S( 326,  425), S( 352,  435), S( 352,  436), S( 350,  441), S( 355,  445), S( 357,  436), S( 356,  426), S( 350,  418),
        S( 345,  427), S( 349,  415), S( 358,  415), S( 344,  428), S( 356,  423), S( 365,  418), S( 373,  418), S( 355,  406),
        S( 336,  417), S( 357,  429), S( 336,  422), S( 333,  424), S( 338,  421), S( 334,  432), S( 353,  413), S( 359,  391),
    ],
    // Knight
    [
        S( 174,  355), S( 185,  407), S( 262,  425), S( 293,  409), S( 347,  405), S( 228,  403), S( 215,  398), S( 213,  331),
        S( 301,  408), S( 318,  421), S( 322,  423), S( 342,  422), S( 337,  416), S( 367,  403), S( 311,  416), S( 332,  390),
        S( 318,  412), S( 338,  422), S( 349,  446), S( 358,  449), S( 371,  444), S( 405,  426), S( 350,  424), S( 352,  400),
        S( 330,  425), S( 337,  436), S( 357,  452), S( 380,  453), S( 351,  461), S( 375,  453), S( 336,  451), S( 362,  423),
        S( 322,  431), S( 336,  428), S( 344,  450), S( 355,  450), S( 363,  455), S( 353,  447), S( 365,  430), S( 336,  427),
        S( 300,  410), S( 322,  419), S( 330,  428), S( 341,  443), S( 356,  440), S( 341,  421), S( 344,  413), S( 327,  414),
        S( 296,  408), S( 311,  417), S( 319,  414), S( 336,  419), S( 338,  417), S( 337,  413), S( 331,  406), S( 328,  422),
        S( 267,  406), S( 301,  403), S( 302,  409), S( 318,  417), S( 326,  414), S( 330,  404), S( 307,  410), S( 299,  411),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 183,  306), S( 179,  309), S( 144,  315), S( 165,  278), S( 133,  294), S( 160,  297), S( 143,  337), S( 134,  322),
        S(  87,  180), S(  79,  199), S( 105,  157), S( 102,  129), S(  99,  136), S( 115,  160), S( 101,  201), S(  58,  184),
        S(  68,  151), S(  73,  152), S(  80,  128), S(  79,  113), S(  97,  113), S(  88,  120), S(  82,  147), S(  64,  130),
        S(  63,  129), S(  65,  139), S(  76,  119), S(  89,  112), S(  90,  113), S(  85,  114), S(  84,  129), S(  63,  112),
        S(  58,  124), S(  67,  133), S(  73,  118), S(  74,  121), S(  83,  125), S(  77,  118), S(  92,  122), S(  62,  109),
        S(  58,  126), S(  65,  134), S(  67,  124), S(  59,  129), S(  69,  135), S(  80,  123), S(  97,  124), S(  52,  113),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-0, 65),
    S(12, 130),
    S(12, 67),
    S(-12, 40),
    S(-12, 14),
    S(-7, 9),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-20, -37),
    S(2, -28),
    S(-4, -19),
    S(-5, -7),
    S(-13, -4),
    S(-6, -16),
    S(-1, -24),
    S(-10, -39),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-8, -1),
    S(-8, -18),
    S(-18, -13),
    S(-17, -22),
    S(-19, -23),
    S(-6, -14),
    S(-10, -18),
    S(1, -2),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(21, 67);

pub const KING_SAFETY: [PhasedScore; Piece::COUNT - 1] =
    [S(-16, -11), S(-20, 7), S(-24, 6), S(-13, 9), S(-13, 7)];

pub const PAWN_THREAT: [PhasedScore; Piece::COUNT] = [
    S(0, 0),    //King
    S(80, -40), //Queen
    S(90, 10),  //Rook
    S(65, 53),  //Bishop
    S(64, 31),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; Piece::COUNT] = [
    S(0, 0),    //King
    S(53, -17), //Queen
    S(72, 15),  //Rook
    S(33, 34),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; Piece::COUNT] = [
    S(0, 0),   //King
    S(74, 40), //Queen
    S(58, 30), //Rook
    S(0, 0),   //Bishop
    S(24, 25), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-41, -91),
    S(-19, -54),
    S(-8, -32),
    S(-1, -22),
    S(6, -15),
    S(13, -6),
    S(21, -8),
    S(28, -12),
    S(35, -27),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-27, -75),
    S(-16, -56),
    S(-5, -41),
    S(2, -27),
    S(9, -16),
    S(12, -6),
    S(16, -1),
    S(18, 2),
    S(19, 5),
    S(25, 1),
    S(32, -3),
    S(37, -4),
    S(34, 4),
    S(58, -22),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-32, -95),
    S(-22, -72),
    S(-18, -68),
    S(-14, -62),
    S(-17, -53),
    S(-11, -50),
    S(-10, -43),
    S(-7, -42),
    S(-3, -39),
    S(-2, -34),
    S(2, -33),
    S(1, -28),
    S(4, -27),
    S(2, -28),
    S(0, -31),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-33, -258),
    S(-21, -215),
    S(-28, -157),
    S(-25, -125),
    S(-23, -109),
    S(-19, -103),
    S(-17, -87),
    S(-16, -73),
    S(-13, -65),
    S(-11, -62),
    S(-9, -56),
    S(-7, -50),
    S(-5, -49),
    S(-6, -43),
    S(-2, -44),
    S(-1, -41),
    S(-2, -33),
    S(1, -37),
    S(8, -40),
    S(24, -53),
    S(28, -53),
    S(73, -82),
    S(62, -78),
    S(97, -109),
    S(188, -145),
    S(210, -177),
    S(145, -120),
    S(81, -125),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(27, 23);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(33, 6),
    S(31, 1),
    S(26, 12),
    S(30, 10),
    S(32, 16),
    S(48, -3),
    S(65, -7),
    S(81, -4),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(1, 47),
    S(10, 18),
    S(9, 19),
    S(17, 8),
    S(7, 15),
    S(23, -1),
    S(30, 3),
    S(8, 33),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(65, -19), S(53, -9), S(40, -6), S(29, 6)],
    // Left adjacent
    [S(44, -9), S(23, -6), S(17, -3), S(12, 6)],
    // Right adjacent
    [S(40, -20), S(34, -9), S(25, -1), S(16, 7)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(132, 185), S(-40, 112), S(-5, 24), S(9, 3)],
    // Left adjacent
    [S(-3, 199), S(-71, 98), S(-27, 25), S(-3, 4)],
    // Right adjacent
    [S(-47, 223), S(-60, 85), S(-22, 22), S(-2, 2)],
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
