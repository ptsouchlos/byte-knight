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
    }
}
