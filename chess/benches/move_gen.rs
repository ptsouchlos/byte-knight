use chess::{board::Board, move_generation};
use criterion::Criterion;

pub fn move_gen_benchmark(c: &mut Criterion) {
    let board = Board::from_fen("R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1")
        .expect("Invalid FEN");

    c.bench_function("move-gen", |b| {
        b.iter(|| {
            let _moves = move_generation::generate_legal_moves(&board, chess::moves::MoveType::All);
        });
    });
}
