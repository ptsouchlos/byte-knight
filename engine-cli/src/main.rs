// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

mod bench;
mod input_handler;
mod perft;
mod uci_handler;

use crate::uci_handler::UciHandler;
use chess::definitions::DEFAULT_FEN;
use clap::{Parser, Subcommand};
use engine::defs::About;

shadow_rs::shadow!(build);

#[derive(Parser)]
#[command(
    version = build::CLAP_LONG_VERSION, about = About::SHORT_DESCRIPTION, long_about = About::SHORT_DESCRIPTION
)]
struct Options {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
#[command(about = "Available commands")]
enum Command {
    #[command(about = "Run fixed depth search")]
    Bench {
        #[arg(short, long, default_value = "8")]
        depth: u8,

        #[arg(short, long)]
        epd_file: Option<String>,
    },
    Perft {
        #[arg(short, long, default_value_t = 6)]
        depth: usize,
        #[arg(
            short,
            long,
            default_value_t = DEFAULT_FEN.to_string()
        )]
        fen: String,
        #[arg(short, long)]
        epd_file: Option<String>,
    },
    SplitPerft {
        #[arg(short, long, default_value_t = 6)]
        depth: usize,
        #[arg(
            short,
            long,
            default_value_t = DEFAULT_FEN.to_string()
        )]
        fen: String,
        #[arg(short, long, default_value_t = false)]
        print_moves: bool,
    },
}

fn run_uci() {
    // Spawn UCI handler on a thread with 8 MiB stack — the search is deeply recursive
    // and the default main thread stack size is insufficient on some platforms.
    let handle = std::thread::Builder::new()
        .name("bk-main".to_string())
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let mut handler = UciHandler::new();
            let result = handler.run();
            if let Err(e) = result {
                eprintln!("Error running engine: {e}");
            }
        })
        .unwrap();
    handle.join().unwrap();
}

fn main() {
    let args = Options::parse();
    match args.command {
        Some(command) => match command {
            Command::Bench { depth, epd_file } => {
                // Spawn bench on a thread with 8 MiB stack to match the UCI handler.
                let handle = std::thread::Builder::new()
                    .name("bench".to_string())
                    .stack_size(8 * 1024 * 1024)
                    .spawn(move || bench::bench(depth, &epd_file))
                    .unwrap();
                handle.join().unwrap();
            }
            Command::Perft {
                depth,
                fen,
                epd_file,
            } => {
                let board = &mut chess::board::Board::from_fen(&fen).unwrap();
                if let Some(epd) = epd_file {
                    perft::process_epd_file(&epd);
                } else {
                    for i in 1..depth + 1 {
                        let now = std::time::Instant::now();
                        let nodes = chess::perft::perft(board, i, false).unwrap();
                        let elapsed = now.elapsed();
                        let nps = nodes as f64 / elapsed.as_secs_f64();
                        println!(
                            "perft {} = {:>12} {:.2} sec {:>12} nps",
                            i,
                            nodes,
                            elapsed.as_secs_f64(),
                            nps.round()
                        );
                    }
                }
            }
            Command::SplitPerft {
                depth,
                fen,
                print_moves,
            } => {
                println!("running split perft at depth {}", depth);
                let board = &mut chess::board::Board::from_fen(&fen).unwrap();
                let move_results = chess::perft::split_perft(board, depth, print_moves).unwrap();
                for res in &move_results {
                    println!("{}: {}", res.mv.to_long_algebraic(), res.nodes);
                }
                println!();
                // print the total nodes
                println!("{}", move_results.iter().map(|r| r.nodes).sum::<u64>());
            }
        },
        None => run_uci(),
    }
}
