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
        S(   7,  -89), S(  27,  -25), S(  14,  -14), S( -94,   25), S( -33,    8), S(   1,   16), S(  61,   -6), S( 152, -122), 
        S( -56,    6), S( -25,   46), S( -76,   57), S(  31,   36), S( -29,   57), S( -23,   70), S(  10,   58), S( -42,   20), 
        S( -69,   25), S(   1,   50), S( -46,   67), S( -75,   76), S( -55,   81), S(  20,   74), S(  -6,   72), S( -25,   35), 
        S( -73,   14), S( -40,   44), S( -86,   66), S(-127,   77), S(-108,   76), S( -80,   71), S( -78,   60), S(-137,   38), 
        S( -81,    2), S( -57,   26), S( -82,   50), S(-113,   64), S(-115,   62), S( -73,   46), S( -79,   33), S(-135,   25), 
        S( -34,  -12), S( -10,    7), S( -61,   29), S( -85,   45), S( -75,   41), S( -63,   30), S( -28,    9), S( -52,    0), 
        S(  26,  -29), S( -13,   -4), S( -31,    8), S( -62,   18), S( -62,   23), S( -46,   16), S( -14,   -2), S(  11,  -30), 
        S( -10,  -74), S(   1,  -43), S( -17,  -27), S( -84,  -14), S( -34,  -34), S( -71,   -9), S( -18,  -33), S(   1,  -79), 
    ],
    // Queen
    [
        S( 917, 1405), S( 936, 1399), S( 959, 1417), S( 971, 1407), S( 982, 1405), S( 990, 1401), S(1015, 1358), S( 972, 1389), 
        S( 963, 1384), S( 947, 1409), S( 943, 1446), S( 934, 1463), S( 927, 1480), S( 975, 1435), S( 980, 1417), S(1004, 1405), 
        S( 968, 1389), S( 967, 1408), S( 969, 1441), S( 966, 1447), S( 976, 1452), S(1006, 1427), S(1014, 1404), S( 990, 1407), 
        S( 957, 1405), S( 965, 1417), S( 958, 1432), S( 956, 1455), S( 971, 1448), S( 970, 1443), S( 982, 1435), S( 974, 1420), 
        S( 967, 1388), S( 955, 1413), S( 964, 1414), S( 969, 1434), S( 974, 1428), S( 966, 1430), S( 979, 1414), S( 979, 1403), 
        S( 957, 1375), S( 967, 1389), S( 972, 1405), S( 966, 1407), S( 972, 1415), S( 973, 1410), S( 983, 1386), S( 979, 1369), 
        S( 971, 1359), S( 972, 1365), S( 979, 1368), S( 987, 1375), S( 986, 1382), S( 994, 1347), S( 996, 1332), S( 995, 1311), 
        S( 963, 1355), S( 969, 1352), S( 974, 1363), S( 977, 1380), S( 983, 1360), S( 969, 1344), S( 981, 1326), S( 981, 1323), 
    ],
    // Rook
    [
        S( 479,  773), S( 478,  783), S( 461,  789), S( 471,  782), S( 487,  770), S( 487,  781), S( 477,  785), S( 469,  776), 
        S( 450,  784), S( 447,  801), S( 468,  794), S( 490,  783), S( 477,  778), S( 495,  780), S( 466,  785), S( 464,  778), 
        S( 448,  780), S( 472,  782), S( 474,  777), S( 479,  772), S( 511,  755), S( 510,  763), S( 527,  763), S( 471,  763), 
        S( 436,  778), S( 449,  781), S( 457,  780), S( 468,  774), S( 474,  758), S( 473,  769), S( 466,  771), S( 451,  763), 
        S( 430,  767), S( 427,  778), S( 447,  771), S( 450,  769), S( 457,  759), S( 433,  777), S( 456,  763), S( 435,  753), 
        S( 423,  759), S( 427,  765), S( 440,  762), S( 436,  764), S( 447,  752), S( 443,  759), S( 461,  739), S( 443,  735), 
        S( 425,  752), S( 431,  761), S( 449,  757), S( 447,  760), S( 457,  748), S( 455,  752), S( 456,  739), S( 422,  741), 
        S( 438,  757), S( 445,  758), S( 452,  761), S( 457,  756), S( 465,  746), S( 457,  756), S( 444,  752), S( 443,  740), 
    ],
    // Bishop
    [
        S( 308,  429), S( 298,  436), S( 268,  435), S( 232,  445), S( 254,  436), S( 229,  435), S( 317,  424), S( 285,  423), 
        S( 324,  415), S( 326,  428), S( 323,  428), S( 313,  430), S( 310,  427), S( 324,  425), S( 301,  433), S( 311,  417), 
        S( 336,  430), S( 348,  427), S( 339,  436), S( 341,  428), S( 350,  428), S( 374,  436), S( 358,  429), S( 338,  435), 
        S( 324,  427), S( 338,  433), S( 342,  434), S( 360,  450), S( 352,  440), S( 354,  435), S( 333,  435), S( 325,  430), 
        S( 332,  421), S( 324,  434), S( 339,  442), S( 360,  442), S( 359,  441), S( 341,  436), S( 340,  428), S( 341,  416), 
        S( 327,  424), S( 349,  431), S( 351,  433), S( 349,  438), S( 352,  440), S( 354,  432), S( 352,  421), S( 350,  416), 
        S( 344,  425), S( 351,  415), S( 359,  416), S( 346,  428), S( 357,  423), S( 364,  417), S( 374,  416), S( 355,  403), 
        S( 338,  415), S( 357,  427), S( 338,  421), S( 335,  422), S( 341,  419), S( 335,  432), S( 354,  412), S( 359,  391), 
    ],
    // Knight
    [
        S( 172,  354), S( 188,  409), S( 261,  428), S( 291,  412), S( 354,  403), S( 222,  415), S( 227,  400), S( 205,  334), 
        S( 303,  408), S( 319,  420), S( 327,  422), S( 347,  423), S( 340,  416), S( 372,  402), S( 312,  417), S( 338,  391), 
        S( 318,  412), S( 328,  422), S( 344,  443), S( 348,  447), S( 364,  441), S( 395,  424), S( 346,  422), S( 351,  400), 
        S( 325,  422), S( 333,  430), S( 349,  445), S( 372,  447), S( 346,  456), S( 373,  446), S( 336,  445), S( 361,  420), 
        S( 318,  423), S( 333,  424), S( 343,  444), S( 352,  443), S( 363,  450), S( 353,  440), S( 367,  424), S( 333,  421), 
        S( 300,  408), S( 322,  417), S( 330,  426), S( 343,  443), S( 354,  438), S( 339,  419), S( 343,  410), S( 327,  411), 
        S( 300,  408), S( 316,  417), S( 323,  415), S( 340,  420), S( 342,  419), S( 340,  414), S( 336,  407), S( 331,  422), 
        S( 270,  406), S( 305,  406), S( 307,  410), S( 324,  416), S( 330,  414), S( 333,  403), S( 312,  411), S( 301,  411), 
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), 
        S( 183,  303), S( 171,  311), S( 141,  313), S( 170,  275), S( 139,  291), S( 157,  297), S( 144,  335), S( 136,  316), 
        S(  85,  172), S(  79,  193), S( 103,  151), S(  99,  123), S(  99,  130), S( 113,  154), S( 101,  194), S(  60,  176), 
        S(  67,  142), S(  71,  145), S(  77,  120), S(  77,  105), S(  94,  105), S(  85,  113), S(  81,  140), S(  63,  123), 
        S(  61,  121), S(  62,  132), S(  73,  111), S(  84,  102), S(  86,  105), S(  81,  105), S(  81,  122), S(  60,  105), 
        S(  54,  115), S(  59,  124), S(  66,  110), S(  65,  113), S(  73,  116), S(  68,  110), S(  84,  113), S(  57,  101), 
        S(  57,  120), S(  61,  130), S(  63,  119), S(  55,  121), S(  62,  129), S(  76,  116), S(  92,  119), S(  51,  107), 
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), 
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(2, 54),
    S(13, 125),
    S(11, 66),
    S(-12, 40),
    S(-11, 15),
    S(-6, 10),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; File::COUNT] = [
    S(-19, -36),
    S(4, -27),
    S(-3, -18),
    S(-3, -5),
    S(-10, -3),
    S(-4, -14),
    S(1, -24),
    S(-10, -37),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; File::COUNT] = [
    S(-7, 2),
    S(-6, -15),
    S(-16, -10),
    S(-14, -17),
    S(-17, -18),
    S(-5, -10),
    S(-7, -15),
    S(2, 2),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(20, 68);

pub const KING_SAFETY: [PhasedScore; Piece::COUNT - 1] =
    [S(-16, -11), S(-20, 7), S(-24, 7), S(-15, 10), S(-13, 6)];

pub const PAWN_THREAT: [PhasedScore; Piece::COUNT] = [
    S(0, 0),    //King
    S(77, -33), //Queen
    S(87, 14),  //Rook
    S(66, 53),  //Bishop
    S(63, 29),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; Piece::COUNT] = [
    S(0, 0),    //King
    S(55, -15), //Queen
    S(73, 19),  //Rook
    S(33, 35),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; Piece::COUNT] = [
    S(0, 0),   //King
    S(72, 37), //Queen
    S(57, 31), //Rook
    S(0, 0),   //Bishop
    S(25, 27), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(-43, -99),
    S(-22, -64),
    S(-11, -40),
    S(-4, -31),
    S(3, -24),
    S(9, -16),
    S(18, -18),
    S(24, -21),
    S(33, -37),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(-26, -80),
    S(-14, -62),
    S(-4, -47),
    S(2, -34),
    S(8, -24),
    S(12, -14),
    S(15, -9),
    S(19, -6),
    S(21, -3),
    S(28, -6),
    S(36, -10),
    S(41, -11),
    S(38, -3),
    S(65, -27),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(-31, -104),
    S(-20, -85),
    S(-16, -80),
    S(-12, -74),
    S(-15, -65),
    S(-9, -62),
    S(-8, -56),
    S(-5, -55),
    S(-1, -52),
    S(1, -47),
    S(5, -46),
    S(6, -42),
    S(8, -40),
    S(7, -41),
    S(10, -47),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(-21, -273),
    S(-9, -228),
    S(-16, -182),
    S(-13, -151),
    S(-12, -134),
    S(-8, -127),
    S(-5, -111),
    S(-4, -98),
    S(-2, -91),
    S(1, -88),
    S(2, -80),
    S(4, -76),
    S(7, -75),
    S(6, -69),
    S(9, -70),
    S(12, -68),
    S(10, -60),
    S(13, -63),
    S(22, -68),
    S(36, -79),
    S(39, -79),
    S(81, -106),
    S(82, -108),
    S(115, -138),
    S(177, -160),
    S(214, -195),
    S(140, -124),
    S(82, -145),
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
    S(65, -9),
    S(80, -6),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; File::COUNT] = [
    S(1, 46),
    S(10, 18),
    S(9, 18),
    S(16, 7),
    S(6, 14),
    S(24, -2),
    S(32, 2),
    S(10, 31),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(66, -16), S(54, -10), S(41, -5), S(31, 6)],
    // Left adjacent
    [S(44, -8), S(24, -5), S(17, -2), S(12, 6)],
    // Right adjacent
    [S(40, -18), S(33, -8), S(24, -0), S(16, 8)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(139, 170), S(-37, 111), S(-5, 23), S(8, 3)],
    // Left adjacent
    [S(-2, 191), S(-70, 96), S(-27, 25), S(-3, 4)],
    // Right adjacent
    [S(-41, 213), S(-61, 81), S(-22, 21), S(-2, 2)],
];

pub const PAWN_DEFENSE: PhasedScore = S(7, 10);

pub const KNIGHT_OUTPOST: PhasedScore = S(27, 11);

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

    fn knight_outpost_bonus(&self, count: i16, _side: Side) -> Self::ReturnScore {
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
