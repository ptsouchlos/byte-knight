// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{
    board::Board,
    definitions::{CastlingAvailability, DEFAULT_FEN},
    file::File,
    pieces::Piece,
    rank::Rank,
    side::Side,
    square::to_square,
};

#[test]
fn test_board_construction() {
    let board = Board::default_board();
    assert_eq!(board.all_pieces(), 0xFFFF00000000FFFF);
}

#[test]
fn construct_board_from_fen_string() {
    let board_result = Board::from_fen(DEFAULT_FEN);
    assert!(board_result.is_ok());
    let board = board_result.unwrap();
    let all_pieces = board.all_pieces();
    println!("all {all_pieces}");
    assert_eq!(board.all_pieces(), 0xFFFF00000000FFFF);
}

#[test]
fn construct_board_from_fen_string_2() {
    let fen_2 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let board_result = Board::from_fen(fen_2);
    assert!(board_result.is_ok());
    let board = board_result.unwrap();

    let white_pawn_bb = board.piece_bitboard(Piece::Pawn, Side::White);
    assert_eq!(white_pawn_bb.as_number(), 0x1000EF00);

    let black_pawn_bb = board.piece_bitboard(Piece::Pawn, Side::Black);
    assert_eq!(black_pawn_bb.as_number(), 0xFF000000000000);

    let white_knight_bb = board.piece_bitboard(Piece::Knight, Side::White);
    assert_eq!(white_knight_bb.as_number(), 0x42);

    let black_knight_bb = board.piece_bitboard(Piece::Knight, Side::Black);
    assert_eq!(black_knight_bb.as_number(), 0x4200000000000000);

    let white_bishop_bb = board.piece_bitboard(Piece::Bishop, Side::White);
    assert_eq!(white_bishop_bb.as_number(), 0x24);

    let black_bishop_bb = board.piece_bitboard(Piece::Bishop, Side::Black);
    assert_eq!(black_bishop_bb.as_number(), 0x2400000000000000);

    let white_rook_bb = board.piece_bitboard(Piece::Rook, Side::White);
    assert_eq!(white_rook_bb.as_number(), 0x81);

    let black_rook_bb = board.piece_bitboard(Piece::Rook, Side::Black);
    assert_eq!(black_rook_bb.as_number(), 0x8100000000000000);

    let white_queen_bb = board.piece_bitboard(Piece::Queen, Side::White);
    assert_eq!(white_queen_bb.as_number(), 0x8);

    let black_queen_bb = board.piece_bitboard(Piece::Queen, Side::Black);
    assert_eq!(black_queen_bb.as_number(), 0x800000000000000);

    let white_king_bb = board.piece_bitboard(Piece::King, Side::White);
    assert_eq!(white_king_bb.as_number(), 0x10);

    let black_king_bb = board.piece_bitboard(Piece::King, Side::Black);
    assert_eq!(black_king_bb.as_number(), 0x1000000000000000);

    let all_pieces = board.all_pieces();
    let white_pieces = board.white_pieces();
    let black_pieces = board.black_pieces();
    let black_pieces_bb = *black_pawn_bb
        | *black_knight_bb
        | *black_bishop_bb
        | *black_rook_bb
        | *black_queen_bb
        | *black_king_bb;

    let white_pieces_bb = *white_pawn_bb
        | *white_knight_bb
        | *white_bishop_bb
        | *white_rook_bb
        | *white_queen_bb
        | *white_king_bb;

    assert_eq!(white_pieces, white_pieces_bb);
    assert_eq!(black_pieces, black_pieces_bb);

    let all_pieces_bb = white_pieces_bb | black_pieces_bb;
    assert_eq!(all_pieces, all_pieces_bb);

    println!("{all_pieces}");

    assert_eq!(board.all_pieces(), all_pieces_bb);
    assert!(board.en_passant_square().is_some());
    assert_eq!(
        board.en_passant_square().unwrap(),
        to_square(File::E as u8, Rank::R3 as u8)
    );
    assert_eq!(board.half_move_clock(), 0);
    assert_eq!(board.full_move_number(), 1);
    assert_eq!(board.side_to_move(), Side::Black);
    assert_eq!(board.castling_rights(), CastlingAvailability::ALL);
}

#[test]
fn construct_board_from_fen_string_3() {
    let fen = "rnbqkbnr/pp1p1ppp/8/2p5/1P1pP3/8/P1P2PPP/RNBQKBNR w KQkq - 0 4";

    let board_result = Board::from_fen(fen);
    assert!(board_result.is_ok());
    let board = board_result.unwrap();

    let all_pieces = board.all_pieces();
    println!("all {all_pieces}");
    assert_eq!(board.all_pieces(), 0xFFEB00041A00E5FF);
}

#[test]
fn board_to_fen() {
    let target_fen = DEFAULT_FEN;
    let board = Board::default_board();
    let fen = board.to_fen();
    assert_eq!(fen, target_fen);
}
