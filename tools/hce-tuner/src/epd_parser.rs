// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use anyhow::{Result, anyhow, bail};
use chess::{board::Board, side::Side};
use engine::{evaluation::Evaluation, traits::Eval};

use crate::{tracing_values::TracingValues, tuning_position::TuningPosition};

pub(crate) fn parse_epd_file(file_path: &str) -> Vec<TuningPosition> {
    let mut positions = Vec::new();
    let file =
        File::open(file_path).unwrap_or_else(|_| panic!("Failed to open file: {}", file_path));
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let pos = parse_epd_line(line.as_str());
        if let Ok(pos) = pos {
            positions.push(pos);
        } else {
            println!("Error processing {line}, {}", pos.err().unwrap());
        }
    }
    positions
}

pub(crate) fn process_epd_line(line: &str) -> Result<(Board, f64)> {
    // find the split point between the FEN and the result
    if line.is_empty() {
        bail!("Empty line")
    }

    let mut replace_pattern = String::default();
    let split_point = if let Some(idx) = line.rfind("ce") {
        replace_pattern = "ce".to_string();
        idx
    } else if let Some(idx) = (1..10).find_map(|num| line.rfind(format!("c{}", num).as_str())) {
        replace_pattern = line.get(idx..idx + 2).unwrap().to_owned();
        idx
    } else {
        line.rfind(' ').unwrap()
    };

    let fen_split_point = if let Some(idx) = line.rfind("ce") {
        idx
    } else if let Some(idx) = (0..10).find_map(|num| line.find(format!("c{}", num).as_str())) {
        idx
    } else {
        split_point
    };

    let fen = &line[..fen_split_point].trim();
    let result = &line[split_point..]
        .replace(replace_pattern.as_str(), "")
        .trim()
        .to_string();

    // EPD result
    let game_result = get_game_result(result)?;

    // FEN
    let board = Board::from_fen(fen)?;

    Ok((board, game_result))
}

pub(crate) fn parse_epd_line(line: &str) -> Result<TuningPosition> {
    if let Ok((board, game_result)) = process_epd_line(line) {
        let tracing = TracingValues::new();
        let eval = Evaluation::new(tracing);
        let _ = eval.eval(&board);
        let (white_indexes, black_indexes, scaled_phase) = eval.into_values().into_trace();

        let is_white_relative = matches!(game_result, 0.0 | 0.5 | 1.0);
        let result = if is_white_relative {
            game_result
        } else {
            match board.side_to_move() {
                Side::White => game_result,
                Side::Black => 1.0 - game_result,
            }
        };

        return Ok(TuningPosition::new(
            white_indexes,
            black_indexes,
            scaled_phase,
            result,
        ));
    }

    bail!("Could not process {line}")
}

/// Parse the game result from part of the EPD line.
/// The game result can be in the following formats:
/// - 0.0
/// - 1.0
/// - 0.5
/// - 1-0
/// - 0-1
/// - 1/2-1/2
/// - [0.75]
/// - 0.75;
/// - [1/2-1/2]
/// - 1/2-1/2;
/// - [1-0]
/// - 1-0;
/// - [0-1]
/// - 0-1;
/// - [draw]
/// - draw;
///
/// The function will return the game result as a f64.
///
/// # Arguments
/// - `part` - A part of the EPD line that contains the game result.
///
/// # Returns
/// A f64 representing the game result. 0.0 for a loss, 0.5 for a draw, and 1.0 for a win.
fn get_game_result(part: &str) -> Result<f64> {
    // first sanitize the string
    let part = part.trim();
    // remove any brackets, braces, parenthesis, semicolons, and double quotes
    let part = part.replace(&['[', ']', '{', '}', '(', ')', ';', '"'][..], "");

    if part.starts_with("draw") || part.starts_with("1/2") {
        Ok(0.5)
    } else if part.starts_with("1-0") {
        Ok(1.0)
    } else if part.starts_with("0-1") {
        Ok(0.0)
    } else {
        // try to parse as f64 directly
        part.parse::<f64>()
            .map_err(|_| anyhow!("Failed to parse game result"))
    }
}

#[cfg(test)]
mod tests {
    use chess::{board::Board, side::Side};
    use engine::{evaluation::ByteKnightEvaluation, hce_values::GAME_PHASE_MAX, traits::Eval};

    use crate::{
        epd_parser::{get_game_result, process_epd_line},
        parameters::Parameters,
        tuning_position::TuningPosition,
    };

    #[test]
    fn game_result() {
        let results = [
            "[0.75]",
            "0.75;",
            "[1/2-1/2]",
            "    1/2-1/2;",
            "[1-0]  ",
            " 1-0;",
            "\"0-1\"",
        ];
        let values = [0.75, 0.75, 0.5, 0.5, 1.0, 1.0, 0.0];
        for (i, &result) in results.iter().enumerate() {
            let game_result = get_game_result(result).unwrap();
            assert_eq!(game_result, values[i]);
        }
    }

    fn test_epd_lines(lines: &[&str]) -> Vec<(TuningPosition, Board, f64)> {
        let mut results = Vec::new();
        for line in lines.iter() {
            let position: Result<TuningPosition, anyhow::Error> = super::parse_epd_line(line);
            assert!(position.is_ok());
            let pos = position.unwrap();
            let (board, result) = process_epd_line(line).unwrap();
            let total_piece_count = board.all_pieces().as_number().count_ones();
            assert!(
                pos.parameter_indexes[Side::White as usize].len()
                    + pos.parameter_indexes[Side::Black as usize].len()
                    >= total_piece_count as usize
            );
            results.push((pos, board, result));
        }
        results
    }

    #[test]
    fn epd_line() {
        let epd_lines = [
            // from lichess big3
            "5r2/p4pk1/2pb4/8/1p2rN2/4p3/PPPB4/3K4 w - - 0 3 [0.0]",
            "r2q1rk1/3n1p2/2pp3p/1pb1p1p1/p3P3/P1NP1N1P/RPP2PP1/5QK1 b - - 0 2 [0.0]",
            "rn2r2k/p1R4p/4bp2/8/1Q6/6P1/1P3P1P/6K1 w - - 0 1 [0.0]",
            "1r4k1/6p1/7p/4p3/R7/3rPNP1/1b3P1P/5RK1 b - - 0 1 [1.0]",
            "1nn3kr/1R1p2pp/5p2/N1p5/3PP3/3B4/P1P2PPP/R5K1 b - - 0 3 [1.0]",
            "6k1/1p2b1pp/p4p2/4pb2/1P1pN3/P2P1P1P/2r3P1/1R3NK1 w - - 0 1 [0.0]",
            "rn1q2k1/ppp2ppp/3p1n2/2bb4/8/5NP1/PPP1NPBP/R4RK1 w - - 0 1 [0.0]",
            "3r1rk1/pR3pbp/2p1pnp1/4q3/2P4P/P3P1P1/2Q2PB1/2B2RK1 b - - 0 4 [0.0]",
            "3b4/5k2/6r1/3pP3/p1pP1p1p/P1P2P1P/1PR3P1/6K1 b - - 0 1 [0.0]",
            "r2q1rk1/ppp1npbp/4b1p1/1P3nN1/2Pp4/3P4/PB1NBPPP/R2QR1K1 b - - 0 1 [0.0]",
        ];

        let mut expected_game_phases: [f64; 10] = [7., 18., 12., 10., 10., 8., 17., 20., 5., 24.];
        for phase in &mut expected_game_phases {
            *phase /= GAME_PHASE_MAX as f64;
        }

        const EXPECTED_GAME_RESULTS: [f64; 10] = [0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let eval = ByteKnightEvaluation::default();
        let params = Parameters::create_from_engine_values();

        let parsed_results = test_epd_lines(&epd_lines);

        for (i, (position, board, result)) in parsed_results.iter().enumerate() {
            assert_eq!(position.phase, expected_game_phases[i]);
            assert_eq!(position.game_result, EXPECTED_GAME_RESULTS[i]);
            assert_eq!(*result, EXPECTED_GAME_RESULTS[i]);
            // also verify that the evaluation matches
            let expected_value = eval.eval(board);

            // tuning position evaluation is always from white's perspective
            let val = match board.side_to_move() {
                Side::White => position.evaluate(&params),
                Side::Black => -position.evaluate(&params),
            };

            println!("{expected_value} // {val}");
            assert!((expected_value.0 as f64 - val).abs().round() <= 1.0)
        }
    }

    #[test]
    fn gedas_epd_data() {
        let epd_lines = [
            "8/8/7p/1P2k2P/4p1P1/1p1r4/1R2K3/8 b - - ce 0.7306",
            "2r3k1/1pr2qp1/p2bpp1p/3p1n2/3P1PP1/2P2N2/RQ2NP1P/4R2K b - - ce 0.8325",
            "r2q1rk1/p1bb1ppp/P1n1pn2/2p5/1pN5/1Q1P1NP1/1P1BPPBP/2R2RK1 b - - ce 0.4102",
            "4k3/3n1p2/p3p1rp/4P1B1/3p1P2/b1p1q3/P1R3PP/2RQ3K w - - ce 0.2295",
            "3q1rk1/3bpp1p/6p1/1Bn1P1P1/3p1B2/8/2P3PP/Q4RK1 w - - ce 0.4457",
            "2rk4/3r2p1/p1pb1p1p/P3p3/1PR4P/3RP1P1/3BKP2/8 b - - ce 0.4194",
            "1R6/2p3pk/3n1q1p/2Q1p3/2p1P3/6P1/4K2P/8 w - - ce 0.5295",
            "4r1k1/Rb5p/5pp1/3pn3/3N1B2/4P2P/5PP1/6K1 b - - ce 0.4183",
            "8/4k3/R2b4/4pp2/5r2/2P2P1P/1P3KB1/8 b - - ce 0.2446",
            "2R5/r3p1kp/5pp1/pR2n3/4P3/PP2K1PP/1r6/5B2 w - - ce 0.4323",
            "2b1r1k1/4q2p/4P1p1/3NQp2/1p1Rn3/5BP1/PP5P/1K6 b - - ce 0.2024",
            "8/6k1/5p2/2pPqB1P/2Pp2K1/8/5Q1b/8 b - - ce 0.4936",
            "2r1k3/8/b2p1p2/p3p1r1/Pp2P1Bp/1Pq4P/2P2R2/2Q1R2K b - - ce 0.6721",
        ];

        // note we do 1 - game result if black is to move
        const EXPECTED_PARSED_GAME_RESULTS: [f64; 13] = [
            0.7306, 0.8325, 0.4102, 0.2295, 0.4457, 0.4194, 0.5295, 0.4183, 0.2446, 0.4323, 0.2024,
            0.4936, 0.6721,
        ];
        let eval = ByteKnightEvaluation::default();
        let params = Parameters::create_from_engine_values();

        let parsed_results = test_epd_lines(&epd_lines);
        for (i, (position, board, result)) in parsed_results.iter().enumerate() {
            // note we do 1 - game result if black is to move
            let expected_game_result: f64 = match board.side_to_move() {
                Side::Black => 1.0 - EXPECTED_PARSED_GAME_RESULTS[i],
                Side::White => EXPECTED_PARSED_GAME_RESULTS[i],
            };

            assert_eq!(position.game_result, expected_game_result);
            assert_eq!(*result, EXPECTED_PARSED_GAME_RESULTS[i]);
            let expected_value = eval.eval(board);

            // tuning position evaluation is always from white's perspective
            let val = match board.side_to_move() {
                Side::White => position.evaluate(&params),
                Side::Black => -position.evaluate(&params),
            };
            println!("{expected_value} // {val}");
            assert!((expected_value.0 as f64 - val).abs().round() <= 1.0)
        }
    }

    #[test]
    fn zurichess_epd_data() {
        let epd_lines = [
            "r2qkr2/p1pp1ppp/1pn1pn2/2P5/3Pb3/2N1P3/PP3PPP/R1B1KB1R b KQq - c9 \"0-1\";",
            "r4rk1/3bppb1/p3q1p1/1p1p3p/2pPn3/P1P1PN1P/1PB1QPPB/1R3RK1 b - - c9 \"1/2-1/2\";",
            "4Q3/8/8/8/6k1/4K2p/3N4/5q2 b - - c9 \"0-1\";",
            "r4rk1/1Qpbq1bp/p1n2np1/3p1p2/3P1P2/P1NBPN1P/1P1B2P1/R4RK1 b - - c9 \"0-1\";",
            "r1bqk2r/2p2ppp/2p5/p3pn2/1bB5/2NP2P1/PPP1NP1P/R1B1K2R w KQkq - c9 \"0-1\";",
            "8/8/4kp2/8/5K2/6p1/6P1/8 b - - c9 \"1/2-1/2\";",
            "r4rk1/3p2pp/p7/1pq2p2/2n2P2/P2Q3P/2P1NRP1/R5K1 w - - c9 \"1/2-1/2\";",
            "2rqk1n1/p6p/1p1pp3/8/4P3/P1b5/R2N1PPP/3QR1K1 w - - c9 \"1-0\";",
            "1r4k1/2qb1pb1/2p2P1p/8/p7/N1BB3P/P5P1/2Q2R1K b - - c9 \"1-0\";",
            "R7/1r6/5p2/8/P4k2/8/1p6/4K3 w - - c9 \"0-1\";",
        ];

        // note we do 1 - game result if black is to move
        const EXPECTED_PARSED_GAME_RESULTS: [f64; 10] =
            [0.0, 0.5, 0.0, 0.0, 0.0, 0.5, 0.5, 1.0, 1.0, 0.0];

        let eval = ByteKnightEvaluation::default();
        let params = Parameters::create_from_engine_values();

        let parsed_results = test_epd_lines(&epd_lines);
        for (i, (position, board, result)) in parsed_results.iter().enumerate() {
            // in this case no adjustment is needed since the game result is already adjusted
            assert_eq!(position.game_result, EXPECTED_PARSED_GAME_RESULTS[i]);
            assert_eq!(*result, EXPECTED_PARSED_GAME_RESULTS[i]);
            let expected_value = eval.eval(board);
            // tuning position evaluation is always from white's perspective
            let val = match board.side_to_move() {
                Side::White => position.evaluate(&params),
                Side::Black => -position.evaluate(&params),
            };

            println!("pos: {}\n{}", board.to_fen(), board);
            println!("{expected_value} // {val}");
            assert!((expected_value.0 as f64 - val).abs().round() <= 1.0)
        }
    }

    #[test]
    fn custom_filtered_epd_data() {
        let lines = [
            "r3rbk1/p1pbqpp1/2p4p/3pP3/1P1B1P2/3B4/P1P3PP/1R1Q1R1K b - b3 c0 i-play-atomic-Larsen-b3 Rated Bullet game https://lichess.org/UkOcG61J 2026.01.01; c1 0-1;",
            "r3rbk1/2pbqpp1/2p4p/p2pP3/1P1B1P2/3B4/P1P3PP/1R1Q1R1K w - a6 c0 i-play-atomic-Larsen-b3 Rated Bullet game https://lichess.org/UkOcG61J 2026.01.01; c1 0-1;",
            "r3rbk1/2pbqpp1/2p4p/p2pP3/1P1B1P2/P2B4/2P3PP/1R1Q1R1K b - - c0 i-play-atomic-Larsen-b3 Rated Bullet game https://lichess.org/UkOcG61J 2026.01.01; c1 0-1;",
            "r3rbk1/2pbqpp1/2p4p/3pP3/1p1B1P2/P2B4/2P3PP/1R1Q1R1K w - - c0 i-play-atomic-Larsen-b3 Rated Bullet game https://lichess.org/UkOcG61J 2026.01.01; c1 0-1;",
            "r3rbk1/2pbqpp1/2p4p/3pP3/1P1B1P2/3B4/2P3PP/1R1Q1R1K b - - c0 i-play-atomic-Larsen-b3 Rated Bullet game https://lichess.org/UkOcG61J 2026.01.01; c1 0-1;",
            "1r2rbk1/2pbqpp1/2p4p/3pP3/1P1B1P2/3B4/2P3PP/1R1Q1R1K w - - c0 i-play-atomic-Larsen-b3 Rated Bullet game https://lichess.org/UkOcG61J 2026.01.01; c1 0-1;",
            "1r2rbk1/2pbqpp1/2p4p/3pP3/1P1B1P2/2PB4/6PP/1R1Q1R1K b - - c0 i-play-atomic-Larsen-b3 Rated Bullet game https://lichess.org/UkOcG61J 2026.01.01; c1 0-1;",
            "1r2rbk1/2pbqp2/2p3pp/3pP3/1P1B1P2/2PB4/6PP/1R1Q1R1K w - - c0 i-play-atomic-Larsen-b3 Rated Bullet game https://lichess.org/UkOcG61J 2026.01.01; c1 0-1;",
        ];

        let eval = ByteKnightEvaluation::default();
        let params = Parameters::create_from_engine_values();

        let parsed_results = test_epd_lines(&lines);
        for (pos, board, result) in parsed_results {
            assert_eq!(result, 0.0);
            assert_eq!(pos.game_result, result);
            let expected_value = eval.eval(&board);
            // tuning position evaluation is always from white's perspective
            let val = match board.side_to_move() {
                Side::White => pos.evaluate(&params),
                Side::Black => -pos.evaluate(&params),
            };

            println!("pos: {}\n{}", board.to_fen(), board);
            println!("{expected_value} // {val}");
            assert!((expected_value.0 as f64 - val).abs().round() <= 1.0)
        }
    }
}
