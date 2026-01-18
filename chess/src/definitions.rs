// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::bitboard::Bitboard;

pub const SPACE: char = ' ';
pub const NEWLINE: char = '\n';
pub const DASH: char = '-';
pub const EM_DASH: char = '–';
pub const SLASH: char = '/';

/// max number of moves in a game from this pos R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1
pub const MAX_MOVE_LIST_SIZE: usize = 218;
/// Maximum number of moves saved in the history
pub const MAX_MOVES: usize = 3072;
pub const MAX_MOVE_RULE: u32 = 100;

// see the tests in move_generation.rs for how these numbers were calculated
pub const ROOK_BLOCKER_PERMUTATIONS: usize = 102_400;
pub const BISHOP_BLOCKER_PERMUTATIONS: usize = 5_248;
pub(crate) const MAX_REPETITION_COUNT: usize = 2;

pub const QUEEN_OFFSETS: [(i8, i8); 8] = [
    // diagonals (bishop)
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
    // straight lines (rook)
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
];

pub const BISHOP_OFFSETS: [(i8, i8); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
pub const ROOK_OFFSETS: [(i8, i8); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

pub struct NumberOf;
impl NumberOf {
    pub const PIECE_TYPES: usize = 6;
    pub const SQUARES: usize = 64;
    pub const FILES: usize = 8;
    pub const RANKS: usize = 8;
    pub const SIDES: usize = 2;
    pub const CASTLING_OPTIONS: usize = 16;
    // Passed pawns cannot be on ranks 1 or 8
    pub const PASSED_PAWN_RANKS: usize = 6;
}

pub const EMPTY: u64 = 0;

pub struct CastlingAvailability;
impl CastlingAvailability {
    pub const NONE: u8 = 0;
    pub const WHITE_KINGSIDE: u8 = 1;
    pub const WHITE_QUEENSIDE: u8 = 2;
    pub const BLACK_KINGSIDE: u8 = 4;
    pub const BLACK_QUEENSIDE: u8 = 8;
    pub const ALL: u8 =
        Self::WHITE_KINGSIDE | Self::WHITE_QUEENSIDE | Self::BLACK_KINGSIDE | Self::BLACK_QUEENSIDE;
}

pub static DEFAULT_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// 0001 0000 0001 0000 0001 0000 0001 0000 0001 0000 0001 0000 0001 0000 0001
// 72,340,172,838,076,673 as decimal
pub const FILE_A: u64 = 0x0101010101010101;
pub const RANK_1: u64 = 0xFF;

pub struct Squares;
impl Squares {
    pub const A1: u8 = 0;
    pub const B1: u8 = 1;
    pub const C1: u8 = 2;
    pub const D1: u8 = 3;
    pub const E1: u8 = 4;
    pub const F1: u8 = 5;
    pub const G1: u8 = 6;
    pub const H1: u8 = 7;

    pub const A2: u8 = 8;
    pub const B2: u8 = 9;
    pub const C2: u8 = 10;
    pub const D2: u8 = 11;
    pub const E2: u8 = 12;
    pub const F2: u8 = 13;
    pub const G2: u8 = 14;
    pub const H2: u8 = 15;

    pub const A3: u8 = 16;
    pub const B3: u8 = 17;
    pub const C3: u8 = 18;
    pub const D3: u8 = 19;
    pub const E3: u8 = 20;
    pub const F3: u8 = 21;
    pub const G3: u8 = 22;
    pub const H3: u8 = 23;

    pub const A4: u8 = 24;
    pub const B4: u8 = 25;
    pub const C4: u8 = 26;
    pub const D4: u8 = 27;
    pub const E4: u8 = 28;
    pub const F4: u8 = 29;
    pub const G4: u8 = 30;
    pub const H4: u8 = 31;

    pub const A5: u8 = 32;
    pub const B5: u8 = 33;
    pub const C5: u8 = 34;
    pub const D5: u8 = 35;
    pub const E5: u8 = 36;
    pub const F5: u8 = 37;
    pub const G5: u8 = 38;
    pub const H5: u8 = 39;

    pub const A6: u8 = 40;
    pub const B6: u8 = 41;
    pub const C6: u8 = 42;
    pub const D6: u8 = 43;
    pub const E6: u8 = 44;
    pub const F6: u8 = 45;
    pub const G6: u8 = 46;
    pub const H6: u8 = 47;

    pub const A7: u8 = 48;
    pub const B7: u8 = 49;
    pub const C7: u8 = 50;
    pub const D7: u8 = 51;
    pub const E7: u8 = 52;
    pub const F7: u8 = 53;
    pub const G7: u8 = 54;
    pub const H7: u8 = 55;

    pub const A8: u8 = 56;
    pub const B8: u8 = 57;
    pub const C8: u8 = 58;
    pub const D8: u8 = 59;
    pub const E8: u8 = 60;
    pub const F8: u8 = 61;
    pub const G8: u8 = 62;
    pub const H8: u8 = 63;
}

pub const DARK_SQUARES: u64 = 0xAA55AA55AA55AA55;

type FileBitboards = [Bitboard; NumberOf::FILES];
type RankBitboards = [Bitboard; NumberOf::RANKS];

pub const FILE_BITBOARDS: FileBitboards = [
    Bitboard::new(72340172838076673),
    Bitboard::new(144680345676153346),
    Bitboard::new(289360691352306692),
    Bitboard::new(578721382704613384),
    Bitboard::new(1157442765409226768),
    Bitboard::new(2314885530818453536),
    Bitboard::new(4629771061636907072),
    Bitboard::new(9259542123273814144),
];

pub const RANK_BITBOARDS: RankBitboards = [
    Bitboard::new(255),
    Bitboard::new(65280),
    Bitboard::new(16711680),
    Bitboard::new(4278190080),
    Bitboard::new(1095216660480),
    Bitboard::new(280375465082880),
    Bitboard::new(71776119061217280),
    Bitboard::new(18374686479671623680),
];
