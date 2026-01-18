// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{
    attacks,
    bitboard::Bitboard,
    bitboard_helpers,
    board::Board,
    definitions::{FILE_BITBOARDS, RANK_BITBOARDS, Squares},
    file::File,
    move_list::MoveList,
    moves::{Move, MoveDescriptor, MoveType, PromotionDescriptor},
    non_slider_piece::NonSliderPiece,
    piece_category::PieceCategory,
    pieces::Piece,
    rank::Rank,
    rays,
    side::Side,
    sliding_piece_attacks::SlidingPieceAttacks,
    square::{self, Square},
};

pub(crate) const NORTH: u64 = 8;
pub(crate) const SOUTH: u64 = 8;

/// The MoveGenerator struct is responsible for generating moves for a given board state.
pub struct MoveGenerator {
    pub(crate) sliding_piece_attacks: SlidingPieceAttacks,
}

impl Default for MoveGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveGenerator {
    pub fn new() -> Self {
        Self {
            sliding_piece_attacks: SlidingPieceAttacks::new(),
        }
    }

    fn edges(file: u8, rank: u8) -> Bitboard {
        // need to get the rank and file of the square
        let file_bb = FILE_BITBOARDS[file as usize];
        let rank_bb = RANK_BITBOARDS[rank as usize];
        // get the edges of the board
        (FILE_BITBOARDS[File::A as usize] & !file_bb)
            | (FILE_BITBOARDS[File::H as usize] & !file_bb)
            | (RANK_BITBOARDS[Rank::R1 as usize] & !rank_bb)
            | (RANK_BITBOARDS[Rank::R8 as usize] & !rank_bb)
    }

    #[allow(dead_code)]
    pub(crate) fn edges_from_square(square: u8) -> Bitboard {
        let (file, rank) = square::from_square(square);
        MoveGenerator::edges(file, rank)
    }

    fn orthogonal_ray_attacks(square: u8, occupied: u64) -> Bitboard {
        let mut attacks = Bitboard::default();
        let bb = Bitboard::from_square(square);
        let not_a_file = !FILE_BITBOARDS[File::A as usize];
        let not_h_file = !FILE_BITBOARDS[File::H as usize];

        // North
        let mut ray = bb;
        while ray != 0 {
            ray <<= 8;
            attacks |= ray;
            if ray & occupied != 0 {
                break;
            }
        }

        // South
        let mut ray = bb;
        while ray != 0 {
            ray >>= 8;
            attacks |= ray;
            if ray & occupied != 0 {
                break;
            }
        }

        // East
        let mut ray = bb;
        while ray != 0 && ray & not_h_file != 0 {
            ray <<= 1;
            attacks |= ray;
            if ray & occupied != 0 {
                break;
            }
        }

        // West
        let mut ray = bb;
        while ray != 0 && ray & not_a_file != 0 {
            ray >>= 1;
            attacks |= ray;
            if ray & occupied != 0 {
                break;
            }
        }

        attacks
    }

    fn diagonal_ray_attacks(square: u8, occupied: u64) -> Bitboard {
        let mut attacks = Bitboard::default();
        let bb = Bitboard::from_square(square);
        let not_a_file = !FILE_BITBOARDS[File::A as usize];
        let not_h_file = !FILE_BITBOARDS[File::H as usize];

        // Northeast
        let mut ray = bb;
        while ray != 0 && ray & not_h_file != 0 {
            ray <<= 9;
            attacks |= ray;
            if ray & occupied != 0 {
                break;
            }
        }

        // Northwest
        let mut ray = bb;
        while ray != 0 && ray & not_a_file != 0 {
            ray <<= 7;
            attacks |= ray;
            if ray & occupied != 0 {
                break;
            }
        }

        // Southeast
        let mut ray = bb;
        while ray != 0 && ray & not_h_file != 0 {
            ray >>= 7;
            attacks |= ray;
            if ray & occupied != 0 {
                break;
            }
        }

        // Southwest
        let mut ray = bb;
        while ray != 0 && ray & not_a_file != 0 {
            ray >>= 9;
            attacks |= ray;
            if ray & occupied != 0 {
                break;
            }
        }

        attacks
    }

    /// Calculate the "relevant" bits for rook attacks at a given square.
    ///
    /// The relevant bits are the squares that the rook can attack from a given square.
    /// The returned bitboard does not include edges.
    ///
    /// # Arguments
    ///
    /// - square - The square to calculate the relevant bits for.
    ///
    /// # Returns
    ///
    /// A bitboard representing the relevant bits for the rook attacks at the given square.
    pub fn relevant_rook_bits(square: u8) -> Bitboard {
        let mut bb = Bitboard::default();
        bb.set_square(square);

        // need to get the rank and file of the square
        let (file, rank) = square::from_square(square);
        let rook_rays_bb = MoveGenerator::orthogonal_ray_attacks(square, 0);
        // get the edges of the board
        let edges = MoveGenerator::edges(file, rank);

        rook_rays_bb & !edges & !bb
    }

    /// Calculate the "relevant" bits for bishop attacks at a given square.
    ///
    /// The relevant bits are the squares that the bishop can attack from a given square.
    /// The returned bitboard does not include edges.
    ///
    /// # Arguments
    ///
    /// - square - The square to calculate the relevant bits for.
    ///
    /// # Returns
    ///
    /// A bitboard representing the relevant bits for the bishop attacks at the given square.
    pub fn relevant_bishop_bits(square: u8) -> Bitboard {
        let mut bb = Bitboard::default();
        bb.set_square(square);

        let (file, rank) = square::from_square(square);
        let edges = MoveGenerator::edges(file, rank);

        // need to calculate ray attacks for the bishop from its square
        let bishop_ray_attacks = MoveGenerator::diagonal_ray_attacks(square, 0);

        bishop_ray_attacks & !edges & !bb
    }

    /// Generate all possible blocker permutations for a given bitboard.
    ///
    /// # Arguments
    ///
    /// - bb - The bitboard to generate the blocker permutations for.
    ///
    /// # Returns
    ///
    /// A vector of bitboards representing all possible blocker permutations for the given bitboard.
    pub fn create_blocker_permutations(bb: Bitboard) -> Vec<Bitboard> {
        // use the carry-rippler method to cycle through all possible permutations of the given bitboard
        let mask = bb;
        let mut subset = Bitboard::default();

        const BASE: u64 = 2_u64;
        let total_permutations = BASE.pow(bb.as_number().count_ones());

        let mut blocker_bitboards = Vec::with_capacity(total_permutations as usize);
        loop {
            // there could be no blockers, so start with that...
            blocker_bitboards.push(subset);
            // carry-rippler method to generate all possible permutations of the given bitboard
            subset = Bitboard::new(subset.as_number().wrapping_sub(mask.as_number())) & mask;
            if subset == 0 {
                break;
            }
        }
        blocker_bitboards
    }

    /// Generate rook attacks for all possible blocker permutations at a given square.
    ///
    /// # Arguments
    /// - square - The square to generate the attacks for
    /// - blockers - The list of blocker permutations
    ///
    /// # Returns
    ///
    /// A vector of bitboards representing the rook attacks for each blocker permutation.
    pub fn rook_attacks(square: u8, blockers: &Vec<Bitboard>) -> Vec<Bitboard> {
        let mut attacks = Vec::with_capacity(blockers.len());
        for blocker in blockers {
            attacks.push(MoveGenerator::calculate_rook_attack(square, blocker));
        }
        attacks
    }

    /// Calculates rook attacks from a given square with a given blocker bitboard.
    ///
    /// # Arguments
    ///
    /// - square - The square to calculate the attacks from
    /// - blocker - The blocker bitboard
    ///
    /// # Returns
    ///
    /// A bitboard representing the rook attacks from the given square with the given blocker bitboard.
    pub fn calculate_rook_attack(square: u8, blocker: &Bitboard) -> Bitboard {
        // calculate ray attacks for the rook from its square
        MoveGenerator::orthogonal_ray_attacks(square, blocker.as_number())
    }

    /// Calculates bishop attacks from a given square with a given blocker bitboard.
    ///
    /// # Arguments
    ///
    /// - square - The square to calculate the attacks from
    /// - blocker - The blocker bitboard
    ///
    /// # Returns
    ///
    /// A vector of bitboards representing the bishop attacks from the given square with the given blocker bitboard.
    pub fn bishop_attacks(square: u8, blockers: &Vec<Bitboard>) -> Vec<Bitboard> {
        let mut attacks = Vec::with_capacity(blockers.len());
        for blocker in blockers {
            attacks.push(MoveGenerator::calculate_bishop_attack(square, blocker));
        }
        attacks
    }

    /// Calculates bishop attacks from a given square with a given blocker bitboard.
    ///
    /// # Arguments
    ///
    /// - square - The square to calculate the attacks from
    /// - blocker - The blocker bitboard
    ///
    /// # Returns
    ///
    /// A bitboard representing the bishop attacks from the given square with the given blocker bitboard.
    pub fn calculate_bishop_attack(square: u8, blocker: &Bitboard) -> Bitboard {
        MoveGenerator::diagonal_ray_attacks(square, blocker.as_number())
    }

    /// Calculate the ray between two squares. This is simply a look up as we pre-calculate all rays between all squares.
    pub(crate) fn ray_between(&self, from: Square, to: Square) -> Bitboard {
        rays::between(from.to_square_index(), to.to_square_index())
    }

    /// Calculate all squares currently being attacked by a given side.
    ///
    /// # Arguments
    /// - board - The current board state
    /// - side - The side to calculate the attacked squares for
    ///
    /// # Returns
    ///
    /// A bitboard representing all squares currently being attacked by the given side.
    pub(crate) fn get_attacked_squares(
        &self,
        board: &Board,
        side: Side,
        occupancy: &Bitboard,
    ) -> Bitboard {
        let mut attacks = Bitboard::default();

        // get the squares attacked by each piece
        for piece in [
            Piece::Bishop,
            Piece::Rook,
            Piece::Queen,
            Piece::King,
            Piece::Knight,
            Piece::Pawn,
        ]
        .iter()
        {
            let mut piece_bb = *board.piece_bitboard(*piece, side);
            if piece_bb.as_number() == 0 {
                continue;
            }

            while piece_bb.as_number() > 0 {
                let from = bitboard_helpers::next_bit(&mut piece_bb) as u8;
                let attacks_bb = match PieceCategory::from(*piece) {
                    PieceCategory::NonSlider(non_slider) => {
                        self.get_non_slider_attacks(side, non_slider, from)
                    }
                    PieceCategory::Slider(slider) => self
                        .sliding_piece_attacks
                        .get_slider_attack(slider, from, occupancy),
                };

                attacks |= attacks_bb;
            }
        }

        attacks
    }

    /// Get attacks for a given piece.
    ///
    /// # Arguments
    ///
    /// - piece - The piece to get the attacks for
    /// - square - The square the piece is on
    /// - attacking_side - The side that is attacking
    /// - occupancy - The current occupancy of the board
    pub(crate) fn get_piece_attacks(
        &self,
        piece: Piece,
        square: u8,
        attacking_side: Side,
        occupancy: &Bitboard,
    ) -> Bitboard {
        match PieceCategory::from(piece) {
            PieceCategory::Slider(slider) => self
                .sliding_piece_attacks
                .get_slider_attack(slider, square, occupancy),
            PieceCategory::NonSlider(non_slider) => {
                self.get_non_slider_attacks(Side::opposite(attacking_side), non_slider, square)
            }
        }
    }

    /// Generates pseudo-legal moves for the current board state.
    /// This function does not check for legality of the moves.
    ///
    /// # Arguments
    /// - board - The current board state
    /// - move_list - The list of moves to append to.
    /// - move_type - The type of moves to generate
    pub fn generate_moves(&self, board: &Board, move_list: &mut MoveList, move_type: MoveType) {
        // get moves for each piece except pawns
        for piece in [
            Piece::King,
            Piece::Knight,
            Piece::Rook,
            Piece::Bishop,
            Piece::Queen,
        ] {
            self.get_piece_moves(piece, board, move_list, &move_type);
        }
        // handle pawn moves separately
        self.get_pawn_moves(board, move_list, &move_type);

        if move_type == MoveType::All || move_type == MoveType::Quiet {
            // handle castling moves
            self.get_castling_moves(board, move_list);
        }
    }

    fn get_castling_moves(&self, board: &Board, move_list: &mut MoveList) {
        /*
         * For castling, the king and rook must not have moved.
         * The squares between the king and rook must be empty.
         * The squares the king moves through must not be under attack (including start and end).
         * The king must not be in check.
         * The king must not move through check.
         * The king must not end up in check.
         *
         * FIDE Laws of Chess:
         * 3.8.2.1 The right to castle has been lost:
         *     3.8.2.1.1 if the king has already moved, or
         *     3.8.2.1.2 with a rook that has already moved.
         *
         * 3.8.2.2 Castling is prevented temporarily:
         *     3.8.2.2.1 if the square on which the king stands, or the square which it must cross, or the square which it is to occupy, is attacked by one or more of the opponent's pieces, or
         *     3.8.2.2.2 if there is any piece between the king and the rook with which castling is to be effected.
         */

        let occupancy = board.all_pieces();

        // white king side castling
        if board.can_castle_kingside(Side::White) && board.side_to_move() == Side::White {
            let king_from = Square::from_square_index(Squares::E1); // e1
            let king_to = Square::from_square_index(Squares::G1); // g1
            let blockers = Bitboard::from_square(Squares::F1) | Bitboard::from_square(Squares::G1);
            let king_ray = [Squares::E1, Squares::F1, Squares::G1];

            let is_blocked = (blockers & occupancy) > 0;
            let are_any_attacked = king_ray.iter().any(|&square| {
                self.is_square_attacked(board, &Square::from_square_index(square), Side::Black)
            });

            if !is_blocked
                && !are_any_attacked
                && !self.is_square_attacked(board, &king_from, Side::Black)
                && !self.is_square_attacked(board, &king_to, Side::Black)
            {
                move_list.push(Move::new_castle(&king_from, &king_to));
            }
        }

        if board.can_castle_queenside(Side::White) && board.side_to_move() == Side::White {
            let king_from = Square::from_square_index(Squares::E1);
            let king_to = Square::from_square_index(Squares::C1);
            let blockers = Bitboard::from_square(Squares::D1)
                | Bitboard::from_square(Squares::C1)
                | Bitboard::from_square(Squares::B1);
            let king_ray = [Squares::E1, Squares::D1, Squares::C1];

            let is_blocked = (blockers & occupancy) > 0;
            let are_any_attacked = king_ray.iter().any(|&square| {
                self.is_square_attacked(board, &Square::from_square_index(square), Side::Black)
            });

            if !is_blocked
                && !are_any_attacked
                && !self.is_square_attacked(board, &king_from, Side::Black)
                && !self.is_square_attacked(board, &king_to, Side::Black)
            {
                move_list.push(Move::new_castle(&king_from, &king_to));
            }
        }

        if board.can_castle_kingside(Side::Black) && board.side_to_move() == Side::Black {
            let king_from = Square::from_square_index(Squares::E8);
            let king_to = Square::from_square_index(Squares::G8);
            let blockers = Bitboard::from_square(Squares::F8) | Bitboard::from_square(Squares::G8);
            let king_ray = [Squares::E8, Squares::F8, Squares::G8];
            let is_blocked = (blockers & occupancy) > 0;
            let are_any_attacked = king_ray.iter().any(|&square| {
                self.is_square_attacked(board, &Square::from_square_index(square), Side::White)
            });

            if !is_blocked
                && !are_any_attacked
                && !self.is_square_attacked(board, &king_from, Side::White)
                && !self.is_square_attacked(board, &king_to, Side::White)
            {
                move_list.push(Move::new_castle(&king_from, &king_to));
            }
        }

        if board.can_castle_queenside(Side::Black) && board.side_to_move() == Side::Black {
            let king_from = Square::from_square_index(Squares::E8);
            let king_to = Square::from_square_index(Squares::C8);
            let blockers = Bitboard::from_square(Squares::D8)
                | Bitboard::from_square(Squares::C8)
                | Bitboard::from_square(Squares::B8);
            let king_ray = [Squares::E8, Squares::D8, Squares::C8];
            let is_blocked = (blockers & occupancy) > 0;
            let are_any_attacked = king_ray.iter().any(|&square| {
                self.is_square_attacked(board, &Square::from_square_index(square), Side::White)
            });

            if !is_blocked
                && !are_any_attacked
                && !self.is_square_attacked(board, &king_from, Side::White)
                && !self.is_square_attacked(board, &king_to, Side::White)
            {
                move_list.push(Move::new_castle(&king_from, &king_to));
            }
        }
    }

    fn get_piece_moves(
        &self,
        piece: Piece,
        board: &Board,
        move_list: &mut MoveList,
        move_type: &MoveType,
    ) {
        debug_assert!(
            piece != Piece::Pawn,
            "Pawn move enumeration is handle separately."
        );

        let us = board.side_to_move();
        let them = Side::opposite(us);
        let our_pieces = board.pieces(us);
        let their_pieces = board.pieces(them);
        let occupancy = board.all_pieces();
        let empty = !occupancy;

        let mut piece_bb = *board.piece_bitboard(piece, us);
        // loop through all the pieces of the given type
        while piece_bb.as_number() > 0 {
            let from_square = bitboard_helpers::next_bit(&mut piece_bb) as u8;
            let attack_bb = match PieceCategory::from(piece) {
                PieceCategory::Slider(slider) => {
                    self.sliding_piece_attacks
                        .get_slider_attack(slider, from_square, &occupancy)
                }
                PieceCategory::NonSlider(non_slider) => {
                    self.get_non_slider_attacks(us, non_slider, from_square)
                }
            };

            let bb_moves = match move_type {
                MoveType::Capture => attack_bb & their_pieces,
                MoveType::Quiet => attack_bb & empty,
                MoveType::All => attack_bb & !our_pieces,
            };

            self.enumerate_moves(
                &bb_moves,
                &Square::from_square_index(from_square),
                piece,
                board,
                move_list,
            );
        }
    }

    fn get_non_slider_attacks(
        &self,
        attacking_side: Side,
        piece: NonSliderPiece,
        from_square: u8,
    ) -> Bitboard {
        match piece {
            NonSliderPiece::King => attacks::king(from_square),
            NonSliderPiece::Knight => attacks::knight(from_square),
            NonSliderPiece::Pawn => attacks::pawn(from_square, attacking_side),
        }
    }

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn get_pawn_moves(&self, board: &Board, move_list: &mut MoveList, move_type: &MoveType) {
        let us = board.side_to_move();
        let them = Side::opposite(us);
        let their_pieces = board.pieces(them);
        let occupancy = board.all_pieces();
        let empty = !occupancy;
        let direction = if us == Side::White { NORTH } else { SOUTH };
        let pawns_bb = board.piece_bitboard(Piece::Pawn, us);

        let mut bb = *pawns_bb;

        // loop through all the pawns for us
        while bb > 0 {
            let from_square = bitboard_helpers::next_bit(&mut bb) as u8;
            let attack_bb = attacks::pawn(from_square, us);

            let mut bb_moves = Bitboard::default();
            let to_square = match us {
                Side::White => from_square as u64 + direction,
                Side::Black => from_square as u64 - direction,
            };

            // pawn non-capture moves
            if *move_type == MoveType::All || *move_type == MoveType::Quiet {
                let bb_push = Bitboard::new(1u64 << to_square);
                let bb_single_push = bb_push & empty;
                let can_double_push = match us {
                    Side::White => square::is_square_on_rank(from_square, Rank::R2 as u8),
                    Side::Black => square::is_square_on_rank(from_square, Rank::R7 as u8),
                };

                let double_push_square = if can_double_push {
                    match us {
                        Side::White => {
                            let (value, did_overflow) = to_square.overflowing_add(direction);
                            if did_overflow { None } else { Some(value) }
                        }
                        Side::Black => {
                            let (value, did_overflow) = to_square.overflowing_sub(direction);
                            if did_overflow { None } else { Some(value) }
                        }
                    }
                } else {
                    None
                };

                // note that the single push square has to be empty in addition to the double push square being empty
                let is_double_push_unobstructed = if let Some(push_square) = double_push_square {
                    !occupancy.is_square_occupied(to_square as u8)
                        && !occupancy.is_square_occupied(push_square as u8)
                } else {
                    false
                };

                let bb_double_push = if can_double_push && is_double_push_unobstructed {
                    Bitboard::new(1u64 << double_push_square.unwrap()) & empty
                } else {
                    Bitboard::default()
                };
                bb_moves |= bb_single_push | bb_double_push;
            }

            // pawn captures
            if move_type == &MoveType::All || move_type == &MoveType::Capture {
                let bb_capture = attack_bb & their_pieces;
                // en passant
                let bb_en_passant = match board.en_passant_square() {
                    Some(en_passant_square) => {
                        // we only want to add the en passant square if it is within range of the pawn
                        // this means that the en passant square is within 1 rank of the pawn and the en passant square
                        // is in the pawn's attack table
                        let en_passant_bb = Bitboard::from_square(en_passant_square);
                        let result = en_passant_bb & !(attack_bb);
                        let is_in_range = result == 0;
                        if is_in_range {
                            en_passant_bb
                        } else {
                            Bitboard::default()
                        }
                    }
                    None => Bitboard::default(),
                };
                bb_moves |= bb_capture | bb_en_passant;
            }

            self.enumerate_moves(
                &bb_moves,
                &Square::from_square_index(from_square),
                Piece::Pawn,
                board,
                move_list,
            );
        }
    }

    /// Enumerate all moves in a given bitboard and add them to the given [`MoveList`]
    ///
    /// # Arguments
    ///
    /// - bitboard - The bitboard to enumerate moves for
    /// - from - The square the piece is moving from
    /// - piece - The piece that is moving
    /// - board - The current board state
    /// - move_list - The list of moves to append to
    #[allow(clippy::panic)]
    pub(crate) fn enumerate_moves(
        &self,
        bitboard: &Bitboard,
        from: &Square,
        piece: Piece,
        board: &Board,
        move_list: &mut MoveList,
    ) {
        if bitboard.as_number() == 0 {
            return;
        }

        let mut bb = *bitboard;
        let us = board.side_to_move();
        let them = Side::opposite(us);
        let enemy_pieces = board.pieces(them);
        let promotion_rank = Rank::promotion_rank(us);
        while bb > 0 {
            let to_square = bitboard_helpers::next_bit(&mut bb) as u8;
            let (file, rank) = square::from_square(to_square as u8);
            let (from_file, _) = square::from_square(from.to_square_index());

            let en_passant = match board.en_passant_square() {
                Some(en_passant_square) => en_passant_square == to_square && piece == Piece::Pawn,
                None => false,
            };

            let is_capture: bool = enemy_pieces.is_square_occupied(to_square) || en_passant;
            // 2 rows = 16 squares
            let is_double_move = piece == Piece::Pawn
                && (to_square as i8 - from.to_square_index() as i8).abs() == 16;
            let is_promotion =
                piece == Piece::Pawn && square::is_square_on_rank(to_square, promotion_rank as u8);

            if is_double_move && en_passant {
                panic!("Double move and en passant should not happen");
            }

            // a castle is the only time a king can move 2 squares
            let is_castle = piece == Piece::King && from_file.abs_diff(file) == 2;

            let mut move_desc = MoveDescriptor::None;
            if is_double_move {
                move_desc = MoveDescriptor::PawnTwoUp;
            } else if en_passant {
                move_desc = MoveDescriptor::EnPassantCapture;
            } else if is_castle {
                move_desc = MoveDescriptor::Castle;
            }

            let capture_piece = if is_capture && !en_passant {
                Some(board.piece_on_square(to_square).unwrap().0)
            } else if en_passant {
                Some(Piece::Pawn)
            } else {
                None
            };

            let to_square = square::to_square_object(file, rank);
            if is_promotion {
                // we have to add 4 moves for each promotion type
                for promotion_type in [
                    PromotionDescriptor::Queen,
                    PromotionDescriptor::Rook,
                    PromotionDescriptor::Bishop,
                    PromotionDescriptor::Knight,
                ] {
                    let mv = Move::new(
                        from,
                        &to_square,
                        move_desc,
                        piece,
                        capture_piece,
                        Some(promotion_type.to_piece()),
                    );
                    move_list.push(mv);
                }
            } else if is_castle {
                let mv = Move::new_castle(from, &to_square);
                move_list.push(mv);
            } else {
                let mv = Move::new(from, &to_square, move_desc, piece, capture_piece, None);
                move_list.push(mv);
            }
        }
    }

    /// Returns true if the given square is attacked by any piece that is on the attacking_side.
    /// This method uses the so called "super-piece" method.
    /// See: https://talkchess.com/viewtopic.php?t=27152
    ///
    /// The gist is that we treat the attacking square as the from square and we project the attacks to the same sides pieces.
    /// If there are any collisions, then we know that a piece is attacking that square.
    ///
    /// # Arguments
    /// - board: the current board state
    /// - square: the square to check if it is attacked
    /// - attacking_side: the side that is potentially attacking the square
    ///
    /// # Returns
    /// - true if the square is attacked, false otherwise
    pub fn is_square_attacked_with_occupancy(
        &self,
        board: &Board,
        square: &Square,
        attacking_side: Side,
        occupancy: &Bitboard,
    ) -> bool {
        let king_bb = board.piece_bitboard(Piece::King, attacking_side);
        let knight_bb = board.piece_bitboard(Piece::Knight, attacking_side);
        let bishop_bb = board.piece_bitboard(Piece::Bishop, attacking_side);
        let rook_bb = board.piece_bitboard(Piece::Rook, attacking_side);
        let queen_bb = board.piece_bitboard(Piece::Queen, attacking_side);
        let pawn_bb: &Bitboard = board.piece_bitboard(Piece::Pawn, attacking_side);

        let king_attacks = self.get_piece_attacks(
            Piece::King,
            square.to_square_index(),
            attacking_side,
            occupancy,
        );
        let knight_attacks = self.get_piece_attacks(
            Piece::Knight,
            square.to_square_index(),
            attacking_side,
            occupancy,
        );
        let rook_attacks = self.get_piece_attacks(
            Piece::Rook,
            square.to_square_index(),
            attacking_side,
            occupancy,
        );
        let bishop_attacks = self.get_piece_attacks(
            Piece::Bishop,
            square.to_square_index(),
            attacking_side,
            occupancy,
        );
        let queen_attacks = rook_attacks | bishop_attacks;
        // note we use the opposite side for the pawn attacks
        let pawn_attacks = attacks::pawn(square.to_square_index(), Side::opposite(attacking_side));

        let is_king_attacker = (king_attacks & *king_bb) > 0;
        let is_knight_attacker = (knight_attacks & *knight_bb) > 0;
        let is_rook_attacker = (rook_attacks & *rook_bb) > 0;
        let is_bishop_attacker = (bishop_attacks & *bishop_bb) > 0;
        let is_queen_attacker = (queen_attacks & *queen_bb) > 0;
        let is_pawn_attacker = (pawn_attacks & *pawn_bb) > 0;

        is_king_attacker
            || is_knight_attacker
            || is_rook_attacker
            || is_bishop_attacker
            || is_queen_attacker
            || is_pawn_attacker
    }

    pub fn is_square_attacked(&self, board: &Board, square: &Square, attacking_side: Side) -> bool {
        self.is_square_attacked_with_occupancy(board, square, attacking_side, &board.all_pieces())
    }
}

#[cfg(test)]
mod tests {

    use crate::{board::Board, definitions::NumberOf, move_generation, slider_pieces::SliderPiece};

    use super::*;

    #[test]
    fn check_is_square_attacked() {
        let board = Board::default_board();
        let move_gen = MoveGenerator::new();
        // loop through all the occupied squares and check if they are attacked
        let mut occupancy = board.all_pieces();
        let mut sq = bitboard_helpers::next_bit(&mut occupancy);
        while sq > 0 {
            let square = Square::from_square_index(sq as u8);
            let is_attacked = move_gen.is_square_attacked(&board, &square, Side::White);
            assert!(!is_attacked);
            sq = bitboard_helpers::next_bit(&mut occupancy);
        }

        // now generate moves and check if the squares that pieces can move to are attacked
        let mut move_list = MoveList::new();
        move_gen.generate_moves(&board, &mut move_list, MoveType::All);
        let side_to_move = board.side_to_move();
        // we ignore pawn two up moves because they are not "attacks"
        for mv in move_list.iter().filter(|mv| !mv.is_pawn_two_up()) {
            let to = mv.to();
            let is_attacked =
                move_gen.is_square_attacked(&board, &Square::from_square_index(to), side_to_move);
            assert!(is_attacked, "Square {to} is not attacked by move\n\t{mv}",);
        }

        {
            let board = Board::from_fen("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2").unwrap();
            let mut king_bb = *board.piece_bitboard(Piece::King, board.side_to_move());
            let square = bitboard_helpers::next_bit(&mut king_bb) as u8;
            assert_eq!(board.side_to_move(), Side::Black);
            assert!(move_gen.is_square_attacked(
                &board,
                &Square::from_square_index(square),
                Side::opposite(board.side_to_move())
            ));
        }

        {
            let mut board =
                Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                    .unwrap();
            move_gen.generate_moves(&board, &mut move_list, MoveType::All);
            let mv = move_list
                .iter()
                .find(|mv| mv.to_long_algebraic() == "b1c3")
                .unwrap();
            assert!(board.make_move(mv, &move_gen).is_ok());

            // did we leave the king in check?
            let mut king_bb = *board.piece_bitboard(Piece::King, Side::White);
            let square = bitboard_helpers::next_bit(&mut king_bb) as u8;
            assert_eq!(board.side_to_move(), Side::Black);
            // there should be no attacks on the king
            assert!(!move_gen.is_square_attacked(
                &board,
                &Square::from_square_index(square),
                Side::Black
            ));
        }
    }

    #[test]
    fn check_rook_relevant_bits() {
        let rook_relevant_bit_expected: [u64; NumberOf::SQUARES] = [
            282578800148862,
            565157600297596,
            1130315200595066,
            2260630401190006,
            4521260802379886,
            9042521604759646,
            18085043209519166,
            36170086419038334,
            282578800180736,
            565157600328704,
            1130315200625152,
            2260630401218048,
            4521260802403840,
            9042521604775424,
            18085043209518592,
            36170086419037696,
            282578808340736,
            565157608292864,
            1130315208328192,
            2260630408398848,
            4521260808540160,
            9042521608822784,
            18085043209388032,
            36170086418907136,
            282580897300736,
            565159647117824,
            1130317180306432,
            2260632246683648,
            4521262379438080,
            9042522644946944,
            18085043175964672,
            36170086385483776,
            283115671060736,
            565681586307584,
            1130822006735872,
            2261102847592448,
            4521664529305600,
            9042787892731904,
            18085034619584512,
            36170077829103616,
            420017753620736,
            699298018886144,
            1260057572672512,
            2381576680245248,
            4624614895390720,
            9110691325681664,
            18082844186263552,
            36167887395782656,
            35466950888980736,
            34905104758997504,
            34344362452452352,
            33222877839362048,
            30979908613181440,
            26493970160820224,
            17522093256097792,
            35607136465616896,
            9079539427579068672,
            8935706818303361536,
            8792156787827803136,
            8505056726876686336,
            7930856604974452736,
            6782456361169985536,
            4485655873561051136,
            9115426935197958144,
        ];

        let mut offset_sum: u64 = 0;
        const BASE: u64 = 2_u64;
        for (square, value) in rook_relevant_bit_expected.into_iter().enumerate() {
            let rook_bits = move_generation::MoveGenerator::relevant_rook_bits(square as u8);
            assert_eq!(rook_bits.as_number(), value);

            offset_sum += BASE.pow(rook_bits.as_number().count_ones());
        }
        println!("rook offset sum: {offset_sum}");
    }

    #[test]
    fn check_relevant_bishop_bits() {
        let bishop_relevant_bit_expected: [u64; NumberOf::SQUARES] = [
            18049651735527936,
            70506452091904,
            275415828992,
            1075975168,
            38021120,
            8657588224,
            2216338399232,
            567382630219776,
            9024825867763712,
            18049651735527424,
            70506452221952,
            275449643008,
            9733406720,
            2216342585344,
            567382630203392,
            1134765260406784,
            4512412933816832,
            9024825867633664,
            18049651768822272,
            70515108615168,
            2491752130560,
            567383701868544,
            1134765256220672,
            2269530512441344,
            2256206450263040,
            4512412900526080,
            9024834391117824,
            18051867805491712,
            637888545440768,
            1135039602493440,
            2269529440784384,
            4539058881568768,
            1128098963916800,
            2256197927833600,
            4514594912477184,
            9592139778506752,
            19184279556981248,
            2339762086609920,
            4538784537380864,
            9077569074761728,
            562958610993152,
            1125917221986304,
            2814792987328512,
            5629586008178688,
            11259172008099840,
            22518341868716544,
            9007336962655232,
            18014673925310464,
            2216338399232,
            4432676798464,
            11064376819712,
            22137335185408,
            44272556441600,
            87995357200384,
            35253226045952,
            70506452091904,
            567382630219776,
            1134765260406784,
            2832480465846272,
            5667157807464448,
            11333774449049600,
            22526811443298304,
            9024825867763712,
            18049651735527936,
        ];

        let mut offset_sum: u64 = 0;
        const BASE: u64 = 2_u64;

        for (square, value) in bishop_relevant_bit_expected.into_iter().enumerate() {
            let bishop_bits = move_generation::MoveGenerator::relevant_bishop_bits(square as u8);
            assert_eq!(bishop_bits.as_number(), value);

            offset_sum += BASE.pow(bishop_bits.as_number().count_ones());
        }

        println!("bishop offset sum: {offset_sum}");
    }

    #[test]
    fn check_rook_attacks() {
        let move_gen = MoveGenerator::new();
        let occupancy = Bitboard::default();
        const EXPECTED_ATTACKS: [u64; NumberOf::SQUARES] = [
            0x1010101010101fe,
            0x2020202020202fd,
            0x4040404040404fb,
            0x8080808080808f7,
            0x10101010101010ef,
            0x20202020202020df,
            0x40404040404040bf,
            0x808080808080807f,
            0x10101010101fe01,
            0x20202020202fd02,
            0x40404040404fb04,
            0x80808080808f708,
            0x101010101010ef10,
            0x202020202020df20,
            0x404040404040bf40,
            0x8080808080807f80,
            0x101010101fe0101,
            0x202020202fd0202,
            0x404040404fb0404,
            0x808080808f70808,
            0x1010101010ef1010,
            0x2020202020df2020,
            0x4040404040bf4040,
            0x80808080807f8080,
            0x1010101fe010101,
            0x2020202fd020202,
            0x4040404fb040404,
            0x8080808f7080808,
            0x10101010ef101010,
            0x20202020df202020,
            0x40404040bf404040,
            0x808080807f808080,
            0x10101fe01010101,
            0x20202fd02020202,
            0x40404fb04040404,
            0x80808f708080808,
            0x101010ef10101010,
            0x202020df20202020,
            0x404040bf40404040,
            0x8080807f80808080,
            0x101fe0101010101,
            0x202fd0202020202,
            0x404fb0404040404,
            0x808f70808080808,
            0x1010ef1010101010,
            0x2020df2020202020,
            0x4040bf4040404040,
            0x80807f8080808080,
            0x1fe010101010101,
            0x2fd020202020202,
            0x4fb040404040404,
            0x8f7080808080808,
            0x10ef101010101010,
            0x20df202020202020,
            0x40bf404040404040,
            0x807f808080808080,
            0xfe01010101010101,
            0xfd02020202020202,
            0xfb04040404040404,
            0xf708080808080808,
            0xef10101010101010,
            0xdf20202020202020,
            0xbf40404040404040,
            0x7f80808080808080,
        ];

        for (sq, expected) in EXPECTED_ATTACKS.iter().enumerate() {
            let rook_attack_bb = move_gen.sliding_piece_attacks.get_slider_attack(
                SliderPiece::Rook,
                sq as u8,
                &occupancy,
            );
            // println!("{:#x},", rook_attack_bb.as_number())
            assert_eq!(rook_attack_bb.as_number(), *expected);
        }
    }

    #[test]
    fn check_blocker_permutations() {
        const BASE: u64 = 2_u64;

        for sq in 0..NumberOf::SQUARES {
            let rook_bb = MoveGenerator::relevant_rook_bits(sq as u8);
            let permutations = MoveGenerator::create_blocker_permutations(rook_bb);
            let total_permutations = BASE.pow(rook_bb.as_number().count_ones());
            assert_eq!(permutations.len(), total_permutations as usize);
            for bb in permutations {
                // check that the permutation is a subset of the rook bitboard
                if (bb) != 0 {
                    assert_eq!(bb & !rook_bb, 0);
                }
            }
        }
    }

    #[test]
    fn check_basic_move_gen() {
        let board = Board::default_board();
        let mut move_list = MoveList::new();
        let move_gen = MoveGenerator::new();
        move_gen.generate_moves(&board, &mut move_list, MoveType::All);

        for mv in move_list.iter() {
            println!("{mv}");
            assert!(!mv.is_castle());
            assert!(!mv.is_en_passant_capture());
            assert!(!mv.is_promotion());
        }

        assert_eq!(move_list.len(), 20);

        move_list.clear();
        move_gen.generate_legal_moves(&board, &mut move_list);

        for mv in move_list.iter() {
            println!("{mv}");
        }
        assert_eq!(move_list.len(), 20);
    }

    #[test]
    fn check_en_passant_capture_move_gen() {
        let board = Board::from_fen("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3").unwrap();
        assert!(board.en_passant_square().is_some());

        assert_eq!(board.side_to_move(), Side::Black);
        let mut move_list = MoveList::new();
        let move_gen = MoveGenerator::new();
        move_gen.generate_moves(&board, &mut move_list, MoveType::All);
        let en_passant_move = move_list.iter().find(|mv| mv.is_en_passant_capture());
        assert!(en_passant_move.is_some());
        assert!(move_list.len() >= 8);
    }
}
