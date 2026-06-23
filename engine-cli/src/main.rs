// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

mod commands;
mod input_handler;
mod perft;
mod uci_handler;

use std::backtrace::Backtrace;
use std::io::Write;

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

fn install_crash_logger() {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let backtrace = Backtrace::force_capture();
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");

        // Save a file with the panic message and backtrace, but we cannot panic here, so we ignore any errors.
        // Also save a unique file for each process so the files don't overlap if multiple instances are running and crash.

        let path =
            std::env::temp_dir().join(format!("byte-knight-crash-{}.log", std::process::id()));
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            // Ignore all write errors, no panics
            let _ = writeln!(file, "version: {}", build::CLAP_LONG_VERSION);
            let _ = writeln!(file, "thread: {thread_name}");
            let _ = writeln!(file, "{info}");
            let _ = writeln!(file, "backtrace:\n{backtrace}");
            // Ensure file is flushed to disk
            let _ = file.flush();
        }

        // Still emit panic message to stderr
        default_hook(info);
    }));
}

fn main() {
    // Install a panic hook to log crashes to a file in the temp directory.
    install_crash_logger();

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
