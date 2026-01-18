// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    fs,
    io::{self, BufRead},
    path::Path,
};

use colored::Colorize;
use rayon::prelude::*;

use chess::{
    board::Board,
    definitions::DEFAULT_FEN,
    move_generation::MoveGenerator,
    perft::{self},
};
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 6)]
    depth: usize,
    #[arg(
        short,
        long,
        default_value_t = DEFAULT_FEN.to_string()
    )]
    fen: String,
    #[arg(short, long)]
    split_perft: bool,

    #[arg(short, long, default_value_t = false)]
    print_moves: bool,

    #[arg(short, long)]
    epd_file: Option<String>,
}

fn read_lines<P>(filename: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let file = fs::File::open(filename)?;
    let reader = io::BufReader::new(file);
    Ok(reader.lines().map(|l| l.unwrap()).collect())
}

/// Process an EPD file and run perft tests on each position.
/// This function assumes the EPD file has fen strings followed by perft information like "D1 20; D2 400; D3 8902".
/// See also the `stardard.epd` file in the data directory of this project.
///
/// # Arguments
/// - `path` - The path to the EPD file.
/// - `move_generation` - The move generator to use for perft calculations.
pub(crate) fn process_epd_file(path: &str, move_generation: &MoveGenerator) {
    let mut all_failures = Vec::new();
    let lines = read_lines(path).unwrap();
    let now = std::time::Instant::now();
    lines
        .par_iter()
        .map(|line| {
            let parts: Vec<&str> = line.split(';').collect();
            let fen = parts[0];
            let mut failures = Vec::new();
            for part in parts.iter().skip(1) {
                let parts = part.split_whitespace().collect::<Vec<&str>>();
                let depth = parts[0].replace('D', "").parse::<usize>().unwrap();
                let expected_nodes = parts[1].parse::<u64>().unwrap();
                let mut board = Board::from_fen(fen).unwrap();
                let nodes = perft::perft(&mut board, move_generation, depth, false).unwrap();
                if expected_nodes != nodes {
                    print!("{} ", "[FAIL]".red().bold());
                    println!("{fen:<30}: {depth:2} {expected_nodes:^10} != {nodes:^10}");
                    failures.push((fen.to_string(), depth, expected_nodes, nodes));
                } else {
                    print!("{} ", "[PASS]".green());
                    println!("{fen:<30}: {depth:2} {expected_nodes:^10} == {nodes:^10}",);
                }
            }

            failures
        })
        .collect_into_vec(&mut all_failures);
    let elapsed = now.elapsed();

    println!(
        "Summary:\n\t{} failed\n\t{:.2} seconds",
        all_failures.iter().map(|f| f.len()).sum::<usize>(),
        elapsed.as_secs_f64()
    );

    for (fen, depth, expected, actual) in all_failures.iter().flatten() {
        println!("{fen:<30}: {depth:2} {expected:^10} != {actual:^10}",);
    }
}
