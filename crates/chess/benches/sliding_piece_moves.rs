use chess::{attacks, board::Board, pieces::Piece, side::Side};
use criterion::Criterion;

pub fn sliding_piece_benchmark(c: &mut Criterion) {
    let board = Board::from_fen("R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1").unwrap();
    let queen_bb = board.piece_bitboard(Piece::Queen, Side::White);
    let next_queen = queen_bb.into_iter().next().unwrap();
    c.bench_function("queen attacks", |b| {
        b.iter(|| attacks::queen(next_queen, board.all_pieces()))
    });
}
