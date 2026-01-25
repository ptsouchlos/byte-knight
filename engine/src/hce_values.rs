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
        S(  19,  -94), S(  -1,  -40), S(  14,  -22), S(-116,   25), S( -76,    9), S(  -6,    6), S(  57,   -6), S( 176, -117),
        S(-112,    7), S( -71,   38), S(-108,   50), S(  -9,   35), S( -53,   52), S( -48,   64), S(   0,   50), S( -17,   16),
        S(-128,   20), S( -18,   40), S( -89,   60), S(-108,   72), S( -68,   72), S(  14,   63), S(  -5,   59), S( -47,   28),
        S( -94,   10), S(-105,   45), S(-123,   66), S(-171,   79), S(-159,   79), S(-120,   73), S(-114,   61), S(-137,   33),
        S( -97,    0), S(-100,   32), S(-135,   58), S(-164,   74), S(-161,   73), S(-122,   59), S(-125,   45), S(-155,   29),
        S( -45,  -12), S( -30,   15), S( -91,   39), S(-104,   51), S( -98,   51), S( -95,   42), S( -48,   21), S( -65,    7),
        S(  48,  -31), S(   3,   -2), S( -13,   11), S( -48,   22), S( -51,   26), S( -32,   16), S(  19,   -3), S(  29,  -23),
        S(  43,  -69), S(  64,  -48), S(  36,  -28), S( -67,   -9), S(  -1,  -35), S( -41,  -10), S(  45,  -38), S(  47,  -68),
    ],
    // Queen
    [
        S( 778, 1285), S( 788, 1298), S( 816, 1316), S( 851, 1304), S( 847, 1303), S( 861, 1289), S( 879, 1245), S( 820, 1281),
        S( 817, 1251), S( 792, 1295), S( 799, 1329), S( 789, 1351), S( 798, 1366), S( 837, 1320), S( 813, 1309), S( 863, 1281),
        S( 818, 1257), S( 815, 1278), S( 813, 1318), S( 830, 1321), S( 835, 1337), S( 877, 1317), S( 881, 1278), S( 878, 1266),
        S( 801, 1272), S( 806, 1294), S( 811, 1308), S( 809, 1334), S( 811, 1348), S( 823, 1333), S( 823, 1319), S( 830, 1296),
        S( 804, 1267), S( 801, 1296), S( 801, 1305), S( 808, 1328), S( 807, 1326), S( 807, 1316), S( 818, 1295), S( 821, 1285),
        S( 802, 1251), S( 809, 1271), S( 803, 1297), S( 803, 1293), S( 806, 1298), S( 813, 1289), S( 826, 1264), S( 819, 1252),
        S( 799, 1248), S( 805, 1253), S( 816, 1250), S( 815, 1261), S( 813, 1263), S( 822, 1235), S( 828, 1203), S( 839, 1175),
        S( 795, 1244), S( 787, 1250), S( 794, 1253), S( 809, 1243), S( 801, 1248), S( 787, 1246), S( 810, 1215), S( 802, 1219),
    ],
    // Rook
    [
        S( 431,  691), S( 420,  699), S( 428,  707), S( 432,  704), S( 451,  694), S( 467,  684), S( 441,  691), S( 471,  681),
        S( 411,  691), S( 410,  703), S( 429,  706), S( 448,  698), S( 435,  697), S( 466,  682), S( 453,  678), S( 484,  666),
        S( 391,  690), S( 413,  691), S( 415,  693), S( 418,  691), S( 448,  677), S( 452,  669), S( 490,  661), S( 465,  658),
        S( 374,  693), S( 388,  691), S( 392,  698), S( 400,  695), S( 406,  680), S( 409,  673), S( 419,  671), S( 419,  663),
        S( 355,  687), S( 358,  691), S( 369,  691), S( 381,  690), S( 382,  685), S( 367,  684), S( 392,  670), S( 382,  667),
        S( 348,  681), S( 358,  680), S( 367,  679), S( 366,  684), S( 373,  679), S( 370,  672), S( 406,  650), S( 383,  653),
        S( 345,  675), S( 358,  679), S( 373,  678), S( 370,  680), S( 375,  671), S( 377,  667), S( 396,  657), S( 362,  664),
        S( 364,  669), S( 366,  679), S( 377,  687), S( 382,  685), S( 386,  678), S( 375,  672), S( 392,  669), S( 367,  659),
    ],
    // Bishop
    [
        S( 268,  376), S( 252,  387), S( 260,  382), S( 219,  395), S( 233,  390), S( 246,  380), S( 278,  380), S( 239,  371),
        S( 285,  363), S( 308,  381), S( 302,  385), S( 287,  389), S( 318,  376), S( 315,  380), S( 304,  384), S( 297,  360),
        S( 299,  387), S( 322,  384), S( 322,  395), S( 345,  384), S( 332,  389), S( 364,  391), S( 342,  384), S( 329,  383),
        S( 289,  385), S( 305,  401), S( 325,  396), S( 337,  412), S( 333,  402), S( 329,  400), S( 304,  397), S( 291,  384),
        S( 284,  383), S( 296,  398), S( 304,  407), S( 323,  406), S( 321,  405), S( 304,  402), S( 297,  395), S( 291,  371),
        S( 293,  380), S( 302,  392), S( 300,  400), S( 305,  401), S( 305,  405), S( 301,  399), S( 303,  383), S( 307,  371),
        S( 297,  377), S( 297,  374), S( 310,  374), S( 286,  388), S( 295,  390), S( 306,  379), S( 313,  381), S( 300,  356),
        S( 272,  355), S( 296,  375), S( 277,  355), S( 271,  376), S( 274,  372), S( 273,  374), S( 296,  361), S( 285,  341),
    ],
    // Knight
    [
        S( 131,  286), S( 170,  346), S( 232,  364), S( 266,  354), S( 300,  358), S( 239,  336), S( 186,  347), S( 187,  263),
        S( 268,  344), S( 285,  364), S( 312,  371), S( 332,  370), S( 313,  364), S( 376,  347), S( 283,  360), S( 308,  327),
        S( 285,  356), S( 321,  373), S( 338,  391), S( 349,  392), S( 387,  375), S( 391,  366), S( 346,  363), S( 314,  345),
        S( 282,  371), S( 295,  391), S( 321,  402), S( 342,  405), S( 325,  405), S( 349,  399), S( 306,  391), S( 317,  361),
        S( 268,  372), S( 283,  382), S( 299,  404), S( 300,  405), S( 310,  409), S( 304,  397), S( 303,  381), S( 279,  363),
        S( 248,  356), S( 272,  376), S( 286,  386), S( 290,  399), S( 301,  398), S( 290,  382), S( 294,  370), S( 265,  356),
        S( 236,  348), S( 247,  363), S( 265,  374), S( 277,  376), S( 278,  374), S( 280,  370), S( 266,  352), S( 263,  356),
        S( 192,  339), S( 245,  325), S( 233,  358), S( 248,  358), S( 253,  360), S( 266,  348), S( 248,  331), S( 221,  333),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S( 172,  308), S( 187,  311), S( 171,  306), S( 204,  255), S( 179,  254), S( 163,  262), S(  94,  315), S(  80,  313),
        S(  75,  147), S(  83,  163), S( 117,  114), S( 116,   79), S( 125,   75), S( 159,   90), S( 130,  141), S(  98,  128),
        S(  52,  129), S(  69,  134), S(  79,  111), S(  79,   94), S( 102,   94), S(  97,   96), S(  90,  125), S(  80,  105),
        S(  42,  112), S(  61,  124), S(  66,  105), S(  82,  100), S(  81,  100), S(  76,  100), S(  74,  117), S(  65,   95),
        S(  40,  107), S(  56,  121), S(  63,  105), S(  62,  109), S(  77,  111), S(  70,  104), S(  91,  113), S(  71,   91),
        S(  39,  111), S(  56,  123), S(  58,  112), S(  46,  109), S(  66,  120), S(  83,  107), S(  98,  111), S(  63,   92),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(-27, 2),
    S(8, 123),
    S(13, 59),
    S(-9, 35),
    S(-12, 12),
    S(-6, 8),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-14, -29),
    S(6, -25),
    S(-6, -16),
    S(-1, -7),
    S(-11, -3),
    S(-8, -16),
    S(-3, -29),
    S(-12, -39),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-3, 0),
    S(-10, -17),
    S(-20, -13),
    S(-19, -22),
    S(-20, -23),
    S(-14, -10),
    S(-13, -15),
    S(-12, 4),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(22, 67);

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
