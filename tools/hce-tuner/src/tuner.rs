// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSlice,
};

use crate::{offsets::PARAMETER_COUNT, parameters::Parameters, tuning_position::TuningPosition};

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
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        ctrlc::try_set_handler(move || {
            println!("Received Ctrl+C, stopping training...");
            stop_clone.store(true, Ordering::Relaxed);
        })
        .expect("Error setting Ctrl+C handler");

        println!("Computing optimal K value...");
        let computed_k: f64 = self.compute_k();
        println!("Optimal K value: {computed_k:.8}");
        println!("Using {} positions", self.positions.len());

        let mut last_error = f64::MAX;
        let patience = 5;
        let mut stale_epochs = 0;
        let mse_error_delta_threshold = 0.00000001;

        for epoch in 1..=self.max_epochs {
            if stop.load(Ordering::Relaxed) {
                println!("Early stopping at epoch {epoch} due to interrupt signal.");
                break;
            }

            self.run_epoch(computed_k);

            if epoch % 100 == 0 {
                // Calculate the MSE
                let error = self.mean_square_error(computed_k);
                println!("Epoch: {epoch} error {error:.8}");
                // Check for improvement
                if last_error - error < mse_error_delta_threshold {
                    // Stale epoch, no significant improvement
                    stale_epochs += 1;
                    // Have we exceeded our patience?
                    if stale_epochs >= patience {
                        println!("Early stopping at epoch {epoch} due to lack of improvement.");
                        break;
                    }
                } else {
                    // Improving epoch, reset stale counter
                    stale_epochs = 0;
                }
                // Update the error
                last_error = error;
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

    fn gradients(&self, k: f64) -> Parameters {
        let chunk_size = self.positions.len().div_ceil(rayon::current_num_threads());
        self.positions
            .par_chunks(chunk_size)
            .map(|chunk| self.weights.gradient_batch(k, chunk))
            .reduce(Parameters::default, |a, b| a + b)
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
        assert_eq!(PARAMETER_COUNT, 537);
    }

    #[test]
    fn construct_tuner() {
        let positions = vec![]; // Add appropriate Board instances here
        let params = Parameters::create_from_engine_values();
        let _ = Tuner::new(params, &positions, 5000);
    }
}
