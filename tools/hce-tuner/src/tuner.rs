// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use chess::side::Side;
use rayon::prelude::*;

use crate::{
    math, offsets::PARAMETER_COUNT, parameters::Parameters, tuner_score::TuningScore,
    tuning_position::TuningPosition,
};

pub(crate) struct Tuner<'a> {
    positions: &'a Vec<TuningPosition>,
    weights: Parameters,
    momentum: Parameters,
    velocity: Parameters,
    learning_rate: f64,
    beta1: f64,
    beta2: f64,
    max_epochs: usize,
}

impl<'a> Tuner<'a> {
    pub(crate) fn new(
        initial_params: Parameters,
        positions: &'a Vec<TuningPosition>,
        max_epochs: usize,
    ) -> Self {
        Self {
            positions,
            weights: initial_params,
            momentum: Parameters::default(),
            velocity: Parameters::default(),
            learning_rate: 0.05,
            beta1: 0.9,
            beta2: 0.999,
            max_epochs,
        }
    }

    pub(crate) fn tune(&mut self) -> &Parameters {
        println!("Computing optimal K value...");
        let computed_k: f64 = self.compute_k();
        println!("Optimal K value: {computed_k:.8}");
        println!("Using {} positions", self.positions.len());

        for epoch in 1..=self.max_epochs {
            self.run_epoch(computed_k);

            if epoch % 100 == 0 {
                let error = self.mean_square_error(computed_k);
                println!("Epoch: {epoch} error {error:.7}");
            }
        }

        &self.weights
    }

    pub(crate) fn run_epoch(&mut self, k: f64) {
        let gradients = self.gradients(k);

        for i in 0..PARAMETER_COUNT {
            let adj = (-2. * k / self.positions.len() as f64) * gradients[i];
            self.momentum[i] = self.beta1 * self.momentum[i] + (1. - self.beta1) * adj;
            self.velocity[i] = self.beta2 * self.velocity[i] + (1. - self.beta2) * adj * adj;
            self.weights[i] -=
                self.learning_rate * self.momentum[i] / (self.velocity[i].sqrt() + 0.00000001);
        }
    }

    pub(crate) fn gradients(&self, k: f64) -> Parameters {
        let chunk_size = self
            .positions
            .len()
            .div_ceil(rayon::current_num_threads());
        self.positions
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut gradient = Parameters::default();
                for point in chunk {
                    // Inline evaluate
                    let mut score = TuningScore::default();
                    for &idx in &point.parameter_indexes[Side::White as usize] {
                        score += self.weights[idx];
                    }
                    for &idx in &point.parameter_indexes[Side::Black as usize] {
                        score -= self.weights[idx];
                    }
                    let eval = score.taper(point.phase);

                    // Gradient coefficient
                    let sigmoid_result = math::sigmoid(k * eval);
                    let term = (point.game_result - sigmoid_result)
                        * (1.0 - sigmoid_result)
                        * sigmoid_result;
                    let phase_adj = term * point.phase_score;

                    // Accumulate
                    for &idx in &point.parameter_indexes[Side::White as usize] {
                        gradient[idx] += phase_adj;
                    }
                    for &idx in &point.parameter_indexes[Side::Black as usize] {
                        gradient[idx] -= phase_adj;
                    }
                }
                gradient
            })
            .reduce(Parameters::default, |mut a, b| {
                a += b;
                a
            })
    }

    pub(crate) fn mean_square_error(&self, k: f64) -> f64 {
        let total_error: f64 = self
            .positions
            .par_iter()
            .map(|point| point.error(k, &self.weights))
            .sum();
        total_error / self.positions.len() as f64
    }

    /// Computes the optimal K value to minimize the error of the initial parameters.
    /// Taken from https://github.com/jw1912/hce-tuner/
    pub(crate) fn compute_k(&self) -> f64 {
        let mut k = 0.009;
        let delta = 0.00001;
        let goal = 0.000001;
        let mut dev = 1f64;

        while dev.abs() > goal {
            let right = self.mean_square_error(k + delta);
            let left = self.mean_square_error(k - delta);
            dev = (right - left) / (100000. * delta);
            k -= dev;
            if k <= 0.0 {
                println!("k {k:.4} decr {left:.5} incr {right:.5}");
            }
        }

        k
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offsets::PARAMETER_COUNT;

    #[test]
    fn offsets() {
        assert_eq!(PARAMETER_COUNT, 497);
    }

    #[test]
    fn construct_tuner() {
        let positions = vec![]; // Add appropriate Board instances here
        let params = Parameters::create_from_engine_values();
        let _ = Tuner::new(params, &positions, 5000);
    }

    #[test]
    fn tuner_reduces_error() {
        let positions = crate::epd_parser::parse_epd_file("../../data/lichess-test.book");
        assert!(!positions.is_empty(), "Test book should have positions");

        let params = Parameters::create_from_engine_values();
        let mut tuner = Tuner::new(params, &positions, 100);

        let k = tuner.compute_k();
        let initial_error = tuner.mean_square_error(k);

        for _ in 0..100 {
            tuner.run_epoch(k);
        }

        let final_error = tuner.mean_square_error(k);
        assert!(
            final_error < initial_error,
            "Error should decrease: initial={initial_error}, final={final_error}"
        );
    }
}
