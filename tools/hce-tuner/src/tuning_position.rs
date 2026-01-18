// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use chess::{definitions::NumberOf, side::Side};

use crate::{math, parameters::Parameters, tuner_score::TuningScore};

pub(crate) struct TuningPosition {
    pub(crate) parameter_indexes: [Vec<usize>; NumberOf::SIDES],
    pub(crate) phase: f64,
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
        Self {
            parameter_indexes,
            phase,
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
