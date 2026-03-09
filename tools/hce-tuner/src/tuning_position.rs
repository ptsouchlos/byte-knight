// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use chess::{definitions::NumberOf, side::Side};

use crate::{math, parameters::Parameters, tuner_score::TuningScore};

pub(crate) struct TuningPosition {
    pub(crate) parameter_indexes: [Vec<usize>; NumberOf::SIDES],
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
    use crate::{epd_parser, parameters::Parameters};

    #[test]
    fn evaluate_snapshot() {
        let params = Parameters::create_from_engine_values();

        // Starting position (opening, high phase)
        let pos_start = epd_parser::parse_epd_line(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 [0.5]",
        )
        .unwrap();
        let eval_start = pos_start.evaluate(&params);
        assert!(
            (eval_start - 30.0).abs() < 1e-10,
            "start position snapshot changed: got {eval_start}"
        );

        // Middlegame position
        let pos_mid = epd_parser::parse_epd_line(
            "r2q1rk1/3n1p2/2pp3p/1pb1p1p1/p3P3/P1NP1N1P/RPP2PP1/5QK1 b - - 0 2 [0.0]",
        )
        .unwrap();
        let eval_mid = pos_mid.evaluate(&params);
        assert!(
            (eval_mid - (-613.0)).abs() < 1e-10,
            "middlegame snapshot changed: got {eval_mid}"
        );

        // Endgame position (low phase)
        let pos_end = epd_parser::parse_epd_line(
            "8/8/7p/1P2k2P/4p1P1/1p1r4/1R2K3/8 b - - ce 0.7306",
        )
        .unwrap();
        let eval_end = pos_end.evaluate(&params);
        assert!(
            (eval_end - (-177.166_666_666_666_69)).abs() < 1e-6,
            "endgame snapshot changed: got {eval_end}"
        );
    }
}
