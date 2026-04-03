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
        S(  -0,  -68), S(  41,  -25), S(  41,  -17), S( -61,   21), S(  14,    2), S(  24,   10), S(  89,  -10), S(  75,  -85),
        S( -30,    4), S(   2,   37), S( -44,   46), S(  52,   30), S(  -0,   48), S(  15,   56), S(  36,   47), S(  -2,   15),
        S( -46,   20), S(  29,   41), S( -16,   56), S( -44,   64), S( -16,   68), S(  47,   60), S(  17,   60), S(   2,   28),
        S( -40,   10), S(  -8,   36), S( -51,   55), S( -84,   65), S( -69,   64), S( -42,   60), S( -53,   52), S( -93,   31),
        S( -40,   -2), S( -21,   20), S( -43,   40), S( -72,   53), S( -72,   51), S( -38,   37), S( -40,   26), S( -90,   20),
        S(  -1,  -12), S(  23,    3), S( -21,   22), S( -44,   35), S( -35,   33), S( -25,   23), S(   3,    7), S( -15,   -3),
        S(  47,  -26), S(  13,   -5), S(   0,    4), S( -27,   12), S( -26,   16), S( -14,   11), S(  13,   -4), S(  34,  -28),
        S(  21,  -65), S(  27,  -38), S(  12,  -25), S( -44,  -16), S(  -3,  -31), S( -33,   -9), S(  11,  -30), S(  28,  -70),
    ],
    // Queen
    [
        S( 384,  541), S( 393,  541), S( 413,  556), S( 427,  547), S( 438,  541), S( 446,  535), S( 464,  505), S( 432,  523),
        S( 424,  526), S( 410,  548), S( 407,  578), S( 398,  595), S( 393,  606), S( 433,  568), S( 438,  552), S( 464,  544),
        S( 430,  528), S( 425,  546), S( 428,  577), S( 425,  580), S( 434,  584), S( 462,  560), S( 469,  543), S( 447,  544),
        S( 418,  545), S( 426,  555), S( 421,  566), S( 420,  585), S( 431,  582), S( 431,  575), S( 440,  569), S( 435,  554),
        S( 427,  531), S( 419,  550), S( 426,  554), S( 430,  572), S( 433,  566), S( 427,  567), S( 438,  554), S( 439,  542),
        S( 421,  516), S( 430,  532), S( 434,  544), S( 428,  545), S( 434,  554), S( 437,  549), S( 446,  528), S( 443,  514),
        S( 431,  506), S( 432,  510), S( 437,  513), S( 443,  519), S( 443,  525), S( 451,  497), S( 454,  480), S( 451,  465),
        S( 423,  505), S( 430,  501), S( 433,  511), S( 436,  525), S( 440,  508), S( 430,  495), S( 444,  475), S( 438,  478),
    ],
    // Rook
    [
        S( 193,  284), S( 193,  292), S( 178,  298), S( 184,  292), S( 203,  279), S( 202,  291), S( 190,  295), S( 184,  286),
        S( 172,  291), S( 171,  306), S( 187,  300), S( 206,  290), S( 192,  288), S( 210,  290), S( 187,  294), S( 184,  287),
        S( 172,  288), S( 194,  291), S( 194,  286), S( 196,  283), S( 223,  267), S( 225,  275), S( 243,  274), S( 191,  274),
        S( 161,  288), S( 174,  291), S( 181,  290), S( 189,  285), S( 193,  270), S( 193,  280), S( 188,  282), S( 174,  275),
        S( 157,  278), S( 155,  288), S( 173,  280), S( 175,  280), S( 180,  271), S( 160,  287), S( 182,  275), S( 162,  265),
        S( 152,  271), S( 156,  276), S( 166,  271), S( 163,  274), S( 175,  264), S( 172,  270), S( 189,  254), S( 172,  250),
        S( 152,  264), S( 157,  273), S( 172,  268), S( 171,  271), S( 179,  260), S( 178,  264), S( 181,  254), S( 152,  254),
        S( 163,  269), S( 169,  270), S( 176,  271), S( 179,  268), S( 186,  258), S( 180,  268), S( 170,  264), S( 168,  254),
    ],
    // Bishop
    [
        S( 109,  147), S( 102,  151), S(  81,  150), S(  49,  157), S(  57,  152), S(  46,  148), S( 116,  142), S(  87,  141),
        S( 122,  134), S( 125,  146), S( 123,  145), S( 113,  146), S( 110,  144), S( 123,  143), S( 101,  151), S( 115,  135),
        S( 132,  148), S( 144,  146), S( 136,  154), S( 138,  146), S( 145,  147), S( 164,  154), S( 152,  148), S( 133,  152),
        S( 124,  146), S( 136,  153), S( 140,  153), S( 157,  166), S( 148,  159), S( 149,  155), S( 131,  153), S( 122,  148),
        S( 131,  141), S( 124,  153), S( 137,  159), S( 155,  160), S( 155,  159), S( 140,  155), S( 138,  148), S( 138,  136),
        S( 126,  142), S( 148,  151), S( 148,  152), S( 146,  155), S( 151,  159), S( 153,  151), S( 152,  143), S( 147,  136),
        S( 142,  144), S( 146,  134), S( 153,  134), S( 142,  145), S( 152,  141), S( 159,  136), S( 166,  136), S( 150,  126),
        S( 135,  135), S( 152,  146), S( 135,  139), S( 132,  141), S( 136,  139), S( 133,  148), S( 149,  132), S( 154,  113),
    ],
    // Knight
    [
        S( -11,   68), S(  -3,  113), S(  65,  128), S(  91,  114), S( 138,  111), S(  34,  110), S(  22,  105), S(  22,   49),
        S(  98,  113), S( 113,  124), S( 116,  126), S( 133,  125), S( 129,  120), S( 155,  109), S( 107,  120), S( 124,   98),
        S( 113,  117), S( 130,  125), S( 139,  145), S( 146,  148), S( 158,  144), S( 187,  128), S( 139,  127), S( 141,  107),
        S( 123,  127), S( 129,  137), S( 146,  151), S( 166,  152), S( 141,  159), S( 162,  152), S( 128,  149), S( 150,  126),
        S( 115,  133), S( 128,  130), S( 135,  149), S( 144,  149), S( 151,  153), S( 143,  147), S( 153,  132), S( 128,  129),
        S(  97,  115), S( 116,  123), S( 123,  130), S( 132,  143), S( 145,  141), S( 132,  124), S( 135,  118), S( 120,  119),
        S(  94,  113), S( 107,  121), S( 113,  119), S( 128,  122), S( 130,  121), S( 128,  117), S( 124,  112), S( 121,  125),
        S(  69,  112), S(  98,  109), S(  99,  114), S( 113,  121), S( 119,  119), S( 123,  110), S( 103,  115), S(  97,  116),
    ],
    // Pawn
    [
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
        S(  87,  133), S(  83,  135), S(  54,  141), S(  71,  110), S(  44,  123), S(  67,  126), S(  53,  160), S(  45,  147),
        S(  74,  153), S(  68,  169), S(  90,  134), S(  88,  110), S(  85,  116), S(  98,  136), S(  87,  171), S(  50,  157),
        S(  58,  128), S(  63,  129), S(  68,  109), S(  68,   96), S(  83,   96), S(  75,  102), S(  70,  125), S(  55,  111),
        S(  54,  110), S(  56,  118), S(  66,  101), S(  77,   95), S(  77,   96), S(  73,   97), S(  72,  110), S(  54,   95),
        S(  50,  106), S(  58,  113), S(  62,  101), S(  63,  103), S(  71,  106), S(  66,  100), S(  79,  104), S(  53,   93),
        S(  50,  107), S(  56,  115), S(  57,  106), S(  51,  110), S(  59,  115), S(  69,  104), S(  83,  106), S(  44,   97),
        S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0), S(   0,    0),
    ],
];

pub const PASSED_PAWN_BONUS: [PhasedScore; NumberOf::PASSED_PAWN_RANKS] = [
    S(70, 182),
    S(10, 111),
    S(10, 57),
    S(-10, 34),
    S(-10, 12),
    S(-6, 8),
];

pub const DOUBLED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-17, -31),
    S(2, -24),
    S(-4, -16),
    S(-4, -6),
    S(-11, -4),
    S(-5, -13),
    S(-1, -21),
    S(-9, -33),
];

pub const ISOLATED_PAWN_VALUES: [PhasedScore; NumberOf::FILES] = [
    S(-6, -1),
    S(-7, -15),
    S(-15, -11),
    S(-15, -19),
    S(-17, -19),
    S(-5, -12),
    S(-8, -15),
    S(1, -1),
];

pub const BISHOP_PAIR_BONUS: PhasedScore = S(18, 57);

pub const KING_SAFETY: [PhasedScore; NumberOf::PIECE_TYPES - 1] =
    [S(-14, -10), S(-17, 6), S(-21, 5), S(-11, 8), S(-11, 6)];

pub const PAWN_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(68, -34), //Queen
    S(77, 8),   //Rook
    S(56, 45),  //Bishop
    S(55, 26),  //Knight
    S(0, 0),    //Pawn
];

pub const KNIGHT_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),    //King
    S(45, -14), //Queen
    S(62, 13),  //Rook
    S(29, 29),  //Bishop
    S(0, 0),    //Knight
    S(0, 0),    //Pawn
];

pub const BISHOP_THREAT: [PhasedScore; NumberOf::PIECE_TYPES] = [
    S(0, 0),   //King
    S(63, 34), //Queen
    S(49, 26), //Rook
    S(0, 0),   //Bishop
    S(21, 21), //Knight
    S(0, 0),   //Pawn
];

pub const KNIGHT_MOBILITY: [PhasedScore; NumberOf::KNIGHT_MOVES + 1] = [
    S(126, 156),
    S(144, 187),
    S(154, 206),
    S(160, 214),
    S(166, 221),
    S(172, 229),
    S(179, 227),
    S(185, 224),
    S(191, 210),
];

pub const BISHOP_MOBILITY: [PhasedScore; NumberOf::BISHOP_MOVES + 1] = [
    S(130, 155),
    S(140, 172),
    S(150, 184),
    S(156, 196),
    S(162, 205),
    S(165, 214),
    S(168, 218),
    S(170, 220),
    S(171, 223),
    S(175, 220),
    S(182, 216),
    S(185, 215),
    S(183, 222),
    S(203, 200),
];

pub const ROOK_MOBILITY: [PhasedScore; NumberOf::ROOK_MOVES + 1] = [
    S(185, 296),
    S(194, 317),
    S(198, 320),
    S(200, 325),
    S(198, 332),
    S(203, 335),
    S(204, 341),
    S(206, 342),
    S(210, 344),
    S(211, 348),
    S(214, 349),
    S(214, 354),
    S(216, 355),
    S(215, 354),
    S(213, 351),
];

pub const QUEEN_MOBILITY: [PhasedScore; NumberOf::QUEEN_MOVES + 1] = [
    S(396, 247),
    S(398, 447),
    S(391, 510),
    S(393, 539),
    S(394, 553),
    S(398, 559),
    S(400, 573),
    S(401, 585),
    S(403, 592),
    S(405, 594),
    S(406, 600),
    S(408, 605),
    S(410, 606),
    S(409, 612),
    S(412, 610),
    S(413, 614),
    S(412, 620),
    S(415, 617),
    S(421, 614),
    S(435, 604),
    S(438, 604),
    S(477, 578),
    S(471, 579),
    S(509, 548),
    S(575, 524),
    S(572, 510),
    S(484, 574),
    S(445, 555),
];

// Small bonus for being the side to move.
pub const TEMPO_BONUS: PhasedScore = S(23, 20);

pub const ROOK_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(28, 5),
    S(26, 1),
    S(22, 10),
    S(26, 8),
    S(28, 14),
    S(41, -2),
    S(55, -6),
    S(69, -4),
];

pub const ROOK_SEMI_OPEN_FILE_BONUS: [PhasedScore; NumberOf::FILES] = [
    S(1, 40),
    S(8, 16),
    S(8, 16),
    S(15, 7),
    S(6, 13),
    S(20, -1),
    S(26, 3),
    S(7, 28),
];

pub const PAWN_SHIELD: [[PhasedScore; NumberOf::PAWN_SHIELD_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(56, -16), S(46, -8), S(34, -5), S(25, 5)],
    // Left adjacent
    [S(37, -8), S(20, -5), S(14, -3), S(10, 5)],
    // Right adjacent
    [S(34, -17), S(29, -8), S(21, -1), S(13, 6)],
];

pub const PAWN_STORM: [[PhasedScore; NumberOf::PAWN_STORM_RANKS]; NumberOf::KING_FLANK_FILES] = [
    // King file
    [S(113, 157), S(-34, 96), S(-4, 20), S(7, 3)],
    // Left adjacent
    [S(-1, 169), S(-61, 84), S(-23, 22), S(-3, 4)],
    // Right adjacent
    [S(-40, 190), S(-52, 72), S(-19, 18), S(-2, 2)],
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
