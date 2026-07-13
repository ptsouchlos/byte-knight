// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::io;

use chess::board::Board;
use engine::{
    bench_positions::BENCH_FENS,
    log_level::LogNone,
    search::{Search, limits::SearchLimits},
    thread_data::ThreadData,
};

#[derive(clap::Args, Debug)]
pub(crate) struct BenchArgs {
    #[arg(short, long, default_value = "10")]
    depth: u8,

    #[arg(short, long)]
    epd_file: Option<String>,
}

/// Execute the bench command with the bench arguments.
pub(crate) fn execute(args: BenchArgs) {
    // Spawn bench on a thread with 8 MiB stack to match the UCI handler.
    let handle = std::thread::Builder::new()
        .name("bench".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || run_bench(args.depth, &args.epd_file))
        .unwrap();
    handle.join().unwrap();
}

fn run_bench(depth: u8, epd_file: &Option<String>) {
    let benchmark_strings: Vec<String> = match epd_file {
        Some(file) => {
            let str = std::fs::read_to_string(file).unwrap();
            str.lines().map(|s| s.to_string()).collect()
        }
        None => BENCH_FENS.into_iter().map(|s| s.to_string()).collect(),
    };

    println!(
        "Running fixed depth (d={depth}) search on {} positions.",
        benchmark_strings.len()
    );

    let config = SearchLimits {
        max_depth: depth,
        ..Default::default()
    };

    let mut nodes = 0u64;
    let mut td = ThreadData::from_limits(config);
    let mut sink = io::sink();
    let mut search = Search::<LogNone>::new(&mut sink);

    let max_fen_width = benchmark_strings.iter().map(|s| s.len()).max().unwrap();

    for (idx, bench) in benchmark_strings.iter().enumerate() {
        // Reset params tracked for each search.
        td.reset();

        let fen: &str = bench.split(';').next().unwrap();
        let mut board = Board::from_fen(fen).unwrap();

        let result = search.search(&mut board, &mut td, None);
        nodes += result.nodes;

        println!(
            "{:>2}/{:>2}: {:<max_fen_width$} => {}",
            idx + 1,
            benchmark_strings.len(),
            fen,
            result.nodes
        );
    }
    let elapsed_time = td.time().as_secs_f64();
    let nps = (nodes as f64 / elapsed_time).trunc();
    println!("{nodes} nodes / {elapsed_time}s => {nps} nps");
}
