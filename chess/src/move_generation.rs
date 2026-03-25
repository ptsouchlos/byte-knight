// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{
    attacks,
    bitboard::Bitboard,
    board::Board,
    move_generation::{self, enumerate::enumerate_moves},
    move_list::MoveList,
    moves::{Move, MoveType},
    pieces::Piece,
    rank::Rank,
    rays,
    side::Side,
    square::{self, Square},
};

pub mod castling;
pub mod enumerate;
pub mod metadata;
pub mod square_state;

pub(crate) const NORTH: u64 = 8;
pub(crate) const SOUTH: u64 = 8;

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

    let (file, rank) = square::from_square(square);
    let rook_rays_bb = attacks::orthogonal_ray_attacks(square, 0);
    let edges = rays::edges(file, rank);

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
    let edges = rays::edges(file, rank);

    let bishop_ray_attacks = attacks::diagonal_ray_attacks(square, 0);

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
    let mask = bb;
    let mut subset = Bitboard::default();

    const BASE: u64 = 2_u64;
    let total_permutations = BASE.pow(bb.as_number().count_ones());

    let mut blocker_bitboards = Vec::with_capacity(total_permutations as usize);
    loop {
        blocker_bitboards.push(subset);
        subset = Bitboard::new(subset.as_number().wrapping_sub(mask.as_number())) & mask;
        if subset == 0 {
            break;
        }
    }
    blocker_bitboards
}

/// Calculate all squares currently being attacked by a given side.
pub(crate) fn get_attacked_squares(board: &Board, side: Side, occupancy: Bitboard) -> Bitboard {
    Piece::iter().fold(Bitboard::default(), |acc, piece| {
        acc | attacks::for_piece(piece, board, occupancy, side)
    })
}

/// Generates pseudo-legal moves for the current board state.
/// This function does not check for legality of the moves.
pub fn generate_moves(board: &Board, move_list: &mut MoveList, move_type: MoveType) {
    for piece in Piece::iter().filter(|p| *p != Piece::Pawn) {
        get_piece_moves(piece, board, move_list, &move_type);
    }

    get_pawn_moves(board, move_list, &move_type);

    if move_type == MoveType::All || move_type == MoveType::Quiet {
        get_castling_moves(board, move_list);
    }
}

/// Calculate 'checkers' and 'pinned' bitboard masks for the current position.
///
/// # Arguments
/// - board - The current board state
/// - occupancy - The occupancy bitboard
///
/// # Returns
///
/// A [`Bitboard`] representing the squares that are checking the king.
pub(crate) fn calculate_checkers(board: &Board, occupancy: Bitboard) -> Bitboard {
    let us = board.side_to_move();
    let king_bb = board.piece_bitboard(Piece::King, us);
    let king_square = board.king_square(us);
    let kingless_occupancy = occupancy & !(king_bb);

    attacks::all_attackers_of(king_square, board, us.opposite(), kingless_occupancy)
}

fn get_castling_moves(board: &Board, move_list: &mut MoveList) {
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
    let checkers = calculate_checkers(board, occupancy);
    let legal_castling_mobility = move_generation::castling::legal_mobility(board, checkers);
    let king_sq = Square::from_square_index(board.king_square(board.side_to_move()));
    enumerate_moves(
        &legal_castling_mobility,
        king_sq,
        Piece::King,
        board,
        move_list,
    );
}

fn get_piece_moves(piece: Piece, board: &Board, move_list: &mut MoveList, move_type: &MoveType) {
    debug_assert!(
        piece != Piece::Pawn,
        "Pawn move enumeration is handle separately."
    );

    let us = board.side_to_move();
    let them = us.opposite();
    let our_pieces = board.pieces(us);
    let their_pieces = board.pieces(them);
    let occupancy = board.all_pieces();
    let empty = !occupancy;

    let piece_bb = board.piece_bitboard(piece, us);
    for from_sq in piece_bb.iter() {
        let attack_bb = attacks::for_piece_on_square(piece, from_sq, occupancy, us);

        let bb_moves = match move_type {
            MoveType::Capture => attack_bb & their_pieces,
            MoveType::Quiet => attack_bb & empty,
            MoveType::All => attack_bb & !our_pieces,
        };

        enumerate::enumerate_moves(
            &bb_moves,
            Square::from_square_index(from_sq),
            piece,
            board,
            move_list,
        );
    }
}

#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(debug_assertions, inline(never))]
fn get_pawn_moves(board: &Board, move_list: &mut MoveList, move_type: &MoveType) {
    let us = board.side_to_move();
    let them = us.opposite();
    let their_pieces = board.pieces(them);
    let occupancy = board.all_pieces();
    let empty = !occupancy;
    let direction = if us == Side::White { NORTH } else { SOUTH };
    let pawns_bb = board.piece_bitboard(Piece::Pawn, us);

    // loop through all the pawns for us
    for from_square in pawns_bb.iter() {
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
            // En passant
            let bb_en_passant = match board.en_passant_square() {
                Some(en_passant_square) => {
                    // We only want to add the en passant square if it is within range of the pawn.
                    // This means that the en passant square is within 1 rank of the pawn and the en passant square
                    // is in the pawn's attack table.
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

        enumerate::enumerate_moves(
            &bb_moves,
            Square::from_square_index(from_square),
            Piece::Pawn,
            board,
            move_list,
        );
    }
}

/// Check if the side to move is in check.
pub fn is_in_check(board: &Board) -> bool {
    let king_square = board.king_square(board.side_to_move());
    square_state::is_square_attacked(
        board,
        Square::from_square_index(king_square),
        board.side_to_move().opposite(),
    )
}

/// Check if the side to move is in checkmate.
/// Checkmate = in check and no legal moves.
pub fn is_checkmate(board: &Board) -> bool {
    if !is_in_check(board) {
        return false;
    }

    let move_list = generate_legal_moves(board, MoveType::All);
    move_list.is_empty()
}

/// Check if a given move is legal. This function does not alter the board state.
/// Instead it makes a copy of the board and tries to make the move.
pub fn is_legal(board: &Board, mv: &Move) -> bool {
    let mut board_copy = board.clone();
    board_copy.make_move(mv).is_ok()
}

/// Check if a list of moves are legal. This function does not alter the board state.
pub fn are_legal(board: &Board, list: &MoveList) -> bool {
    let mut board_copy = board.clone();
    for mv in list.iter() {
        if board_copy.make_move(mv).is_err() {
            return false;
        }
    }
    true
}

/// Re-export from legal_move_generation for convenience.
pub use crate::legal_move_generation::generate_legal_moves;

#[cfg(test)]
mod tests {

    use crate::{
        board::Board,
        definitions::{NumberOf, Squares},
        move_generation,
    };

    use super::*;

    #[test]
    fn calculate_pinned_pieces() {
        let board =
            Board::from_fen("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2")
                .unwrap();
        let occupancy = board.all_pieces();
        let meta = metadata::compute(&board);
        let checkers = calculate_checkers(&board, occupancy);
        assert_eq!(checkers, 0);
        assert_eq!(meta.pinned, Bitboard::from_square(Squares::D7));
    }

    #[test]
    fn calculate_pinned_pieces_2() {
        let board = Board::from_fen("8/8/8/8/k2Pp2Q/8/8/3K4 b - d3 0 1").unwrap();
        let occupancy = board.all_pieces();
        let meta = metadata::compute(&board);
        let checkers = calculate_checkers(&board, occupancy);
        assert_eq!(checkers, 0);
        assert_eq!(meta.pinned, Bitboard::default());
    }

    #[test]
    fn calculate_pinned_pieces_3() {
        let board =
            Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQKR2 b Q - 2 8").unwrap();

        let occupancy = board.all_pieces();
        let meta = metadata::compute(&board);
        let pin_rays = meta.orthogonal_pin_rays | meta.diagonal_pin_rays;
        let checkers = calculate_checkers(&board, occupancy);
        assert_eq!(checkers, 0);
        assert_eq!(meta.pinned, 0);
        assert_eq!(pin_rays, 0);
    }

    #[test]
    fn calculate_pins() {
        let board =
            Board::from_fen("r3k2r/Pppp1ppp/1b3nbN/nPB5/B1P1P3/5N2/q2P1KPP/b2Q1R2 w kq - 0 3")
                .unwrap();
        let meta = metadata::compute(&board);

        assert_eq!(meta.pinned.number_of_occupied_squares(), 2);
        println!("horizontal pin rays:\n{}", meta.orthogonal_pin_rays);
        println!("diagonal pin rays:\n{}", meta.diagonal_pin_rays);

        assert!(meta.pinned.intersects(Bitboard::from_square(Squares::C5)));
        assert!(meta.pinned.intersects(Bitboard::from_square(Squares::D2)));
    }

    #[test]
    fn check_pinned_and_capture_mask() {
        let board =
            Board::from_fen("rnQq1k1r/pp2bppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R b KQ - 0 8").unwrap();
        let meta = metadata::compute(&board);
        println!("checkers:\n{}", meta.checkers);
        println!("check mask:\n{}", meta.capture_mask);
        println!("push mask:\n{}", meta.push_mask);
        println!("pinned:\n{}", meta.pinned);
        println!("orthogonal rays:\n{}", meta.orthogonal_pin_rays);
        println!("diagonal rays:\n{}", meta.diagonal_pin_rays);

        assert_eq!(meta.checkers, 0);
        assert_eq!(meta.pinned, Bitboard::from_square(Squares::D8));
    }

    #[test]
    fn check_pinned_and_capture_mask_2() {
        let board = Board::from_fen("4B1r1/2q2p2/QP4k1/3P2p1/7B/8/6K1/7R b - - 3 59").unwrap();
        let meta = metadata::compute(&board);
        println!("checkers:\n{}", meta.checkers);
        println!("check mask:\n{}", meta.capture_mask);
        println!("push mask:\n{}", meta.push_mask);
        println!("pinned:\n{}", meta.pinned);
        println!("orthogonal rays:\n{}", meta.orthogonal_pin_rays);
        println!("diagonal rays:\n{}", meta.diagonal_pin_rays);

        assert_eq!(meta.checkers, 0);
        assert_eq!(meta.pinned, Bitboard::from_square(Squares::F7));
        assert_eq!(meta.orthogonal_pin_rays, 0);
        assert!(meta.diagonal_pin_rays > 0);
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
            let rook_bits = move_generation::relevant_rook_bits(square as u8);
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
            let bishop_bits = move_generation::relevant_bishop_bits(square as u8);
            assert_eq!(bishop_bits.as_number(), value);

            offset_sum += BASE.pow(bishop_bits.as_number().count_ones());
        }

        println!("bishop offset sum: {offset_sum}");
    }

    #[test]
    fn check_rook_attacks() {
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
            let rook_attack_bb = attacks::rook(sq as u8, occupancy);
            // println!("{:#x},", rook_attack_bb.as_number())
            assert_eq!(rook_attack_bb.as_number(), *expected);
        }
    }

    #[test]
    fn check_blocker_permutations() {
        const BASE: u64 = 2_u64;

        for sq in 0..NumberOf::SQUARES {
            let rook_bb = relevant_rook_bits(sq as u8);
            let permutations = create_blocker_permutations(rook_bb);
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
        generate_moves(&board, &mut move_list, MoveType::All);

        for mv in move_list.iter() {
            println!("{mv}");
            assert!(!mv.is_castle());
            assert!(!mv.is_en_passant_capture());
            assert!(!mv.is_promotion());
        }

        assert_eq!(move_list.len(), 20);

        move_list.clear();
        let move_list = move_generation::generate_legal_moves(&board, MoveType::All);

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
        generate_moves(&board, &mut move_list, MoveType::All);
        let en_passant_move = move_list.iter().find(|mv| mv.is_en_passant_capture());
        assert!(en_passant_move.is_some());
        assert!(move_list.len() >= 8);
    }
}
