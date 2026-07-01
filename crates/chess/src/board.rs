// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::fmt::Display;
use std::iter::zip;

use crate::board_state::BoardState;
use crate::definitions::{CastlingAvailability, MAX_MOVE_RULE, MAX_REPETITION_COUNT, SPACE};
use crate::fen::FenError;
use crate::file::File;
use crate::move_history::BoardHistory;
use crate::moves::Move;
use crate::rank::Rank;
use crate::square::Square;
use crate::zobrist;
use crate::zobrist::{Hashes, keys};

use super::definitions::NumberOf;
use super::fen;
use super::side::Side;
use super::{bitboard::Bitboard, pieces::Piece};

/// Represents a chessboard position.
#[derive(Debug, Clone)]
pub struct Board {
    bitboards: [Bitboard; NumberOf::PIECE_TYPES + NumberOf::SIDES],
    pieces: [Option<Piece>; NumberOf::SQUARES],
    pub(crate) history: BoardHistory,
    state: BoardState,
}

impl Default for Board {
    fn default() -> Self {
        Board::default_board()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const FILES: &str = "   a b c d e f g h\n";
        const LINES: &str = "  +----------------+\n";

        write!(f, "{}", FILES)?;
        write!(f, "{}", LINES)?;

        for rank in (0..8).rev() {
            write!(f, "{} |", rank + 1)?;
            for file in 0..8 {
                let square =
                    Square::new(File::try_from(file).unwrap(), Rank::try_from(rank).unwrap());
                if let Some((piece, side)) = self.piece_on_square(square) {
                    let piece_char = match piece {
                        Piece::King => 'K',
                        Piece::Queen => 'Q',
                        Piece::Rook => 'R',
                        Piece::Bishop => 'B',
                        Piece::Knight => 'N',
                        Piece::Pawn => 'P',
                    };
                    let display_char = if side == Side::White {
                        piece_char
                    } else {
                        piece_char.to_ascii_lowercase()
                    };
                    write!(f, "{} ", display_char)?;
                } else {
                    write!(f, ". ")?;
                }

                if file == 7 {
                    write!(f, "|")?;
                }
            }
            writeln!(f)?;
        }
        write!(f, "{}", LINES)?;
        write!(f, "{}", FILES)?;
        Ok(())
    }
}

// Private methods
impl Board {
    /// Create a new board in the default, *uninitialized*, state.
    fn new() -> Self {
        Board {
            bitboards: [Bitboard::default(); NumberOf::PIECE_TYPES + NumberOf::SIDES],
            pieces: [None; NumberOf::SQUARES],
            history: BoardHistory::new(),
            state: BoardState::new(),
        }
    }

    fn initialize(&mut self) {
        self.state.hashes = self.initialize_zobrist_hash();
    }

    fn initialize_zobrist_hash(&self) -> zobrist::Hashes {
        // TODO
        zobrist::Hashes::new(self)
    }

    /// Initialize bitboards for a given side
    fn initialize_piece_bbs(&mut self) {
        // Set up the board with the starting position
        let layout: &[(Piece, u64, u64)] = &[
            (Piece::Pawn, 0xFF00, 0xFF000000000000),
            (Piece::Knight, 0x42, 0x4200000000000000),
            (Piece::Bishop, 0x24, 0x2400000000000000),
            (Piece::Rook, 0x81, 0x8100000000000000),
            (Piece::Queen, 0x8, 0x800000000000000),
            (Piece::King, 0x10, 0x1000000000000000),
        ];

        for &(piece, white_bits, black_bits) in layout {
            let white_bb = Bitboard::new(white_bits);
            let black_bb = Bitboard::new(black_bits);
            for sq in white_bb {
                self.set_piece_square(piece, Side::White, sq);
            }
            for sq in black_bb {
                self.set_piece_square(piece, Side::Black, sq);
            }
        }
    }

    pub(crate) fn set_piece_square(&mut self, piece: Piece, side: Side, square: Square) {
        let piece_bb = &mut self.bitboards[piece as usize];
        piece_bb.set_square(square);

        let side_bb = &mut self.bitboards[NumberOf::PIECE_TYPES + side as usize];
        side_bb.set_square(square);

        self.pieces[square] = Some(piece);
    }

    pub(crate) fn remove_piece_from_square(&mut self, piece: Piece, side: Side, square: Square) {
        let piece_bb = &mut self.bitboards[piece as usize];
        piece_bb.clear_square(square);

        let side_bb = &mut self.bitboards[NumberOf::PIECE_TYPES + side as usize];
        side_bb.clear_square(square);

        self.pieces[square] = None;
    }

    /// Sets the side to move and updates the zobrist hash.
    pub(crate) fn set_side_to_move(&mut self, side: Side) {
        // undo the current side to move in the hash
        self.state
            .hashes
            .update_hash(keys::side_hash(self.state.side_to_move));
        // set the new side to move
        self.state.side_to_move = side;
        // update zobrist hash with the new side to move
        self.state
            .hashes
            .update_hash(keys::side_hash(self.state.side_to_move));
    }

    /// Set the en passant square and update the zobrist hash.
    pub(crate) fn set_en_passant_square(&mut self, square: Option<Square>) {
        self.state.hashes.update_hash(keys::ep_hash(
            self.state.en_passant_square.map(Square::from_square_index),
        ));
        self.state.en_passant_square = square.map(|sq| sq.inner());
        self.state.hashes.update_hash(keys::ep_hash(
            self.state.en_passant_square.map(Square::from_square_index),
        ));
    }

    pub(crate) fn set_half_move_clock(&mut self, half_move_clock: u32) {
        self.state.half_move_clock = half_move_clock;
    }

    pub(crate) fn set_full_move_number(&mut self, full_move_number: u32) {
        self.state.full_move_number = full_move_number;
    }

    pub(crate) fn set_castling_rights(&mut self, castling_rights: u8) {
        self.state
            .hashes
            .update_hash(keys::castling_hash(self.state.castling_rights));
        self.state.castling_rights = castling_rights;
        self.state
            .hashes
            .update_hash(keys::castling_hash(self.state.castling_rights));
    }

    pub(crate) fn update_zobrist_hash_for_piece(
        &mut self,
        square: Square,
        piece: Piece,
        side: Side,
    ) {
        self.state
            .hashes
            .update_hash(keys::sq_hash(piece, side, square));
        if piece == Piece::Pawn {
            self.state
                .hashes
                .update_pawn_hash(keys::sq_hash(piece, side, square));
        }
    }

    pub(crate) fn board_state(&self) -> &BoardState {
        &self.state
    }

    pub(crate) fn set_board_state(&mut self, state: BoardState) {
        self.state = state;
    }
}

// Public API
impl Board {
    /// Create a new board with the default starting position.
    pub fn default_board() -> Board {
        let mut board = Board::new();
        // Set up the board with the starting position
        board.initialize_piece_bbs();
        board.set_en_passant_square(None);
        board.set_half_move_clock(0);
        board.set_full_move_number(1);
        board.set_side_to_move(Side::White);
        board.set_castling_rights(CastlingAvailability::ALL);
        board.state.hashes = Hashes::new(&board);
        board
    }

    /// Create a new board from a FEN string.
    ///
    /// # Arguments
    ///
    /// - `fen` - A FEN string representing the board state.
    ///
    /// # Returns
    ///
    /// - a Result containing a [`Board`] if parsing was successful or
    ///   [`FenError`] if the FEN string is invalid or cannot be parsed.
    pub fn from_fen(fen: &str) -> Result<Board, FenError> {
        let mut board = Board::new();

        // parse the FEN string
        let fen_parts = fen::split_fen_string(fen);
        match fen_parts {
            Ok(parts) => {
                let fen_part_parsers = fen::FEN_PART_PARSERS;
                for (part, parser) in zip(parts, fen_part_parsers) {
                    parser(&mut board, &part)?;
                }
            }
            Err(e) => {
                return Err(e);
            }
        }

        // the parser initializes most of the board state, but we need to set the zobrist hash
        // initializing the board will handle initializing anything that isn't set by the FEN parser
        board.initialize();

        Ok(board)
    }

    /// Convert the board to a FEN string.
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        // Piece placement
        fen.push_str(&fen::piece_placement_to_fen(self));
        fen.push(SPACE);
        // Active color
        fen.push_str(&fen::active_color_to_fen(self));
        fen.push(SPACE);
        // Castling availability
        fen.push_str(&fen::castling_availability_to_fen(self));
        fen.push(SPACE);
        // En passant target square
        fen.push_str(&fen::en_passant_target_square_to_fen(self));
        fen.push(SPACE);
        // Halfmove clock
        fen.push_str(&fen::halfmove_clock_to_fen(self));
        fen.push(SPACE);
        // Fullmove number
        fen.push_str(&fen::fullmove_number_to_fen(self));

        fen
    }

    /// Returns the all pieces of this [`Board`].
    /// This is also known as the occupancy bitboard.
    pub fn all_pieces(&self) -> Bitboard {
        self.white_pieces() | self.black_pieces()
    }

    /// Returns all the pieces of a given side in a single [`Bitboard`].
    pub fn pieces(&self, side: Side) -> Bitboard {
        self.bitboards[NumberOf::PIECE_TYPES + side as usize]
    }

    /// Returns the white pieces of this [`Board`] in a single [`Bitboard`].
    pub fn white_pieces(&self) -> Bitboard {
        self.pieces(Side::White)
    }

    /// Returns the black pieces of this [`Board`] in a single [`Bitboard`].
    pub fn black_pieces(&self) -> Bitboard {
        self.pieces(Side::Black)
    }

    /// Returns the bitboard for a specific piece and side.
    pub fn piece_bitboard(&self, piece: Piece, side: Side) -> Bitboard {
        self.piece_kind_bitboard(piece) & self.pieces(side)
    }

    /// Returns a combined [`Bitboard`] of all pieces of a given type for both sides.
    pub fn piece_kind_bitboard(&self, piece: Piece) -> Bitboard {
        self.bitboards[piece as usize]
    }

    /// Returns the current square of the king for a given side.
    pub fn king_square(&self, side: Side) -> Square {
        let king_bb = self.piece_bitboard(Piece::King, side);
        king_bb.lsb().unwrap()
    }

    /// Find what piece is on a given square.
    ///
    /// # Arguments
    ///
    /// - `square` - The square to check.
    ///
    /// # Returns
    ///
    /// - Optional tuple of the piece and the side that the piece belongs to. (Piece, Side)
    pub fn piece_on_square(&self, square: Square) -> Option<(Piece, Side)> {
        let piece = self.pieces[square];
        piece?;

        let bb = Bitboard::from(square);
        let side = if !(bb & self.pieces(Side::White)).is_empty() {
            Side::White
        } else {
            Side::Black
        };

        Some((piece.unwrap(), side))
    }

    /// Returns the side to move of this [`Board`].
    pub fn side_to_move(&self) -> Side {
        self.state.side_to_move
    }

    /// Returns the en passant square of this [`Board`] (if it exists)
    pub fn en_passant_square(&self) -> Option<Square> {
        self.state.en_passant_square.map(Square::from_square_index)
    }

    /// Returns the half move clock of this [`Board`].
    pub fn half_move_clock(&self) -> u32 {
        self.state.half_move_clock
    }

    /// Returns the full move number of this [`Board`].
    pub fn full_move_number(&self) -> u32 {
        self.state.full_move_number
    }

    /// Returns the castling rights of this [`Board`].
    pub fn castling_rights(&self) -> u8 {
        self.state.castling_rights
    }

    /// Returns the Zobrist hash of this [`Board`].
    pub fn zobrist_hash(&self) -> u64 {
        self.state.hashes.board_hash()
    }

    /// Returns the Zobrist hash of the pawn structure of this [`Board`].
    pub fn pawn_hash(&self) -> u64 {
        self.state.hashes.pawn_hash()
    }

    /// Checks if a given square is empty.
    pub fn is_square_empty(&self, square: Square) -> bool {
        !self.all_pieces().is_square_occupied(square)
    }

    /// Helper function to check if a given side has kingside castling rights.
    ///
    /// # Arguments
    ///
    /// - `side` - The side to check.
    ///
    /// # Returns
    ///
    /// - `true` if the side has kingside castling rights, otherwise `false`.
    pub fn can_castle_kingside(&self, side: Side) -> bool {
        let castling_rights = self.castling_rights();
        match side {
            Side::White => castling_rights & CastlingAvailability::WHITE_KINGSIDE != 0,
            Side::Black => castling_rights & CastlingAvailability::BLACK_KINGSIDE != 0,
        }
    }

    /// Helper function to check if a given side has queenside castling rights.
    ///
    /// # Arguments
    ///
    /// - `side` - The side to check.
    ///
    /// # Returns
    ///
    /// - `true` if the side has queenside castling rights, otherwise `false`.
    pub fn can_castle_queenside(&self, side: Side) -> bool {
        let castling_rights = self.castling_rights();
        match side {
            Side::White => castling_rights & CastlingAvailability::WHITE_QUEENSIDE != 0,
            Side::Black => castling_rights & CastlingAvailability::BLACK_QUEENSIDE != 0,
        }
    }

    /// Get the color of the piece on a given square.
    ///
    /// Returns `Some(Side)` if the square is occupied, otherwise `None`.
    pub fn color_on(&self, square: Square) -> Option<Side> {
        let white_pieces = self.white_pieces();
        let black_pieces = self.black_pieces();
        if white_pieces.is_square_occupied(square) {
            Some(Side::White)
        } else if black_pieces.is_square_occupied(square) {
            Some(Side::Black)
        } else {
            None
        }
    }

    /// Checks for draws for the current [`Board`].
    ///
    /// This function checks for:
    /// - Fifty move rule
    /// - Insufficient material
    /// - Threefold repetition
    pub fn is_draw(&self) -> bool {
        self.is_draw_by_fifty_move_rule() || self.insufficient_material() || self.is_repetition()
    }

    /// Check if the game is a draw by insufficient material. We use the FIDE rules for this check.
    ///
    /// Returns true if the game is a draw by insufficient material, otherwise false.
    pub fn insufficient_material(&self) -> bool {
        // If any side has a Queen, Rook or Pawn, there's sufficient material
        let pawns = self.piece_kind_bitboard(Piece::Pawn);
        let rooks = self.piece_kind_bitboard(Piece::Rook);
        let queens = self.piece_kind_bitboard(Piece::Queen);
        if (pawns | rooks | queens).number_of_occupied_squares() > 0 {
            return false;
        }

        let knights = self.piece_kind_bitboard(Piece::Knight);
        let bishops = self.piece_kind_bitboard(Piece::Bishop);

        let minor_pieces = knights | bishops;
        if minor_pieces.number_of_occupied_squares() <= 1 {
            return true;
        }

        // check bishops and knights
        let white_bishops = bishops & self.pieces(Side::White);
        let black_bishops = bishops & self.pieces(Side::Black);
        let white_knights = knights & self.pieces(Side::White);
        let black_knights = knights & self.pieces(Side::Black);

        let wb_count = white_bishops.number_of_occupied_squares();
        let bb_count = black_bishops.number_of_occupied_squares();
        let wn_count = white_knights.number_of_occupied_squares();
        let bn_count = black_knights.number_of_occupied_squares();

        match (wb_count, bb_count, wn_count, bn_count) {
            // only kings left
            (0, 0, 0, 0) => true,
            // single bishops
            (1, 0, 0, 0) => true,
            (0, 1, 0, 0) => true,
            // single knight
            (0, 0, 1, 0) => true,
            (0, 0, 0, 1) => true,
            (1, 1, 0, 0) => {
                // bishops on the same color
                Square::from_bitboard(&white_bishops).color()
                    == Square::from_bitboard(&black_bishops).color()
            }
            _ => false,
        }
    }

    /// Check if the game is a draw by the fifty move rule.
    pub fn is_draw_by_fifty_move_rule(&self) -> bool {
        self.half_move_clock() >= MAX_MOVE_RULE
    }

    /// Check if the game is a draw by threefold repetition.
    pub fn is_repetition(&self) -> bool {
        let mut repetition_count = 0;
        // go through the history and check if the current position has been repeated
        for previous_state in self.history.iter().rev().skip(1) {
            // we found a match, increment the repetition count
            if previous_state.hashes.board_hash() == self.zobrist_hash() {
                repetition_count += 1;
                if repetition_count >= MAX_REPETITION_COUNT {
                    // break out early
                    return true;
                }
            }

            // we only need to go back up to the last pawn move, castle, or capture as these moves reset the half-move clock
            // beyond this point, there can't be a repeated position
            if previous_state.half_move_clock == 0 {
                return false;
            }
        }

        repetition_count >= 2
    }

    /// Get the last move made on the board.
    ///
    /// Returns `None` if there are no moves in the history.
    /// Otherwise, returns the last move made.
    pub fn last_move(&self) -> Option<Move> {
        self.history.iter().last().map(|m| m.next_move)
    }

    /// Returns a new board with the position mirrored vertically (ranks flipped).
    ///
    /// White and black pieces are swapped, the side to move is toggled, castling
    /// rights are swapped between the two sides, and the en passant square is
    /// flipped.
    pub fn flip(&self) -> Board {
        let mut flipped = Board::new();

        // Swap sides and flip each bitboard vertically (swap_bytes mirrors ranks).
        for piece in Piece::iter() {
            let white_flipped = Bitboard::new(
                self.piece_bitboard(piece, Side::White)
                    .as_number()
                    .swap_bytes(),
            );
            let black_flipped = Bitboard::new(
                self.piece_bitboard(piece, Side::Black)
                    .as_number()
                    .swap_bytes(),
            );

            for sq in black_flipped {
                flipped.set_piece_square(piece, Side::White, sq);
            }
            for sq in white_flipped {
                flipped.set_piece_square(piece, Side::Black, sq);
            }
        }

        flipped.state.side_to_move = self.state.side_to_move.opposite();
        flipped.state.half_move_clock = self.state.half_move_clock;
        flipped.state.full_move_number = self.state.full_move_number;

        // Swap castling rights between sides.
        let cr = self.state.castling_rights;
        let mut flipped_cr = CastlingAvailability::NONE;
        if cr & CastlingAvailability::WHITE_KINGSIDE != 0 {
            flipped_cr |= CastlingAvailability::BLACK_KINGSIDE;
        }
        if cr & CastlingAvailability::WHITE_QUEENSIDE != 0 {
            flipped_cr |= CastlingAvailability::BLACK_QUEENSIDE;
        }
        if cr & CastlingAvailability::BLACK_KINGSIDE != 0 {
            flipped_cr |= CastlingAvailability::WHITE_KINGSIDE;
        }
        if cr & CastlingAvailability::BLACK_QUEENSIDE != 0 {
            flipped_cr |= CastlingAvailability::WHITE_QUEENSIDE;
        }
        flipped.state.castling_rights = flipped_cr;

        flipped.state.en_passant_square = self.state.en_passant_square.map(crate::square::flip);

        flipped.state.hashes = flipped.initialize_zobrist_hash();
        flipped
    }

    /// Helper to get the captured piece for a given move.
    /// Use this instead of checking piece_on_square for the move's destination square,
    /// as this will correctly handle en passant captures and castling moves.
    ///
    /// # Arguments
    /// - `mv` - The move for which to get the captured piece.
    ///
    /// # Returns
    /// - `Some(Piece)` if the move is a capture, otherwise `None`.
    #[inline]
    pub fn captured(&self, mv: &Move) -> Option<Piece> {
        if mv.is_castle() {
            return None;
        }
        if mv.is_en_passant_capture() {
            return Some(Piece::Pawn);
        }
        self.piece_on_square(mv.to()).map(|(piece, _side)| piece)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        definitions::DEFAULT_FEN,
        file::File,
        move_generation::{self, move_filter::MoveFilter},
        move_list::MoveList,
        moves::MoveFlag,
        rank::Rank,
        side::Side,
        square,
    };

    use super::*;
    #[test]
    fn threefold_repetition_detection() {
        let mut board = Board::from_fen("k7/8/KQ6/8/8/8/8/8 w - - 0 1").unwrap();

        let bk_square_1 = Square::A8;
        let bk_square_2 = Square::B8;

        let wq_square_1 = Square::B6;
        let wq_square_2 = Square::C5;

        let white_queen_move = Move::new(wq_square_1, wq_square_2, MoveFlag::Standard);
        let while_queen_reverse_move = Move::new(wq_square_2, wq_square_1, MoveFlag::Standard);

        let black_king_move = Move::new(bk_square_1, bk_square_2, MoveFlag::Standard);

        let black_king_reverse_move = Move::new(bk_square_2, bk_square_1, MoveFlag::Standard);

        for _i in 0..2 {
            assert!(board.make_move_unchecked(&white_queen_move).is_ok());
            assert!(board.make_move_unchecked(&black_king_move).is_ok());
            assert!(board.make_move_unchecked(&while_queen_reverse_move).is_ok());
            assert!(board.make_move_unchecked(&black_king_reverse_move).is_ok());
        }

        assert!(board.is_repetition());
    }

    #[test]
    fn checkmate() {
        {
            let board =
                Board::from_fen("r1b1k1nr/pppp1ppp/2n5/4P3/8/2Q2N2/P1P1PPPP/RNq1KB1R w KQkq - 1 9")
                    .unwrap();

            assert!(move_generation::is_in_check(&board));
            assert!(move_generation::is_checkmate(&board));
        }
        {
            let board =
                Board::from_fen("r1b3nr/5ppp/3pk2R/8/2Q5/4R1PB/2PPPP1P/RNB1K1NR b KQ - 0 1")
                    .unwrap();
            assert!(move_generation::is_in_check(&board));
            assert!(move_generation::is_checkmate(&board));
        }
    }

    #[test]
    fn test_default_board() {
        let board = Board::default_board();
        assert_eq!(board.all_pieces(), 0xFFFF00000000FFFF);
        assert_eq!(board.to_fen(), DEFAULT_FEN);
    }

    #[test]
    fn make_and_unmake_move_changes_hash() {
        static FEN: &str = "6nr/pp3p1p/k1p5/8/1QN5/2P1P3/4KPqP/8 b - - 5 26";
        let mut move_list = MoveList::new();
        let mut board = Board::from_fen(FEN).unwrap();
        let hash = board.zobrist_hash();

        move_generation::generate_moves(&board, &mut move_list, MoveFilter::All);

        for mv in move_list.iter() {
            let mv_ok = board.make_move(mv);
            if mv_ok.is_ok() {
                // legal move, check that the new hash is different
                let move_hash = board.zobrist_hash();
                assert_ne!(hash, move_hash);
                // undo the move
                let undo_result = board.unmake_move();
                assert!(undo_result.is_ok());
                // check that the hash is back to the original value
                assert_eq!(hash, board.zobrist_hash());
            }
        }
    }

    #[test]
    fn make_move_updates_castling_rights() {
        // TODO
    }

    #[test]
    fn insufficient_material_check() {
        // test cases taken from https://github.com/dannyhammer/chessie/blob/b9ff0e4340b4600c497570ed11cd18c3654c99b9/chessie/src/position.rs#L412
        // Lone Kings
        let kk = Board::from_fen("8/4k3/8/8/3K4/8/8/8 w - - 0 1").unwrap();
        assert!(kk.insufficient_material());

        // A single Bishop (either color)
        let kbk = Board::from_fen("8/4k3/8/8/3K4/8/5B2/8 w - - 0 1").unwrap();
        assert!(kbk.insufficient_material());

        // A single Knight
        let knk = Board::from_fen("8/4k3/2n5/8/3K4/8/8/8 w - - 0 1").unwrap();
        assert!(knk.insufficient_material());

        // Opposing Bishops on the same color square
        let same_square_bishops = Board::from_fen("8/2b1k3/8/8/3K4/8/5B2/8 w - - 0 1").unwrap();
        assert!(same_square_bishops.insufficient_material());

        // Opposing Bishops on different color squares
        let diff_square_bishops = Board::from_fen("8/3bk3/8/8/3K4/8/5B2/8 w - - 0 1").unwrap();
        assert!(!diff_square_bishops.insufficient_material());
    }

    #[test]
    fn check_square_is_empty() {
        let board = Board::default_board();
        // All of these squares should be empty.
        // TODO: Use proper Rank and File iteration
        for rank in (Rank::R3 as u8)..=(Rank::R6 as u8) {
            for file in (File::A as u8)..=(File::H as u8) {
                let square = square::to_square_object(file, rank);
                assert!(board.is_square_empty(square));
            }
        }
    }

    #[test]
    fn from_fen_round_trip() {
        // load Pohl.epd from data and go through each FEN. Load it into the board and then output the FEN to see if they match
        let path = format!("{}/{}/{}", env!("CARGO_WORKSPACE_DIR"), "data", "Pohl.epd");
        let lines = std::fs::read_to_string(path).unwrap();
        println!("Loaded {} FEN strings from Pohl.epd", lines.lines().count());
        // It seems that the Pohl.epd test data uses strict FEN encoding which means the EP square is set after every double pawn push.
        // See https://www.chessprogramming.org/Forsyth-Edwards_Notation#En_passant_target_square
        // Parsing strips EP targets that aren't capturable, so instead of checking for equivalent FENs, we parse,
        // emit a new FEN and parse again to ensure they match.
        for fen in lines.lines() {
            let board = Board::from_fen(fen).unwrap();
            let emitted = board.to_fen();
            let reparsed = Board::from_fen(&emitted).unwrap();
            assert_eq!(emitted, reparsed.to_fen());
        }
    }

    #[test]
    fn from_invalid_fen() {
        let maybe_board = Board::from_fen("");
        assert!(maybe_board.is_err());
        let err = maybe_board.unwrap_err();
        let message = format!("{err}");
        // check that the message contains something about the FEN being empty
        assert!(message.to_lowercase().contains("empty"));
    }

    #[test]
    fn from_fen_illegal_ep() {
        // f6 EP would expose the white king on h2 to a discovered check from the
        // black queen on c7, and exf6 has no opposing pawn from g5, so the EP
        // target should be stripped on parse.
        const FENS: [&str; 2] = [
            "1r6/2q4k/2P1b1pp/bB2Ppn1/R2B2PN/p4P1P/P1Q4K/1R6 w - f6 0 39",
            "8/p2r2K1/6p1/1kp1PpP1/2p5/2P5/8/4R3 w - f6 0 44",
        ];

        for fen in FENS {
            let maybe_board = Board::from_fen(fen);
            assert!(maybe_board.is_ok(), "failed to parse {fen}");
            assert!(
                maybe_board.unwrap().en_passant_square().is_none(),
                "expected EP to be stripped for {fen}"
            );
        }
    }

    #[test]
    fn from_fen_legal_ep_kept() {
        // Standard positions where the EP capture is legal: parsing must keep
        // the EP target.
        const FENS: [&str; 2] = [
            // white e5, black d5 just pushed.
            "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
            // mirror: white d4 just pushed, black to move with c4 pawn able to capture.
            "rnbqkbnr/pp1ppppp/8/8/2pP4/8/PPP1PPPP/RNBQKBNR b KQkq d3 0 3",
        ];

        for fen in FENS {
            let maybe_board = Board::from_fen(fen);
            assert!(maybe_board.is_ok(), "failed to parse {fen}");
            assert!(
                maybe_board.unwrap().en_passant_square().is_some(),
                "expected EP to be kept for {fen}"
            );
        }
    }

    #[test]
    fn color_on() {
        let board = Board::default_board();
        for sq in Bitboard::filled() {
            let color = board.color_on(sq);
            if sq.index() <= 15 {
                assert!(color.is_some_and(|c| c == Side::White));
            } else if sq.index() >= 48 {
                assert!(color.is_some_and(|c| c == Side::Black));
            } else {
                assert!(color.is_none());
            }
        }
    }

    #[test]
    fn get_last_move() {
        let mut board = Board::default_board();
        let mut move_list = MoveList::new();
        move_generation::generate_moves(&board, &mut move_list, MoveFilter::All);

        let first_move = move_list.iter().next().unwrap();
        let mv_ok = board.make_move(first_move);
        assert!(mv_ok.is_ok());

        let last_move = board.last_move().unwrap();
        assert_eq!(last_move, *first_move);

        // undo the move
        let undo_result = board.unmake_move();
        assert!(undo_result.is_ok());

        // now make a null move
        board.null_move();
        let last_move = board.last_move().unwrap();
        assert!(last_move.is_null_move());

        // undo the null move
        let undo_result = board.unmake_move();
        assert!(undo_result.is_ok());
        assert!(board.to_fen() == Board::default_board().to_fen());
    }

    #[test]
    fn piece_kind_bitboard() {
        let board = Board::default_board();
        let piece_kind_bb = board.piece_kind_bitboard(Piece::Pawn);

        let black_pawns_bb = board.piece_bitboard(Piece::Pawn, Side::Black);
        let white_pawns_bb = board.piece_bitboard(Piece::Pawn, Side::White);

        assert_eq!(piece_kind_bb, black_pawns_bb | white_pawns_bb);
        assert_eq!(piece_kind_bb.number_of_occupied_squares(), 16);
    }

    #[test]
    fn piece_bitboards_for_side() {
        let board = Board::default_board();
        let white_pieces_bb = board.pieces(Side::White);
        let black_pieces_bb = board.pieces(Side::Black);

        let expected_white_bb = 0x000000000000FFFF;
        let expected_black_bb = 0xFFFF000000000000;

        assert_eq!(white_pieces_bb, Bitboard::new(expected_white_bb));
        assert_eq!(black_pieces_bb, Bitboard::new(expected_black_bb));

        let all = board.all_pieces();
        assert_eq!(all, white_pieces_bb | black_pieces_bb);
    }

    #[test]
    fn flip() {
        // standard EPD suite FEN positions
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "4k3/8/8/8/8/8/8/4K2R w K - 0 1",
            "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1",
            "4k2r/8/8/8/8/8/8/4K3 w k - 0 1",
            "r3k3/8/8/8/8/8/8/4K3 w q - 0 1",
            "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1",
            "r3k2r/8/8/8/8/8/8/4K3 w kq - 0 1",
            "8/8/8/8/8/8/6k1/4K2R w K - 0 1",
            "8/8/8/8/8/8/1k6/R3K3 w Q - 0 1",
            "4k2r/6K1/8/8/8/8/8/8 w k - 0 1",
            "r3k3/1K6/8/8/8/8/8/8 w q - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            "r3k2r/8/8/8/8/8/8/1R2K2R w Kkq - 0 1",
            "r3k2r/8/8/8/8/8/8/2R1K2R w Kkq - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K1R1 w Qkq - 0 1",
            "1r2k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1",
            "2r1k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1",
            "r3k1r1/8/8/8/8/8/8/R3K2R w KQq - 0 1",
            "4k3/8/8/8/8/8/8/4K2R b K - 0 1",
            "4k3/8/8/8/8/8/8/R3K3 b Q - 0 1",
            "4k2r/8/8/8/8/8/8/4K3 b k - 0 1",
            "r3k3/8/8/8/8/8/8/4K3 b q - 0 1",
            "4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1",
            "r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1",
            "8/8/8/8/8/8/6k1/4K2R b K - 0 1",
            "8/8/8/8/8/8/1k6/R3K3 b Q - 0 1",
            "4k2r/6K1/8/8/8/8/8/8 b k - 0 1",
            "r3k3/1K6/8/8/8/8/8/8 b q - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
            "r3k2r/8/8/8/8/8/8/1R2K2R b Kkq - 0 1",
            "r3k2r/8/8/8/8/8/8/2R1K2R b Kkq - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1",
            "1r2k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1",
            "2r1k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1",
            "r3k1r1/8/8/8/8/8/8/R3K2R b KQq - 0 1",
            "8/1n4N1/2k5/8/8/5K2/1N4n1/8 w - - 0 1",
            "8/1k6/8/5N2/8/4n3/8/2K5 w - - 0 1",
            "8/8/4k3/3Nn3/3nN3/4K3/8/8 w - - 0 1",
            "K7/8/2n5/1n6/8/8/8/k6N w - - 0 1",
            "k7/8/2N5/1N6/8/8/8/K6n w - - 0 1",
            "8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1",
            "8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1",
            "8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1",
            "K7/8/2n5/1n6/8/8/8/k6N b - - 0 1",
            "k7/8/2N5/1N6/8/8/8/K6n b - - 0 1",
            "B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1",
            "8/8/1B6/7b/7k/8/2B1b3/7K w - - 0 1",
            "k7/B7/1B6/1B6/8/8/8/K6b w - - 0 1",
            "K7/b7/1b6/1b6/8/8/8/k6B w - - 0 1",
            "B6b/8/8/8/2K5/5k2/8/b6B b - - 0 1",
            "8/8/1B6/7b/7k/8/2B1b3/7K b - - 0 1",
            "k7/B7/1B6/1B6/8/8/8/K6b b - - 0 1",
            "K7/b7/1b6/1b6/8/8/8/k6B b - - 0 1",
            "7k/RR6/8/8/8/8/rr6/7K w - - 0 1",
            "R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1",
            "7k/RR6/8/8/8/8/rr6/7K b - - 0 1",
            "R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1",
            "6kq/8/8/8/8/8/8/7K w - - 0 1",
            "6KQ/8/8/8/8/8/8/7k b - - 0 1",
            "K7/8/8/3Q4/4q3/8/8/7k w - - 0 1",
            "6qk/8/8/8/8/8/8/7K b - - 0 1",
            "6KQ/8/8/8/8/8/8/7k b - - 0 1",
            "K7/8/8/3Q4/4q3/8/8/7k b - - 0 1",
            "8/8/8/8/8/K7/P7/k7 w - - 0 1",
            "8/8/8/8/8/7K/7P/7k w - - 0 1",
            "K7/p7/k7/8/8/8/8/8 w - - 0 1",
            "7K/7p/7k/8/8/8/8/8 w - - 0 1",
            "8/2k1p3/3pP3/3P2K1/8/8/8/8 w - - 0 1",
            "8/8/8/8/8/K7/P7/k7 b - - 0 1",
            "8/8/8/8/8/7K/7P/7k b - - 0 1",
            "K7/p7/k7/8/8/8/8/8 b - - 0 1",
            "7K/7p/7k/8/8/8/8/8 b - - 0 1",
            "8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1",
            "8/8/8/8/8/4k3/4P3/4K3 w - - 0 1",
            "4k3/4p3/4K3/8/8/8/8/8 b - - 0 1",
            "8/8/7k/7p/7P/7K/8/8 w - - 0 1",
            "8/8/k7/p7/P7/K7/8/8 w - - 0 1",
            "8/8/3k4/3p4/3P4/3K4/8/8 w - - 0 1",
            "8/3k4/3p4/8/3P4/3K4/8/8 w - - 0 1",
            "8/8/3k4/3p4/8/3P4/3K4/8 w - - 0 1",
            "k7/8/3p4/8/3P4/8/8/7K w - - 0 1",
            "8/8/7k/7p/7P/7K/8/8 b - - 0 1",
            "8/8/k7/p7/P7/K7/8/8 b - - 0 1",
            "8/8/3k4/3p4/3P4/3K4/8/8 b - - 0 1",
            "8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1",
            "8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1",
            "k7/8/3p4/8/3P4/8/8/7K b - - 0 1",
            "7k/3p4/8/8/3P4/8/8/K7 w - - 0 1",
            "7k/8/8/3p4/8/8/3P4/K7 w - - 0 1",
            "k7/8/8/7p/6P1/8/8/K7 w - - 0 1",
            "k7/8/7p/8/8/6P1/8/K7 w - - 0 1",
            "k7/8/8/6p1/7P/8/8/K7 w - - 0 1",
            "k7/8/6p1/8/8/7P/8/K7 w - - 0 1",
            "k7/8/8/3p4/4p3/8/8/7K w - - 0 1",
            "k7/8/3p4/8/8/4P3/8/7K w - - 0 1",
            "7k/3p4/8/8/3P4/8/8/K7 b - - 0 1",
            "7k/8/8/3p4/8/8/3P4/K7 b - - 0 1",
            "k7/8/8/7p/6P1/8/8/K7 b - - 0 1",
            "k7/8/7p/8/8/6P1/8/K7 b - - 0 1",
            "k7/8/8/6p1/7P/8/8/K7 b - - 0 1",
            "k7/8/6p1/8/8/7P/8/K7 b - - 0 1",
            "k7/8/8/3p4/4p3/8/8/7K b - - 0 1",
            "k7/8/3p4/8/8/4P3/8/7K b - - 0 1",
            "7k/8/8/p7/1P6/8/8/7K w - - 0 1",
            "7k/8/p7/8/8/1P6/8/7K w - - 0 1",
            "7k/8/8/1p6/P7/8/8/7K w - - 0 1",
            "7k/8/1p6/8/8/P7/8/7K w - - 0 1",
            "k7/7p/8/8/8/8/6P1/K7 w - - 0 1",
            "k7/6p1/8/8/8/8/7P/K7 w - - 0 1",
            "3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1",
            "7k/8/8/p7/1P6/8/8/7K b - - 0 1",
            "7k/8/p7/8/8/1P6/8/7K b - - 0 1",
            "7k/8/8/1p6/P7/8/8/7K b - - 0 1",
            "7k/8/1p6/8/8/P7/8/7K b - - 0 1",
            "k7/7p/8/8/8/8/6P1/K7 b - - 0 1",
            "k7/6p1/8/8/8/8/7P/K7 b - - 0 1",
            "3k4/3pp3/8/8/8/8/3PP3/3K4 b - - 0 1",
            "8/Pk6/8/8/8/8/6Kp/8 w - - 0 1",
            "n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1",
            "8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1",
            "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1",
            "8/Pk6/8/8/8/8/6Kp/8 b - - 0 1",
            "n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1",
            "8/PPPk4/8/8/8/8/4Kppp/8 b - - 0 1",
            "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "rnbqkb1r/ppppp1pp/7n/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        ];

        for fen in positions {
            let board = Board::from_fen(fen).unwrap();
            let flipped = board.flip();
            let flipped_back = flipped.flip();
            assert_eq!(
                board.to_fen(),
                flipped_back.to_fen(),
                "Failed on FEN: {fen}"
            );
        }
    }
}
