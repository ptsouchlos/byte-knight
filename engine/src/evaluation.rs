// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{bitboard_helpers, board::Board, pieces::Piece, side::Side};

use crate::{
    hce_values::{ByteKnightValues, GAME_PHASE_INC, GAME_PHASE_MAX},
    pawn_structure::PawnEvaluator,
    phased_score::{PhaseType, PhasedScore},
    score::{LargeScoreType, Score, ScoreType},
    traits::{Eval, EvalValues},
};

/// Provides static evaluation of a given chess position.
pub struct Evaluation<Values>
where
    Values: EvalValues,
{
    values: Values,
    pawn_evaluator: PawnEvaluator,
}

impl<Values: EvalValues> Evaluation<Values> {
    pub fn new(values: Values) -> Self {
        Evaluation {
            values,
            pawn_evaluator: PawnEvaluator::new(),
        }
    }

    pub fn values(&self) -> &Values {
        &self.values
    }

    pub fn mutable_values(&mut self) -> &mut Values {
        &mut self.values
    }

    pub(crate) fn mvv_lva(captured: Piece, capturing: Piece) -> LargeScoreType {
        let can_capture = captured != Piece::King;
        ((can_capture as LargeScoreType)
            * (25 * Self::piece_value(captured) - Self::piece_value(capturing)))
            << 16
    }

    pub(crate) fn piece_value(piece: Piece) -> LargeScoreType {
        match piece {
            Piece::King => 0,
            Piece::Queen => 5,
            Piece::Rook => 4,
            Piece::Bishop => 3,
            Piece::Knight => 2,
            Piece::Pawn => 1,
        }
    }
}

impl<Values: EvalValues<ReturnScore = PhasedScore>> Eval<Board> for Evaluation<Values> {
    /// Evaluates the given position.
    ///
    /// # Arguments
    ///
    /// - `board`: The [`Board`] to evaluate.
    fn eval(&self, board: &Board) -> Score {
        let side_to_move = board.side_to_move();
        let mut mg: [i32; 2] = [0; 2];
        let mut eg: [i32; 2] = [0; 2];
        let mut game_phase = 0_i32;

        let pawn_structure = self.pawn_evaluator.detect_pawn_structure(board);

        let mut occupancy = board.all_pieces();
        // loop through occupied squares
        while occupancy.as_number() > 0 {
            let sq = bitboard_helpers::next_bit(&mut occupancy);
            let maybe_piece = board.piece_on_square(sq as u8);
            if let Some((piece, side)) = maybe_piece {
                if piece == Piece::Pawn {
                    // add passed pawn bonus if applicable
                    let passed_pawn_bonus = self.values.passed_pawn_bonus(sq as u8, side);
                    if pawn_structure.passed_pawns[side as usize].is_square_occupied(sq as u8) {
                        mg[side as usize] += passed_pawn_bonus.mg() as i32;
                        eg[side as usize] += passed_pawn_bonus.eg() as i32;
                    }

                    let doubled_pawn_value = self.values.doubled_pawn_value(sq as u8, side);
                    if pawn_structure.doubled_pawns[side as usize].is_square_occupied(sq as u8) {
                        mg[side as usize] += doubled_pawn_value.mg() as i32;
                        eg[side as usize] += doubled_pawn_value.eg() as i32;
                    }

                    let isolated_pawn_value = self.values.isolated_pawn_value(sq as u8, side);
                    if pawn_structure.isolated_pawns[side as usize].is_square_occupied(sq as u8) {
                        mg[side as usize] += isolated_pawn_value.mg() as i32;
                        eg[side as usize] += isolated_pawn_value.eg() as i32;
                    }
                }
                let phased_score: PhasedScore = self.values.psqt(sq as u8, piece, side);
                mg[side as usize] += phased_score.mg() as i32;
                eg[side as usize] += phased_score.eg() as i32;

                game_phase += GAME_PHASE_INC[piece as usize] as i32;
            }
        }

        let stm_idx = side_to_move as usize;
        let opposite = Side::opposite(side_to_move) as usize;
        let mg_score = mg[stm_idx] - mg[opposite];
        let eg_score = eg[stm_idx] - eg[opposite];
        let score = PhasedScore::new(mg_score as ScoreType, eg_score as ScoreType);
        // taper the score based on the game phase
        let val = score.taper(game_phase.min(GAME_PHASE_MAX) as PhaseType, GAME_PHASE_MAX);
        Score::new(val)
    }
}

pub type ByteKnightEvaluation = Evaluation<ByteKnightValues>;

impl Default for ByteKnightEvaluation {
    fn default() -> Self {
        Self::new(ByteKnightValues::default())
    }
}

#[cfg(test)]
mod tests {
    use chess::{
        board::Board,
        pieces::{ALL_PIECES, PIECE_SHORT_NAMES},
    };

    use crate::{
        evaluation::ByteKnightEvaluation,
        score::{LargeScoreType, ScoreType},
        traits::Eval,
    };

    #[test]
    fn mvv_lva_scaling() {
        for captured in ALL_PIECES {
            for capturing in ALL_PIECES {
                let score = ByteKnightEvaluation::mvv_lva(captured, capturing);
                println!(
                    "{} x {} -> {}",
                    PIECE_SHORT_NAMES[capturing as usize],
                    PIECE_SHORT_NAMES[captured as usize],
                    score
                );
                assert!((score as i64) < (LargeScoreType::MIN as i64).abs());
            }
        }
    }

    #[test]
    fn score_stability() {
        // These values were determined empirically by running this test and manually copy/pasting the results.
        // If any changes are made to the evaluation function, these values will need to be updated or the test
        // will need to be augmented with the new evaluation values.

        // standard EPD suite FEN positions
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "4k3/8/8/8/8/8/8/4K2R w K - 0 1",
            "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1",
            "4k2r/8/8/8/8/8/8/4K3 w k - 0 1",
            "r3k3/8/8/8/8/8/8/4K3 w q - 0 1",
            "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1",
            "r3k2r/8/8/8/8/8/8/4K3 w kq - 0 1",
            "8/8/8/8/8/8/6k1/4K2R w K - 0 1",
            "8/8/8/8/8/8/1k6/R3K3 w Q - 0 1",
            "4k2r/6K1/8/8/8/8/8/8 w k - 0 1",
            "r3k3/1K6/8/8/8/8/8/8 w q - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            "r3k2r/8/8/8/8/8/8/1R2K2R w Kkq - 0 1",
            "r3k2r/8/8/8/8/8/8/2R1K2R w Kkq - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K1R1 w Qkq - 0 1",
            "1r2k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1",
            "2r1k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1",
            "r3k1r1/8/8/8/8/8/8/R3K2R w KQq - 0 1",
            "4k3/8/8/8/8/8/8/4K2R b K - 0 1",
            "4k3/8/8/8/8/8/8/R3K3 b Q - 0 1",
            "4k2r/8/8/8/8/8/8/4K3 b k - 0 1",
            "r3k3/8/8/8/8/8/8/4K3 b q - 0 1",
            "4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1",
            "r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1",
            "8/8/8/8/8/8/6k1/4K2R b K - 0 1",
            "8/8/8/8/8/8/1k6/R3K3 b Q - 0 1",
            "4k2r/6K1/8/8/8/8/8/8 b k - 0 1",
            "r3k3/1K6/8/8/8/8/8/8 b q - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
            "r3k2r/8/8/8/8/8/8/1R2K2R b Kkq - 0 1",
            "r3k2r/8/8/8/8/8/8/2R1K2R b Kkq - 0 1",
            "r3k2r/8/8/8/8/8/8/R3K1R1 b Qkq - 0 1",
            "1r2k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1",
            "2r1k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1",
            "r3k1r1/8/8/8/8/8/8/R3K2R b KQq - 0 1",
            "8/1n4N1/2k5/8/8/5K2/1N4n1/8 w - - 0 1",
            "8/1k6/8/5N2/8/4n3/8/2K5 w - - 0 1",
            "8/8/4k3/3Nn3/3nN3/4K3/8/8 w - - 0 1",
            "K7/8/2n5/1n6/8/8/8/k6N w - - 0 1",
            "k7/8/2N5/1N6/8/8/8/K6n w - - 0 1",
            "8/1n4N1/2k5/8/8/5K2/1N4n1/8 b - - 0 1",
            "8/1k6/8/5N2/8/4n3/8/2K5 b - - 0 1",
            "8/8/3K4/3Nn3/3nN3/4k3/8/8 b - - 0 1",
            "K7/8/2n5/1n6/8/8/8/k6N b - - 0 1",
            "k7/8/2N5/1N6/8/8/8/K6n b - - 0 1",
            "B6b/8/8/8/2K5/4k3/8/b6B w - - 0 1",
            "8/8/1B6/7b/7k/8/2B1b3/7K w - - 0 1",
            "k7/B7/1B6/1B6/8/8/8/K6b w - - 0 1",
            "K7/b7/1b6/1b6/8/8/8/k6B w - - 0 1",
            "B6b/8/8/8/2K5/5k2/8/b6B b - - 0 1",
            "8/8/1B6/7b/7k/8/2B1b3/7K b - - 0 1",
            "k7/B7/1B6/1B6/8/8/8/K6b b - - 0 1",
            "K7/b7/1b6/1b6/8/8/8/k6B b - - 0 1",
            "7k/RR6/8/8/8/8/rr6/7K w - - 0 1",
            "R6r/8/8/2K5/5k2/8/8/r6R w - - 0 1",
            "7k/RR6/8/8/8/8/rr6/7K b - - 0 1",
            "R6r/8/8/2K5/5k2/8/8/r6R b - - 0 1",
            "6kq/8/8/8/8/8/8/7K w - - 0 1",
            "6KQ/8/8/8/8/8/8/7k b - - 0 1",
            "K7/8/8/3Q4/4q3/8/8/7k w - - 0 1",
            "6qk/8/8/8/8/8/8/7K b - - 0 1",
            "6KQ/8/8/8/8/8/8/7k b - - 0 1",
            "K7/8/8/3Q4/4q3/8/8/7k b - - 0 1",
            "8/8/8/8/8/K7/P7/k7 w - - 0 1",
            "8/8/8/8/8/7K/7P/7k w - - 0 1",
            "K7/p7/k7/8/8/8/8/8 w - - 0 1",
            "7K/7p/7k/8/8/8/8/8 w - - 0 1",
            "8/2k1p3/3pP3/3P2K1/8/8/8/8 w - - 0 1",
            "8/8/8/8/8/K7/P7/k7 b - - 0 1",
            "8/8/8/8/8/7K/7P/7k b - - 0 1",
            "K7/p7/k7/8/8/8/8/8 b - - 0 1",
            "7K/7p/7k/8/8/8/8/8 b - - 0 1",
            "8/2k1p3/3pP3/3P2K1/8/8/8/8 b - - 0 1",
            "8/8/8/8/8/4k3/4P3/4K3 w - - 0 1",
            "4k3/4p3/4K3/8/8/8/8/8 b - - 0 1",
            "8/8/7k/7p/7P/7K/8/8 w - - 0 1",
            "8/8/k7/p7/P7/K7/8/8 w - - 0 1",
            "8/8/3k4/3p4/3P4/3K4/8/8 w - - 0 1",
            "8/3k4/3p4/8/3P4/3K4/8/8 w - - 0 1",
            "8/8/3k4/3p4/8/3P4/3K4/8 w - - 0 1",
            "k7/8/3p4/8/3P4/8/8/7K w - - 0 1",
            "8/8/7k/7p/7P/7K/8/8 b - - 0 1",
            "8/8/k7/p7/P7/K7/8/8 b - - 0 1",
            "8/8/3k4/3p4/3P4/3K4/8/8 b - - 0 1",
            "8/3k4/3p4/8/3P4/3K4/8/8 b - - 0 1",
            "8/8/3k4/3p4/8/3P4/3K4/8 b - - 0 1",
            "k7/8/3p4/8/3P4/8/8/7K b - - 0 1",
            "7k/3p4/8/8/3P4/8/8/K7 w - - 0 1",
            "7k/8/8/3p4/8/8/3P4/K7 w - - 0 1",
            "k7/8/8/7p/6P1/8/8/K7 w - - 0 1",
            "k7/8/7p/8/8/6P1/8/K7 w - - 0 1",
            "k7/8/8/6p1/7P/8/8/K7 w - - 0 1",
            "k7/8/6p1/8/8/7P/8/K7 w - - 0 1",
            "k7/8/8/3p4/4p3/8/8/7K w - - 0 1",
            "k7/8/3p4/8/8/4P3/8/7K w - - 0 1",
            "7k/3p4/8/8/3P4/8/8/K7 b - - 0 1",
            "7k/8/8/3p4/8/8/3P4/K7 b - - 0 1",
            "k7/8/8/7p/6P1/8/8/K7 b - - 0 1",
            "k7/8/7p/8/8/6P1/8/K7 b - - 0 1",
            "k7/8/8/6p1/7P/8/8/K7 b - - 0 1",
            "k7/8/6p1/8/8/7P/8/K7 b - - 0 1",
            "k7/8/8/3p4/4p3/8/8/7K b - - 0 1",
            "k7/8/3p4/8/8/4P3/8/7K b - - 0 1",
            "7k/8/8/p7/1P6/8/8/7K w - - 0 1",
            "7k/8/p7/8/8/1P6/8/7K w - - 0 1",
            "7k/8/8/1p6/P7/8/8/7K w - - 0 1",
            "7k/8/1p6/8/8/P7/8/7K w - - 0 1",
            "k7/7p/8/8/8/8/6P1/K7 w - - 0 1",
            "k7/6p1/8/8/8/8/7P/K7 w - - 0 1",
            "3k4/3pp3/8/8/8/8/3PP3/3K4 w - - 0 1",
            "7k/8/8/p7/1P6/8/8/7K b - - 0 1",
            "7k/8/p7/8/8/1P6/8/7K b - - 0 1",
            "7k/8/8/1p6/P7/8/8/7K b - - 0 1",
            "7k/8/1p6/8/8/P7/8/7K b - - 0 1",
            "k7/7p/8/8/8/8/6P1/K7 b - - 0 1",
            "k7/6p1/8/8/8/8/7P/K7 b - - 0 1",
            "3k4/3pp3/8/8/8/8/3PP3/3K4 b - - 0 1",
            "8/Pk6/8/8/8/8/6Kp/8 w - - 0 1",
            "n1n5/1Pk5/8/8/8/8/5Kp1/5N1N w - - 0 1",
            "8/PPPk4/8/8/8/8/4Kppp/8 w - - 0 1",
            "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1",
            "8/Pk6/8/8/8/8/6Kp/8 b - - 0 1",
            "n1n5/1Pk5/8/8/8/8/5Kp1/5N1N b - - 0 1",
            "8/PPPk4/8/8/8/8/4Kppp/8 b - - 0 1",
            "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "rnbqkb1r/ppppp1pp/7n/4Pp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        ];

        let scores: [ScoreType; 128] = [
            0, 19, 649, 657, -649, -657, 1255, -1255, 568, 594, -568, -594, 0, 8, 17, 15, -8, -17,
            -15, -649, -657, 649, 657, -1255, 1255, -568, -594, 568, 594, 0, -8, -17, -15, 8, 17,
            15, 2, -2, 0, -431, 523, -2, 2, 7, 431, -523, -23, -42, 793, -825, 30, 42, -793, 825,
            0, -7, 0, 7, -1206, -1315, -46, 1180, -1315, 46, 207, 236, -207, -236, -12, -207, -236,
            207, 236, 12, -4, -4, 0, 0, 0, 20, -20, -8, 0, 0, 0, -20, 20, 8, -11, 7, 2, 1, -2, -1,
            -291, 2, 11, -7, -2, -1, 2, 1, 291, -2, -6, -4, 6, 4, -2, 2, 0, 6, 4, -6, -4, 2, -2, 0,
            -12, 12, 40, 65, 12, -12, -40, -65, 2, 27,
        ];

        let eval = ByteKnightEvaluation::default();

        for (i, fen) in positions.iter().enumerate() {
            println!("Position {i}: {fen}");
            let board = Board::from_fen(fen).unwrap();
            let score = eval.eval(&board);
            println!("{},", score.0);
            assert_eq!(score.0, scores[i]);
        }
    }
}
