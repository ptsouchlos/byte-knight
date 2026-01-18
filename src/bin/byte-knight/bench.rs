// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::board::Board;
use engine::{
    log_level::LogNone,
    search::{Search, SearchParameters},
};

const BENCHMARKS: [&str; 56] = [
    // From [Stormphrax](https://github.com/Ciekce/Stormphrax/blob/correct_ep_handling/src/bench.cpp#L29) and
    // [toad](https://github.com/dannyhammer/toad/blob/a5b4a5a5300a15e30e582217b5694f5fa6f2276e/src/utils.rs#L58)
    "q5k1/5ppp/1r3bn1/1B6/P1N2P2/BQ2P1P1/5K1P/8 b - - 2 34",
    "6r1/5k2/p1b1r2p/1pB1p1p1/1Pp3PP/2P1R1K1/2P2P2/3R4 w - - 1 36",
    "r1bq2k1/p4r1p/1pp2pp1/3p4/1P1B3Q/P2B1N2/2P3PP/4R1K1 b - - 2 19",
    "2rr2k1/1p4bp/p1q1p1p1/4Pp1n/2PB4/1PN3P1/P3Q2P/2RR2K1 w - f6 0 20",
    "3br1k1/p1pn3p/1p3n2/5pNq/2P1p3/1PN3PP/P2Q1PB1/4R1K1 w - - 0 23",
    "2r2b2/5p2/5k2/p1r1pP2/P2pB3/1P3P2/K1P3R1/7R w - - 23 93",
    "8/8/1p1kp1p1/p1pr1n1p/P6P/1R4P1/1P3PK1/1R6 b - - 15 45",
    "8/8/1p1k2p1/p1prp2p/P2n3P/6P1/1P1R1PK1/4R3 b - - 5 49",
    "8/8/1p4p1/p1p2k1p/P2npP1P/4K1P1/1P6/3R4 w - - 6 54",
    "8/8/1p4p1/p1p2k1p/P2n1P1P/4K1P1/1P6/6R1 b - - 6 59",
    "8/5k2/1p4p1/p1pK3p/P2n1P1P/6P1/1P6/4R3 b - - 14 63",
    "8/1R6/1p1K1kp1/p6p/P1p2P1P/6P1/1Pn5/8 w - - 0 67",
    // 218 legal moves available
    "R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1",
    "r6r/3q4/1q4q1/4q3/2q4q/q4q2/PP1q4/Kbnn1kb1 b - - 0 1",
    // Checkmated positions
    "4k3/4Q3/4K3/8/8/8/8/8 b - - 0 1",
    "8/8/8/8/8/2k5/1q6/K7 w - - 0 1",
    // Stalemated positions
    "K7/8/kq6/8/8/8/8/8 w - - 0 1",
    "8/8/8/8/8/5QK1/8/6k1 b - - 0 1",
    // 3-fold repetition; best move here is c2c1
    "7k/2QQ4/8/8/8/PPP5/2q5/K7 b - - 0 1",
    // 50 move rule; best move here is h2h3
    "7k/8/R7/1R6/7K/8/7P/8 w - - 99 1",
    // A stalemate is better than losing; best move here is a1a7
    "k5q1/p7/8/6q1/6q1/6q1/8/Q6K w - - 0 1",
    // Under-promotion; f2f1n is best
    "8/2n5/1b6/8/4b1k1/8/5p1K/8 b - - 0 1",
    // Zugzwang positions where null moves may fail
    "8/8/p1p5/1p5p/1P5p/8/PPP2K1p/4R1rk w - - 0 1",
    "1q1k4/2Rr4/8/2Q3K1/8/8/8/8 w - - 0 1",
    "7k/5K2/5P1p/3p4/6P1/3p4/8/8 w - - 0 1",
    "8/6B1/p5p1/Pp4kp/1P5r/5P1Q/4q1PK/8 w - - 0 32",
    "8/8/1p1r1k2/p1pPN1p1/P3KnP1/1P6/8/3R4 b - - 0 1",
    // Positions with only 1 legal move available
    "k7/8/4rr2/7r/4K3/7r/8/8 w - - 0 1",
    "k7/8/Q7/K7/8/8/8/8 b - - 0 1",
    // Mate-in-1
    "3k3B/7p/p1Q1p3/2n5/6P1/K3b3/PP5q/R7 w - - 0 1",
    "4bk2/ppp3p1/2np3p/2b5/2B2Bnq/2N5/PP4PP/4RR1K w - - 0 1",
    "r3k1nr/p1p2p1p/2pP4/8/7q/7b/PPPP3P/RNBQ2KR b kq - 0 1",
    // Mate-in-2
    "1B1Q1R2/8/qNrn3p/2p1rp2/Rn3k1K/8/5P2/bbN4B w - - 0 1",
    "1B6/2R2PN1/8/7P/2p1pk2/2Q1pN1P/8/1B5K w - - 0 1",
    "3q4/pp6/6p1/3Pp2k/1Q3p2/4r2P/P5RK/6R1 b - - 0 1",
    // Mate-in-3
    "8/8/8/8/1p1N4/1Bk1K3/3N4/b7 w - - 0 1",
    "5K1k/6R1/8/3b2P1/5p2/p6p/q7/8 w - - 0 1",
    "2q3k1/1p4pp/3R1r2/p2bQ3/P7/1N2B3/1PP3rP/R3K3 b - - 0 1",
    // Mate-in-4
    "8/3p1p2/5Ppp/K2R2bk/4pPrr/6Pp/4B2P/3N4 w - - 0 1",
    "8/5p2/5p1p/5KPk/7p/7P/8/8 w - - 0 1",
    "1r5k/1p1P2b1/p2Q3p/7P/2q5/2B4R/PP2n1r1/1K1R4 b - - 0 1",
    // Mate-in-5
    "6b1/4Kpk1/5r2/8/3B2P1/7R/8/8 w - - 0 1",
    "3k4/2pPp1n1/2K1p2b/1p2P1p1/bP1N1pP1/1p3Pp1/1P4P1/6B1 w - - 0 1",
    "1b6/kPp5/p1P5/R5Rr/P1N1P3/8/6p1/6Kb w - - 0 1",
    "4r1k1/pp6/6p1/3q4/1P1p2Pn/6K1/1B1Q1P2/3B4 b - - 0 1",
    // Mate-in-6
    "1B3Nbb/1r2pn2/Bnp1P3/3kP3/p2PR3/1Pp1P1N1/5K2/8 w - - 0 1",
    "1B4q1/1p6/4prb1/p3pr1p/P2RBkN1/5ppP/3N1RP1/1K6 w - - 0 1",
    "1K1k1BB1/8/4P3/2p1P3/2p4b/8/8/8 w - - 0 1",
    "r3k2r/pp1n1pp1/2p3p1/3p4/Pb1P4/1B2PPqP/1P4P1/R1BQ1R1K b kq - 0 1",
    // Mate-in-7
    "1B2n3/8/2R5/5p2/3kp1n1/4p3/B3K3/b7 w - - 0 1",
    "1B5b/1p1Np3/1Pp5/2P3p1/K2k3p/2N5/2nP1p2/5B2 w - - 0 1",
    "1B5r/8/8/1b6/8/3p2RB/7k/5K2 w - - 0 1",
    "r1b3k1/pp3pp1/5q1p/3pr3/1Q1n4/P1NB4/1P3PPP/R4RK1 b - - 0 1",
    // Mate-in-8
    "1B1k1NRK/1p1BpP1N/P4p1p/4r2q/8/3b4/8/8 w - - 0 1",
    "1B1rN3/p6q/5pp1/PPk4b/R1P1Pp1r/2pP1Bn1/2N1P2p/4KR2 w - - 0 1",
    "4r1k1/8/2p5/2p1Rpq1/6P1/1PP4Q/P5K1/RN1r4 b - - 0 1",
];

pub(crate) fn bench(depth: u8, epd_file: &Option<String>) {
    let benchmark_strings: Vec<String> = match epd_file {
        Some(file) => {
            let str = std::fs::read_to_string(file).unwrap();
            str.lines().map(|s| s.to_string()).collect()
        }
        None => BENCHMARKS.into_iter().map(|s| s.to_string()).collect(),
    };

    println!(
        "Running fixed depth (d={depth}) search on {} positions.",
        benchmark_strings.len()
    );

    let config = SearchParameters {
        max_depth: depth,
        ..Default::default()
    };

    let mut nodes = 0u64;
    let mut tt = Default::default();
    let mut hist = Default::default();
    let mut search = Search::<LogNone>::new(&config, &mut tt, &mut hist);

    let max_fen_width = benchmark_strings.iter().map(|s| s.len()).max().unwrap();

    for (idx, bench) in benchmark_strings.iter().enumerate() {
        let fen: &str = bench.split(';').next().unwrap();
        let mut board = Board::from_fen(fen).unwrap();

        let result = search.search(&mut board, None);
        nodes += result.nodes;

        println!(
            "{:>2}/{:>2}: {:<max_fen_width$} => {}",
            idx + 1,
            benchmark_strings.len(),
            fen,
            result.nodes
        );
    }
    let elapsed_time = config.start_time.elapsed().as_secs_f64();
    let nps = (nodes as f64 / elapsed_time).trunc();
    println!("{nodes} nodes / {elapsed_time}s => {nps} nps");
}
