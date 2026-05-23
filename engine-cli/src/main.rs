// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

mod commands;
mod input_handler;
mod perft;
mod uci_handler;

use crate::{
    commands::{bench::BenchArgs, perft::PerftArgs, split_perft::SplitPerftArgs},
    uci_handler::UciHandler,
};

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
    Bench(BenchArgs),
    Perft(PerftArgs),
    SplitPerft(SplitPerftArgs),
}

/// Run the UCI handler for the engine.
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
            Command::Bench(bench_args) => commands::bench::execute(bench_args),
            Command::Perft(perft_args) => commands::perft::execute(perft_args),
            Command::SplitPerft(split_perft_args) => {
                commands::split_perft::execute(split_perft_args)
            }
        },
        None => run_uci(),
    }
}
