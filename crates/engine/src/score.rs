// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::ops::{Div, DivAssign, Mul, MulAssign, Shl, Sub, SubAssign};
use std::{
    fmt::{self, Display, Formatter},
    ops::{Add, AddAssign, Neg},
};
use uci_parser::UciScore;

use crate::defs::MAX_PLY;

pub type ScoreType = i16;
pub(crate) type LargeScoreType = i32;

/// Helper to check if a given score is valid. Validity is
/// determined by a score being within the bounds of [-MATE, MATE]
/// inclusive.
///
/// # Arguments
/// - `score`: The score to check
///
/// # Returns
/// True if the score is within the bounds [-MATE, MATE] (inclusive), false otherwise.
pub fn is_valid(score: i32) -> bool {
    score >= -Score::MATE.0 as i32 && score <= Score::MATE.0 as i32
}

/// Represents a score in centipawns.
///
/// This type has saturating add/sub operations to prevent overflow.
/// It will not wrap around on overflow, but instead saturate to the internal types min/max.
///
/// The score is represented as a signed 16-bit integer, which allows for a range of -32,768 to 32,767.
///
/// Example usage:
/// ```rust
/// use engine::score::{Score, ScoreType};
/// let score = Score::new(150); // Represents a score of 150 centipawns
/// let mate_score = Score::MATE; // Represents a checkmate score
/// let draw_score = Score::DRAW; // Represents a draw score
/// let mut s = Score::INF / 2;
/// s += Score::INF;
/// assert_eq!(s, Score::INF); // Saturating addition
/// let mut ss = -Score::INF;
/// ss -= Score::INF;
/// assert_eq!(ss, Score::new(ScoreType::MIN)); // Saturating subtraction
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Score(pub ScoreType);

impl Score {
    /// Score for a draw.
    pub const DRAW: Score = Score(0);
    /// Numerical value for max mate value
    pub const MATE_VALUE: ScoreType = 30_000;
    /// Mate score - this is the upper limit.
    pub const MATE: Score = Score(Score::MATE_VALUE as ScoreType);
    /// The minimum mate score. This is the maximum score minus the maximum depth.
    pub const MINIMUM_MATE: Score = Score(Score::MATE.0 - MAX_PLY);
    /// "Infinity" score
    pub const INF: Score = Score(ScoreType::MAX as ScoreType);
    /// Multiplier for the history bonus calculation.
    pub const HISTORY_MULT: ScoreType = 300;
    /// Offset for the history bonus calculation.
    pub const HISTORY_OFFSET: ScoreType = 250;
    /// Max/min score for history heuristic
    /// Must be lower then the minimum score for captures in MVV_LVA
    pub const MAX_HISTORY: LargeScoreType = 16_384;

    pub fn new(score: ScoreType) -> Score {
        Score(score)
    }

    /// Returns a new "mate" score (positive).
    /// This is the largest mate score possible.
    pub fn new_mate() -> Score {
        Score::MATE
    }

    /// Returns a new "mated" score (negative).
    /// This is the largest mate score possible.
    pub fn new_mated() -> Score {
        -Score::MATE
    }

    pub fn mate_in(ply: ScoreType) -> Score {
        Score::new_mate() - ply
    }

    pub fn mated_in(ply: ScoreType) -> Score {
        Score::new_mated() + ply
    }

    pub fn clamp(&self, min: ScoreType, max: ScoreType) -> Score {
        Score(self.0.clamp(min, max))
    }

    /// Check if the score is a mate score. This is true if the score is between [MINIMUM_MATE, INF).
    ///
    /// # Returns
    /// - true if the score is a mate score, false otherwise.
    pub fn is_mate(&self) -> bool {
        self.0.abs() >= Score::MINIMUM_MATE.0.abs() && self.0.abs() < Score::INF.0
    }

    pub fn pow(&self, exp: u32) -> Score {
        Score(self.0.pow(exp))
    }

    /// Check if a score is a "mated" score, meaning we are being mated.
    ///
    /// # Returns
    /// - True if the score indicates that you are being mated, false otherwise.
    pub fn mated(&self) -> bool {
        self.0 <= -Score::MINIMUM_MATE.0 && self.0 > -Score::INF.0
    }

    /// Add ply bias to the current score if it is a mate score.
    ///
    /// # Arguments
    /// - `ply`: The ply bias to add to the mate score.
    ///
    /// # Returns
    /// A new score with ply bias if it is a mate score.
    pub fn ply_relative(&self, ply: ScoreType) -> Score {
        if self.0 >= Score::MINIMUM_MATE.0 {
            Score::new(self.0 - ply)
        } else if self.0 <= -Score::MINIMUM_MATE.0 {
            Score::new(self.0 + ply)
        } else {
            *self
        }
    }

    /// Remove ply bias from a score if it is a mate score.
    ///
    /// # Arguments
    /// - `ply`: The ply to remove from the mate score.
    ///
    /// # Returns
    /// A new score with the ply bias removed if the current score is a mate score.
    /// Otherwise, it just returns a clone of the same score.
    pub fn remove_ply_bias(&self, ply: ScoreType) -> Score {
        if self.0 >= Score::MINIMUM_MATE.0 {
            Score::new(self.0.saturating_add(ply))
        } else if self.0 <= -Score::MINIMUM_MATE.0 {
            Score::new(self.0.saturating_sub(ply))
        } else {
            *self
        }
    }

    pub fn as_i32(self) -> i32 {
        self.0 as i32
    }
}

impl From<Score> for UciScore {
    fn from(value: Score) -> Self {
        if value.is_mate() {
            // Mate scores are MATE - d where d is the number of plies to mate.
            // With UCI, "mate N" means we're mating and "mate -N" means we're being mated
            // So we can figure out the plies based on the difference between the score
            // and the mate score since that's the upper limit and the minimum is scaled by max plies.
            let plies = Score::MATE.0 - value.0.abs();
            let moves = (plies + 1) / 2;
            if value.0 > 0 {
                UciScore::mate(moves as i32)
            } else {
                UciScore::mate(-(moves as i32))
            }
        } else {
            UciScore::cp(value.0.into())
        }
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.is_mate() {
            let plies = Score::MATE.0 - self.0.abs();
            let moves = (plies + 1) / 2;
            if self.0 > 0 {
                write!(f, "mate {}", moves)
            } else {
                write!(f, "mate -{}", moves)
            }
        } else {
            write!(f, "cp {}", self.0)
        }
    }
}

impl Neg for Score {
    type Output = Score;

    fn neg(self) -> Score {
        Score(-self.0)
    }
}

impl AddAssign for Score {
    fn add_assign(&mut self, other: Score) {
        *self = *self + other;
    }
}

impl AddAssign<ScoreType> for Score {
    fn add_assign(&mut self, other: ScoreType) {
        *self = *self + other;
    }
}

impl Add for Score {
    type Output = Score;

    fn add(self, other: Score) -> Self::Output {
        Score(self.0.saturating_add(other.0))
    }
}

impl Add<ScoreType> for Score {
    type Output = Score;

    fn add(self, other: ScoreType) -> Self::Output {
        Score(self.0.saturating_add(other))
    }
}

impl Sub for Score {
    type Output = Score;
    fn sub(self, other: Score) -> Self::Output {
        Score(self.0.saturating_sub(other.0))
    }
}

impl Sub<ScoreType> for Score {
    type Output = Score;
    fn sub(self, other: ScoreType) -> Score {
        Score(self.0.saturating_sub(other))
    }
}

impl SubAssign for Score {
    fn sub_assign(&mut self, other: Score) {
        *self = *self - other;
    }
}

impl SubAssign<ScoreType> for Score {
    fn sub_assign(&mut self, rhs: ScoreType) {
        *self = *self - rhs;
    }
}

impl Div<ScoreType> for Score {
    type Output = Score;
    fn div(self, rhs: ScoreType) -> Score {
        Score(self.0 / rhs)
    }
}

impl Div<Score> for Score {
    type Output = Score;
    fn div(self, rhs: Score) -> Score {
        Score(self.0 / rhs.0)
    }
}

impl DivAssign<ScoreType> for Score {
    fn div_assign(&mut self, rhs: ScoreType) {
        self.0 /= rhs;
    }
}

impl DivAssign<Score> for Score {
    fn div_assign(&mut self, rhs: Score) {
        self.0 /= rhs.0;
    }
}

impl Mul<ScoreType> for Score {
    type Output = Score;
    fn mul(self, rhs: ScoreType) -> Score {
        Score(self.0 * rhs)
    }
}

impl Mul<Score> for Score {
    type Output = Score;
    fn mul(self, rhs: Score) -> Score {
        Score(self.0 * rhs.0)
    }
}

impl MulAssign<ScoreType> for Score {
    fn mul_assign(&mut self, rhs: ScoreType) {
        self.0 *= rhs;
    }
}

impl MulAssign<Score> for Score {
    fn mul_assign(&mut self, rhs: Score) {
        self.0 *= rhs.0;
    }
}

impl Shl<u32> for Score {
    type Output = Score;
    fn shl(self, rhs: u32) -> Score {
        Score(self.0 << rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn add_assign() {
        let mut right = Score::INF / 2;
        right += Score::INF;
        assert_eq!(right, Score::INF);
    }

    #[test]
    fn display_mate_score() {
        // Mate scores are MATE - d plies; moves = d/2
        // d=1 → mate in 1 move (we move once, opponent is mated)
        assert_eq!(Score::new(Score::MATE.0 - 1).to_string(), "mate 1");
        // d=3 → mate in 2 moves
        assert_eq!(Score::new(Score::MATE.0 - 3).to_string(), "mate 2");
        // d=2 → mate in 1 move (ceiling of 2/2)
        assert_eq!(Score::new(Score::MATE.0 - 2).to_string(), "mate 1");
        // Being mated in 1 (negative score)
        assert_eq!(Score::new(-(Score::MATE.0 - 1)).to_string(), "mate -1");
        // Being mated in 2 moves
        assert_eq!(Score::new(-(Score::MATE.0 - 3)).to_string(), "mate -2");
        // Regular cp score never shows as mate
        assert_eq!(Score::new(100).to_string(), "cp 100");
        assert_eq!(Score::new(-500).to_string(), "cp -500");
        assert_eq!(Score::new(0).to_string(), "cp 0");
    }

    #[test]
    fn uci_score_from_mate() {
        // Mate in 1 move
        assert_eq!(
            UciScore::from(Score::new(Score::MATE.0 - 1)),
            UciScore::mate(1)
        );
        // Mate in 2 moves
        assert_eq!(
            UciScore::from(Score::new(Score::MATE.0 - 3)),
            UciScore::mate(2)
        );
        // Being mated in 1 move
        assert_eq!(
            UciScore::from(Score::new(-(Score::MATE.0 - 1))),
            UciScore::mate(-1)
        );
        // Being mated in 2 moves
        assert_eq!(
            UciScore::from(Score::new(-(Score::MATE.0 - 3))),
            UciScore::mate(-2)
        );
        // Regular scores convert to centipawns
        assert_eq!(UciScore::from(Score::new(150)), UciScore::cp(150));
        assert_eq!(UciScore::from(Score::new(-300)), UciScore::cp(-300));
    }

    #[test]
    fn is_mate() {
        let mut score = Score::INF;
        assert!(!score.is_mate());

        score = Score::MATE;
        assert!(score.is_mate());

        score = -Score::MINIMUM_MATE;
        assert!(score.mated());

        score = Score::MINIMUM_MATE;
        assert!(score.is_mate());

        score = Score::new_mate();
        assert!(score.is_mate());
        assert_eq!(score.0, Score::MATE.0);

        score = Score::new_mated();
        assert!(score.is_mate());
        assert_eq!(score.0, -Score::MATE.0);
    }

    #[test]
    fn ply_bias_non_mate_no_change() {
        // Non-mate scores should pass through unchanged regardless of ply
        for cp in [0, 100, -100, 500, -500, 1, -1] {
            let score = Score::new(cp);
            assert_eq!(score.ply_relative(0), score);
            assert_eq!(score.ply_relative(5), score);
            assert_eq!(score.ply_relative(50), score);
            assert_eq!(score.remove_ply_bias(0), score);
            assert_eq!(score.remove_ply_bias(5), score);
            assert_eq!(score.remove_ply_bias(50), score);
        }
    }

    #[test]
    fn ply_relative_positive_mate() {
        // Positive mate: ply_relative subtracts ply
        // MATE - 5 at ply 0 → stays MATE - 5
        let score = Score::new_mate().ply_relative(5);
        assert_eq!(score.ply_relative(0), score);
        // MATE - 2 (stored ply-independent) retrieved at ply 3 → MATE - 5
        let offset_ply = 2;
        let lookup_ply = 5;
        let stored = Score::new_mate() - offset_ply;
        assert_eq!(
            stored.ply_relative(lookup_ply - offset_ply),
            Score::new_mate().ply_relative(lookup_ply)
        );
    }

    #[test]
    fn ply_relative_negative_mate() {
        // Negative mate (being mated): ply_relative adds ply
        // -MATE + 2 (stored ply-independent) retrieved at ply 3 → -MATE + 5
        let stored = Score::new(-Score::MATE.0 + 2);
        assert_eq!(stored.ply_relative(3), Score::new(-Score::MATE.0 + 5));
        // At ply 0, no adjustment
        assert_eq!(stored.ply_relative(0), stored);
    }

    #[test]
    fn remove_ply_bias_positive_mate() {
        // Positive mate: remove_ply_bias adds ply
        // MATE - 5 at ply 3 → stored as MATE - 5 + 3 = MATE - 2
        let score = Score::new(Score::MATE.0 - 5);
        assert_eq!(score.remove_ply_bias(3), Score::new(Score::MATE.0 - 2));
        // At ply 0, no adjustment
        assert_eq!(score.remove_ply_bias(0), score);
    }

    #[test]
    fn remove_ply_bias_negative_mate() {
        // Negative mate (being mated): remove_ply_bias subtracts ply
        // -MATE + 5 at ply 3 → stored as -MATE + 5 - 3 = -MATE + 2
        let score = Score::new(-Score::MATE.0 + 5);
        assert_eq!(score.remove_ply_bias(3), Score::new(-Score::MATE.0 + 2));
        assert_eq!(score.remove_ply_bias(0), score);
    }

    #[test]
    fn ply_bias_round_trip_same_ply() {
        // store then retrieve at the same ply → original score
        let plies = [0, 1, 5, 10, 50];
        let scores = [
            Score::new_mate() - 3,
            Score::new_mate() - 10,
            Score::new_mated() + 3,
            Score::new_mated() + 10,
            Score::MATE,
            Score::MINIMUM_MATE,
            -Score::MINIMUM_MATE,
        ];
        for ply in plies {
            for score in scores {
                assert_eq!(
                    score.remove_ply_bias(ply).ply_relative(ply),
                    score,
                    "round-trip failed for score {} at ply {ply}",
                    score.0
                );
            }
        }
    }

    #[test]
    fn ply_bias_round_trip_different_ply() {
        // Store at ply 4, retrieve at ply 10.
        // Positive mate: MATE - 5 at ply 4. Distance from position = 5 - 4 = 1.
        // Stored: MATE - 5 + 4 = MATE - 1.
        // Retrieved at ply 10: MATE - 1 - 10 = MATE - 11.
        // This means mate at ply 11 from root, which is 1 ply from ply 10. Correct.
        let score_at_ply4 = Score::new(Score::MATE.0 - 5);
        let stored = score_at_ply4.remove_ply_bias(4);
        assert_eq!(stored, Score::new(Score::MATE.0 - 1));
        let retrieved_at_ply10 = stored.ply_relative(10);
        assert_eq!(retrieved_at_ply10, Score::new(Score::MATE.0 - 11));
        // The distance from the position is preserved: (MATE - score) - ply
        // At ply 4:  (MATE - (MATE-5)) - 4 = 5 - 4 = 1
        // At ply 10: (MATE - (MATE-11)) - 10 = 11 - 10 = 1
        assert_eq!(Score::MATE.0 - score_at_ply4.0 - 4, 1);
        assert_eq!(Score::MATE.0 - retrieved_at_ply10.0 - 10, 1);
    }

    #[test]
    fn ply_bias_boundary_minimum_mate() {
        // MINIMUM_MATE is the lowest score that counts as a mate
        let score = Score::MINIMUM_MATE;
        assert!(score.is_mate());
        // Should be adjusted by ply_relative/remove_ply_bias
        assert_eq!(score.ply_relative(5), Score::new(Score::MINIMUM_MATE.0 - 5));
        assert_eq!(
            score.remove_ply_bias(5),
            Score::new(Score::MINIMUM_MATE.0 + 5)
        );

        // One below MINIMUM_MATE is NOT a mate score → no adjustment
        let below = Score::new(Score::MINIMUM_MATE.0 - 1);
        assert!(!below.is_mate());
        assert_eq!(below.ply_relative(5), below);
        assert_eq!(below.remove_ply_bias(5), below);

        // Negative boundary: -MINIMUM_MATE
        let neg = -Score::MINIMUM_MATE;
        assert!(neg.mated());
        assert_eq!(neg.ply_relative(5), Score::new(-Score::MINIMUM_MATE.0 + 5));
        assert_eq!(
            neg.remove_ply_bias(5),
            Score::new(-Score::MINIMUM_MATE.0 - 5)
        );

        // One above -MINIMUM_MATE is NOT a mated score → no adjustment
        let above_neg = Score::new(-Score::MINIMUM_MATE.0 + 1);
        assert!(!above_neg.mated());
        assert_eq!(above_neg.ply_relative(5), above_neg);
        assert_eq!(above_neg.remove_ply_bias(5), above_neg);
    }

    #[test]
    fn mate_in_and_mated_in() {
        assert_eq!(Score::mate_in(0), Score::MATE);
        assert_eq!(Score::mate_in(5), Score::new(Score::MATE.0 - 5));
        assert_eq!(Score::mated_in(0), -Score::MATE);
        assert_eq!(Score::mated_in(5), Score::new(-Score::MATE.0 + 5));
        // they are negations of each other
        assert_eq!(-Score::mated_in(5), Score::mate_in(5));
        // both return mate scores
        assert!(Score::mate_in(1).is_mate());
        assert!(Score::mated_in(1).mated());
    }
}
