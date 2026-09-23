// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{
    definitions::NumberOf,
    file::File,
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
        S(  11,  -90), S(  28,  -26), S(  18,  -15), S( -94,   25), S( -34,    8), S(  -1,   17), S(  60,   -6), S( 160, -125),
        S( -60,    7), S( -25,   47), S( -77,   57), S(  31,   36), S( -29,   58), S( -22,   70), S(  11,   59), S( -42,   20),
        S( -72,   25), S(   2,   51), S( -47,   68), S( -76,   76), S( -54,   82), S(  20,   74), S(  -6,   72), S( -25,   35),
        S( -73,   14), S( -40,   44), S( -86,   66), S(-128,   78), S(-109,   77), S( -80,   72), S( -78,   61), S(-138,   39),
        S( -81,    2), S( -57,   26), S( -83,   50), S(-114,   64), S(-115,   63), S( -74,   46), S( -79,   33), S(-136,   25),
        S( -34,  -12), S( -10,    6), S( -61,   30), S( -86,   45), S( -76,   42), S( -64,   30), S( -29,    9), S( -52,   -0),
        S(  27,  -30), S( -13,   -5), S( -30,    8), S( -62,   18), S( -62,   23), S( -46,   16), S( -14,   -3), S(  11,  -31),
        S(  -9,  -75), S(   1,  -44), S( -17,  -27), S( -84,  -15), S( -34,  -34), S( -71,   -9), S( -18,  -33), S(   1,  -80),
    ],
    // Queen
    [
        S( 919, 1409), S( 937, 1404), S( 960, 1422), S( 972, 1413), S( 983, 1410), S( 991, 1406), S(1017, 1363), S( 974, 1392),
        S( 965, 1388), S( 949, 1414), S( 945, 1451), S( 935, 1468), S( 928, 1485), S( 977, 1440), S( 982, 1422), S(1006, 1410),
        S( 969, 1394), S( 968, 1412), S( 970, 1446), S( 967, 1452), S( 978, 1458), S(1007, 1432), S(1016, 1409), S( 992, 1412),
        S( 958, 1410), S( 966, 1422), S( 959, 1437), S( 957, 1460), S( 972, 1453), S( 971, 1448), S( 984, 1440), S( 976, 1425),
        S( 968, 1393), S( 956, 1418), S( 965, 1419), S( 970, 1440), S( 975, 1433), S( 967, 1435), S( 980, 1419), S( 981, 1408),
        S( 958, 1380), S( 968, 1394), S( 973, 1411), S( 967, 1412), S( 972, 1420), S( 974, 1416), S( 984, 1391), S( 980, 1374),
        S( 973, 1363), S( 974, 1370), S( 981, 1373), S( 988, 1380), S( 987, 1387), S( 997, 1351), S( 998, 1336), S( 996, 1315),
        S( 965, 1360), S( 971, 1356), S( 975, 1368), S( 979, 1385), S( 984, 1365), S( 970, 1349), S( 983, 1330), S( 982, 1328),
    ],
    // Rook
    [
        S( 480,  775), S( 479,  785), S( 462,  791), S( 472,  784), S( 488,  771), S( 488,  783), S( 477,  787), S( 470,  778),
        S( 451,  785), S( 448,  803), S( 469,  796), S( 491,  785), S( 478,  780), S( 496,  782), S( 467,  787), S( 465,  780),
        S( 449,  782), S( 473,  784), S( 475,  778), S( 480,  774), S( 513,  756), S( 511,  765), S( 528,  764), S( 472,  765),
        S( 437,  780), S( 449,  783), S( 458,  782), S( 469,  776), S( 474,  759), S( 474,  770), S( 467,  773), S( 452,  765),
        S( 430,  769), S( 427,  780), S( 447,  773), S( 450,  770), S( 458,  761), S( 433,  778), S( 456,  765), S( 435,  754),
        S( 422,  761), S( 427,  767), S( 440,  763), S( 437,  766), S( 448,  754), S( 443,  760), S( 461,  740), S( 443,  737),
        S( 425,  754), S( 431,  763), S( 450,  759), S( 448,  761), S( 458,  750), S( 457,  753), S( 457,  741), S( 423,  743),
        S( 439,  758), S( 445,  759), S( 453,  763), S( 458,  758), S( 466,  748), S( 458,  758), S( 445,  753), S( 444,  741),
    ],
    // Bishop
    [
        S( 309,  430), S( 299,  437), S( 269,  437), S( 233,  446), S( 254,  437), S( 229,  436), S( 316,  425), S( 285,  424),
        S( 325,  416), S( 327,  429), S( 323,  429), S( 313,  431), S( 310,  427), S( 325,  426), S( 302,  433), S( 312,  418),
        S( 336,  431), S( 349,  427), S( 340,  437), S( 342,  428), S( 351,  428), S( 375,  436), S( 359,  430), S( 339,  437),
        S( 324,  428), S( 339,  434), S( 343,  435), S( 361,  450), S( 352,  441), S( 355,  436), S( 334,  436), S( 326,  431),
        S( 332,  422), S( 324,  434), S( 340,  443), S( 360,  442), S( 360,  442), S( 342,  437), S( 340,  429), S( 341,  417),
        S( 327,  425), S( 349,  432), S( 351,  434), S( 350,  439), S( 352,  441), S( 355,  432), S( 352,  421), S( 350,  417),
        S( 345,  426), S( 352,  416), S( 360,  417), S( 347,  429), S( 358,  424), S( 365,  418), S( 375,  417), S( 356,  404),
        S( 338,  416), S( 357,  428), S( 338,  422), S( 335,  423), S( 341,  420), S( 335,  433), S( 354,  413), S( 360,  392),
    ],
    // Knight
    [
        S( 170,  354), S( 187,  409), S( 260,  427), S( 290,  412), S( 353,  403), S( 221,  414), S( 225,  400), S( 204,  333),
        S( 302,  408), S( 319,  420), S( 329,  421), S( 350,  422), S( 343,  415), S( 374,  401), S( 313,  417), S( 338,  390),
        S( 319,  412), S( 338,  421), S( 352,  444), S( 359,  447), S( 374,  441), S( 407,  425), S( 353,  423), S( 354,  400),
        S( 330,  422), S( 337,  434), S( 356,  448), S( 380,  451), S( 350,  460), S( 376,  450), S( 337,  448), S( 362,  422),
        S( 321,  426), S( 334,  426), S( 344,  447), S( 354,  447), S( 363,  452), S( 352,  443), S( 366,  426), S( 334,  425),
        S( 297,  408), S( 319,  417), S( 328,  425), S( 341,  443), S( 352,  438), S( 337,  418), S( 340,  409), S( 325,  411),
        S( 297,  407), S( 313,  417), S( 321,  415), S( 338,  419), S( 340,  419), S( 338,  413), S( 334,  407), S( 329,  421),
        S( 268,  406), S( 303,  405), S( 304,  410), S( 322,  416), S( 328,  414), S( 331,  403), S( 310,  411), S( 299,  411),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 183,  304), S( 172,  312), S( 141,  314), S( 170,  275), S( 139,  292), S( 157,  297), S( 144,  336), S( 135,  317),
        S(  86,  174), S(  80,  195), S( 104,  153), S( 101,  124), S(  99,  132), S( 114,  155), S( 102,  196), S(  60,  178),
        S(  68,  144), S(  72,  146), S(  77,  122), S(  77,  106), S(  95,  106), S(  85,  114), S(  81,  142), S(  63,  124),
        S(  61,  122), S(  63,  133), S(  73,  112), S(  85,  103), S(  86,  106), S(  82,  106), S(  82,  123), S(  60,  106),
        S(  54,  116), S(  60,  124), S(  66,  111), S(  66,  114), S(  73,  117), S(  69,  110), S(  84,  114), S(  56,  102),
        S(  57,  121), S(  62,  131), S(  64,  120), S(  56,  122), S(  62,  130), S(  78,  117), S(  93,  120), S(  51,  108),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(2, 57),
    S(13, 126),
    S(10, 66),
    S(-12, 41),
    S(-11, 15),
    S(-6, 10),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; File::COUNT] = [
    S(-19, -37),
    S(4, -27),
    S(-3, -18),
    S(-3, -5),
    S(-10, -3),
    S(-4, -14),
    S(0, -24),
    S(-10, -38),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; File::COUNT] = [
    S(-6, 3),
    S(-6, -15),
    S(-16, -10),
    S(-15, -17),
    S(-17, -18),
    S(-5, -10),
    S(-7, -15),
    S(2, 2),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(20, 68);

pub const KING_SAFETY: [PhasedScore; Piece::COUNT - 1] =
    [S(-16, -11), S(-20, 8), S(-24, 7), S(-14, 9), S(-13, 6)];

pub const PAWN_THREAT: [PhasedScore; Piece::COUNT] = [
    S(0, 0),    //King
    S(77, -32), //Queen
    S(88, 15),  //Rook
    S(67, 53),  //Bishop
    S(66, 33),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; Piece::COUNT] = [
    S(0, 0),    //King
    S(53, -15), //Queen
    S(72, 18),  //Rook
    S(33, 36),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; Piece::COUNT] = [
    S(0, 0),   //King
    S(73, 38), //Queen
    S(58, 31), //Rook
    S(0, 0),   //Bishop
    S(25, 26), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-39, -96),
    S(-17, -60),
    S(-6, -37),
    S(1, -27),
    S(8, -19),
    S(14, -11),
    S(23, -13),
    S(30, -17),
    S(39, -33),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-25, -77),
    S(-13, -60),
    S(-2, -45),
    S(4, -31),
    S(10, -21),
    S(14, -11),
    S(17, -6),
    S(21, -3),
    S(23, 0),
    S(30, -2),
    S(39, -6),
    S(44, -6),
    S(39, 1),
    S(66, -23),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-29, -99),
    S(-18, -79),
    S(-13, -75),
    S(-10, -68),
    S(-13, -60),
    S(-7, -57),
    S(-6, -50),
    S(-3, -49),
    S(1, -46),
    S(4, -41),
    S(8, -40),
    S(8, -36),
    S(10, -34),
    S(9, -35),
    S(12, -41),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-18, -268),
    S(-6, -221),
    S(-13, -174),
    S(-10, -141),
    S(-9, -124),
    S(-5, -117),
    S(-2, -101),
    S(-1, -87),
    S(1, -80),
    S(3, -77),
    S(4, -69),
    S(7, -65),
    S(9, -64),
    S(9, -58),
    S(12, -59),
    S(15, -57),
    S(13, -49),
    S(17, -52),
    S(25, -56),
    S(40, -67),
    S(42, -68),
    S(84, -94),
    S(85, -96),
    S(116, -125),
    S(182, -150),
    S(216, -183),
    S(144, -118),
    S(83, -137),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(26, 22);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; File::COUNT] = [
    S(32, 5),
    S(32, 0),
    S(26, 10),
    S(30, 8),
    S(32, 14),
    S(48, -4),
    S(66, -9),
    S(80, -6),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; File::COUNT] = [
    S(1, 47),
    S(10, 18),
    S(9, 18),
    S(16, 7),
    S(6, 14),
    S(24, -2),
    S(32, 2),
    S(9, 31),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(67, -16), S(55, -10), S(41, -5), S(31, 6)],
    // Left adjacent
    [S(44, -8), S(25, -5), S(17, -2), S(12, 6)],
    // Right adjacent
    [S(40, -18), S(33, -8), S(24, -0), S(16, 8)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(141, 172), S(-37, 112), S(-5, 24), S(9, 3)],
    // Left adjacent
    [S(-2, 193), S(-70, 97), S(-28, 25), S(-3, 4)],
    // Right adjacent
    [S(-41, 215), S(-61, 82), S(-22, 21), S(-3, 2)],
];

pub const PAWN_DEFENSE: PhasedScore = S(8, 10);

pub const KNIGHT_OUTPOST: PhasedScore = S(12, 8);

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

    fn pawn_defense_bonus(&self, count: i16, _side: Side) -> Self::ReturnScore {
        PAWN_DEFENSE * count
    }

    fn knight_outputs(&self, count: i16, _side: Side) -> Self::ReturnScore {
        KNIGHT_OUTPOST * count
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
