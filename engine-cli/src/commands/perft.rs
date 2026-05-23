// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::definitions::DEFAULT_FEN;

use crate::perft;

#[derive(clap::Args, Debug)]
pub(crate) struct PerftArgs {
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
}

pub(crate) fn execute(args: PerftArgs) {
    let board = &mut chess::board::Board::from_fen(&args.fen).unwrap();
    if let Some(epd) = args.epd_file {
        perft::process_epd_file(&epd);
    } else {
        for i in 1..args.depth + 1 {
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
