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
        S(  19, -104), S(  -1,  -42), S(  16,  -23), S(-129,   31), S( -82,   13), S( -24,   11), S(  41,   -3), S( 193, -131),
        S(-129,   12), S( -47,   42), S( -91,   55), S(  33,   36), S( -20,   56), S( -21,   70), S(  20,   57), S( -49,   24),
        S(-152,   26), S(   6,   44), S( -71,   64), S( -84,   75), S( -37,   75), S(  53,   62), S(   5,   65), S( -57,   31),
        S(-103,   10), S( -80,   43), S( -95,   62), S(-144,   76), S(-129,   74), S( -88,   63), S( -97,   54), S(-163,   34),
        S(-109,   -0), S( -79,   27), S( -97,   48), S(-128,   66), S(-121,   61), S( -80,   43), S( -99,   33), S(-176,   27),
        S( -53,  -13), S(  -9,   10), S( -63,   32), S( -73,   45), S( -62,   44), S( -62,   32), S( -22,   13), S( -74,    6),
        S(  41,  -32), S(  11,   -1), S( -10,   15), S( -48,   27), S( -47,   31), S( -29,   21), S(  24,   -1), S(  20,  -22),
        S(  25,  -75), S(  55,  -48), S(  15,  -20), S( -94,   -1), S( -26,  -25), S( -63,   -2), S(  26,  -34), S(  24,  -72),
    ],
    // Queen
    [
        S( 917, 1464), S( 928, 1471), S( 956, 1489), S( 989, 1470), S( 970, 1473), S( 981, 1469), S(1010, 1422), S( 949, 1458),
        S( 971, 1431), S( 946, 1472), S( 946, 1511), S( 933, 1532), S( 921, 1552), S( 970, 1492), S( 953, 1488), S(1012, 1455),
        S( 981, 1444), S( 977, 1466), S( 976, 1504), S( 978, 1504), S( 980, 1510), S(1006, 1486), S(1014, 1452), S( 998, 1440),
        S( 958, 1472), S( 968, 1489), S( 964, 1497), S( 958, 1522), S( 967, 1519), S( 972, 1502), S( 977, 1497), S( 973, 1467),
        S( 966, 1460), S( 957, 1488), S( 957, 1497), S( 968, 1512), S( 967, 1511), S( 965, 1496), S( 978, 1471), S( 976, 1461),
        S( 959, 1439), S( 970, 1459), S( 966, 1485), S( 961, 1488), S( 969, 1496), S( 973, 1476), S( 986, 1445), S( 977, 1430),
        S( 958, 1433), S( 966, 1440), S( 978, 1439), S( 979, 1459), S( 978, 1462), S( 989, 1417), S( 995, 1377), S(1006, 1343),
        S( 954, 1434), S( 945, 1438), S( 955, 1449), S( 972, 1443), S( 963, 1445), S( 948, 1427), S( 967, 1399), S( 963, 1400),
    ],
    // Rook
    [
        S( 477,  812), S( 460,  822), S( 461,  832), S( 457,  831), S( 469,  822), S( 495,  808), S( 473,  814), S( 509,  803),
        S( 467,  808), S( 459,  823), S( 476,  828), S( 493,  821), S( 471,  822), S( 494,  806), S( 486,  801), S( 520,  787),
        S( 462,  803), S( 488,  804), S( 489,  806), S( 489,  803), S( 524,  787), S( 506,  784), S( 544,  777), S( 508,  775),
        S( 447,  807), S( 464,  804), S( 468,  813), S( 478,  808), S( 483,  790), S( 475,  785), S( 474,  785), S( 475,  777),
        S( 425,  801), S( 428,  806), S( 443,  805), S( 456,  803), S( 459,  797), S( 425,  798), S( 449,  783), S( 437,  781),
        S( 416,  796), S( 429,  794), S( 440,  793), S( 438,  797), S( 445,  791), S( 433,  782), S( 465,  761), S( 444,  764),
        S( 411,  790), S( 427,  794), S( 446,  792), S( 443,  793), S( 447,  784), S( 444,  778), S( 462,  766), S( 427,  775),
        S( 433,  785), S( 436,  794), S( 449,  802), S( 454,  799), S( 459,  791), S( 445,  784), S( 458,  782), S( 433,  775),
    ],
    // Bishop
    [
        S( 315,  439), S( 298,  449), S( 299,  443), S( 248,  457), S( 249,  453), S( 264,  444), S( 309,  443), S( 271,  434),
        S( 324,  427), S( 342,  448), S( 342,  450), S( 329,  450), S( 325,  445), S( 337,  445), S( 303,  456), S( 321,  425),
        S( 348,  453), S( 370,  449), S( 362,  462), S( 366,  454), S( 354,  458), S( 385,  461), S( 366,  452), S( 343,  455),
        S( 331,  451), S( 345,  470), S( 355,  465), S( 365,  481), S( 362,  469), S( 361,  468), S( 340,  466), S( 322,  449),
        S( 328,  447), S( 331,  467), S( 346,  471), S( 362,  472), S( 360,  470), S( 357,  465), S( 345,  460), S( 330,  435),
        S( 333,  443), S( 349,  454), S( 351,  462), S( 354,  465), S( 358,  470), S( 354,  462), S( 356,  445), S( 356,  432),
        S( 342,  436), S( 348,  432), S( 361,  434), S( 341,  449), S( 351,  452), S( 363,  439), S( 370,  440), S( 353,  413),
        S( 320,  411), S( 343,  437), S( 329,  415), S( 321,  438), S( 326,  433), S( 325,  436), S( 348,  419), S( 340,  392),
    ],
    // Knight
    [
        S( 155,  337), S( 202,  403), S( 274,  428), S( 309,  417), S( 347,  422), S( 281,  395), S( 228,  405), S( 213,  313),
        S( 303,  406), S( 327,  430), S( 344,  443), S( 362,  444), S( 355,  434), S( 401,  420), S( 329,  424), S( 335,  390),
        S( 333,  419), S( 367,  441), S( 396,  459), S( 404,  463), S( 419,  453), S( 452,  435), S( 370,  439), S( 361,  409),
        S( 337,  434), S( 364,  453), S( 393,  467), S( 415,  471), S( 389,  474), S( 418,  468), S( 360,  461), S( 368,  426),
        S( 321,  435), S( 349,  445), S( 369,  469), S( 370,  471), S( 379,  475), S( 373,  461), S( 367,  445), S( 331,  426),
        S( 300,  417), S( 328,  440), S( 349,  451), S( 351,  466), S( 361,  466), S( 351,  445), S( 348,  434), S( 319,  418),
        S( 282,  408), S( 293,  426), S( 315,  438), S( 330,  441), S( 329,  439), S( 332,  434), S( 314,  414), S( 310,  422),
        S( 230,  398), S( 289,  384), S( 276,  419), S( 293,  419), S( 298,  422), S( 315,  408), S( 293,  392), S( 263,  391),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 188,  319), S( 191,  329), S( 184,  321), S( 224,  262), S( 184,  269), S( 176,  272), S(  64,  342), S(  67,  329),
        S(  85,  175), S(  94,  196), S( 126,  142), S( 127,  104), S( 130,  105), S( 175,  121), S( 135,  179), S( 104,  158),
        S(  60,  153), S(  79,  161), S(  88,  135), S(  84,  119), S( 105,  122), S( 112,  121), S( 100,  155), S(  89,  126),
        S(  49,  132), S(  69,  149), S(  75,  126), S(  91,  123), S(  88,  125), S(  87,  122), S(  80,  144), S(  75,  112),
        S(  46,  126), S(  67,  143), S(  72,  125), S(  71,  129), S(  85,  133), S(  80,  125), S( 101,  135), S(  81,  108),
        S(  45,  131), S(  67,  145), S(  67,  132), S(  54,  128), S(  77,  141), S( 100,  125), S( 114,  131), S(  73,  108),
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
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(27, -49), //Queen
    S(56, -16), //Rook
    S(26, 28),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(48, 44), //Queen
    S(42, 4),  //Rook
    S(0, 0),   //Bishop
    S(23, 19), //Knight
    S(0, 0),   //Pawn
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
