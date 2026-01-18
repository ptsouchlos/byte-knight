// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::attacks;
use crate::definitions::RANK_BITBOARDS;
use crate::move_generation::NORTH;
use crate::move_generation::SOUTH;
use crate::move_list::MoveList;
use crate::square;
use crate::{
    bitboard::Bitboard, bitboard_helpers, board::Board, definitions::Squares,
    move_generation::MoveGenerator, pieces::Piece, rank::Rank, side::Side, square::Square,
};

impl MoveGenerator {
    /// Calculates checkers, pinned pieces, capture mask, push mask and pin rays for the current position.
    ///
    /// # Arguments
    ///
    /// - board - The current board state
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - A [`Bitboard`] representing the squares that are checking the king
    /// - A [`Bitboard`] representing the squares can be attacked
    /// - A [`Bitboard`] representing the squares that can be pushed to
    /// - A [`Bitboard`] representing the squares that are pinned
    /// - A [`Bitboard`] representing the orthogonal pin rays
    /// - A [`Bitboard`] representing the diagonal pin rays
    ///
    fn calculate_check_and_pin_metadata(
        &self,
        board: &Board,
    ) -> (Bitboard, Bitboard, Bitboard, Bitboard, Bitboard, Bitboard) {
        // helpers to simplify things later
        let us = board.side_to_move();
        let them = Side::opposite(us);
        let occupancy = board.all_pieces();
        let empty = !occupancy;
        let their_pieces = board.pieces(them);
        let our_pieces = board.pieces(us);
        let enemy_or_empty = their_pieces | empty;
        let king_sq = board.king_square(us);

        // initialize variables that we will use later
        let mut pinned = Bitboard::default();
        let mut capture_mask = enemy_or_empty & !(*board.piece_bitboard(Piece::King, them));
        let mut orthogonal_pin_rays = Bitboard::default();
        let mut diagonal_pin_rays = Bitboard::default();

        // calculate checkers, we first start with non-sliding pieces
        // note that the opponent's king cannot check our king
        let mut checkers = *board.piece_bitboard(Piece::Knight, them)
            & self.get_piece_attacks(Piece::Knight, king_sq, them, &occupancy)
            | *board.piece_bitboard(Piece::Pawn, them)
                & self.get_piece_attacks(Piece::Pawn, king_sq, them, &occupancy);

        // calculate sliding attacks from our king square on an empty board and then "and" with the enemy pieces
        // the overlap of these two bitboards will give us the sliding pieces that can "see" our king and are
        // potentially attacking it.
        let mut enemy_sliding_attacks =
            self.get_piece_attacks(Piece::Rook, king_sq, them, &Bitboard::default())
                & (*board.piece_bitboard(Piece::Rook, them)
                    | *board.piece_bitboard(Piece::Queen, them))
                | self.get_piece_attacks(Piece::Bishop, king_sq, them, &Bitboard::default())
                    & (*board.piece_bitboard(Piece::Bishop, them)
                        | *board.piece_bitboard(Piece::Queen, them));

        // loop through all sliding attackers
        while enemy_sliding_attacks.as_number() > 0 {
            let next_attacker = bitboard_helpers::next_bit(&mut enemy_sliding_attacks) as u8;
            let attacker_bb = Bitboard::from_square(next_attacker);

            // get the ray from the attacker to the king square (not inclusive)
            let ray = self.ray_between(
                Square::from_square_index(king_sq),
                Square::from_square_index(next_attacker),
            );

            let (king_file, king_rank) = square::from_square(king_sq);
            let (attacker_file, attacker_rank) = square::from_square(next_attacker as u8);
            // check if the ray is orthogonal or diagonal
            let is_orthogonal = king_file == attacker_file || king_rank == attacker_rank;
            let is_diagonal = (king_sq as i16 - next_attacker as i16).abs() % 9 == 0
                || (king_sq as i16 - next_attacker as i16).abs() % 7 == 0;

            // check if the ray is blocked by any pieces
            match (ray & occupancy).number_of_occupied_squares() {
                // not blocked so the attacker is checking our king
                0 => {
                    checkers |= Bitboard::from_square(next_attacker as u8);
                }
                // exactly 1 blockers, so this piece is pinned
                1 => {
                    // check that the blocking piece is ours
                    let overlap = ray & our_pieces;
                    if overlap.number_of_occupied_squares() == 1 {
                        // we found a real pin
                        pinned |= ray & our_pieces;
                        if is_orthogonal {
                            orthogonal_pin_rays |= ray | attacker_bb;
                        } else if is_diagonal {
                            diagonal_pin_rays |= ray | attacker_bb;
                        }
                    }
                }
                // more than 1 piece in ray, so we don't care about this ray
                // regardless of whose pieces are blocking the ray
                _ => {}
            }
        }

        // by default, we can push to all squares
        let mut push_mask = Bitboard::from(u64::MAX);

        // check if we have checkers
        if checkers.number_of_occupied_squares() >= 1 {
            // check if this is a single check
            let is_single_check = checkers.number_of_occupied_squares() == 1;

            // update our capture mask to be the checkers, but exclude the king
            capture_mask = checkers & !(*board.piece_bitboard(Piece::King, them));

            // special case for single check
            if is_single_check {
                // if we're in single check, we can block the checker or capture it
                let mut checkers_clone = checkers;
                let checker = bitboard_helpers::next_bit(&mut checkers_clone) as u8;

                // calculate the ray between the checker and the king
                let ray = self.ray_between(
                    Square::from_square_index(king_sq),
                    Square::from_square_index(checker as u8),
                );

                // update the push mask if the attacker is a slider piece
                if let Some((piece, side)) = board.piece_on_square(checker as u8) {
                    debug_assert!(side == them);
                    let is_slider = piece.is_slider();
                    if is_slider {
                        // attacker is a slider, so we can block by pushing into the ray
                        push_mask = ray;
                    } else {
                        // we can only capture, so push mask must be empty
                        push_mask = Bitboard::default();
                    }
                }
            }
        }

        // Add the en passant square to our capture mask if it's viable
        let en_passant_bb = board
            .en_passant_square()
            .map(Bitboard::from)
            .unwrap_or_default();
        // we need to first check if the en passant square would capture a checker
        // if "us" is white, then we should shift the en passant square left
        // if "us" is black, then we should shift the en passant square right
        match board.side_to_move() {
            Side::White => {
                let left = en_passant_bb >> SOUTH;
                if left & checkers != 0 {
                    capture_mask |= en_passant_bb;
                }
            }
            Side::Black => {
                let right = en_passant_bb << NORTH;
                if right & checkers != 0 {
                    capture_mask |= en_passant_bb;
                }
            }
        }

        (
            checkers,
            capture_mask,
            push_mask,
            pinned,
            orthogonal_pin_rays,
            diagonal_pin_rays,
        )
    }

    /// Calculate 'checkers' and 'pinned' bitboard masks for the current position.
    ///
    /// # Arguments
    /// - board - The current board state
    /// - occupancy - The occupancy bitboard
    ///
    /// # Returns
    ///
    /// A [`Bitboard`] representing the squares that are checking the king
    fn calculate_checkers(&self, board: &Board, occupancy: &Bitboard) -> Bitboard {
        let us = board.side_to_move();
        let them = Side::opposite(us);
        let king_bb = board.piece_bitboard(Piece::King, us);
        let king_square = bitboard_helpers::next_bit(&mut king_bb.clone()) as u8;

        // ensure we definitely don't have the king in the occupancy
        let kingless_occupancy = *occupancy & !(*king_bb);
        // an enemy king cannot check our king, so we ignore it
        let knight_attacks =
            self.get_piece_attacks(Piece::Knight, king_square, us, &kingless_occupancy);
        let rook_attacks =
            self.get_piece_attacks(Piece::Rook, king_square, us, &kingless_occupancy);
        let bishop_attacks =
            self.get_piece_attacks(Piece::Bishop, king_square, us, &kingless_occupancy);
        let queen_attacks = rook_attacks | bishop_attacks;
        // note we use the opposite side for the pawn attacks
        let pawn_attacks = attacks::pawn(king_square, Side::opposite(them));

        let enemy_pawns = board.piece_bitboard(Piece::Pawn, them);
        let enemy_knights = board.piece_bitboard(Piece::Knight, them);
        let enemy_bishops = board.piece_bitboard(Piece::Bishop, them);
        let enemy_rooks = board.piece_bitboard(Piece::Rook, them);
        let enemy_queens = board.piece_bitboard(Piece::Queen, them);

        // calculate our checkers bb
        knight_attacks & *enemy_knights
            | rook_attacks & *enemy_rooks
            | bishop_attacks & *enemy_bishops
            | queen_attacks & *enemy_queens
            | pawn_attacks & *enemy_pawns
    }

    /// Calculate the en passant bitboard for the current position.
    /// This will return a bitboard with the en passant square set if it is a valid move.
    ///
    /// # Arguments
    /// - from - The square the pawn is moving from
    /// - board - The current board state
    /// - push_mask - The push mask for the king. See [calculate_capture_and_push_masks][MoveGenerator::calculate_capture_and_push_masks] for more.
    /// - checkers - The squares that are attacking the king. See [calculate_checkers][MoveGenerator::calculate_checkers] for more.
    ///
    /// # Returns
    /// A [`Bitboard`] with the en passant square set if it is a valid move, otherwise an empty bitboard.
    fn calculate_en_passant_bitboard(
        &self,
        from: u8,
        board: &Board,
        push_mask: &Bitboard,
        checkers: &Bitboard,
    ) -> Bitboard {
        let en_passant_sq = board.en_passant_square();

        match en_passant_sq {
            Some(sq) => {
                let en_passant_bb = en_passant_sq.map(Bitboard::from).unwrap_or_default();

                // check for discovered checks
                // remove the captured and capturing pawns from the bitboard
                let mut occupancy = board.all_pieces();
                // remove the attacking pawn
                occupancy &= !(Bitboard::from_square(from));
                // remove the captured pawn
                let captured_sq = match board.side_to_move() {
                    Side::White => sq - SOUTH as u8,
                    Side::Black => sq + NORTH as u8,
                };
                occupancy &= !(Bitboard::from_square(captured_sq));
                // get the squares attacked by the sliding pieces
                let mut discovered_checkers = self.calculate_checkers(board, &occupancy);
                // filter checkers to the same rank as the king
                let king_sq = bitboard_helpers::next_bit(
                    &mut board
                        .piece_bitboard(Piece::King, board.side_to_move())
                        .clone(),
                ) as u8;
                let king_rank = square::from_square(king_sq).1;
                // check if the checkers are on the same rank as the king and the en passant square is not
                discovered_checkers &= RANK_BITBOARDS[king_rank as usize];

                // it's a discovered check if our discovered check is not empty and we're currently not in check
                let is_discovered_check = discovered_checkers.number_of_occupied_squares() > 0
                    && checkers.number_of_occupied_squares() == 0;
                let ep_is_blocker = en_passant_bb.intersects(*push_mask) && !is_discovered_check;

                // check if there are no checkers along the king rank or, the en
                if !is_discovered_check || ep_is_blocker {
                    en_passant_bb
                } else {
                    // we are now in check, so we cannot use this en passant capture
                    Bitboard::default()
                }
            }
            None => Bitboard::default(),
        }
    }

    /// Generate the legal pawn moves from the given square with the given board state.
    ///
    /// # Arguments
    ///
    /// - board - The current board state
    /// - square - The square to generate moves for
    /// - pinned_pieces - The pinned pieces on the board
    /// - capture_mask - The mask of squares that can be captured. Will be all squares if king is not in check.
    /// - push_mask - The mask of squares that can be pushed to. Will be all squares if king is not in check.
    /// - orthogonal_pin_rays - The rays of orthogonal pins
    /// - diagonal_pin_rays - The rays of diagonal pins
    /// - checkers - The squares that are attacking the king
    ///
    /// # Returns
    /// A [`Bitboard`] with the legal moves for the pawn.
    ///
    /// These moves need to be enumerated to get the actual moves. See [`MoveGenerator::enumerate_moves`]
    #[allow(clippy::too_many_arguments)]
    fn generate_legal_pawn_mobility(
        &self,
        board: &Board,
        square: &Square,
        pinned_pieces: &Bitboard,
        capture_mask: &Bitboard,
        push_mask: &Bitboard,
        orthogonal_pin_rays: &Bitboard,
        diagonal_pin_rays: &Bitboard,
        checkers: &Bitboard,
    ) -> Bitboard {
        // pawns can get complex because of en passant and promotion
        // also, we need to take into account the pin directions
        let is_pinned = pinned_pieces.intersects(Bitboard::from_square(square.to_square_index()));
        let us = board.side_to_move();
        let their_pieces = board.pieces(Side::opposite(us));
        let direction = match us {
            Side::White => NORTH as u8,
            Side::Black => SOUTH as u8,
        };
        let from_square = square.to_square_index();
        let to_square = match us {
            Side::White => {
                let (result, did_overflow) = from_square.overflowing_add(direction);
                match did_overflow {
                    true => None,
                    false => Some(result),
                }
            }
            Side::Black => {
                let (result, did_overflow) = from_square.overflowing_sub(direction);
                match did_overflow {
                    true => None,
                    false => Some(result),
                }
            }
        };

        let mut pushes: Bitboard = match to_square {
            Some(to) => Bitboard::from_square(to),
            None => Bitboard::default(),
        };

        let occupancy = board.all_pieces();
        let is_unobstructed = pushes & !occupancy == Bitboard::default();

        let can_double_push = match us {
            Side::White => square::is_square_on_rank(from_square, Rank::R2 as u8),
            Side::Black => square::is_square_on_rank(from_square, Rank::R7 as u8),
        };

        // if single push is obstructed, we can't double push
        if can_double_push && !is_unobstructed {
            let double_push_sq = match us {
                Side::White => {
                    let (result, did_overflow) = from_square.overflowing_add(2 * NORTH as u8);
                    match did_overflow {
                        true => None,
                        false => Some(result),
                    }
                }
                Side::Black => {
                    let (result, did_overflow) = from_square.overflowing_sub(2 * SOUTH as u8);
                    match did_overflow {
                        true => None,
                        false => Some(result),
                    }
                }
            };

            if let Some(to) = double_push_sq {
                let bb = Bitboard::from_square(to);
                // append to our pushes
                pushes |= bb;
            }
        }

        let en_passant_bb: Bitboard =
            self.calculate_en_passant_bitboard(from_square, board, push_mask, checkers);

        let hv_pin_ray_mask = if is_pinned {
            *orthogonal_pin_rays
        } else {
            Bitboard::from(u64::MAX)
        };

        let diag_pin_ray_mask = if is_pinned {
            *diagonal_pin_rays
        } else {
            Bitboard::from(u64::MAX)
        };

        // filter pushes by the occupancy
        let legal_pushes = (pushes & !occupancy) & hv_pin_ray_mask;
        let attacks = attacks::pawn(square.to_square_index(), us)
            & (their_pieces | en_passant_bb)
            & diag_pin_ray_mask;

        (legal_pushes | attacks) & (*capture_mask | *push_mask)
    }

    /// Generate the legal moves for a normal piece (not a pawn or king) from the given square.
    /// This function will take into account pinned pieces and generate the legal moves for the piece.
    ///
    /// # Arguments
    ///
    /// - piece - The piece to generate moves for
    /// - square - The square to generate moves for
    /// - board - The current board state
    /// - capture_mask - The mask of squares that can be captured. Will be all squares if king is not in check.
    /// - pinned_mask - The mask of squares that are pinned
    /// - push_mask - The mask of squares that can be pushed to. Will be all squares if king is not in check.
    /// - orthogonal_pin_rays - The rays of orthogonal pins
    /// - diagonal_pin_rays - The rays of diagonal pins
    ///
    /// # Returns
    ///
    /// A [`Bitboard`] with the legal moves for the piece.
    ///
    /// These moves need to be enumerated to get the actual moves. See [`MoveGenerator::enumerate_moves`]
    #[allow(clippy::too_many_arguments)]
    fn generate_normal_piece_legal_mobility(
        &self,
        piece: Piece,
        square: &Square,
        board: &Board,
        capture_mask: &Bitboard,
        pinned_mask: &Bitboard,
        push_mask: &Bitboard,
        orthogonal_pin_rays: &Bitboard,
        diagonal_pin_rays: &Bitboard,
    ) -> Bitboard {
        let is_pinned = pinned_mask.intersects(Bitboard::from_square(square.to_square_index()));
        let us = board.side_to_move();
        let their_pieces = board.pieces(Side::opposite(us));
        let from_square = square.to_square_index();
        let occupancy = board.all_pieces();
        // TODO: properly use orthogonal and diagonal pin rays
        let pin_rays = *orthogonal_pin_rays | *diagonal_pin_rays;

        assert!(!piece.is_king() && !piece.is_pawn());

        let attacks = self.get_piece_attacks(piece, from_square, us, &occupancy);

        // we need to further filter out moves and ensure they are along the correct pin ray

        let our_pieces = board.pieces(us);
        let empty = !(their_pieces | our_pieces);

        // empty pushes or captures masked by capture mask and push mask in the case that we are
        // single checked
        // note: everything gets masked by the pin ray mask

        // calculate the correct pin ray mask, only using pin rays that intersect with the piece
        // and the king
        let pin_ray_mask = if is_pinned {
            let king_sq = board.king_square(us);

            // at least one of our moves intersects with the pin ray, so we're attacking the pinner or moving along the pin ray
            // we need to ensure that we move along the correct pin ray
            let mut pinners = their_pieces & pin_rays;
            let piece_bb = Bitboard::from_square(square.to_square_index());
            let mut true_ray_mask = Bitboard::default();

            while pinners.as_number() > 0 {
                let pinner_sq = bitboard_helpers::next_bit(&mut pinners) as u8;
                let ray = self.ray_between(
                    Square::from_square_index(pinner_sq),
                    Square::from_square_index(king_sq),
                );

                if ray.intersects(piece_bb) {
                    // the ray intersects with our piece,
                    true_ray_mask |= ray | Bitboard::from_square(pinner_sq);
                }
            }

            true_ray_mask
        } else {
            Bitboard::from(u64::MAX)
        };

        ((attacks & *capture_mask & their_pieces) | (attacks & empty & *push_mask)) & pin_ray_mask
    }

    /// Generate legal castling moves for the king.
    ///
    /// # Arguments
    ///
    /// - square - The square the king is on
    /// - board - The current board state
    /// - attacked_squares - The squares that are attacked by the opponent
    /// - checkers - The squares that are checking the king
    ///
    /// # Returns
    ///
    /// A [`Bitboard`] with the legal castling moves for the king.
    fn generate_legal_castling_mobility(
        &self,
        square: &Square,
        board: &Board,
        attacked_squares: &Bitboard,
        checkers: &Bitboard,
    ) -> Bitboard {
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

        // we cannot castle when in check
        let in_check = checkers.number_of_occupied_squares() > 0;
        if in_check {
            return Bitboard::default();
        }

        let us = board.side_to_move();
        let occupancy = board.all_pieces();
        let mut castling_moves = Bitboard::default();
        let king_side_castle = board.can_castle_kingside(us);
        let queen_side_castle = board.can_castle_queenside(us);

        let king_sq = match us {
            Side::White => Squares::E1,
            Side::Black => Squares::E8,
        };

        // sanity check
        let king_in_place = king_sq == square.to_square_index();
        if !king_in_place {
            return Bitboard::default();
        }

        if king_side_castle {
            let king_side_rook = match us {
                Side::White => Squares::H1,
                Side::Black => Squares::H8,
            };
            // sanity check for the rook placement
            let maybe_rook = board.piece_on_square(king_side_rook);
            let rook_in_place = match maybe_rook {
                Some((Piece::Rook, side)) => side == us,
                _ => false,
            };

            let king_side_empty = match us {
                Side::White => {
                    Bitboard::from_square(Squares::F1) | Bitboard::from_square(Squares::G1)
                }
                Side::Black => {
                    Bitboard::from_square(Squares::F8) | Bitboard::from_square(Squares::G8)
                }
            };

            let king_side_target_sq = match us {
                Side::White => Squares::G1,
                Side::Black => Squares::G8,
            };

            let is_king_ray_empty = king_side_empty & occupancy == Bitboard::default();
            let is_king_ray_attacked = king_side_empty & *attacked_squares != Bitboard::default();
            if is_king_ray_empty && !is_king_ray_attacked && rook_in_place && king_in_place {
                castling_moves |= Bitboard::from_square(king_side_target_sq);
            }
        }

        if queen_side_castle {
            let queen_side_rook = match us {
                Side::White => Squares::A1,
                Side::Black => Squares::A8,
            };
            // sanity check for the rook placement
            let maybe_rook = board.piece_on_square(queen_side_rook);
            let rook_in_place = match maybe_rook {
                Some((Piece::Rook, side)) => side == us,
                _ => false,
            };

            let queen_side_no_attack = match us {
                Side::White => {
                    Bitboard::from_square(Squares::C1) | Bitboard::from_square(Squares::D1)
                }
                Side::Black => {
                    Bitboard::from_square(Squares::C8) | Bitboard::from_square(Squares::D8)
                }
            };
            let queen_side_empty = match us {
                Side::White => queen_side_no_attack | Bitboard::from_square(Squares::B1),
                Side::Black => queen_side_no_attack | Bitboard::from_square(Squares::B8),
            };

            let queen_side_target_sq = match us {
                Side::White => Squares::C1,
                Side::Black => Squares::C8,
            };

            let is_king_ray_empty = queen_side_empty & occupancy == Bitboard::default();
            let is_king_ray_attacked =
                queen_side_no_attack & *attacked_squares != Bitboard::default();
            if is_king_ray_empty && !is_king_ray_attacked && rook_in_place && king_in_place {
                castling_moves |= Bitboard::from_square(queen_side_target_sq);
            }
        }
        castling_moves
    }

    /// Generate legal moves for the king
    ///
    /// # Arguments
    ///
    /// - `square` - The square index of the king
    /// - `board` - The board state
    /// - `capture_mask` - The mask of squares that can be captured
    /// - `checkers` - The mask of squares that are checking the king
    ///
    /// # Returns
    ///
    /// A [`Bitboard`] of legal moves for the king
    fn generate_king_legal_mobility(
        &self,
        square: &Square,
        board: &Board,
        capture_mask: &Bitboard,
        checkers: &Bitboard,
    ) -> Bitboard {
        let us = board.side_to_move();
        let them = Side::opposite(us);
        let our_pieces = board.pieces(us);
        let their_pieces = board.pieces(them);
        let occupancy = our_pieces | their_pieces;

        let king_bb = board.piece_bitboard(Piece::King, us);

        // generate king moves
        // calculate attacked squares
        let king_moves_bb =
            self.get_piece_attacks(Piece::King, square.to_square_index(), us, &occupancy);

        // remove the king from the attacked squares occupancy
        let attacked_squares_occupancy = occupancy & !*king_bb;
        let attacked_squares = self.get_attacked_squares(board, them, &attacked_squares_occupancy);
        let king_pushes = king_moves_bb & !attacked_squares & !our_pieces & !their_pieces;

        // also add castling if possible
        let castling_moves =
            self.generate_legal_castling_mobility(square, board, &attacked_squares, checkers);

        let king_non_checker_attacks =
            (king_moves_bb & their_pieces & !*checkers) & !attacked_squares;

        let mut king_attacks = (king_moves_bb & *capture_mask & their_pieces & !attacked_squares)
            | king_non_checker_attacks;

        // do any of our attacks put us in check?
        let mut k_att = king_attacks;
        while k_att.as_number() > 0 {
            let capture_sq = bitboard_helpers::next_bit(&mut k_att) as u8;
            // remove the piece we're capturing and the king from the occupancy
            let modified_occupancy = occupancy & !Bitboard::from_square(capture_sq) & !*king_bb;
            let is_invalid_capture = self.is_square_attacked_with_occupancy(
                board,
                &Square::from_square_index(capture_sq),
                them,
                &modified_occupancy,
            );
            if is_invalid_capture {
                // if the capture puts us in check, remove it
                king_attacks &= !Bitboard::from_square(capture_sq);
            }
        }

        king_pushes | king_attacks | castling_moves
    }

    /// Generate legal moves for the given piece. This is a delegating function
    /// that calls the appropriate function to generate legal moves for the piece.
    ///
    /// # Arguments
    ///
    /// - `piece` - The piece to generate legal moves for
    /// - `square` - The square index of the piece
    /// - `board` - The board state
    /// - `pinned_mask` - The mask of pinned pieces
    /// - `capture_mask` - The mask of squares that can be captured
    /// - `push_mask` - The mask of squares that can be pushed to
    /// - `orthogonal_pin_rays` - The mask of orthogonal pin rays
    /// - `diagonal_pin_rays` - The mask of diagonal pin rays
    /// - `checkers` - The mask of squares that are checking the king
    ///
    /// # Returns
    ///
    /// A [`Bitboard`] of legal moves for the piece that can them be enumerated.
    #[allow(clippy::too_many_arguments)]
    fn generate_legal_mobility(
        &self,
        piece: Piece,
        square: &Square,
        board: &Board,
        pinned_mask: &Bitboard,
        capture_mask: &Bitboard,
        push_mask: &Bitboard,
        orthogonal_pin_rays: &Bitboard,
        diagonal_pin_rays: &Bitboard,
        checkers: &Bitboard,
    ) -> Bitboard {
        match piece {
            Piece::Pawn => self.generate_legal_pawn_mobility(
                board,
                square,
                pinned_mask,
                capture_mask,
                push_mask,
                orthogonal_pin_rays,
                diagonal_pin_rays,
                checkers,
            ),
            Piece::King => self.generate_king_legal_mobility(square, board, capture_mask, checkers),
            _ => self.generate_normal_piece_legal_mobility(
                piece,
                square,
                board,
                capture_mask,
                pinned_mask,
                push_mask,
                orthogonal_pin_rays,
                diagonal_pin_rays,
            ),
        }
    }

    /// Generate all legal moves for the current [`Board`] state.
    ///
    /// # Arguments
    ///
    /// - `board` - The current board state
    /// - `move_list` - The list of moves to append to
    ///
    /// # Returns
    ///
    /// A list of legal moves for the current board state.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::board::Board;
    /// use chess::move_list::MoveList;
    /// use chess::move_generation::MoveGenerator;
    ///
    /// let board = Board::default_board();
    /// let mut move_list = MoveList::new();
    /// let movegen = MoveGenerator::new();
    /// movegen.generate_legal_moves(&board, &mut move_list);
    /// assert_eq!(20, move_list.len())
    /// ```
    pub fn generate_legal_moves(&self, board: &Board, move_list: &mut MoveList) {
        // get board state info to make things simpler
        let us = board.side_to_move();
        let our_pieces = board.pieces(us);

        // get the king square and bitboard
        let king_bb = board.piece_bitboard(Piece::King, us);
        let king_square = board.king_square(us);

        // calculate checkers and pins
        let (checkers, capture_mask, push_mask, pinned, orthogonal_pin_rays, diagonal_pin_rays) =
            self.calculate_check_and_pin_metadata(board);

        // convert to Square object
        let king_sq = Square::from_square_index(king_square);
        // generate the king mobility first because king can always move (unless checkmate)
        let king_moves =
            self.generate_king_legal_mobility(&king_sq, board, &capture_mask, &checkers);

        // enumerate the king moves
        self.enumerate_moves(&king_moves, &king_sq, Piece::King, board, move_list);

        // check if we are more than single checked
        let num_checkers = checkers.as_number().count_ones();
        if num_checkers > 1 {
            // if there are multiple checkers, the king must move
            return;
        }

        // now we generate moves for the rest of the pieces
        let mut moveable_pieces = our_pieces & !(*king_bb);
        while moveable_pieces.as_number() > 0 {
            // Generate moves for each piece
            let from = bitboard_helpers::next_bit(&mut moveable_pieces) as u8;
            let piece = match board.piece_on_square(from) {
                Some((piece, _)) => piece,
                None => continue,
            };

            let from_square = Square::from_square_index(from);
            // calculate our legal mobility
            let moves = self.generate_legal_mobility(
                piece,
                &from_square,
                board,
                &pinned,
                &capture_mask,
                &push_mask,
                &orthogonal_pin_rays,
                &diagonal_pin_rays,
                &checkers,
            );

            // enumerate the moves and add them to the move list
            self.enumerate_moves(&moves, &from_square, piece, board, move_list);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_pinned_pieces() {
        let move_gen = MoveGenerator::new();
        let board =
            Board::from_fen("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2")
                .unwrap();
        let occupancy = board.all_pieces();
        let (_, _, _, pinned, _, _) = move_gen.calculate_check_and_pin_metadata(&board);
        let checkers = move_gen.calculate_checkers(&board, &occupancy);
        assert_eq!(checkers, 0);
        assert_eq!(pinned, Bitboard::from_square(Squares::D7));
    }

    #[test]
    fn calculate_pinned_pieces_2() {
        let move_gen = MoveGenerator::new();
        let board = Board::from_fen("8/8/8/8/k2Pp2Q/8/8/3K4 b - d3 0 1").unwrap();
        let occupancy = board.all_pieces();
        let (_, _, _, pinned, _, _) = move_gen.calculate_check_and_pin_metadata(&board);
        let checkers = move_gen.calculate_checkers(&board, &occupancy);
        assert_eq!(checkers, 0);
        assert_eq!(pinned, Bitboard::default());
    }

    #[test]
    fn calculate_pinned_pieces_3() {
        let move_gen = MoveGenerator::new();
        let board =
            Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQKR2 b Q - 2 8").unwrap();

        let occupancy = board.all_pieces();
        let (_, _, _, pinned, orthogonal_rays, diagonal_rays) =
            move_gen.calculate_check_and_pin_metadata(&board);
        let pin_rays = orthogonal_rays | diagonal_rays;
        let checkers = move_gen.calculate_checkers(&board, &occupancy);
        assert_eq!(checkers, 0);
        assert_eq!(pinned, 0);
        assert_eq!(pin_rays, 0);
    }

    #[test]
    fn calculate_pins() {
        let move_gen = MoveGenerator::new();
        let board =
            Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nPB5/B1P1P3/5N2/q2P1KPP/b2Q1R2 w kq - 0 3")
                .unwrap();
        let (_, _, _, pinned_pieces, horizontal_pin_rays, diagonal_pin_rays) =
            move_gen.calculate_check_and_pin_metadata(&board);

        assert_eq!(pinned_pieces.number_of_occupied_squares(), 2);
        println!("horizontal pin rays:\n{horizontal_pin_rays}");
        println!("diagonal pin rays:\n{diagonal_pin_rays}");

        assert!(pinned_pieces.intersects(Bitboard::from_square(Squares::C5)));
        assert!(pinned_pieces.intersects(Bitboard::from_square(Squares::D2)));
    }

    #[test]
    fn check_pinned_and_capture_mask() {
        let move_gen = MoveGenerator::new();
        let board =
            Board::from_fen("rnQq1k1r/pp2bppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R b KQ - 0 8").unwrap();
        let (checkers, capture_mask, push_mask, pinned, orthogonal_rays, diagonal_rays) =
            move_gen.calculate_check_and_pin_metadata(&board);
        println!("checkers:\n{checkers}");
        println!("check mask:\n{capture_mask}");
        println!("push mask:\n{push_mask}");
        println!("pinned:\n{pinned}");
        println!("orthogonal rays:\n{orthogonal_rays}");
        println!("diagonal rays:\n{diagonal_rays}");

        assert_eq!(checkers, 0);
        assert_eq!(pinned, Bitboard::from_square(Squares::D8));
        println!("capture mask:\n{capture_mask}");
        println!("push mask:\n{push_mask}");
    }

    #[test]
    fn check_pinned_and_capture_mask_2() {
        let move_gen = MoveGenerator::new();
        let board = Board::from_fen("4B1r1/2q2p2/QP4k1/3P2p1/7B/8/6K1/7R b - - 3 59").unwrap();
        let (checkers, capture_mask, push_mask, pinned, orthogonal_rays, diagonal_rays) =
            move_gen.calculate_check_and_pin_metadata(&board);
        println!("checkers:\n{checkers}");
        println!("check mask:\n{capture_mask}");
        println!("push mask:\n{push_mask}");
        println!("pinned:\n{pinned}");
        println!("orthogonal rays:\n{orthogonal_rays}");
        println!("diagonal rays:\n{diagonal_rays}");

        assert_eq!(checkers, 0);
        assert_eq!(pinned, Bitboard::from_square(Squares::F7));
        assert_eq!(orthogonal_rays, 0);
        assert!(diagonal_rays > 0);
    }

    #[test]
    fn en_passant_capture_causes_discovered_check() {
        let move_gen = MoveGenerator::new();
        let board = Board::from_fen("8/8/8/8/k2Pp2Q/8/8/3K4 b - d3 0 1").unwrap();
        let mut move_list = MoveList::new();
        move_gen.generate_legal_moves(&board, &mut move_list);

        for mv in move_list.iter() {
            println!("{mv}");
        }

        assert_eq!(move_list.len(), 6);
    }

    #[test]
    fn king_cannot_move_away_from_slider() {
        let move_gen = MoveGenerator::new();
        let board = Board::from_fen("4k3/8/8/8/4R3/8/8/4K3 b - - 0 1").unwrap();

        let mut move_list = MoveList::new();
        move_gen.generate_legal_moves(&board, &mut move_list);
        assert_eq!(move_list.len(), 4);
    }

    #[test]
    fn king_cannot_slide_away_from_bishop() {
        let move_gen = MoveGenerator::new();
        let board = Board::from_fen("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2").unwrap();

        let mut move_list = MoveList::new();
        move_gen.generate_legal_moves(&board, &mut move_list);
        assert_eq!(move_list.len(), 8);
    }

    #[test]
    fn evade_check_with_en_passant_capture() {
        let move_gen = MoveGenerator::new();
        let board = Board::from_fen("8/8/8/2k5/3Pp3/8/8/4K3 b - d3 0 1").unwrap();
        let mut move_list = MoveList::new();
        move_gen.generate_legal_moves(&board, &mut move_list);

        for mv in move_list.iter() {
            println!("{mv}");
        }

        assert_eq!(move_list.len(), 9);
    }

    #[test]
    fn rays_between_verification() {
        let move_gen = MoveGenerator::new();
        let from = Square::from_square_index(Squares::A1);
        let to = Square::from_square_index(Squares::H8);
        let rays = move_gen.ray_between(from, to);

        let expected = Bitboard::from_square(Squares::B2)
            | Bitboard::from_square(Squares::C3)
            | Bitboard::from_square(Squares::D4)
            | Bitboard::from_square(Squares::E5)
            | Bitboard::from_square(Squares::F6)
            | Bitboard::from_square(Squares::G7);
        println!("{rays}");
        assert_eq!(rays, expected);

        let from = Square::from_square_index(Squares::H1);
        let to = Square::from_square_index(Squares::A8);
        let rays = move_gen.ray_between(from, to);

        let expected = Bitboard::from_square(Squares::G2)
            | Bitboard::from_square(Squares::F3)
            | Bitboard::from_square(Squares::E4)
            | Bitboard::from_square(Squares::D5)
            | Bitboard::from_square(Squares::C6)
            | Bitboard::from_square(Squares::B7);
        assert_eq!(rays, expected);

        let rays = move_gen.ray_between(
            Square::from_square_index(Squares::A1),
            Square::from_square_index(Squares::C2),
        );
        assert!(rays == Bitboard::default());
    }
}
