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
        S(  19, -104), S(  -1,  -42), S(  16,  -23), S(-128,   32), S( -82,   13), S( -23,   11), S(  41,   -3), S( 189, -131),
        S(-129,   12), S( -47,   42), S( -91,   55), S(  33,   36), S( -20,   56), S( -21,   70), S(  20,   57), S( -49,   24),
        S(-151,   26), S(   6,   44), S( -71,   64), S( -84,   75), S( -37,   75), S(  53,   62), S(   6,   65), S( -57,   32),
        S(-103,   10), S( -80,   43), S( -95,   62), S(-144,   76), S(-129,   74), S( -88,   63), S( -97,   54), S(-163,   34),
        S(-109,   -0), S( -79,   27), S( -97,   48), S(-128,   66), S(-121,   61), S( -80,   43), S( -99,   33), S(-176,   27),
        S( -53,  -13), S(  -9,   10), S( -63,   32), S( -73,   45), S( -62,   44), S( -62,   32), S( -22,   13), S( -74,    6),
        S(  41,  -32), S(  11,   -1), S(  -9,   15), S( -48,   27), S( -46,   31), S( -29,   21), S(  24,   -1), S(  20,  -22),
        S(  25,  -75), S(  55,  -48), S(  15,  -20), S( -94,   -1), S( -26,  -25), S( -63,   -2), S(  26,  -34), S(  24,  -72),
    ],
    // Queen
    [
        S( 914, 1465), S( 925, 1472), S( 954, 1490), S( 986, 1471), S( 968, 1474), S( 978, 1471), S(1007, 1424), S( 946, 1460),
        S( 968, 1433), S( 943, 1473), S( 943, 1513), S( 931, 1534), S( 918, 1553), S( 967, 1493), S( 950, 1490), S(1009, 1457),
        S( 978, 1445), S( 974, 1467), S( 973, 1505), S( 975, 1505), S( 977, 1512), S(1003, 1488), S(1011, 1454), S( 995, 1442),
        S( 956, 1473), S( 966, 1491), S( 961, 1499), S( 955, 1524), S( 964, 1520), S( 970, 1503), S( 974, 1499), S( 970, 1469),
        S( 963, 1462), S( 954, 1489), S( 954, 1498), S( 965, 1513), S( 964, 1513), S( 962, 1497), S( 975, 1473), S( 973, 1463),
        S( 957, 1440), S( 967, 1461), S( 964, 1486), S( 958, 1490), S( 966, 1498), S( 971, 1477), S( 984, 1446), S( 975, 1432),
        S( 956, 1434), S( 963, 1442), S( 975, 1441), S( 976, 1460), S( 975, 1463), S( 986, 1418), S( 992, 1379), S(1003, 1345),
        S( 951, 1436), S( 942, 1439), S( 952, 1450), S( 970, 1445), S( 960, 1446), S( 945, 1428), S( 965, 1401), S( 960, 1402),
    ],
    // Rook
    [
        S( 476,  812), S( 459,  822), S( 461,  832), S( 457,  831), S( 469,  822), S( 494,  808), S( 472,  814), S( 509,  803),
        S( 467,  808), S( 459,  823), S( 476,  828), S( 493,  821), S( 470,  822), S( 494,  806), S( 486,  801), S( 519,  787),
        S( 462,  803), S( 488,  804), S( 489,  806), S( 489,  803), S( 523,  787), S( 505,  784), S( 543,  777), S( 507,  776),
        S( 446,  807), S( 463,  804), S( 468,  813), S( 478,  808), S( 483,  790), S( 474,  785), S( 474,  785), S( 474,  777),
        S( 424,  801), S( 427,  806), S( 442,  806), S( 456,  803), S( 459,  797), S( 425,  798), S( 448,  783), S( 436,  781),
        S( 415,  796), S( 428,  794), S( 440,  793), S( 438,  797), S( 444,  791), S( 433,  782), S( 464,  761), S( 443,  764),
        S( 411,  790), S( 427,  794), S( 445,  792), S( 442,  793), S( 446,  784), S( 443,  778), S( 461,  766), S( 426,  776),
        S( 433,  786), S( 435,  794), S( 448,  802), S( 453,  799), S( 458,  791), S( 445,  784), S( 457,  782), S( 433,  775),
    ],
    // Bishop
    [
        S( 314,  439), S( 298,  449), S( 299,  443), S( 248,  457), S( 249,  453), S( 263,  445), S( 308,  443), S( 270,  434),
        S( 324,  427), S( 341,  448), S( 342,  450), S( 329,  450), S( 325,  445), S( 337,  445), S( 303,  456), S( 321,  425),
        S( 348,  453), S( 370,  449), S( 362,  462), S( 366,  454), S( 354,  458), S( 384,  461), S( 366,  452), S( 343,  455),
        S( 331,  451), S( 345,  470), S( 355,  466), S( 365,  481), S( 361,  469), S( 361,  468), S( 340,  466), S( 321,  449),
        S( 327,  447), S( 330,  467), S( 346,  471), S( 361,  472), S( 360,  470), S( 357,  465), S( 345,  460), S( 330,  435),
        S( 332,  443), S( 349,  454), S( 351,  462), S( 353,  465), S( 358,  470), S( 354,  462), S( 356,  445), S( 355,  432),
        S( 342,  436), S( 348,  432), S( 360,  434), S( 341,  449), S( 351,  452), S( 362,  440), S( 370,  440), S( 353,  413),
        S( 320,  411), S( 343,  437), S( 328,  415), S( 321,  438), S( 326,  433), S( 325,  436), S( 347,  419), S( 340,  392),
    ],
    // Knight
    [
        S( 155,  337), S( 202,  403), S( 274,  428), S( 309,  417), S( 346,  422), S( 281,  395), S( 228,  405), S( 213,  313),
        S( 303,  406), S( 327,  430), S( 344,  443), S( 361,  444), S( 355,  434), S( 401,  420), S( 329,  424), S( 335,  390),
        S( 333,  419), S( 366,  441), S( 396,  459), S( 404,  463), S( 418,  453), S( 451,  435), S( 370,  439), S( 361,  409),
        S( 337,  434), S( 363,  453), S( 393,  467), S( 414,  471), S( 388,  474), S( 418,  468), S( 360,  461), S( 368,  426),
        S( 321,  435), S( 349,  445), S( 369,  469), S( 370,  471), S( 379,  475), S( 372,  461), S( 366,  445), S( 330,  426),
        S( 299,  417), S( 328,  440), S( 348,  451), S( 351,  466), S( 361,  466), S( 351,  445), S( 347,  435), S( 318,  418),
        S( 281,  408), S( 292,  426), S( 315,  438), S( 329,  441), S( 329,  439), S( 331,  434), S( 313,  414), S( 310,  422),
        S( 230,  398), S( 289,  384), S( 276,  419), S( 292,  419), S( 297,  423), S( 315,  408), S( 293,  392), S( 263,  391),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 188,  319), S( 191,  329), S( 184,  321), S( 224,  262), S( 184,  269), S( 176,  272), S(  64,  342), S(  67,  329),
        S(  85,  175), S(  94,  196), S( 126,  142), S( 127,  104), S( 130,  105), S( 175,  121), S( 135,  179), S( 104,  158),
        S(  60,  153), S(  79,  161), S(  88,  135), S(  84,  119), S( 105,  122), S( 111,  121), S( 100,  155), S(  89,  126),
        S(  49,  132), S(  69,  149), S(  75,  126), S(  91,  123), S(  88,  125), S(  87,  122), S(  80,  144), S(  75,  112),
        S(  46,  126), S(  66,  143), S(  72,  125), S(  71,  129), S(  85,  133), S(  80,  125), S( 101,  135), S(  81,  108),
        S(  45,  130), S(  67,  145), S(  67,  132), S(  54,  128), S(  77,  141), S( 100,  125), S( 114,  131), S(  73,  108),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-22, 45),
    S(9, 143),
    S(13, 71),
    S(-13, 43),
    S(-15, 15),
    S(-7, 9),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-19, -34),
    S(4, -30),
    S(-7, -20),
    S(-2, -9),
    S(-14, -4),
    S(-13, -20),
    S(-7, -34),
    S(-16, -46),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-2, 0),
    S(-12, -19),
    S(-23, -14),
    S(-22, -25),
    S(-23, -26),
    S(-17, -10),
    S(-13, -19),
    S(-13, 6),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(23, 76);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-21, -16), S(-42, 9), S(-34, 5), S(-19, 9), S(-16, 16)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(54, -62), //Queen
    S(76, -18), //Rook
    S(45, 42),  //Bishop
    S(46, 2),   //Knight
    S(30, 0),   //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(27, -49), //Queen
    S(56, -16), //Rook
    S(26, 28),  //Bishop
    S(10, 32),  //Knight
    S(-16, 7),  //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(48, 44), //Queen
    S(42, 4),  //Rook
    S(20, 20), //Bishop
    S(23, 19), //Knight
    S(-2, 8),  //Pawn
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

    fn bishop_pair_bonus_value(&self) -> Self::ReturnScore {
        BISHOP_PAIR_BONUS
    }

    fn king_safety_value(&self, piece: Piece) -> Self::ReturnScore {
        assert!(piece != Piece::King);
        KING_SAFETY[piece as usize - 1]
    }

    fn threat_value(&self, piece: Piece, attacked_piece: Piece) -> Self::ReturnScore {
        match piece {
            Piece::Pawn => PAWN_THREAT[attacked_piece as usize],
            Piece::Knight => KNIGHT_THREAT[attacked_piece as usize],
            Piece::Bishop => BISHOP_THREAT[attacked_piece as usize],
            _ => S(0, 0),
        }
    }
}
