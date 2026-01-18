// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use anyhow::Result;

use crate::{bitboard::Bitboard, definitions::FILE_BITBOARDS};

/// Represents a file on the chess board.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl File {
    /// Returns the file offset by `delta` if it is within range.
    /// Returns `None` if the resulting file is out of bounds.
    ///
    /// # Arguments
    ///
    /// - `delta`: The offset to apply to the file.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::file::File;
    ///
    /// assert_eq!(File::A.offset(1), Some(File::B));
    /// assert_eq!(File::A.offset(-1), None);
    /// assert_eq!(File::H.offset(1), None);
    /// assert_eq!(File::H.offset(-1), Some(File::G));
    /// ```
    pub const fn offset(&self, delta: i8) -> Option<Self> {
        let new_file = (*self as i8) + delta;
        if new_file >= 0 && new_file <= 7 {
            return Some(unsafe { std::mem::transmute::<u8, File>(new_file as u8) });
        }
        None
    }

    pub const fn to_bitboard(self) -> Bitboard {
        FILE_BITBOARDS[self as usize]
    }

    pub const fn of(sq: u8) -> Self {
        match sq & 7u8 {
            0 => Self::A,
            1 => Self::B,
            2 => Self::C,
            3 => Self::D,
            4 => Self::E,
            5 => Self::F,
            6 => Self::G,
            7 => Self::H,
            _ => unreachable!(),
        }
    }

    /// Returns the character representation of the file (lowercase)
    pub fn to_char(&self) -> char {
        match self {
            Self::A => 'a',
            Self::B => 'b',
            Self::C => 'c',
            Self::D => 'd',
            Self::E => 'e',
            Self::F => 'f',
            Self::G => 'g',
            Self::H => 'h',
        }
    }
}

impl TryFrom<u8> for File {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            2 => Ok(Self::C),
            3 => Ok(Self::D),
            4 => Ok(Self::E),
            5 => Ok(Self::F),
            6 => Ok(Self::G),
            7 => Ok(Self::H),
            _ => Err(anyhow::Error::msg(format!("Invalid file {value}"))),
        }
    }
}

impl TryFrom<char> for File {
    type Error = anyhow::Error;
    fn try_from(value: char) -> Result<Self> {
        match value {
            'a' => Ok(Self::A),
            'b' => Ok(Self::B),
            'c' => Ok(Self::C),
            'd' => Ok(Self::D),
            'e' => Ok(Self::E),
            'f' => Ok(Self::F),
            'g' => Ok(Self::G),
            'h' => Ok(Self::H),
            _ => Err(anyhow::Error::msg(format!("Invalid file {value}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offset() {
        assert_eq!(File::A.offset(1), Some(File::B));
        assert_eq!(File::A.offset(-1), None);
        assert_eq!(File::H.offset(1), None);
        assert_eq!(File::H.offset(-1), Some(File::G));
    }

    #[test]
    fn to_char() {
        assert_eq!(File::A.to_char(), 'a');
        assert_eq!(File::B.to_char(), 'b');
        assert_eq!(File::C.to_char(), 'c');
        assert_eq!(File::D.to_char(), 'd');
        assert_eq!(File::E.to_char(), 'e');
        assert_eq!(File::F.to_char(), 'f');
        assert_eq!(File::G.to_char(), 'g');
        assert_eq!(File::H.to_char(), 'h');
    }

    #[test]
    fn from_char() {
        assert_eq!(File::try_from('a').unwrap(), File::A);
        assert_eq!(File::try_from('b').unwrap(), File::B);
        assert_eq!(File::try_from('c').unwrap(), File::C);
        assert_eq!(File::try_from('d').unwrap(), File::D);
        assert_eq!(File::try_from('e').unwrap(), File::E);
        assert_eq!(File::try_from('f').unwrap(), File::F);
        assert_eq!(File::try_from('g').unwrap(), File::G);
        assert_eq!(File::try_from('h').unwrap(), File::H);

        for c in 'i'..='z' {
            assert!(File::try_from(c).is_err());
        }
    }

    #[test]
    fn from_u8() {
        assert_eq!(File::try_from(0).unwrap(), File::A);
        assert_eq!(File::try_from(1).unwrap(), File::B);
        assert_eq!(File::try_from(2).unwrap(), File::C);
        assert_eq!(File::try_from(3).unwrap(), File::D);
        assert_eq!(File::try_from(4).unwrap(), File::E);
        assert_eq!(File::try_from(5).unwrap(), File::F);
        assert_eq!(File::try_from(6).unwrap(), File::G);
        assert_eq!(File::try_from(7).unwrap(), File::H);

        for i in 8..=u8::MAX {
            assert!(File::try_from(i).is_err());
        }
    }

    #[test]
    fn to_bitboard() {
        for file_index in 0..8u8 {
            let file = File::try_from(file_index).unwrap();
            let bb = file.to_bitboard();
            println!("File {:?}\n{}", file, bb);

            let not_bb = Bitboard::new(!(bb.as_number()));
            println!("Not BB:\n{}", not_bb);

            assert_eq!(bb & not_bb, Bitboard::new(0u64));
            assert_eq!(bb | not_bb, Bitboard::new(u64::MAX));
        }
    }
}
