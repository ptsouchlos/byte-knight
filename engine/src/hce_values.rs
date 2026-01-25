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
        S(  20,  -95), S(   0,  -40), S(  15,  -23), S(-117,   25), S( -76,    9), S(  -6,    5), S(  58,   -6), S( 179, -119),
        S(-113,    7), S( -71,   38), S(-109,   51), S(  -9,   35), S( -53,   53), S( -48,   64), S(   1,   51), S( -17,   16),
        S(-129,   20), S( -18,   40), S( -89,   61), S(-109,   72), S( -68,   72), S(  15,   63), S(  -4,   60), S( -47,   28),
        S( -95,    9), S(-105,   46), S(-124,   67), S(-172,   80), S(-160,   80), S(-121,   74), S(-115,   61), S(-138,   33),
        S( -98,    0), S(-101,   32), S(-136,   59), S(-165,   75), S(-162,   74), S(-123,   60), S(-126,   45), S(-156,   29),
        S( -45,  -12), S( -30,   14), S( -91,   39), S(-105,   52), S( -99,   52), S( -96,   42), S( -48,   21), S( -65,    7),
        S(  49,  -32), S(   4,   -2), S( -13,   11), S( -48,   22), S( -51,   26), S( -32,   16), S(  19,   -4), S(  30,  -23),
        S(  44,  -70), S(  65,  -49), S(  37,  -29), S( -67,   -9), S(   0,  -36), S( -41,  -10), S(  46,  -39), S(  48,  -69),
    ],
    // Queen
    [
        S( 785, 1302), S( 794, 1315), S( 822, 1333), S( 858, 1321), S( 854, 1320), S( 868, 1306), S( 887, 1262), S( 826, 1298),
        S( 824, 1267), S( 799, 1312), S( 806, 1346), S( 796, 1369), S( 805, 1384), S( 844, 1338), S( 820, 1327), S( 870, 1298),
        S( 825, 1274), S( 822, 1295), S( 820, 1335), S( 837, 1338), S( 842, 1355), S( 885, 1334), S( 888, 1294), S( 885, 1283),
        S( 808, 1289), S( 813, 1311), S( 817, 1325), S( 816, 1351), S( 817, 1366), S( 830, 1350), S( 830, 1337), S( 837, 1313),
        S( 810, 1284), S( 808, 1313), S( 808, 1322), S( 814, 1345), S( 814, 1343), S( 814, 1333), S( 825, 1312), S( 828, 1302),
        S( 808, 1268), S( 816, 1288), S( 810, 1315), S( 810, 1310), S( 812, 1315), S( 819, 1306), S( 833, 1281), S( 826, 1268),
        S( 805, 1265), S( 812, 1269), S( 823, 1266), S( 822, 1277), S( 820, 1280), S( 829, 1252), S( 835, 1219), S( 846, 1191),
        S( 802, 1261), S( 793, 1266), S( 801, 1270), S( 816, 1259), S( 808, 1264), S( 794, 1262), S( 817, 1232), S( 808, 1235),
    ],
    // Rook
    [
        S( 435,  700), S( 425,  708), S( 433,  715), S( 437,  712), S( 455,  703), S( 472,  692), S( 446,  699), S( 476,  689),
        S( 416,  699), S( 414,  711), S( 434,  715), S( 453,  707), S( 440,  706), S( 471,  690), S( 458,  687), S( 489,  674),
        S( 395,  698), S( 417,  699), S( 419,  701), S( 422,  699), S( 452,  685), S( 457,  677), S( 495,  669), S( 470,  666),
        S( 378,  701), S( 392,  699), S( 396,  707), S( 405,  703), S( 410,  688), S( 413,  681), S( 423,  679), S( 423,  671),
        S( 359,  695), S( 362,  699), S( 373,  700), S( 385,  698), S( 386,  694), S( 371,  692), S( 397,  678), S( 386,  675),
        S( 352,  689), S( 362,  688), S( 371,  687), S( 370,  692), S( 377,  687), S( 374,  680), S( 411,  658), S( 387,  661),
        S( 348,  683), S( 361,  687), S( 377,  687), S( 374,  688), S( 379,  680), S( 381,  675), S( 400,  665), S( 366,  672),
        S( 368,  677), S( 370,  687), S( 381,  695), S( 386,  693), S( 390,  686), S( 379,  680), S( 397,  677), S( 371,  667),
    ],
    // Bishop
    [
        S( 271,  380), S( 254,  392), S( 263,  387), S( 221,  399), S( 236,  394), S( 249,  385), S( 281,  385), S( 241,  376),
        S( 288,  368), S( 311,  386), S( 306,  390), S( 291,  394), S( 322,  381), S( 318,  385), S( 307,  389), S( 301,  364),
        S( 303,  392), S( 326,  389), S( 326,  400), S( 348,  389), S( 336,  394), S( 368,  396), S( 345,  388), S( 332,  387),
        S( 292,  389), S( 308,  405), S( 329,  401), S( 341,  417), S( 337,  407), S( 333,  405), S( 307,  402), S( 294,  389),
        S( 287,  387), S( 299,  403), S( 307,  412), S( 326,  411), S( 324,  410), S( 308,  407), S( 301,  400), S( 294,  375),
        S( 297,  385), S( 306,  397), S( 304,  405), S( 308,  406), S( 308,  410), S( 304,  404), S( 306,  387), S( 310,  376),
        S( 300,  381), S( 300,  379), S( 314,  379), S( 289,  392), S( 298,  394), S( 310,  384), S( 317,  385), S( 303,  360),
        S( 275,  360), S( 300,  380), S( 280,  359), S( 274,  381), S( 277,  377), S( 276,  378), S( 299,  365), S( 288,  345),
    ],
    // Knight
    [
        S( 132,  289), S( 172,  350), S( 235,  369), S( 268,  358), S( 304,  362), S( 241,  340), S( 188,  351), S( 189,  266),
        S( 271,  348), S( 288,  368), S( 316,  375), S( 336,  374), S( 317,  368), S( 380,  351), S( 286,  365), S( 311,  331),
        S( 288,  361), S( 325,  378), S( 341,  395), S( 353,  397), S( 391,  379), S( 396,  370), S( 349,  368), S( 317,  349),
        S( 285,  376), S( 298,  396), S( 325,  407), S( 346,  410), S( 328,  410), S( 353,  404), S( 309,  396), S( 320,  365),
        S( 271,  376), S( 286,  387), S( 303,  409), S( 303,  410), S( 313,  414), S( 308,  401), S( 306,  386), S( 282,  367),
        S( 251,  360), S( 275,  381), S( 289,  391), S( 293,  404), S( 304,  402), S( 294,  386), S( 297,  374), S( 268,  361),
        S( 238,  353), S( 250,  367), S( 268,  379), S( 280,  380), S( 281,  379), S( 283,  374), S( 269,  357), S( 266,  361),
        S( 194,  344), S( 248,  328), S( 236,  362), S( 250,  362), S( 256,  364), S( 268,  352), S( 250,  335), S( 223,  337),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 172,  308), S( 187,  312), S( 172,  307), S( 205,  255), S( 179,  254), S( 163,  262), S(  93,  315), S(  79,  313),
        S(  76,  148), S(  84,  164), S( 118,  115), S( 117,   80), S( 126,   76), S( 161,   91), S( 132,  143), S(  99,  130),
        S(  53,  131), S(  70,  136), S(  80,  112), S(  80,   95), S( 103,   95), S(  98,   97), S(  91,  126), S(  81,  107),
        S(  42,  114), S(  61,  126), S(  67,  106), S(  83,  101), S(  82,  102), S(  77,  101), S(  75,  119), S(  66,   96),
        S(  40,  108), S(  57,  122), S(  64,  106), S(  62,  110), S(  78,  112), S(  71,  106), S(  92,  114), S(  72,   92),
        S(  40,  112), S(  57,  125), S(  58,  114), S(  46,  110), S(  67,  122), S(  84,  108), S(  99,  113), S(  64,   93),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-26, 5),
    S(8, 124),
    S(13, 60),
    S(-9, 36),
    S(-12, 13),
    S(-6, 8),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-14, -29),
    S(6, -26),
    S(-6, -17),
    S(-1, -7),
    S(-11, -3),
    S(-8, -17),
    S(-3, -29),
    S(-12, -40),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-3, 0),
    S(-10, -17),
    S(-21, -13),
    S(-19, -22),
    S(-20, -23),
    S(-14, -10),
    S(-14, -16),
    S(-12, 4),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(22, 68);

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
}
