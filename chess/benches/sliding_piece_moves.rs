use chess::{attacks, bitboard_helpers::next_bit, board::Board, pieces::Piece, side::Side};
use criterion::{Criterion, criterion_group, criterion_main};

pub fn sliding_piece_benchmark(c: &mut Criterion) {
    let board = Board::from_fen("R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1").unwrap();
    let mut queen_bb = *board.piece_bitboard(Piece::Queen, Side::White);
    let next_queen = next_bit(&mut queen_bb);
    c.bench_function("test", |b| {
        b.iter(|| attacks::queen(next_queen as u8, board.all_pieces()))
    });
}

criterion_group!(benches, sliding_piece_benchmark);
criterion_main!(benches);
