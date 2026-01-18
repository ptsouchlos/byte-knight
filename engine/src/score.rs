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

use crate::defs::MAX_DEPTH;

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
    pub const DRAW: Score = Score(0);
    pub const MATE: Score = Score(ScoreType::MAX as ScoreType);
    /// The minimum mate score. This is the maximum score minus the maximum depth.
    pub const MINIMUM_MATE: Score = Score(Score::MATE.0 - MAX_DEPTH as ScoreType);
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

    /// Returns true if the score is a mate score.
    /// This is the case if the absolute value of the score is greater than or equal to `Score::MINIMUM_MATE`.
    pub fn is_mate(&self) -> bool {
        self.0.abs() >= Score::MINIMUM_MATE.0.abs()
    }

    pub fn pow(&self, exp: u32) -> Score {
        Score(self.0.pow(exp))
    }

    /// Returns true if the score indicates that you are being mated
    pub fn mated(&self) -> bool {
        self.0 <= -Score::MINIMUM_MATE.0
    }
}

impl From<Score> for UciScore {
    fn from(value: Score) -> Self {
        UciScore::cp(value.0.into())
    }
}

impl Display for Score {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.0.abs() >= Score::MATE.0.abs() {
            write!(f, "mate {}", (self.0 - Score::MATE.0) / 2)
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
}
