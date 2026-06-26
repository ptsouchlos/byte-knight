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
    pub const KNIGHT_MOVES: usize = 8;
    pub const BISHOP_MOVES: usize = 13;
    pub const ROOK_MOVES: usize = 14;
    pub const QUEEN_MOVES: usize = 27;
    pub const PAWN_SHIELD_RANKS: usize = 4;
    pub const PAWN_STORM_RANKS: usize = 4;
    pub const KING_FLANK_FILES: usize = 3;
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
