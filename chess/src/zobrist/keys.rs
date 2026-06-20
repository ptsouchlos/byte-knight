use crate::{board::Board, definitions::NumberOf, pieces::Piece, side::Side, zobrist::values};

pub(crate) fn sq_hash(piece: Piece, side: Side, square: u8) -> u64 {
    values::PIECE_VALUES[side as usize][piece as usize][square as usize]
}

pub(crate) fn side_hash(side: Side) -> u64 {
    values::SIDE_VALUES[side as usize]
}

pub(crate) fn castling_hash(rights: u8) -> u64 {
    values::CASTLING_VALUES[rights as usize]
}

pub(crate) fn ep_hash(ep_sq: Option<u8>) -> u64 {
    match ep_sq {
        None => values::EN_PASSANT_VALUES[NumberOf::SQUARES],
        Some(sq) => values::EN_PASSANT_VALUES[sq as usize],
    }
}

pub(crate) fn get_hash(board: &Board) -> u64 {
    let mut zobrist_hash = 0;

    for piece in Piece::iter() {
        let bitboard = board.piece_kind_bitboard(piece);
        let white_bb = board.pieces(Side::White);
        let black_bb = board.pieces(Side::Black);

        let white_pieces = bitboard & white_bb;
        let black_pieces = bitboard & black_bb;

        for sq in white_pieces.iter() {
            zobrist_hash ^= sq_hash(piece, Side::White, sq);
        }

        for sq in black_pieces.iter() {
            zobrist_hash ^= sq_hash(piece, Side::Black, sq);
        }
    }

    // XOR the zobrist value for the side to move
    zobrist_hash ^= side_hash(board.side_to_move());

    // XOR the zobrist values for castling rights
    zobrist_hash ^= castling_hash(board.castling_rights());

    // XOR the zobrist value for the en passant square, if any
    zobrist_hash ^= ep_hash(board.en_passant_square());

    zobrist_hash
}

pub(crate) fn get_pawn_hash(board: &Board) -> u64 {
    let mut hash = 0;
    for sq in board.piece_kind_bitboard(Piece::Pawn).iter() {
        if let Some(side) = board.color_on(sq) {
            hash ^= sq_hash(Piece::Pawn, side, sq);
        }
    }
    hash
}
