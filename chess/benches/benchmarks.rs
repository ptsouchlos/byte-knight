use criterion::{criterion_group, criterion_main};

use crate::{move_gen::move_gen_benchmark, sliding_piece_moves::sliding_piece_benchmark};

mod move_gen;
mod sliding_piece_moves;

criterion_group!(attacks, sliding_piece_benchmark);
criterion_group!(move_gen, move_gen_benchmark);
criterion_main!(attacks, move_gen);
