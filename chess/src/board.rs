// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::fmt::Display;
use std::iter::zip;

use crate::bitboard_helpers;
use crate::board_state::BoardState;
use crate::definitions::{CastlingAvailability, MAX_MOVE_RULE, MAX_REPETITION_COUNT, SPACE};
use crate::fen::FenError;
use crate::file::File;
use crate::move_generation::MoveGenerator;
use crate::move_history::BoardHistory;
use crate::move_list::MoveList;
use crate::moves::Move;
use crate::rank::Rank;
use crate::square::Square;
use crate::zobrist::{ZobristHash, ZobristRandomValues};

use super::definitions::NumberOf;
use super::fen;
use super::side::Side;
use super::{bitboard::Bitboard, pieces::Piece};

/// Represents a chessboard position.
#[derive(Debug)]
pub struct Board {
    piece_bitboards: [[Bitboard; NumberOf::PIECE_TYPES]; NumberOf::SIDES],
    pub(crate) history: BoardHistory,
    state: BoardState,
    zobrist_values: ZobristRandomValues,
}

impl Clone for Board {
    fn clone(&self) -> Self {
        Self {
            piece_bitboards: self.piece_bitboards,
            history: self.history.clone(),
            state: self.state,
            zobrist_values: self.zobrist_values.clone(),
        }
    }
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
                if let Some((piece, side)) = self.piece_on_square(square.to_square_index()) {
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
            piece_bitboards: [[Bitboard::default(); NumberOf::PIECE_TYPES]; NumberOf::SIDES],
            history: BoardHistory::new(),
            state: BoardState::new(),
            zobrist_values: ZobristRandomValues::new(),
        }
    }

    pub(crate) fn initialize(&mut self) {
        self.state.zobrist_hash = self.initialize_zobrist_hash();
    }

    fn initialize_zobrist_hash(&self) -> ZobristHash {
        // create the initial zobrist hash based on the starting position
        // for each piece on the board, get the corresponding zobrist value and xor it with the hash
        // for each side to move, xor the hash with the zobrist value for the side
        // for each castling right, xor the hash with the zobrist value for the castling right
        // for the en passant square, xor the hash with the zobrist value for the en passant square
        // Initialize the zobrist hash to 0
        let mut zobrist_hash = ZobristHash::default();

        // XOR the zobrist values for each piece on the board
        for side in 0..NumberOf::SIDES {
            for piece in 0..NumberOf::PIECE_TYPES {
                let mut bitboard = self.piece_bitboards[side][piece];

                while bitboard != 0 {
                    let square = bitboard_helpers::next_bit(&mut bitboard);
                    zobrist_hash ^= self.zobrist_values.get_piece_value(piece, side, square);
                }
            }
        }

        // XOR the zobrist value for the side to move
        zobrist_hash ^= self
            .zobrist_values
            .get_side_value(self.side_to_move() as usize);

        // XOR the zobrist values for castling rights
        zobrist_hash ^= self
            .zobrist_values
            .get_castling_value(self.castling_rights() as usize);

        // XOR the zobrist value for the en passant square, if any
        zobrist_hash ^= self
            .zobrist_values
            .get_en_passant_value(self.state.en_passant_square);

        zobrist_hash
    }

    /// Initialize bitboards for a given side
    fn initialize_piece_bbs(&mut self, side: Side) {
        // Set up the board with the starting position
        match side {
            Side::White => self.initialize_white_bbs(),
            Side::Black => self.initialize_black_bbs(),
        }
    }

    /// Initialize bitboard for all white pieces
    fn initialize_white_bbs(&mut self) {
        let index = Side::White as usize;
        // Set up the board with the starting position
        self.piece_bitboards[index][Piece::Pawn as usize] = Bitboard::new(0xFF00);
        self.piece_bitboards[index][Piece::Knight as usize] = Bitboard::new(0x42);
        self.piece_bitboards[index][Piece::Bishop as usize] = Bitboard::new(0x24);
        self.piece_bitboards[index][Piece::Rook as usize] = Bitboard::new(0x81);
        self.piece_bitboards[index][Piece::Queen as usize] = Bitboard::new(0x8);
        self.piece_bitboards[index][Piece::King as usize] = Bitboard::new(0x10);
    }

    /// Initialize bitboard for all black pieces
    fn initialize_black_bbs(&mut self) {
        let index = Side::Black as usize;
        // Set up the board with the starting position
        self.piece_bitboards[index][Piece::Pawn as usize] = Bitboard::new(0xFF000000000000);
        self.piece_bitboards[index][Piece::Knight as usize] = Bitboard::new(0x4200000000000000);
        self.piece_bitboards[index][Piece::Bishop as usize] = Bitboard::new(0x2400000000000000);
        self.piece_bitboards[index][Piece::Rook as usize] = Bitboard::new(0x8100000000000000);
        self.piece_bitboards[index][Piece::Queen as usize] = Bitboard::new(0x800000000000000);
        self.piece_bitboards[index][Piece::King as usize] = Bitboard::new(0x1000000000000000);
    }

    pub(crate) fn mut_piece_bitboard(&mut self, piece: Piece, side: Side) -> &mut Bitboard {
        &mut self.piece_bitboards[side as usize][piece as usize]
    }

    pub(crate) fn set_piece_square(&mut self, piece: usize, side: usize, square: u8) {
        self.piece_bitboards[side][piece].set_square(square);
    }

    /// Sets the side to move and updates the zobrist hash.
    pub(crate) fn set_side_to_move(&mut self, side: Side) {
        // undo the current side to move in the hash
        self.state.zobrist_hash ^= self
            .zobrist_values
            .get_side_value(self.state.side_to_move as usize);
        // set the new side to move
        self.state.side_to_move = side;
        // update zobrist hash with the new side to move
        self.state.zobrist_hash ^= self
            .zobrist_values
            .get_side_value(self.state.side_to_move as usize);
    }

    /// Set the en passant square and update the zobrist hash.
    pub(crate) fn set_en_passant_square(&mut self, square: Option<u8>) {
        self.state.zobrist_hash ^= self
            .zobrist_values
            .get_en_passant_value(self.state.en_passant_square);
        self.state.en_passant_square = square;
        self.state.zobrist_hash ^= self
            .zobrist_values
            .get_en_passant_value(self.state.en_passant_square);
    }

    pub(crate) fn set_half_move_clock(&mut self, half_move_clock: u32) {
        self.state.half_move_clock = half_move_clock;
    }

    pub(crate) fn set_full_move_number(&mut self, full_move_number: u32) {
        self.state.full_move_number = full_move_number;
    }

    pub(crate) fn set_castling_rights(&mut self, castling_rights: u8) {
        self.state.zobrist_hash ^= self
            .zobrist_values
            .get_castling_value(self.state.castling_rights as usize);
        self.state.castling_rights = castling_rights;
        self.state.zobrist_hash ^= self
            .zobrist_values
            .get_castling_value(self.state.castling_rights as usize);
    }

    pub(crate) fn update_zobrist_hash_for_piece(&mut self, square: u8, piece: Piece, side: Side) {
        self.state.zobrist_hash ^=
            self.zobrist_values
                .get_piece_value(piece as usize, side as usize, square as usize);
    }

    fn set_zobrist_hash(&mut self, hash: u64) {
        self.state.zobrist_hash = hash;
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
        // White pieces
        board.initialize_piece_bbs(Side::White);
        // Black pieces
        board.initialize_piece_bbs(Side::Black);
        board.set_en_passant_square(None);
        board.set_half_move_clock(0);
        board.set_full_move_number(1);
        board.set_side_to_move(Side::White);
        board.set_castling_rights(CastlingAvailability::ALL);
        board.set_zobrist_hash(board.initialize_zobrist_hash());
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
        let mut all_pieces = Bitboard::default();
        for piece_type in 0..NumberOf::PIECE_TYPES {
            for side in 0..NumberOf::SIDES {
                all_pieces |= self.piece_bitboards[side][piece_type];
            }
        }
        all_pieces
    }

    /// Returns all the pieces of a given side in a single [`Bitboard`].
    pub fn pieces(&self, side: Side) -> Bitboard {
        let mut pieces = Bitboard::default();
        for piece_type in 0..NumberOf::PIECE_TYPES {
            pieces |= self.piece_bitboards[side as usize][piece_type];
        }
        pieces
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
    pub fn piece_bitboard(&self, piece: Piece, side: Side) -> &Bitboard {
        &self.piece_bitboards[side as usize][piece as usize]
    }

    /// Returns a combined [`Bitboard`] of all pieces of a given type for both sides.
    pub fn piece_kind_bitboard(&self, piece: Piece) -> Bitboard {
        let mut piece_bb = Bitboard::default();
        for side in 0..NumberOf::SIDES {
            piece_bb |= self.piece_bitboards[side][piece as usize];
        }
        piece_bb
    }

    /// Returns the current square of the king for a given side.
    pub fn king_square(&self, side: Side) -> u8 {
        let king_bb = self.piece_bitboard(Piece::King, side);
        bitboard_helpers::next_bit(&mut king_bb.clone()) as u8
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
    pub fn piece_on_square(&self, square: u8) -> Option<(Piece, Side)> {
        for piece in 0..NumberOf::PIECE_TYPES {
            for side in 0..NumberOf::SIDES {
                if self.piece_bitboards[side][piece].is_square_occupied(square) {
                    return Some((
                        Piece::try_from(piece as u8).unwrap(),
                        Side::try_from(side as u8).unwrap(),
                    ));
                }
            }
        }
        None
    }

    /// Returns the side to move of this [`Board`].
    pub fn side_to_move(&self) -> Side {
        self.state.side_to_move
    }

    /// Returns the en passant square of this [`Board`] (if it exists)
    pub fn en_passant_square(&self) -> Option<u8> {
        self.state.en_passant_square
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
        self.state.zobrist_hash
    }

    /// Checks if a given square is empty.
    pub fn is_square_empty(&self, square: &Square) -> bool {
        !self
            .all_pieces()
            .is_square_occupied(square.to_square_index())
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

    /// Check if the side to move is in check.
    ///
    /// # Arguments
    ///
    /// - `move_gen` - The move generator to use for generating moves.
    ///
    /// # Returns
    ///
    /// - `true` if the side to move is in check, otherwise `false`.
    pub fn is_in_check(&self, move_gen: &MoveGenerator) -> bool {
        // pseudo legal check
        // check if we are in check
        // get the kings location and check if that square is attacked by the opponent
        let mut king_bb = *self.piece_bitboard(Piece::King, self.side_to_move());
        let king_square = bitboard_helpers::next_bit(&mut king_bb) as u8;
        move_gen.is_square_attacked(
            self,
            &Square::from_square_index(king_square),
            Side::opposite(self.side_to_move()),
        )
    }

    /// Check if the side to move is in checkmate.
    pub fn is_checkmate(&self, move_gen: &MoveGenerator) -> bool {
        // if the side to move is not in check, it's not checkmate
        if !self.is_in_check(move_gen) {
            return false;
        }

        let king_bb = self.piece_bitboard(Piece::King, self.side_to_move());
        let king_sq = bitboard_helpers::next_bit(&mut king_bb.clone());

        // check mate happens when we're in check and all the legal moves are illegal but we
        // don't want to try all the moves to check their legality
        // instead we can get king moves only and then check if the possible squares the king can move to are attacked
        let mut occupancy = self.all_pieces();
        let us = self.side_to_move();

        let king_attacks = move_gen.get_piece_attacks(Piece::King, king_sq as u8, us, &occupancy);
        let our_pieces = self.pieces(self.side_to_move());
        let mut king_attacks = king_attacks & !our_pieces;

        // modify occupancy to exclude the king square
        occupancy.clear_square(king_sq as u8);

        // check if the king can move to any of the squares it's attacking
        while king_attacks > 0 {
            let square = bitboard_helpers::next_bit(&mut king_attacks);
            if move_gen.is_square_attacked_with_occupancy(
                self,
                &Square::from_square_index(square as u8),
                Side::opposite(self.side_to_move()),
                &occupancy,
            ) {
                return true;
            }
        }
        false
    }

    /// Get the color of the piece on a given square.
    ///
    /// Returns `Some(Side)` if the square is occupied, otherwise `None`.
    pub fn color_on(&self, square: u8) -> Option<Side> {
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
        // if any side has a Queen, Rook or Pawn, there's sufficient material
        let queen_bbs = *self.piece_bitboard(Piece::Queen, Side::Black)
            | *self.piece_bitboard(Piece::Queen, Side::White);
        let rook_bbs = *self.piece_bitboard(Piece::Rook, Side::Black)
            | *self.piece_bitboard(Piece::Rook, Side::White);
        let pawn_bbs = *self.piece_bitboard(Piece::Pawn, Side::Black)
            | *self.piece_bitboard(Piece::Pawn, Side::White);

        if (queen_bbs | rook_bbs | pawn_bbs).number_of_occupied_squares() > 0 {
            return false;
        }

        // check bishops and knights
        let white_bishops = self.piece_bitboard(Piece::Bishop, Side::White);
        let black_bishops = self.piece_bitboard(Piece::Bishop, Side::Black);
        let white_knights = self.piece_bitboard(Piece::Knight, Side::White);
        let black_knights = self.piece_bitboard(Piece::Knight, Side::Black);

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
                Square::from_bitboard(white_bishops).color()
                    == Square::from_bitboard(black_bishops).color()
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
            if previous_state.zobrist_hash == self.zobrist_hash() {
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

    /// Check if a given move is legal. This function does not alter the current board state.
    /// Instead it makes a copy of the current state and tries to make the move. There is a performance
    /// penalty for this, so use this function sparingly.
    pub fn is_legal(&self, mv: &Move, move_gen: &MoveGenerator) -> bool {
        // check if a move is legal without altering the current board state
        let mut board_copy = self.clone();
        board_copy.make_move(mv, move_gen).is_ok()
    }

    /// Check if a list of moves are legal. This function does not alter the current board state.
    pub fn are_legal(&self, list: &MoveList, move_gen: &MoveGenerator) -> bool {
        // check if a list of moves are legal without altering the current board state
        let mut board_copy = self.clone();
        for mv in list.iter() {
            if board_copy.make_move(mv, move_gen).is_err() {
                return false;
            }
        }
        true
    }

    /// Get the last move made on the board.
    ///
    /// Returns `None` if there are no moves in the history.
    /// Otherwise, returns the last move made.
    pub fn last_move(&self) -> Option<Move> {
        self.history.iter().last().map(|m| m.next_move)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        definitions::{DEFAULT_FEN, Squares},
        file::File,
        move_generation::MoveGenerator,
        move_list::MoveList,
        moves::{MoveDescriptor, MoveType},
        rank::Rank,
        side::Side,
        square,
    };

    use super::*;
    #[test]
    fn threefold_repetition_detection() {
        let mut board = Board::from_fen("k7/8/KQ6/8/8/8/8/8 w - - 0 1").unwrap();

        let bk_square_1 = Square::from_square_index(Squares::A8);
        let bk_square_2 = Square::from_square_index(Squares::B8);

        let wq_square_1 = Square::from_square_index(Squares::B6);
        let wq_square_2 = Square::from_square_index(Squares::C5);

        let white_queen_move = Move::new(
            &wq_square_1,
            &wq_square_2,
            MoveDescriptor::None,
            Piece::Queen,
            None,
            None,
        );

        let while_queen_reverse_move = Move::new(
            &wq_square_2,
            &wq_square_1,
            MoveDescriptor::None,
            Piece::Queen,
            None,
            None,
        );

        let black_king_move = Move::new(
            &bk_square_1,
            &bk_square_2,
            MoveDescriptor::None,
            Piece::King,
            None,
            None,
        );

        let black_king_reverse_move = Move::new(
            &bk_square_2,
            &bk_square_1,
            MoveDescriptor::None,
            Piece::King,
            None,
            None,
        );

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
        let move_gen = MoveGenerator::new();
        {
            let board =
                Board::from_fen("r1b1k1nr/pppp1ppp/2n5/4P3/8/2Q2N2/P1P1PPPP/RNq1KB1R w KQkq - 1 9")
                    .unwrap();

            assert!(board.is_in_check(&move_gen));
            assert!(board.is_checkmate(&move_gen));
        }
        {
            let board =
                Board::from_fen("r1b3nr/5ppp/3pk2R/8/2Q5/4R1PB/2PPPP1P/RNB1K1NR b KQ - 0 1")
                    .unwrap();
            assert!(board.is_in_check(&move_gen));
            assert!(board.is_checkmate(&move_gen));
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
        let move_gen = MoveGenerator::new();
        let mut move_list = MoveList::new();
        let mut board = Board::from_fen(FEN).unwrap();
        let hash = board.zobrist_hash();

        move_gen.generate_moves(&board, &mut move_list, MoveType::All);

        for mv in move_list.iter() {
            let mv_ok = board.make_move(mv, &move_gen);
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
        // all of these squares should be empty.
        for rank in (Rank::R3 as u8)..=(Rank::R6 as u8) {
            for file in (File::A as u8)..=(File::H as u8) {
                let square = square::to_square_object(file, rank);
                assert!(board.is_square_empty(&square));
            }
        }
    }

    #[test]
    fn from_fen_round_trip() {
        // load Pohl.epd from data and go through each FEN. Load it into the board and then output the FEN to see if they match
        let path = format!(
            "{}/../{}/{}",
            env!("CARGO_MANIFEST_DIR"),
            "data",
            "Pohl.epd"
        );
        let lines = std::fs::read_to_string(path).unwrap();
        println!("Loaded {} FEN strings from Pohl.epd", lines.lines().count());
        for fen in lines.lines() {
            let board = Board::from_fen(fen).unwrap();
            assert_eq!(fen, board.to_fen());
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
    fn color_on() {
        let board = Board::default_board();
        for sq in 0..64 {
            let color = board.color_on(sq);
            if sq <= 15 {
                assert!(color.is_some_and(|c| c == Side::White));
            } else if sq >= 48 {
                assert!(color.is_some_and(|c| c == Side::Black));
            } else {
                assert!(color.is_none());
            }
        }
    }

    #[test]
    fn get_last_move() {
        let mut board = Board::default_board();
        let move_gen = MoveGenerator::new();
        let mut move_list = MoveList::new();
        move_gen.generate_moves(&board, &mut move_list, MoveType::All);

        let first_move = move_list.iter().next().unwrap();
        let mv_ok = board.make_move(first_move, &move_gen);
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

        assert_eq!(piece_kind_bb, *black_pawns_bb | *white_pawns_bb);
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
}
