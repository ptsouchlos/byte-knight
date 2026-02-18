// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
};

use crate::score::{LargeScoreType, ScoreType};

/// Represents a phased score in centipawns meaning that the score holds 2 values. One for midgame and one for endgame.
///
/// The mg score is stored in the upper 16 bits and the eg score in the lower 16 bits.
/// MSB mmmmmmmm mmmmmmmm eeeeeeee eeeeeeee LSB
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
#[must_use]
pub struct PhasedScore {
    value: LargeScoreType,
}

pub type PhaseType = i32;
const BITS: usize = ScoreType::BITS as usize;

impl PhasedScore {
    pub const fn new(mg: ScoreType, eg: ScoreType) -> Self {
        // TODO(PT): Check if scores are valid
        Self {
            value: (((mg as LargeScoreType) << BITS) + eg as LargeScoreType),
        }
    }

    pub fn mg(&self) -> ScoreType {
        // shift 16 bits right
        ((self.value + (1 << (BITS - 1))) >> BITS) as ScoreType
    }

    pub fn eg(&self) -> ScoreType {
        // only use the first 16 bits
        (self.value & 0xFFFF) as ScoreType
    }

    pub fn taper(&self, phase: PhaseType, max_phase: PhaseType) -> ScoreType {
        let mg_phase = phase.min(max_phase);
        let eg_phase = max_phase - mg_phase;
        ((self.mg() as PhaseType * mg_phase + self.eg() as PhaseType * eg_phase) / max_phase)
            as ScoreType
    }
}

impl Mul<i32> for PhasedScore {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self::new(
            (self.mg() as i32).saturating_mul(rhs) as ScoreType,
            (self.eg() as i32).saturating_mul(rhs) as ScoreType,
        )
    }
}

impl Add<Self> for PhasedScore {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.mg().saturating_add(rhs.mg()),
            self.eg().saturating_add(rhs.eg()),
        )
    }
}

impl Add<i16> for PhasedScore {
    type Output = Self;

    fn add(self, rhs: i16) -> Self::Output {
        Self::new(
            self.mg().saturating_add(rhs) as ScoreType,
            self.eg().saturating_add(rhs) as ScoreType,
        )
    }
}

impl AddAssign<Self> for PhasedScore {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self::new(
            self.mg().saturating_add(rhs.mg()),
            self.eg().saturating_add(rhs.eg()),
        );
    }
}

impl AddAssign<i16> for PhasedScore {
    fn add_assign(&mut self, rhs: i16) {
        *self = Self::new(
            self.mg().saturating_add(rhs) as ScoreType,
            self.eg().saturating_add(rhs) as ScoreType,
        );
    }
}

const fn phase_score(mg: ScoreType, eg: ScoreType) -> PhasedScore {
    PhasedScore::new(mg, eg)
}

#[allow(non_snake_case)]
pub const fn S(mg: ScoreType, eg: ScoreType) -> PhasedScore {
    phase_score(mg, eg)
}

impl Display for PhasedScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mg: {}, eg: {}", self.mg(), self.eg())
    }
}

#[cfg(test)]
mod tests {
    use crate::{phased_score::PhasedScore, score::ScoreType};

    #[test]
    fn phased_score() {
        use super::PhasedScore;

        let ps = PhasedScore::new(100, 50);
        assert_eq!(ps.mg(), 100);
        assert_eq!(ps.eg(), 50);

        let phase = 50;
        assert_eq!(ps.taper(phase, 100), 75);

        let ps: PhasedScore = PhasedScore::new(40, 80);
        assert_eq!(ps.mg(), 40);
        assert_eq!(ps.eg(), 80);

        let phase = 12;
        assert_eq!(ps.taper(phase, 24), 60);

        let phase = 24;
        let ps = PhasedScore::new(56, -26);
        assert_eq!(ps.mg(), 56);
        assert_eq!(ps.eg(), -26);
        assert_eq!(ps.taper(phase, 24), 56);
    }

    #[test]
    fn phased_score_mul() {
        use super::PhasedScore;

        let ps = PhasedScore::new(100, 50);
        let ps2 = ps * 2;
        assert_eq!(ps2.mg(), 200);
        assert_eq!(ps2.eg(), 100);

        let ps = PhasedScore::new(40, 80);
        let ps2 = ps * 3;
        assert_eq!(ps2.mg(), 120);
        assert_eq!(ps2.eg(), 240);

        let ps = PhasedScore::new(56, -26);
        let ps2 = ps * -1;
        assert_eq!(ps2.mg(), -56);
        assert_eq!(ps2.eg(), 26);

        let ps = PhasedScore::new(100, 500);
        let ps2 = ps * i16::MAX as i32;
        assert_eq!(ps2.mg(), i16::MAX);
        assert_eq!(ps2.eg(), i16::MAX);
    }

    #[test]
    fn store_max_values() {
        let ps = PhasedScore::new(i16::MAX as ScoreType, i16::MAX as ScoreType);
        assert_eq!(ps.mg(), i16::MAX);
        assert_eq!(ps.eg(), i16::MAX);
    }

    #[test]
    fn phased_score_add() {
        use super::PhasedScore;

        let ps1 = PhasedScore::new(100, 50);
        let ps2 = PhasedScore::new(200, 100);
        let ps3 = ps1 + ps2;
        assert_eq!(ps3.mg(), 300);
        assert_eq!(ps3.eg(), 150);

        let ps1 = PhasedScore::new(40, 80);
        let ps2 = PhasedScore::new(120, 240);
        let ps3 = ps1 + ps2;
        assert_eq!(ps3.mg(), 160);
        assert_eq!(ps3.eg(), 320);

        let ps1 = PhasedScore::new(56, -26);
        let ps2 = PhasedScore::new(-56, 26);
        let ps3 = ps1 + ps2;
        assert_eq!(ps3.mg(), 0);
        assert_eq!(ps3.eg(), 0);

        let mut ps1 = PhasedScore::new(100, 500);
        let ps2 = PhasedScore::new(i16::MAX as ScoreType, i16::MAX as ScoreType);
        ps1 += ps2;
        assert_eq!(ps1.mg(), i16::MAX);
        assert_eq!(ps1.eg(), i16::MAX);

        let ps1 = PhasedScore::new(100, 50);
        let ps2 = ps1 + 50;
        assert_eq!(ps2.mg(), 150);
        assert_eq!(ps2.eg(), 100);

        let mut ps1 = PhasedScore::new(100, 50);
        ps1 += 50;
        assert_eq!(ps1.mg(), 150);
        assert_eq!(ps1.eg(), 100);

        let ps1 = PhasedScore::new((i16::MAX - 10) as ScoreType, (i16::MAX - 10) as ScoreType);
        let ps2 = ps1 + 20;
        assert_eq!(ps2.mg(), i16::MAX);
        assert_eq!(ps2.eg(), i16::MAX);
    }
}
