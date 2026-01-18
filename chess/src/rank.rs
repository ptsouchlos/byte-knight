// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::ops::Sub;

use crate::side::Side;
use anyhow::Result;

/// Represents a rank on the chess board.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    R1 = 0,
    R2 = 1,
    R3 = 2,
    R4 = 3,
    R5 = 4,
    R6 = 5,
    R7 = 6,
    R8 = 7,
}

impl Rank {
    /// Returns the rank of the promotion square for the given side.
    pub const fn promotion_rank(side: Side) -> Rank {
        match side {
            Side::White => Rank::R8,
            Side::Black => Rank::R1,
        }
    }

    /// Returns the starting rank for pawns of a given side.
    pub const fn pawn_start_rank(side: Side) -> Rank {
        match side {
            Side::White => Rank::R2,
            Side::Black => Rank::R7,
        }
    }

    /// Returns the rank as a number.
    pub fn as_number(&self) -> u8 {
        *self as u8
    }

    pub const fn of(sq: u8) -> Self {
        match sq >> 3 {
            0 => Self::R1,
            1 => Self::R2,
            2 => Self::R3,
            3 => Self::R4,
            4 => Self::R5,
            5 => Self::R6,
            6 => Self::R7,
            7 => Self::R8,
            _ => unreachable!(),
        }
    }

    /// Offset the rank by the given delta.
    ///
    /// Returns `None` if the resulting rank is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::rank::Rank;
    ///
    /// assert_eq!(Rank::R1.offset(1), Some(Rank::R2));
    /// assert_eq!(Rank::R1.offset(-1), None);
    /// assert_eq!(Rank::R8.offset(1), None);
    /// assert_eq!(Rank::R8.offset(-1), Some(Rank::R7));
    /// ```
    ///
    pub const fn offset(&self, delta: i8) -> Option<Self> {
        let new_rank = (*self as i8) + delta;
        if new_rank >= 0 && new_rank <= 7 {
            return Some(unsafe { std::mem::transmute::<u8, Rank>(new_rank as u8) });
        }
        None
    }
}

impl TryFrom<u8> for Rank {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::R1),
            1 => Ok(Self::R2),
            2 => Ok(Self::R3),
            3 => Ok(Self::R4),
            4 => Ok(Self::R5),
            5 => Ok(Self::R6),
            6 => Ok(Self::R7),
            7 => Ok(Self::R8),
            _ => Err(anyhow::Error::msg(format!("Invalid rank {value}"))),
        }
    }
}

impl Sub for Rank {
    type Output = i8;

    fn sub(self, other: Self) -> i8 {
        self as i8 - other as i8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rank_offset() {
        assert_eq!(Rank::R1.offset(1), Some(Rank::R2));
        assert_eq!(Rank::R1.offset(-1), None);
        assert_eq!(Rank::R8.offset(1), None);
        assert_eq!(Rank::R8.offset(-1), Some(Rank::R7));
    }
}
