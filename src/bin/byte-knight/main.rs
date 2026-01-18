// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

mod bench;
mod perft;

use chess::definitions::DEFAULT_FEN;
use chess::move_generation::MoveGenerator;
use clap::{Parser, Subcommand};
use engine::defs::About;
use engine::engine::ByteKnight;
use std::process::exit;

#[derive(Parser)]
#[command(
    version = About::VERSION, about = About::SHORT_DESCRIPTION, long_about = About::SHORT_DESCRIPTION
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
    let mut engine = ByteKnight::new();
    let engine_run_result = engine.run();
    match engine_run_result {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error running engine: {e}");
            exit(1);
        }
    }
}

fn main() {
    let args = Options::parse();
    match args.command {
        Some(command) => match command {
            Command::Bench { depth, epd_file } => {
                bench::bench(depth, &epd_file);
            }
            Command::Perft {
                depth,
                fen,
                epd_file,
            } => {
                let move_gen = MoveGenerator::new();
                let board = &mut chess::board::Board::from_fen(&fen).unwrap();
                if let Some(epd) = epd_file {
                    perft::process_epd_file(&epd, &move_gen);
                } else {
                    for i in 1..depth + 1 {
                        let now = std::time::Instant::now();
                        let nodes = chess::perft::perft(board, &move_gen, i, false).unwrap();
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
                let move_gen = MoveGenerator::new();
                let board = &mut chess::board::Board::from_fen(&fen).unwrap();
                let move_results =
                    chess::perft::split_perft(board, &move_gen, depth, print_moves).unwrap();
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
