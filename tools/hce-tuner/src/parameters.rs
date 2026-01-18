// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use std::ops::{Add, Index, IndexMut};

use chess::{
    definitions::NumberOf,
    pieces::{ALL_PIECES, Piece},
    side::Side,
    square,
};
use engine::hce_values::{DOUBLED_PAWN_VALUES, ISOLATED_PAWN_VALUES, PASSED_PAWN_BONUS, PSQTS};

use crate::{
    math,
    offsets::{Offsets, PARAMETER_COUNT},
    tuner_score::TuningScore,
    tuning_position::TuningPosition,
};

/// Set of parameters that serve as input for tuning.
pub struct Parameters([TuningScore; PARAMETER_COUNT]);

#[allow(dead_code)]
fn piece_value(piece: Piece) -> f64 {
    match piece {
        Piece::King => 10.,
        Piece::Queen => 900.,
        Piece::Rook => 400.,
        Piece::Bishop => 300.,
        Piece::Knight => 200.,
        Piece::Pawn => 100.,
    }
}

impl Parameters {
    pub(crate) fn as_slice(&self) -> &[TuningScore] {
        &self.0
    }

    #[allow(dead_code)]
    pub(crate) fn value(&self, piece: Piece, square: u8, side: Side) -> TuningScore {
        self[64 * piece as usize + square::flip_if(side == Side::White, square) as usize]
    }

    #[allow(dead_code)]
    pub(crate) fn create_from_engine_values() -> Parameters {
        let mut params = Parameters::default();
        for &piece in ALL_PIECES.iter() {
            for sq in 0..NumberOf::SQUARES {
                // seed from our PSQTS table
                let s = PSQTS[piece as usize][sq].into();
                params[64 * piece as usize + sq] = s;
            }
        }

        // Add passed pawn bonuses
        for (idx, val) in PASSED_PAWN_BONUS.iter().enumerate() {
            params[Offsets::PASSED_PAWN as usize + idx] = (*val).into();
        }

        // Add doubled pawn values
        for (idx, val) in DOUBLED_PAWN_VALUES.iter().enumerate() {
            params[Offsets::DOUBLED_PAWN as usize + idx] = (*val).into();
        }

        for (idx, val) in ISOLATED_PAWN_VALUES.iter().enumerate() {
            params[Offsets::ISOLATED_PAWN as usize + idx] = (*val).into();
        }

        params
    }

    #[allow(dead_code)]
    pub(crate) fn create_from_piece_values() -> Parameters {
        let mut params = Parameters::default();
        for piece in ALL_PIECES {
            for sq in 0..NumberOf::SQUARES {
                let val = piece_value(piece);
                params[64 * piece as usize + sq] = TuningScore::new(val, val);
            }
        }

        // Add passed pawn bonuses
        for (idx, val) in PASSED_PAWN_BONUS.iter().enumerate() {
            params[Offsets::PASSED_PAWN as usize + idx] = (*val).into();
        }

        params
    }

    pub(crate) fn gradient_batch(&self, k: f64, data: &[TuningPosition]) -> Self {
        let mut gradient = Parameters::default();
        for point in data {
            let sigmoid_result = math::sigmoid(k * point.evaluate(self));
            let term =
                (point.game_result - sigmoid_result) * (1. - sigmoid_result) * sigmoid_result;
            let phase_adjustment = term * TuningScore::new(point.phase, 1. - point.phase);

            for idx in &point.parameter_indexes[Side::White as usize] {
                gradient[*idx] += phase_adjustment;
            }

            for idx in &point.parameter_indexes[Side::Black as usize] {
                gradient[*idx] -= phase_adjustment;
            }
        }
        gradient
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self([TuningScore::default(); PARAMETER_COUNT])
    }
}

impl Index<usize> for Parameters {
    type Output = TuningScore;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Parameters {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl Add<Parameters> for Parameters {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result = Parameters::default();
        for i in 0..PARAMETER_COUNT {
            result[i] = self[i] + rhs[i];
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use chess::{definitions::NumberOf, pieces::ALL_PIECES, side::Side};
    use engine::{evaluation::ByteKnightEvaluation, traits::EvalValues};

    use super::Parameters;

    #[test]
    fn parameter_access() {
        // ensure that we can access parameters correctly at the correct index
        let params = Parameters::create_from_engine_values();
        let eval = ByteKnightEvaluation::default();
        for piece in ALL_PIECES {
            for square in 0..NumberOf::SQUARES as u8 {
                let side = Side::White;
                let value = params.value(piece, square, side);
                assert_eq!(value, eval.values().psqt(square, piece, side).into());
            }
        }
    }
}
