// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::hint::black_box;

use chess::board::Board;
use criterion::{Criterion, criterion_group, criterion_main};
use engine::{bench_positions::BENCH_FENS, evaluation::ByteKnightEvaluation, traits::Eval};

/// Benchmarks static evaluation over the standard benchmark position set (the same
/// positions the search bench uses), so eval changes can be measured A/B.
pub fn eval_benchmark(c: &mut Criterion) {
    // Parse the FENs once up front so the benchmark only measures `eval()`.
    let boards: Vec<Board> = BENCH_FENS
        .iter()
        .map(|entry| {
            // Strip any EPD trailer, matching the search bench's FEN handling.
            let fen = entry.split(';').next().expect("empty benchmark entry");
            Board::from_fen(fen).expect("invalid benchmark FEN")
        })
        .collect();

    let eval = ByteKnightEvaluation::default();

    c.bench_function("eval-bench-suite", |b| {
        b.iter(|| {
            for board in &boards {
                black_box(eval.eval(black_box(board)));
            }
        });
    });
}

criterion_group!(eval, eval_benchmark);
criterion_main!(eval);
