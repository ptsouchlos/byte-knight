// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::definitions::DEFAULT_FEN;

#[derive(clap::Args, Debug)]
pub(crate) struct SplitPerftArgs {
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
}

pub(crate) fn execute(args: SplitPerftArgs) {
    println!("running split perft at depth {}", args.depth);
    let board = &mut chess::board::Board::from_fen(&args.fen).unwrap();
    let move_results = chess::perft::split_perft(board, args.depth, args.print_moves).unwrap();
    for res in &move_results {
        println!("{}: {}", res.mv.to_long_algebraic(), res.nodes);
    }
    println!();
    // print the total nodes
    println!("{}", move_results.iter().map(|r| r.nodes).sum::<u64>());
}
