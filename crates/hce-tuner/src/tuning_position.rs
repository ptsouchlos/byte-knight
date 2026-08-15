// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use chess::side::Side;

use crate::{math, parameters::Parameters, tuner_score::TuningScore};

pub(crate) struct TuningPosition {
    pub(crate) parameter_indexes: [Vec<usize>; Side::COUNT],
    pub(crate) phase: f64,
    pub(crate) phase_score: TuningScore,
    pub(crate) game_result: f64,
}

impl TuningPosition {
    pub(crate) fn new(
        white_indexes: Vec<usize>,
        black_indexes: Vec<usize>,
        phase: f64,
        game_result: f64,
    ) -> Self {
        // Side::White == 0, Side::Black == 1
        let parameter_indexes = [white_indexes, black_indexes];
        let phase_score = TuningScore::new(phase, 1.0 - phase);
        Self {
            parameter_indexes,
            phase,
            phase_score,
            game_result,
        }
    }

    /// Evaluate the tuning position based on the given parameters from white's perspective.
    /// # Arguments
    /// * `parameters` - The parameters to evaluate.
    /// # Returns
    /// The evaluated score from white's perspective.
    pub(crate) fn evaluate(&self, parameters: &Parameters) -> f64 {
        let mut score: TuningScore = Default::default();

        for &idx in &self.parameter_indexes[Side::White as usize] {
            score += parameters[idx];
        }

        for &idx in &self.parameter_indexes[Side::Black as usize] {
            score -= parameters[idx];
        }

        score.taper(self.phase)
    }

    pub(crate) fn error(&self, k: f64, params: &Parameters) -> f64 {
        (self.game_result - math::sigmoid(k * self.evaluate(params))).powi(2)
    }
}

#[cfg(test)]
mod tests {
    use chess::side::Side;
    use engine::{evaluation::ByteKnightEvaluation, traits::Eval};

    use crate::{
        epd_parser::{self, WdlModel},
        parameters::Parameters,
    };

    #[test]
    fn tuning_position_eval_matches_engine() {
        let params = Parameters::create_from_engine_values();
        let eval = ByteKnightEvaluation::default();
        let line = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 [0.5]";
        // Starting position (opening, high phase)
        let pos_start = epd_parser::parse_epd_line(line, WdlModel::Auto).unwrap();
        let eval_start = pos_start.evaluate(&params);
        let (board, _) = epd_parser::process_epd_line(line).unwrap();

        let engine_eval_start = match board.side_to_move() {
            Side::White => eval.eval(&board).0 as f64,
            Side::Black => -eval.eval(&board).0 as f64,
        };

        assert!(
            (eval_start - engine_eval_start).abs() < 1.0,
            "start position snapshot changed: got {eval_start}, expected {engine_eval_start}"
        );

        // Middlegame position
        let line_mid = "r2q1rk1/3n1p2/2pp3p/1pb1p1p1/p3P3/P1NP1N1P/RPP2PP1/5QK1 b - - 0 2 [0.0]";
        let pos_mid = epd_parser::parse_epd_line(line_mid, WdlModel::Auto).unwrap();
        let (board_mid, _) = epd_parser::process_epd_line(line_mid).unwrap();

        let eval_mid = pos_mid.evaluate(&params);
        let engine_eval_mid = match board_mid.side_to_move() {
            Side::White => eval.eval(&board_mid).0 as f64,
            Side::Black => -eval.eval(&board_mid).0 as f64,
        };

        assert!(
            (eval_mid - engine_eval_mid).abs() < 1.0,
            "middlegame snapshot changed: got {eval_mid}, expected {engine_eval_mid}"
        );

        // Endgame position (low phase)
        let line_end = "8/8/7p/1P2k2P/4p1P1/1p1r4/1R2K3/8 b - - ce 0.7306";
        let pos_end = epd_parser::parse_epd_line(line_end, WdlModel::Auto).unwrap();
        let (board_end, _) = epd_parser::process_epd_line(line_end).unwrap();
        let eval_end = pos_end.evaluate(&params);
        let engine_eval_end = match board_end.side_to_move() {
            Side::White => eval.eval(&board_end).0 as f64,
            Side::Black => -eval.eval(&board_end).0 as f64,
        };
        assert!(
            (eval_end - engine_eval_end).abs() < 1.0,
            "endgame snapshot changed: got {eval_end}, expected {engine_eval_end}"
        );
    }
}
