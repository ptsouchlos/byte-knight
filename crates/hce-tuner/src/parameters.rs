// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use std::ops::{Add, AddAssign, Index, IndexMut};

use chess::{
    definitions::NumberOf,
    pieces::{ALL_PIECES, Piece},
    side::Side,
    square::{self, Square},
};

use crate::tuning_position::TuningPosition;
use crate::{
    offsets::{Offsets, PARAMETER_COUNT},
    tuner_score::TuningScore,
};

/// Set of parameters that serve as input for tuning.
pub struct Parameters([TuningScore; PARAMETER_COUNT]);

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
        use engine::{evaluation::ByteKnightEvaluation, traits::EvalValues};

        let mut params = Parameters::default();
        let values = ByteKnightEvaluation::default().into_values();

        // PSQTs: enumerate all (piece, square) in Black's perspective (no flip)
        for &piece in ALL_PIECES.iter() {
            for sq in 0..Square::COUNT as u8 {
                let idx = Offsets::offset_for_piece_and_square(sq as usize, piece, Side::Black);
                params[idx] = values.psqt(sq, piece, Side::Black).into();
            }
        }

        // Passed pawn: 6 entries, one per rank 1-6
        for rank in 1..=NumberOf::PASSED_PAWN_RANKS as u8 {
            let sq = rank * 8; // a-file square on this rank; file is irrelevant
            let idx = Offsets::offset_for_passed_pawn(sq as usize, Side::Black);
            params[idx] = values.passed_pawn_bonus(sq, Side::Black).into();
        }

        // Doubled/isolated pawn: 8 entries each, one per file
        for file in 0..8u8 {
            let sq = file; // rank 0; rank is irrelevant for file-indexed features
            params[Offsets::offset_for_doubled_pawn(sq as usize, Side::White)] =
                values.doubled_pawn_value(sq, Side::White).into();
            params[Offsets::offset_for_isolated_pawn(sq as usize, Side::White)] =
                values.isolated_pawn_value(sq, Side::White).into();
        }

        // Mobility values for the pieces we care about.
        for piece in [Piece::Rook, Piece::Bishop, Piece::Knight, Piece::Queen] {
            let num_moves = match piece {
                Piece::Rook => NumberOf::ROOK_MOVES,
                Piece::Bishop => NumberOf::BISHOP_MOVES,
                Piece::Knight => NumberOf::KNIGHT_MOVES,
                Piece::Queen => NumberOf::QUEEN_MOVES,
                _ => unreachable!(),
            };

            for mobility in 0..=num_moves {
                let idx = Offsets::offset_for_mobility(piece, mobility);
                params[idx] = values.mobility_value(piece, mobility, Side::White).into();
            }
        }

        // Bishop pair: single value
        params[Offsets::offset_for_bishop_pair()] =
            values.bishop_pair_bonus_value(Side::White).into();

        // King safety: one per non-King attacker piece
        for &piece in ALL_PIECES.iter().filter(|&&p| p != Piece::King) {
            let idx = Offsets::offset_for_king_safety(piece);
            params[idx] = values.king_safety_value(piece, Side::White).into();
        }

        // Threats (pawn/knight/bishop can threaten non-King pieces)
        // King slot stays TuningScore::default() = 0, matching S(0,0) in the engine arrays
        for &attacker in &[Piece::Pawn, Piece::Knight, Piece::Bishop] {
            for &attacked in ALL_PIECES.iter().filter(|&&p| p != Piece::King) {
                let idx = Offsets::offset_for_threat(attacker, attacked);
                params[idx] = values.threat_value(attacker, attacked, Side::White).into();
            }
        }

        // Tempo bonus
        params[Offsets::offset_for_tempo_bonus()] = values.tempo_bonus(Side::White).into();

        // Rook open/semi-open files
        for file in 0..File::COUNT as u8 {
            params[Offsets::offset_for_rook_open_file(file)] =
                values.open_file_bonus(file, Side::White).into();
            params[Offsets::offset_for_rook_semi_open_file(file)] =
                values.semi_open_file_bonus(file, Side::White).into();
        }

        // Pawn shield & storm
        for file_idx in 0..NumberOf::KING_FLANK_FILES {
            for rank_idx in 0..NumberOf::PAWN_SHIELD_RANKS {
                params[Offsets::offset_for_pawn_shield(file_idx, rank_idx)] = values
                    .pawn_shield_value(file_idx, rank_idx, Side::White)
                    .into();
            }
            for rank_idx in 0..NumberOf::PAWN_STORM_RANKS {
                params[Offsets::offset_for_pawn_storm(file_idx, rank_idx)] = values
                    .pawn_storm_value(file_idx, rank_idx, Side::White)
                    .into();
            }
        }

        params
    }

    #[inline(always)]
    pub(crate) fn gradient_batch(&self, k: f64, data: &[TuningPosition]) -> Self {
        use crate::math;
        let mut gradient = Parameters::default();
        for point in data {
            let sigmoid_result = math::sigmoid(k * point.evaluate(self));
            let term =
                (point.game_result - sigmoid_result) * (1. - sigmoid_result) * sigmoid_result;
            let phase_adjustment = term * point.phase_score;

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

impl AddAssign<Parameters> for Parameters {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..PARAMETER_COUNT {
            self[i] += rhs[i];
        }
    }
}

#[cfg(test)]
mod tests {
    use chess::{
        definitions::NumberOf,
        pieces::{ALL_PIECES, Piece},
        side::Side,
        square::Square,
    };
    use engine::{evaluation::ByteKnightEvaluation, traits::EvalValues};

    use super::Parameters;
    use crate::offsets::Offsets;

    #[test]
    fn parameter_access() {
        // ensure that we can access parameters correctly at the correct index
        let params = Parameters::create_from_engine_values();
        let eval = ByteKnightEvaluation::default();

        // PSQTs
        for &piece in ALL_PIECES.iter() {
            for sq in 0..Square::COUNT as u8 {
                let idx = Offsets::offset_for_piece_and_square(sq as usize, piece, Side::Black);
                assert_eq!(
                    params[idx],
                    eval.values().psqt(sq, piece, Side::Black).into(),
                    "PSQT mismatch for {piece:?} sq={sq}"
                );
            }
        }

        // Passed pawn bonuses (ranks 1-6)
        for rank in 1..=NumberOf::PASSED_PAWN_RANKS as u8 {
            let sq = rank * 8;
            let idx = Offsets::offset_for_passed_pawn(sq as usize, Side::Black);
            assert_eq!(
                params[idx],
                eval.values().passed_pawn_bonus(sq, Side::Black).into(),
                "Passed pawn mismatch at rank={rank}"
            );
        }

        // Doubled / isolated pawn (by file)
        for file in 0..8u8 {
            let sq = file;
            assert_eq!(
                params[Offsets::offset_for_doubled_pawn(sq as usize, Side::White)],
                eval.values().doubled_pawn_value(sq, Side::White).into(),
                "Doubled pawn mismatch at file={file}"
            );
            assert_eq!(
                params[Offsets::offset_for_isolated_pawn(sq as usize, Side::White)],
                eval.values().isolated_pawn_value(sq, Side::White).into(),
                "Isolated pawn mismatch at file={file}"
            );
        }

        // Bishop pair
        assert_eq!(
            params[Offsets::offset_for_bishop_pair()],
            eval.values().bishop_pair_bonus_value(Side::White).into()
        );

        // King safety
        for &piece in ALL_PIECES.iter().filter(|&&p| p != Piece::King) {
            assert_eq!(
                params[Offsets::offset_for_king_safety(piece)],
                eval.values().king_safety_value(piece, Side::White).into(),
                "King safety mismatch for {piece:?}"
            );
        }

        // Threats
        for &attacker in &[Piece::Pawn, Piece::Knight, Piece::Bishop] {
            for &attacked in ALL_PIECES.iter().filter(|&&p| p != Piece::King) {
                assert_eq!(
                    params[Offsets::offset_for_threat(attacker, attacked)],
                    eval.values()
                        .threat_value(attacker, attacked, Side::White)
                        .into(),
                    "Threat mismatch for {attacker:?} -> {attacked:?}"
                );
            }
        }
    }
}
