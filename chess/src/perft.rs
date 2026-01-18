// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{board::Board, move_generation::MoveGenerator, move_list::MoveList, moves::Move};
use anyhow::{Result, bail};

pub struct SplitPerftResult {
    pub mv: Move,
    pub nodes: u64,
}

/// Perform split perft on the given board with the given move generator and depth.
///
/// # Arguments
///
/// - `board` - The board to perform perft on.
/// - `move_gen` - The move generator to use.
/// - `depth` - The depth to perform perft to.
/// - `print_moves` - If true, extra debug information will be printed.
///
/// # Returns
///
/// A vector of `SplitPerftResult` containing the move and the number of nodes at that move.
#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(debug_assertions, inline(never))]
pub fn split_perft(
    board: &mut Board,
    move_gen: &MoveGenerator,
    depth: usize,
    print_moves: bool,
) -> Result<Vec<SplitPerftResult>> {
    let mut move_list = MoveList::new();
    move_gen.generate_legal_moves(board, &mut move_list);

    let mut results = Vec::new();
    for mv in move_list.iter() {
        // first ensure the move is legal
        if print_moves {
            println!("mv - {}", mv.to_long_algebraic());
        }
        let move_res = board.make_move_unchecked(mv);
        if move_res.is_err() {
            bail!("move failed: {:?}", move_res);
        }
        if print_moves {
            println!("node - {}", mv.to_long_algebraic());
        }

        let nodes = if depth > 1 {
            perft(board, move_gen, depth - 1, print_moves)?
        } else {
            1
        };
        if print_moves {
            println!("---");
        }
        board.unmake_move()?;
        results.push(SplitPerftResult { mv: *mv, nodes });
    }

    // sort results alphabetically
    results.sort_by(|a, b| a.mv.to_long_algebraic().cmp(&b.mv.to_long_algebraic()));

    Ok(results)
}

/// Perform perft on the given board with the given move generator and depth.
#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(debug_assertions, inline(never))]
pub fn perft(
    board: &mut Board,
    move_gen: &MoveGenerator,
    depth: usize,
    print_moves: bool,
) -> Result<u64> {
    let mut nodes = 0;
    let mut move_list = MoveList::new();
    move_gen.generate_legal_moves(board, &mut move_list);

    if print_moves {
        for mv in move_list.iter() {
            println!("{mv}");
        }
    }

    if depth == 1 {
        // bulk counting
        return Ok(move_list.len() as u64);
    }

    for mv in move_list.iter() {
        let result = board.make_move_unchecked(mv);
        // this is unexpected as we are generating legal moves
        // if this happens, it is likely a bug in the move generator
        if result.is_err() {
            println!("current board: {}", board.to_fen());
            println!("current move: {mv}");
            bail!("move failed ({}): {:?}", depth, result);
        }
        nodes += perft(board, move_gen, depth - 1, print_moves)?;
        board.unmake_move()?;
    }

    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use crate::side::Side;

    use super::*;

    #[test]
    fn default_board() {
        let mut board = Board::default_board();
        let move_gen = MoveGenerator::new();
        assert!(board.en_passant_square().is_none());
        let result = perft(&mut board, &move_gen, 1, false).unwrap();
        assert_eq!(result, 20);
    }

    #[test]
    fn single_depth_non_standard_positions() {
        // test positions taken from https://gist.github.com/peterellisjones/8c46c28141c162d1d8a0f0badbc9cff9
        let move_gen = MoveGenerator::new();
        {
            let mut board = Board::from_fen("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b KQ - 3 2").unwrap();
            assert_eq!(board.side_to_move(), Side::Black);
            assert!(board.is_in_check(&move_gen));
            let total_moves = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves, 8);
        }

        {
            let mut board = Board::from_fen("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 0 3").unwrap();
            assert_eq!(board.side_to_move(), Side::Black);
            assert!(board.is_in_check(&move_gen));
            assert!(board.en_passant_square().is_some());
            let total_moves = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves, 8);
        }

        {
            let mut board =
                Board::from_fen("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 2 2")
                    .unwrap();
            let total_moves = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves, 19);
        }

        {
            let mut board = Board::from_fen(
                "r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQkq - 3 2",
            )
            .unwrap();
            let total_moves = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves, 5);
        }

        {
            let mut board =
                Board::from_fen("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b KQ - 3 2")
                    .unwrap();
            let total_moves = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves, 44);
        }

        {
            let mut board =
                Board::from_fen("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w KQ - 3 9")
                    .unwrap();
            let total_moves = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves, 39);
        }

        {
            let mut board = Board::from_fen("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4").unwrap();
            let total_moves = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves, 9);
        }

        {
            let mut board =
                Board::from_fen("rnQq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQKR1r b Q - 1 8")
                    .unwrap();
            let total_moves = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves, 32);
        }
    }

    #[test]
    fn multi_depth_non_standard_positions() {
        let move_gen = MoveGenerator::new();

        {
            let mut board = Board::from_fen("8/8/2k5/KpPr4/8/8/8/8 w - b6 0 1").unwrap();
            let mut total_moves = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves, 2);
            total_moves = perft(&mut board, &move_gen, 2, false).unwrap();
            assert_eq!(total_moves, 31);
        }

        {
            let mut board =
                Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8")
                    .unwrap();
            let total_moves_depth1 = perft(&mut board, &move_gen, 1, false).unwrap();
            assert_eq!(total_moves_depth1, 44);
            let total_moves_depth2 = perft(&mut board, &move_gen, 2, false).unwrap();
            assert_eq!(total_moves_depth2, 1486);
            let total_moves = perft(&mut board, &move_gen, 3, false).unwrap();
            assert_eq!(total_moves, 62379);
        }

        {
            let mut board = Board::from_fen("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 6, false).unwrap();
            assert_eq!(total_moves, 1134888);
        }

        {
            let mut board = Board::from_fen(
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            )
            .unwrap();
            let total_moves = perft(&mut board, &move_gen, 3, false).unwrap();
            assert_eq!(total_moves, 89890);
        }

        {
            let mut board = Board::from_fen("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 6, false).unwrap();
            assert_eq!(total_moves, 1015133);
        }

        {
            let mut board = Board::from_fen("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 6, false).unwrap();
            assert_eq!(total_moves, 1440467);
        }

        {
            let mut board = Board::from_fen("5k2/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 6, false).unwrap();
            assert_eq!(total_moves, 661072);
        }

        {
            let mut board = Board::from_fen("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 6, false).unwrap();
            assert_eq!(total_moves, 803711);
        }

        {
            let mut board = Board::from_fen("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 4, false).unwrap();
            assert_eq!(total_moves, 1274206);
        }

        {
            let mut board = Board::from_fen("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 4, false).unwrap();
            assert_eq!(total_moves, 1720476);
        }

        {
            let mut board = Board::from_fen("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 6, false).unwrap();
            assert_eq!(total_moves, 3821001);
        }

        {
            let mut board = Board::from_fen("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 5, false).unwrap();
            assert_eq!(total_moves, 1004658);
        }

        {
            let mut board = Board::from_fen("4k3/1P6/8/8/8/8/K7/8 w - - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 6, false).unwrap();
            assert_eq!(total_moves, 217342);
        }

        {
            let mut board = Board::from_fen("8/P1k5/K7/8/8/8/8/8 w - - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 6, false).unwrap();
            assert_eq!(total_moves, 92683);
        }

        {
            let mut board = Board::from_fen("K1k5/8/P7/8/8/8/8/8 w - - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 6, false).unwrap();
            assert_eq!(total_moves, 2217);
        }

        {
            let mut board = Board::from_fen("8/k1P5/8/1K6/8/8/8/8 w - - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 7, false).unwrap();
            assert_eq!(total_moves, 567584);
        }

        {
            let mut board = Board::from_fen("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1").unwrap();
            let total_moves = perft(&mut board, &move_gen, 4, false).unwrap();
            assert_eq!(total_moves, 23527);
        }
        {
            let mut board =
                Board::from_fen("6r1/2q2pp1/1PB2k2/3P1P2/5Q1B/8/6K1/7R b - - 0 56").unwrap();
            let depths = [1, 2, 3, 4, 5, 6, 7];
            let move_counts = [1, 48, 1060, 42723, 981168, 37765954, 891192699];
            for (depth, count) in depths.iter().zip(move_counts.iter()) {
                let total_moves = perft(&mut board, &move_gen, *depth, false).unwrap();
                assert_eq!(total_moves, *count);
            }
        }
    }

    /// Helper to run the EPD tests below
    fn run_epd_test(tests: &[(&str, Vec<i64>)], move_gen: &MoveGenerator) {
        for (fen, results) in tests.iter() {
            println!("{fen}");
            let mut board = Board::from_fen(fen).unwrap();
            for (idx, result) in results.iter().enumerate() {
                let nodes = perft(&mut board, move_gen, idx + 1, false).unwrap();
                assert_eq!(nodes, *result as u64);
            }
        }
    }

    // tests below are from https://github.com/kz04px/rawr/blob/master/tests/perft_extra.rs
    ////////////////////////////////////////////////////////////////////////////
    #[test]
    fn en_passant() {
        let tests = [
            // EP
            ("8/8/8/8/1k1PpN1R/8/8/4K3 b - d3 0 1", vec![9, 193]),
            ("8/8/8/8/1k1Ppn1R/8/8/4K3 b - d3 0 1", vec![17, 220]),
            ("4k3/8/8/2PpP3/8/8/8/4K3 w - d6 0 1", vec![9, 47, 376]),
            ("4k3/8/8/8/2pPp3/8/8/4K3 b - d3 0 1", vec![9, 47, 376]),
            // EP - pinned diagonal
            ("4k3/b7/8/2Pp4/8/8/8/6K1 w - d6 0 1", vec![5, 45]),
            ("4k3/7b/8/4pP2/8/8/8/1K6 w - e6 0 1", vec![5, 45]),
            ("6k1/8/8/8/2pP4/8/B7/3K4 b - d3 0 1", vec![5, 45]),
            ("1k6/8/8/8/4Pp2/8/7B/4K3 b - e3 0 1", vec![5, 45]),
            ("4k3/b7/8/1pP5/8/8/8/6K1 w - b6 0 1", vec![6, 52]),
            ("4k3/7b/8/5Pp1/8/8/8/1K6 w - g6 0 1", vec![6, 51]),
            ("6k1/8/8/8/1Pp5/8/B7/4K3 b - b3 0 1", vec![6, 52]),
            ("1k6/8/8/8/5pP1/8/7B/4K3 b - g3 0 1", vec![6, 51]),
            ("4k3/K7/8/1pP5/8/8/8/6b1 w - b6 0 1", vec![6, 66]),
            ("4k3/7K/8/5Pp1/8/8/8/1b6 w - g6 0 1", vec![6, 60]),
            ("6B1/8/8/8/1Pp5/8/k7/4K3 b - b3 0 1", vec![6, 66]),
            ("1B6/8/8/8/5pP1/8/7k/4K3 b - g3 0 1", vec![6, 60]),
            ("4k3/b7/8/2Pp4/3K4/8/8/8 w - d6 0 1", vec![5, 44]),
            ("4k3/8/1b6/2Pp4/3K4/8/8/8 w - d6 0 1", vec![6, 59]),
            ("4k3/8/b7/1Pp5/2K5/8/8/8 w - c6 0 1", vec![6, 49]),
            ("4k3/8/7b/5pP1/5K2/8/8/8 w - f6 0 1", vec![6, 49]),
            ("4k3/7b/8/4pP2/4K3/8/8/8 w - e6 0 1", vec![5, 44]),
            ("4k3/8/6b1/4pP2/4K3/8/8/8 w - e6 0 1", vec![6, 53]),
            ("4k3/8/3K4/1pP5/8/q7/8/8 w - b6 0 1", vec![5, 114]),
            ("7k/4K3/8/1pP5/8/q7/8/8 w - b6 0 1", vec![8, 171]),
            // EP - double check
            ("4k3/2rn4/8/2K1pP2/8/8/8/8 w - e6 0 1", vec![4, 75]),
            // EP - pinned horizontal
            ("4k3/8/8/K2pP2r/8/8/8/8 w - d6 0 1", vec![6, 94]),
            ("4k3/8/8/K2pP2q/8/8/8/8 w - d6 0 1", vec![6, 130]),
            ("4k3/8/8/r2pP2K/8/8/8/8 w - d6 0 1", vec![6, 87]),
            ("4k3/8/8/q2pP2K/8/8/8/8 w - d6 0 1", vec![6, 129]),
            ("8/8/8/8/1k1Pp2R/8/8/4K3 b - d3 0 1", vec![8, 125]),
            ("8/8/8/8/1R1Pp2k/8/8/4K3 b - d3 0 1", vec![6, 87]),
            // EP - pinned vertical
            ("k7/8/4r3/3pP3/8/8/8/4K3 w - d6 0 1", vec![5, 70]),
            ("k3K3/8/8/3pP3/8/8/8/4r3 w - d6 0 1", vec![6, 91]),
            // EP - in check
            ("4k3/8/8/4pP2/3K4/8/8/8 w - e6 0 1", vec![9, 49]),
            ("8/8/8/4k3/5Pp1/8/8/3K4 b - f3 0 1", vec![9, 50]),
            // EP - block check
            ("4k3/8/K6r/3pP3/8/8/8/8 w - d6 0 1", vec![6, 109]),
            ("4k3/8/K6q/3pP3/8/8/8/8 w - d6 0 1", vec![6, 151]),
        ];
        let move_gen = &MoveGenerator::new();
        run_epd_test(&tests, move_gen);
    }

    #[test]
    fn double_checked() {
        let tests = [
            ("4k3/8/4r3/8/8/8/3p4/4K3 w - - 0 1", vec![4, 80, 320]),
            ("4k3/8/4q3/8/8/8/3b4/4K3 w - - 0 1", vec![4, 143, 496]),
        ];

        let move_gen = &MoveGenerator::new();
        run_epd_test(&tests, move_gen);
    }

    #[test]
    fn pins() {
        let tests = [
            ("4k3/8/8/8/1b5b/8/3Q4/4K3 w - - 0 1", vec![3, 54, 1256]),
            ("4k3/8/8/8/1b5b/8/3R4/4K3 w - - 0 1", vec![3, 54, 836]),
            ("4k3/8/8/8/1b5b/2Q5/5P2/4K3 w - - 0 1", vec![6, 98, 2274]),
            ("4k3/8/8/8/1b5b/2R5/5P2/4K3 w - - 0 1", vec![4, 72, 1300]),
            ("4k3/8/8/8/1b2r3/8/3Q4/4K3 w - - 0 1", vec![3, 66, 1390]),
            ("4k3/8/8/8/1b2r3/8/3QP3/4K3 w - - 0 1", vec![6, 119, 2074]),
        ];
        let move_gen = &MoveGenerator::new();
        run_epd_test(&tests, move_gen);
    }
    ////////////////////////////////////////////////////////////////////////////

    // taken from the rustic engine: https://github.com/mvanthoor/rustic/blob/eb5284dbb41d171fa6bf023f262f94240dcbe66d/src/extra/epds.rs
    ////////////////////////////////////////////////////////////////////////////
    #[test]
    fn rustic_deep_perft() {
        let tests = [
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                vec![20, 400, 8902, 197281, 4865609, 119060324],
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",
                vec![20, 400, 8902, 197281, 4865609, 119060324],
            ),
            (
                "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                vec![6, 264, 9467, 422333, 15833292, 706045033],
            ),
            (
                "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
                vec![46, 2079, 89890, 3894594, 164075551, 6923051137],
            ),
        ];

        let move_gen = &MoveGenerator::new();
        run_epd_test(&tests, move_gen);
    }

    #[test]
    fn rustic_checks_and_stalemate() {
        let tests = [
            // discovered check
            (
                "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1",
                vec![29, 165, 5160, 31961, 1004658],
            ),
            (
                "5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1",
                vec![29, 165, 5160, 31961, 1004658],
            ),
            // "# self stalemate
            (
                "K1k5/8/P7/8/8/8/8/8 w - - 0 1",
                vec![2, 6, 13, 63, 382, 2217],
            ),
            (
                "8/8/8/8/8/p7/8/k1K5 b - - 0 1",
                vec![2, 6, 13, 63, 382, 2217],
            ),
            // stalemate/checkmate:
            (
                "8/k1P5/8/1K6/8/8/8/8 w - - 0 1",
                vec![10, 25, 268, 926, 10857, 43261, 567584],
            ),
            (
                "8/8/8/8/1k6/8/K1p5/8 b - - 0 1",
                vec![10, 25, 268, 926, 10857, 43261, 567584],
            ),
            // double check
            (
                "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1",
                vec![37, 183, 6559, 23527],
            ),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn rustic_promote() {
        let tests = [
            //  promote out of check
            (
                "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1",
                vec![11, 133, 1442, 19174, 266199, 3821001],
            ),
            (
                "3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1",
                vec![11, 133, 1442, 19174, 266199, 3821001],
            ),
            // promote to give check
            (
                "4k3/1P6/8/8/8/8/K7/8 w - - 0 1",
                vec![9, 40, 472, 2661, 38983, 217342],
            ),
            (
                "8/k7/8/8/8/8/1p6/4K3 b - - 0 1",
                vec![9, 40, 472, 2661, 38983, 217342],
            ),
            // "# underpromote to check
            (
                "8/P1k5/K7/8/8/8/8/8 w - - 0 1",
                vec![6, 27, 273, 1329, 18135, 92683],
            ),
            (
                "8/8/8/8/8/k7/p1K5/8 b - - 0 1",
                vec![6, 27, 273, 1329, 18135, 92683],
            ),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn rustic_en_passant() {
        let tests = [
            // en passant capture checks opponent
            (
                "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1",
                vec![15, 126, 1928, 13931, 206379, 1440467],
            ),
            (
                "8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1",
                vec![15, 126, 1928, 13931, 206379, 1440467],
            ),
            // avoid illegal ep(thanks to Steve Maughan)
            (
                "3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1",
                vec![18, 92, 1670, 10138, 185429, 1134888],
            ),
            (
                "8/8/8/8/k1p4R/8/3P4/3K4 w - - 0 1",
                vec![18, 92, 1670, 10138, 185429, 1134888],
            ),
            // avoid illegal ep #2
            (
                "8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1",
                vec![13, 102, 1266, 10276, 135655, 1015133],
            ),
            (
                "8/b2p2k1/8/2P5/8/4K3/8/8 b - - 0 1",
                vec![13, 102, 1266, 10276, 135655, 1015133],
            ),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn rustic_castling_checks() {
        let tests = [
            // short castling gives check
            (
                "5k2/8/8/8/8/8/8/4K2R w K - 0 1",
                vec![15, 66, 1198, 6399, 120330, 661072],
            ),
            (
                "4k2r/8/8/8/8/8/8/5K2 b k - 0 1",
                vec![15, 66, 1198, 6399, 120330, 661072],
            ),
            // long castling gives check
            (
                "3k4/8/8/8/8/8/8/R3K3 w Q - 0 1",
                vec![16, 71, 1286, 7418, 141077, 803711],
            ),
            (
                "r3k3/8/8/8/8/8/8/3K4 b q - 0 1",
                vec![16, 71, 1286, 7418, 141077, 803711],
            ),
            // castling(including losing cr due to rook capture)
            (
                "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1",
                vec![26, 1141, 27826, 1274206],
            ),
            (
                "r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1",
                vec![26, 1141, 27826, 1274206],
            ),
            // castling prevented
            (
                "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1",
                vec![44, 1494, 50509, 1720476],
            ),
            (
                "r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1",
                vec![44, 1494, 50509, 1720476],
            ),
            // short castling impossible although the rook never moved away from its corner
            (
                "1k6/1b6/8/8/7R/8/8/4K2R b K - 0 1",
                vec![13, 284, 3529, 85765, 1063513],
            ),
            (
                "4k2r/8/8/7r/8/8/1B6/1K6 w k - 0 1",
                vec![13, 284, 3529, 85765, 1063513],
            ),
            // long castling impossible although the rook never moved away from its corner
            (
                "1k6/8/8/8/R7/1n6/8/R3K3 b Q - 0 1",
                vec![9, 193, 1676, 38751, 346695],
            ),
            (
                "r3k3/8/1N6/r7/8/8/8/1K6 w q - 0 1",
                vec![9, 193, 1676, 38751, 346695],
            ),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }
    ////////////////////////////////////////////////////////////////////////////

    // taken from the kz04px engine: https://github.com/kz04px/libchess/blob/master/tests/perft.cpp
    ////////////////////////////////////////////////////////////////////////////

    #[test]
    fn kz_diagonal_pin() {
        let tests = [
            ("4k3/b7/8/2Pp4/8/8/8/6K1 w - d6 0 2", vec![5]),
            ("4k3/7b/8/4pP2/8/8/8/1K6 w - e6 0 2", vec![5]),
            ("6k1/8/8/8/2pP4/8/B7/3K4 b - d3 0 2", vec![5]),
            ("1k6/8/8/8/4Pp2/8/7B/4K3 b - e3 0 2", vec![5]),
            ("4k3/b7/8/1pP5/8/8/8/6K1 w - b6 0 2", vec![6]),
            ("4k3/7b/8/5Pp1/8/8/8/1K6 w - g6 0 2", vec![6]),
            ("6k1/8/8/8/1Pp5/8/B7/4K3 b - b3 0 2", vec![6]),
            ("1k6/8/8/8/5pP1/8/7B/4K3 b - g3 0 2", vec![6]),
            ("4k3/K7/8/1pP5/8/8/8/6b1 w - b6 0 2", vec![6]),
            ("4k3/7K/8/5Pp1/8/8/8/1b6 w - g6 0 2", vec![6]),
            ("6B1/8/8/8/1Pp5/8/k7/4K3 b - b3 0 2", vec![6]),
            ("1B6/8/8/8/5pP1/8/7k/4K3 b - g3 0 2", vec![6]),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn kz_en_passant_horizontal_pin() {
        let tests = [
            ("4k3/8/8/K2pP2r/8/8/8/8 w - d6 0 1", vec![6]),
            ("4k3/8/8/r2pP2K/8/8/8/8 w - d6 0 1", vec![6]),
            ("8/8/8/8/1k1Pp2R/8/8/4K3 b - d3 0 1", vec![8]),
            ("8/8/8/8/1R1Pp2k/8/8/4K3 b - d3 0 1", vec![6]),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn kz_en_passant_vertical_pin() {
        let tests = [
            ("k7/8/4r3/3pP3/8/8/8/4K3 w - d6 0 1", vec![5]),
            ("k3K3/8/8/3pP3/8/8/8/4r3 w - d6 0 1", vec![6]),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn kz_legal_en_passant() {
        let tests = [
            ("8/8/8/8/1k1PpN1R/8/8/4K3 b - d3 0 1", vec![9, 193, 1322]),
            ("8/8/8/8/1k1Ppn1R/8/8/4K3 b - d3 0 1", vec![17, 220, 3001]),
            ("4k3/8/8/2PpP3/8/8/8/4K3 w - d6 0 1", vec![9, 47, 376]),
            ("4k3/8/8/8/2pPp3/8/8/4K3 b - d3 0 1", vec![9, 47, 376]),
            (
                "r3k2r/p2pqpb1/bn2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBBPPP/R4K1R w kq c6 0 2",
                vec![46],
            ),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn kz_en_passant_capture_checker() {
        let tests = [
            ("4k3/8/8/4pP2/3K4/8/8/8 w - e6 0 2", vec![9]),
            ("8/8/8/4k3/5Pp1/8/8/3K4 b - f3 0 1", vec![9]),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn kz_en_passant_in_check() {
        let tests = [
            ("2b1k3/8/8/2Pp4/8/7K/8/8 w - - 0 1", vec![4, 52]),
            ("2b1k3/8/8/2Pp4/8/7K/8/8 w - d6 0 1", vec![4, 52]),
            ("4k3/r6K/8/2Pp4/8/8/8/8 w - - 0 1", vec![4, 77]),
            ("4k3/r6K/8/2Pp4/8/8/8/8 w - d6 0 1", vec![4, 77]),
            ("2K1k3/8/8/2Pp4/8/7b/8/8 w - - 0 1", vec![3, 37]),
            ("2K1k3/8/8/2Pp4/8/7b/8/8 w - d6 0 1", vec![3, 37]),
            ("2K1k3/8/8/2Pp4/8/7q/8/8 w - - 0 1", vec![3, 79]),
            ("2K1k3/8/8/2Pp4/8/7q/8/8 w - d6 0 1", vec![3, 79]),
            ("8/8/7k/8/2pP4/8/8/2B1K3 b - - 0 1", vec![4, 52]),
            ("8/8/7k/8/2pP4/8/8/2B1K3 b - d3 0 1", vec![4, 52]),
            ("8/8/7k/8/2pP4/8/8/2Q1K3 b - - 0 1", vec![4, 76]),
            ("8/8/7k/8/2pP4/8/8/2Q1K3 b - d3 0 1", vec![4, 76]),
            ("8/8/8/8/2pP4/8/R6k/4K3 b - - 0 1", vec![4, 77]),
            ("8/8/8/8/2pP4/8/R6k/4K3 b - d3 0 1", vec![4, 77]),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn kz_en_passant_block_check() {
        let tests = [
            ("4k3/8/K6r/3pP3/8/8/8/8 w - d6 0 1", vec![6, 109]),
            ("4k3/8/K6q/3pP3/8/8/8/8 w - d6 0 1", vec![6, 151]),
            ("4kb2/8/8/3pP3/8/K7/8/8 w - d6 0 1", vec![5, 55]),
            ("4kq2/8/8/3pP3/8/K7/8/8 w - d6 0 1", vec![5, 100]),
            ("4k3/8/r6K/3pP3/8/8/8/8 w - d6 0 1", vec![6, 107]),
            ("4k3/8/q6K/3pP3/8/8/8/8 w - d6 0 1", vec![6, 149]),
            ("3k1K2/8/8/3pP3/8/b7/8/8 w - d6 0 1", vec![4, 44]),
            ("3k1K2/8/8/3pP3/8/q7/8/8 w - d6 0 1", vec![4, 100]),
            ("8/8/8/8/3Pp3/k6R/8/4K3 b - d3 0 1", vec![6, 109]),
            ("8/8/8/8/3Pp3/k6Q/8/4K3 b - d3 0 1", vec![6, 151]),
            ("8/8/k7/8/3Pp3/8/8/4KB2 b - d3 0 1", vec![5, 55]),
            ("8/8/k7/8/3Pp3/8/8/4KQ2 b - d3 0 1", vec![5, 100]),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }

    #[test]
    fn kz_many_moves() {
        let board =
            Board::from_fen("R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1").unwrap();
        let move_gen = MoveGenerator::new();
        let mut move_list = MoveList::new();

        move_gen.generate_legal_moves(&board, &mut move_list);
        assert_eq!(move_list.len(), 218);
    }

    #[test]
    fn kz_perft_shallow() {
        let tests = [
            ("4k3/b7/8/2Pp4/8/8/8/6K1 w - d6 0 2", vec![5]),
            ("4k3/7b/8/4pP2/8/8/8/1K6 w - e6 0 2", vec![5]),
            ("6k1/8/8/8/2pP4/8/B7/3K4 b - d3 0 2", vec![5]),
            ("1k6/8/8/8/4Pp2/8/7B/4K3 b - e3 0 2", vec![5]),
            ("4k3/b7/8/1pP5/8/8/8/6K1 w - b6 0 2", vec![6]),
            ("4k3/7b/8/5Pp1/8/8/8/1K6 w - g6 0 2", vec![6]),
            ("6k1/8/8/8/1Pp5/8/B7/4K3 b - b3 0 2", vec![6]),
            ("1k6/8/8/8/5pP1/8/7B/4K3 b - g3 0 2", vec![6]),
            ("4k3/K7/8/1pP5/8/8/8/6b1 w - b6 0 2", vec![6]),
            ("4k3/7K/8/5Pp1/8/8/8/1b6 w - g6 0 2", vec![6]),
            ("6B1/8/8/8/1Pp5/8/k7/4K3 b - b3 0 2", vec![6]),
            ("1B6/8/8/8/5pP1/8/7k/4K3 b - g3 0 2", vec![6]),
            ("4k3/8/8/K2pP2r/8/8/8/8 w - d6 0 1", vec![6]),
            ("4k3/8/8/r2pP2K/8/8/8/8 w - d6 0 1", vec![6]),
            ("8/8/8/8/1k1Pp2R/8/8/4K3 b - d3 0 1", vec![8]),
            ("8/8/8/8/1R1Pp2k/8/8/4K3 b - d3 0 1", vec![6]),
            ("k7/8/4r3/3pP3/8/8/8/4K3 w - d6 0 1", vec![5]),
            ("k3K3/8/8/3pP3/8/8/8/4r3 w - d6 0 1", vec![6]),
            ("8/8/8/8/1k1PpN1R/8/8/4K3 b - d3 0 1", vec![9, 193, 1322]),
            ("8/8/8/8/1k1Ppn1R/8/8/4K3 b - d3 0 1", vec![17, 220, 3001]),
            ("4k3/8/8/2PpP3/8/8/8/4K3 w - d6 0 1", vec![9, 47, 376]),
            ("4k3/8/8/8/2pPp3/8/8/4K3 b - d3 0 1", vec![9, 47, 376]),
            ("4k3/8/8/4pP2/3K4/8/8/8 w - e6 0 2", vec![9]),
            ("8/8/8/4k3/5Pp1/8/8/3K4 b - f3 0 1", vec![9]),
            ("4k3/8/K6r/3pP3/8/8/8/8 w - d6 0 1", vec![6]),
        ];

        let move_gen = MoveGenerator::new();
        run_epd_test(&tests, &move_gen);
    }
}
