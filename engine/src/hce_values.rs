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
        S(  16, -104), S(  -1,  -42), S(  16,  -23), S(-128,   31), S( -82,   13), S( -21,   10), S(  42,   -3), S( 182, -130),
        S(-127,   11), S( -47,   42), S( -92,   56), S(  34,   36), S( -21,   56), S( -21,   71), S(  19,   57), S( -47,   24),
        S(-152,   26), S(   7,   44), S( -71,   65), S( -85,   76), S( -38,   76), S(  53,   63), S(   6,   65), S( -57,   32),
        S(-104,   10), S( -81,   44), S( -96,   63), S(-145,   77), S(-130,   75), S( -92,   64), S(-100,   56), S(-164,   34),
        S(-109,   -0), S( -79,   27), S(-100,   50), S(-132,   67), S(-125,   63), S( -83,   45), S(-101,   34), S(-178,   27),
        S( -52,  -13), S(  -8,   10), S( -64,   32), S( -75,   46), S( -64,   45), S( -64,   33), S( -22,   13), S( -74,    5),
        S(  44,  -33), S(  13,   -1), S(  -8,   15), S( -47,   27), S( -45,   30), S( -27,   20), S(  25,   -2), S(  22,  -23),
        S(  28,  -77), S(  57,  -49), S(  18,  -21), S( -92,   -2), S( -23,  -26), S( -61,   -3), S(  29,  -35), S(  27,  -74),
    ],
    // Queen
    [
        S( 918, 1480), S( 927, 1488), S( 954, 1507), S( 989, 1487), S( 971, 1489), S( 977, 1489), S(1011, 1438), S( 949, 1475),
        S( 970, 1448), S( 945, 1490), S( 945, 1529), S( 933, 1551), S( 920, 1571), S( 969, 1510), S( 952, 1506), S(1012, 1473),
        S( 980, 1462), S( 975, 1484), S( 975, 1522), S( 977, 1522), S( 979, 1529), S(1004, 1505), S(1011, 1472), S( 996, 1458),
        S( 958, 1490), S( 966, 1509), S( 963, 1516), S( 955, 1542), S( 964, 1539), S( 969, 1522), S( 974, 1517), S( 971, 1488),
        S( 965, 1480), S( 956, 1506), S( 955, 1514), S( 966, 1529), S( 965, 1529), S( 962, 1514), S( 976, 1491), S( 975, 1479),
        S( 959, 1457), S( 970, 1477), S( 965, 1502), S( 961, 1504), S( 968, 1513), S( 972, 1493), S( 986, 1461), S( 976, 1449),
        S( 959, 1449), S( 967, 1455), S( 979, 1456), S( 980, 1474), S( 979, 1478), S( 991, 1431), S( 995, 1393), S(1007, 1359),
        S( 957, 1448), S( 945, 1454), S( 956, 1465), S( 974, 1458), S( 964, 1460), S( 949, 1443), S( 971, 1413), S( 964, 1416),
    ],
    // Rook
    [
        S( 478,  820), S( 460,  830), S( 462,  841), S( 458,  839), S( 472,  830), S( 497,  817), S( 474,  822), S( 509,  812),
        S( 468,  817), S( 460,  831), S( 477,  837), S( 494,  830), S( 471,  831), S( 495,  815), S( 486,  810), S( 521,  796),
        S( 463,  812), S( 488,  813), S( 489,  814), S( 489,  812), S( 523,  796), S( 503,  793), S( 540,  787), S( 506,  784),
        S( 447,  816), S( 463,  813), S( 468,  822), S( 477,  817), S( 480,  800), S( 472,  794), S( 471,  795), S( 473,  786),
        S( 425,  810), S( 427,  814), S( 439,  815), S( 454,  812), S( 456,  806), S( 420,  808), S( 445,  793), S( 437,  789),
        S( 416,  805), S( 427,  803), S( 438,  801), S( 436,  807), S( 443,  800), S( 429,  792), S( 463,  769), S( 443,  773),
        S( 413,  797), S( 428,  802), S( 446,  801), S( 443,  801), S( 447,  792), S( 444,  786), S( 461,  774), S( 428,  783),
        S( 435,  793), S( 437,  802), S( 450,  810), S( 455,  807), S( 460,  799), S( 447,  792), S( 459,  790), S( 435,  782),
    ],
    // Bishop
    [
        S( 318,  443), S( 300,  455), S( 305,  449), S( 254,  464), S( 252,  460), S( 266,  451), S( 312,  449), S( 275,  438),
        S( 331,  430), S( 366,  448), S( 354,  454), S( 339,  456), S( 352,  446), S( 346,  451), S( 327,  457), S( 323,  430),
        S( 351,  457), S( 375,  453), S( 376,  466), S( 381,  456), S( 362,  462), S( 391,  466), S( 367,  457), S( 345,  459),
        S( 338,  454), S( 358,  471), S( 365,  469), S( 375,  487), S( 374,  473), S( 368,  473), S( 356,  465), S( 323,  455),
        S( 335,  451), S( 337,  470), S( 347,  479), S( 371,  478), S( 367,  477), S( 357,  472), S( 349,  465), S( 342,  435),
        S( 338,  447), S( 350,  462), S( 353,  470), S( 354,  473), S( 359,  478), S( 355,  470), S( 356,  451), S( 361,  436),
        S( 344,  444), S( 350,  439), S( 363,  440), S( 340,  456), S( 350,  459), S( 364,  446), S( 370,  448), S( 354,  420),
        S( 321,  418), S( 346,  443), S( 329,  420), S( 322,  443), S( 327,  438), S( 326,  441), S( 348,  425), S( 341,  399),
    ],
    // Knight
    [
        S( 156,  339), S( 202,  408), S( 274,  433), S( 308,  424), S( 350,  428), S( 274,  402), S( 225,  411), S( 215,  315),
        S( 316,  408), S( 338,  432), S( 368,  441), S( 384,  445), S( 359,  439), S( 426,  420), S( 334,  429), S( 353,  391),
        S( 338,  423), S( 381,  443), S( 397,  466), S( 402,  472), S( 435,  456), S( 448,  443), S( 389,  439), S( 353,  417),
        S( 335,  440), S( 349,  465), S( 379,  480), S( 405,  482), S( 366,  491), S( 401,  482), S( 339,  476), S( 364,  433),
        S( 320,  441), S( 338,  453), S( 357,  480), S( 356,  482), S( 369,  485), S( 359,  473), S( 356,  455), S( 330,  431),
        S( 296,  422), S( 325,  446), S( 340,  457), S( 345,  474), S( 357,  472), S( 343,  452), S( 345,  439), S( 316,  423),
        S( 282,  412), S( 295,  429), S( 316,  443), S( 330,  445), S( 330,  442), S( 334,  437), S( 315,  417), S( 313,  423),
        S( 233,  401), S( 292,  385), S( 280,  422), S( 296,  422), S( 302,  425), S( 319,  410), S( 296,  393), S( 266,  394),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 184,  321), S( 201,  327), S( 186,  321), S( 224,  262), S( 190,  267), S( 171,  274), S(  74,  339), S(  60,  331),
        S(  87,  176), S(  99,  196), S( 135,  141), S( 133,  103), S( 138,  104), S( 185,  119), S( 142,  179), S( 107,  158),
        S(  63,  153), S(  84,  161), S(  94,  134), S(  95,  115), S( 118,  118), S( 117,  120), S( 106,  154), S(  94,  126),
        S(  50,  133), S(  74,  148), S(  78,  126), S(  96,  121), S(  96,  122), S(  89,  123), S(  87,  142), S(  77,  113),
        S(  47,  127), S(  69,  143), S(  74,  125), S(  73,  130), S(  88,  133), S(  83,  125), S( 104,  135), S(  84,  108),
        S(  46,  131), S(  68,  146), S(  68,  133), S(  55,  130), S(  78,  143), S( 101,  126), S( 116,  132), S(  74,  109),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-12, 45),
    S(10, 144),
    S(13, 72),
    S(-12, 43),
    S(-14, 15),
    S(-6, 9),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-17, -34),
    S(6, -30),
    S(-7, -20),
    S(-1, -10),
    S(-13, -5),
    S(-14, -20),
    S(-7, -34),
    S(-15, -46),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-2, -0),
    S(-13, -19),
    S(-24, -14),
    S(-22, -25),
    S(-25, -26),
    S(-17, -11),
    S(-15, -18),
    S(-13, 6),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(26, 80);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-22, -16), S(-42, 9), S(-33, 5), S(-16, 9), S(-16, 16)];

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
}
