// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

use engine::phased_score::PhasedScore;

/// A float version of PhasedScore, with some small differences.
/// This is used for tuning only and helps us avoid issues that come up with integer arithmetic.
#[derive(Default, Copy, Clone, PartialEq)]
#[must_use]
pub struct TuningScore {
    mg: f64,
    eg: f64,
}

impl TuningScore {
    pub fn new(mg: f64, eg: f64) -> Self {
        Self { mg, eg }
    }

    pub fn mg(&self) -> f64 {
        self.mg
    }

    pub fn eg(&self) -> f64 {
        self.eg
    }

    pub fn sqrt(&self) -> Self {
        Self::new(self.mg().sqrt(), self.eg().sqrt())
    }

    /// Taper the score based on the phase of the game.
    ///
    /// # Arguments
    /// * `phase` - The current phase of the game. This should already be scaled to the range [0, max_phase].
    ///
    /// # Returns
    /// The tapered score.
    pub fn taper(&self, phase: f64) -> f64 {
        self.mg() * phase + self.eg() * (1. - phase)
    }
}

impl From<PhasedScore> for TuningScore {
    fn from(value: PhasedScore) -> Self {
        Self {
            mg: value.mg() as f64,
            eg: value.eg() as f64,
        }
    }
}

impl TryInto<PhasedScore> for TuningScore {
    type Error = ();

    fn try_into(self) -> Result<PhasedScore, Self::Error> {
        Ok(PhasedScore::new(self.mg as i16, self.eg as i16))
    }
}

impl Add<TuningScore> for TuningScore {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            mg: self.mg + rhs.mg,
            eg: self.eg + rhs.eg,
        }
    }
}

impl Add<f64> for TuningScore {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        Self {
            mg: self.mg + rhs,
            eg: self.eg + rhs,
        }
    }
}

impl Add<TuningScore> for f64 {
    type Output = TuningScore;

    fn add(self, rhs: TuningScore) -> Self::Output {
        TuningScore {
            mg: self + rhs.mg,
            eg: self + rhs.eg,
        }
    }
}

impl AddAssign<TuningScore> for TuningScore {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Mul<TuningScore> for TuningScore {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            mg: self.mg * rhs.mg,
            eg: self.eg * rhs.eg,
        }
    }
}

impl Mul<f64> for TuningScore {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self {
            mg: self.mg * rhs,
            eg: self.eg * rhs,
        }
    }
}

impl Mul<TuningScore> for f64 {
    type Output = TuningScore;

    fn mul(self, rhs: TuningScore) -> Self::Output {
        TuningScore {
            mg: self * rhs.mg,
            eg: self * rhs.eg,
        }
    }
}

impl Sub<TuningScore> for TuningScore {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            mg: self.mg - rhs.mg,
            eg: self.eg - rhs.eg,
        }
    }
}

impl Sub<f64> for TuningScore {
    type Output = Self;

    fn sub(self, rhs: f64) -> Self::Output {
        Self {
            mg: self.mg - rhs,
            eg: self.eg - rhs,
        }
    }
}

impl Sub<TuningScore> for f64 {
    type Output = TuningScore;

    fn sub(self, rhs: TuningScore) -> Self::Output {
        TuningScore {
            mg: self - rhs.mg,
            eg: self - rhs.eg,
        }
    }
}

impl SubAssign<TuningScore> for TuningScore {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Div<TuningScore> for TuningScore {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self {
            mg: self.mg / rhs.mg,
            eg: self.eg / rhs.eg,
        }
    }
}

impl Div<f64> for TuningScore {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        Self {
            mg: self.mg / rhs,
            eg: self.eg / rhs,
        }
    }
}

impl Div<TuningScore> for f64 {
    type Output = TuningScore;

    fn div(self, rhs: TuningScore) -> Self::Output {
        TuningScore {
            mg: self / rhs.mg,
            eg: self / rhs.eg,
        }
    }
}

impl Debug for TuningScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "S({: >4.0}, {: >4.0})", self.mg(), self.eg())
    }
}
