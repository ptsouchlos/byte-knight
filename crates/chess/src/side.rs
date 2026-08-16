// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    fmt::Display,
    ops::{Index, IndexMut, Not},
};

/// Represents a side to play in chess.
#[repr(usize)]
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    #[default]
    White = 0,
    Black = 1,
}

impl Side {
    pub const COUNT: usize = 2;
    const ALL_SIDES: [Self; Self::COUNT] = [Side::White, Side::Black];

    /// Returns the opposite side.
    pub fn opposite(&self) -> Side {
        match self {
            Side::White => Side::Black,
            Side::Black => Side::White,
        }
    }

    /// Returns the rank delta of a single pawn push for this side: `+1` for
    /// White (toward rank 8), `-1` for Black (toward rank 1).
    pub const fn forward_delta(self) -> i8 {
        match self {
            Side::White => 1,
            Side::Black => -1,
        }
    }

    /// Returns `true` if the side is [`White`].
    ///
    /// [`White`]: Side::White
    #[must_use]
    pub fn is_white(&self) -> bool {
        matches!(self, Self::White)
    }

    /// Returns `true` if the side is [`Black`].
    ///
    /// [`Black`]: Side::Black
    #[must_use]
    pub fn is_black(&self) -> bool {
        matches!(self, Self::Black)
    }

    pub fn iter() -> impl Iterator<Item = Side> {
        Side::ALL_SIDES.into_iter()
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::White => write!(f, "W"),
            Self::Black => write!(f, "B"),
        }
    }
}

impl TryFrom<u8> for Side {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::White),
            1 => Ok(Self::Black),
            _ => Err(()),
        }
    }
}

impl TryFrom<char> for Side {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'w' => Ok(Self::White),
            'b' => Ok(Self::Black),
            _ => Err(()),
        }
    }
}

impl Not for Side {
    type Output = Side;

    fn not(self) -> Self::Output {
        self.opposite()
    }
}

impl<T, const N: usize> Index<Side> for [T; N] {
    type Output = T;

    fn index(&self, stm: Side) -> &Self::Output {
        &self[stm as usize]
    }
}

impl<T, const N: usize> IndexMut<Side> for [T; N] {
    fn index_mut(&mut self, stm: Side) -> &mut Self::Output {
        &mut self[stm as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_default() {
        let side: Side = Default::default();
        assert_eq!(side, Side::White);
    }

    #[test]
    fn side_from_u8() {
        assert_eq!(Side::try_from(0), Ok(Side::White));
        assert_eq!(Side::try_from(1), Ok(Side::Black));
        assert_eq!(Side::try_from(3), Err(()));
    }

    #[test]
    fn side_from_char() {
        assert_eq!(Side::try_from('w'), Ok(Side::White));
        assert_eq!(Side::try_from('b'), Ok(Side::Black));

        for char in ('a'..='z').filter(|val| *val != 'w' && *val != 'b') {
            assert!(Side::try_from(char).is_err());
        }
    }

    #[test]
    fn display_side() {
        assert_eq!(Side::White.to_string(), "W");
        assert_eq!(Side::Black.to_string(), "B");
    }

    #[test]
    fn opposite() {
        assert_eq!(Side::White.opposite(), Side::Black);
        assert_eq!(Side::Black.opposite(), Side::White);
    }

    #[test]
    fn not_operator() {
        assert_eq!(!Side::White, Side::Black);
        assert_eq!(!Side::Black, Side::White);
    }

    #[test]
    fn is_white() {
        assert!(Side::White.is_white());
        assert!(!Side::Black.is_white());
    }

    #[test]
    fn is_black() {
        assert!(!Side::White.is_black());
        assert!(Side::Black.is_black());
    }
}
