// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{
    attacks,
    bitboard::Bitboard,
    bitboard_helpers,
    board::Board,
    move_generation::{self, enumerate::enumerate_moves, move_filter::MoveFilter},
    move_list::MoveList,
    moves::{Move, MoveFlag},
    pieces::Piece,
    rank::Rank,
    rays,
    side::Side,
    square::Square,
};

pub mod castling;
pub mod enumerate;
pub mod legal;
pub mod metadata;
pub mod move_filter;
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
pub fn relevant_rook_bits(square: Square) -> Bitboard {
    let mut bb = Bitboard::default();
    bb.set_square(square);

    let rook_rays_bb = attacks::orthogonal_ray_attacks(square, 0);
    let edges = rays::edges(square.file(), square.rank());

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
pub fn relevant_bishop_bits(square: Square) -> Bitboard {
    let mut bb = Bitboard::default();
    bb.set_square(square);

    let edges = rays::edges(square.file(), square.rank());

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
pub fn generate_moves(board: &Board, move_list: &mut MoveList, move_filter: MoveFilter) {
    for piece in Piece::iter().filter(|p| *p != Piece::Pawn) {
        get_piece_moves(piece, board, move_list, move_filter);
    }

    get_pawn_moves(board, move_list, move_filter);

    if matches!(move_filter, MoveFilter::All | MoveFilter::Quiets) {
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
    let occupancy = board.all_pieces();
    let checkers = calculate_checkers(board, occupancy);
    let legal_castling_mobility = move_generation::castling::legal_mobility(board, checkers);
    let king_sq = board.king_square(board.side_to_move());
    enumerate_moves(
        &legal_castling_mobility,
        king_sq,
        Piece::King,
        board,
        MoveFilter::All,
        move_list,
    );
}

fn get_piece_moves(piece: Piece, board: &Board, move_list: &mut MoveList, move_filter: MoveFilter) {
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
    for from_sq in piece_bb {
        let attack_bb = attacks::for_piece_on_square(piece, from_sq, occupancy, us);

        let bb_moves = match move_filter {
            MoveFilter::Captures | MoveFilter::Tacticals => attack_bb & their_pieces,
            MoveFilter::Quiets => attack_bb & empty,
            MoveFilter::All => attack_bb & !our_pieces,
        };

        enumerate::enumerate_moves(&bb_moves, from_sq, piece, board, move_filter, move_list);
    }
}

/// The promotion flags emitted, in queen-first order, whenever a pawn move
/// lands on the promotion rank.
const PROMOTION_FLAGS: [MoveFlag; 4] = [
    MoveFlag::PromotionQueen,
    MoveFlag::PromotionRook,
    MoveFlag::PromotionBishop,
    MoveFlag::PromotionKnight,
];

/// Emit pawn moves for a set of destination squares generated by a single
/// directional shift.
///
/// Each destination's origin is recovered by the inverse shift
/// `(inv_file, inv_rank)`, which is exact because every destination came from
/// shifting a real pawn (see [`Square::offset_unchecked`]). Destinations on the
/// promotion rank expand into the four [`PROMOTION_FLAGS`]; others use
/// `quiet_flag`. `emit_promotions` / `emit_quiets` gate the two cases so a single
/// target set can serve different [`MoveFilter`]s.
#[inline]
fn emit_pawn_targets(
    move_list: &mut MoveList,
    targets: Bitboard,
    inv_file: i8,
    inv_rank: i8,
    quiet_flag: MoveFlag,
    promotion_rank: Rank,
    emit_promotions: bool,
    emit_quiets: bool,
) {
    for to in targets {
        let from = to.offset_unchecked(inv_file, inv_rank);
        if to.rank() == promotion_rank {
            if emit_promotions {
                for flag in PROMOTION_FLAGS {
                    move_list.push(Move::new(from, to, flag));
                }
            }
        } else if emit_quiets {
            move_list.push(Move::new(from, to, quiet_flag));
        }
    }
}

#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(debug_assertions, inline(never))]
fn get_pawn_moves(board: &Board, move_list: &mut MoveList, move_filter: MoveFilter) {
    let us = board.side_to_move();
    let them = us.opposite();
    let pawns = board.piece_bitboard(Piece::Pawn, us);
    if pawns.is_empty() {
        return;
    }

    let enemies = board.pieces(them);
    let empty = !board.all_pieces();
    let promotion_rank = Rank::promotion_rank(us);

    // Non-promotion pushes, promotion pushes, and captures are gated independently
    // so each `MoveFilter` selects exactly the same set the per-pawn generator did:
    // Quiets = non-promo pushes; Captures = all captures/EP; Tacticals = promo
    // pushes + all captures; All = everything.
    let want_quiet_pushes = matches!(move_filter, MoveFilter::All | MoveFilter::Quiets);
    let want_promo_pushes = matches!(move_filter, MoveFilter::All | MoveFilter::Tacticals);
    let want_captures = matches!(
        move_filter,
        MoveFilter::All | MoveFilter::Captures | MoveFilter::Tacticals
    );

    // Forward shift and the third rank (where a legal single push from the start
    // rank lands) depend on the side; captures always shift toward the lower file
    // ("left") and higher file ("right").
    let (push, capture_left, capture_right, third_rank): (
        fn(Bitboard) -> Bitboard,
        fn(Bitboard) -> Bitboard,
        fn(Bitboard) -> Bitboard,
        Bitboard,
    ) = match us {
        Side::White => (
            bitboard_helpers::north,
            bitboard_helpers::north_west,
            bitboard_helpers::north_east,
            Rank::R3.to_bitboard(),
        ),
        Side::Black => (
            bitboard_helpers::south,
            bitboard_helpers::south_west,
            bitboard_helpers::south_east,
            Rank::R6.to_bitboard(),
        ),
    };
    // Inverse rank delta of one forward push (used to recover a move's origin).
    let back = -us.forward_delta();

    // Pushes: the single-push set carries both quiet and promotion pushes.
    if want_quiet_pushes || want_promo_pushes {
        let single = push(pawns) & empty;
        emit_pawn_targets(
            move_list,
            single,
            0,
            back,
            MoveFlag::Standard,
            promotion_rank,
            want_promo_pushes,
            want_quiet_pushes,
        );

        // A double push exists only where the intermediate single-push square is
        // empty (so it must be in `single`) and the pawn started on its home rank.
        if want_quiet_pushes {
            let double = push(single & third_rank) & empty;
            emit_pawn_targets(
                move_list,
                double,
                0,
                2 * back,
                MoveFlag::DoublePush,
                promotion_rank,
                false,
                true,
            );
        }
    }

    // Captures (including capture-promotions) and en passant.
    if want_captures {
        emit_pawn_targets(
            move_list,
            capture_left(pawns) & enemies,
            1,
            back,
            MoveFlag::Standard,
            promotion_rank,
            true,
            true,
        );
        emit_pawn_targets(
            move_list,
            capture_right(pawns) & enemies,
            -1,
            back,
            MoveFlag::Standard,
            promotion_rank,
            true,
            true,
        );

        if let Some(ep_square) = board.en_passant_square() {
            // Our pawns that attack the EP square are exactly those a `them` pawn
            // on the EP square would attack.
            let ep_attackers = pawns & attacks::pawn(ep_square, them);
            for from in ep_attackers {
                move_list.push(Move::new(from, ep_square, MoveFlag::EnPassant));
            }
        }
    }
}

/// Check if the side to move is in check.
pub fn is_in_check(board: &Board) -> bool {
    let king_square = board.king_square(board.side_to_move());
    square_state::is_square_attacked(board, king_square, board.side_to_move().opposite())
}

/// Check if the side to move is in checkmate.
/// Checkmate = in check and no legal moves.
pub fn is_checkmate(board: &Board) -> bool {
    if !is_in_check(board) {
        return false;
    }

    let move_list = legal::generate_moves(board, MoveFilter::All);
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

#[cfg(test)]
mod tests {

    use crate::{board::Board, definitions::NumberOf, move_generation};

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
        assert_eq!(meta.pinned, Bitboard::from(Square::D7));
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

        assert!(meta.pinned.intersects(Bitboard::from(Square::C5)));
        assert!(meta.pinned.intersects(Bitboard::from(Square::D2)));
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
        assert_eq!(meta.pinned, Bitboard::from(Square::D8));
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
        assert_eq!(meta.pinned, Bitboard::from(Square::F7));
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
            let rook_bits =
                move_generation::relevant_rook_bits(Square::from_square_index(square as u8));
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
            let bishop_bits =
                move_generation::relevant_bishop_bits(Square::from_square_index(square as u8));
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
            let rook_attack_bb = attacks::rook(Square::from_square_index(sq as u8), occupancy);
            // println!("{:#x},", rook_attack_bb.as_number())
            assert_eq!(rook_attack_bb.as_number(), *expected);
        }
    }

    #[test]
    fn check_blocker_permutations() {
        const BASE: u64 = 2_u64;

        for sq in 0..NumberOf::SQUARES {
            let rook_bb = relevant_rook_bits(Square::from_square_index(sq as u8));
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
        generate_moves(&board, &mut move_list, MoveFilter::All);

        for mv in move_list.iter() {
            println!("{mv}");
            assert!(!mv.is_castle());
            assert!(!mv.is_en_passant_capture());
            assert!(!mv.is_promotion());
        }

        assert_eq!(move_list.len(), 20);

        move_list.clear();
        let move_list = move_generation::legal::generate_moves(&board, MoveFilter::All);

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
        generate_moves(&board, &mut move_list, MoveFilter::All);
        let en_passant_move = move_list.iter().find(|mv| mv.is_en_passant_capture());
        assert!(en_passant_move.is_some());
        assert!(move_list.len() >= 8);
    }
}
