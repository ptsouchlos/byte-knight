// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{
    bitboard::Bitboard,
    bitboard_helpers,
    color::Color,
    definitions::{DARK_SQUARES, NumberOf},
    file::File,
    rank::Rank,
};

use anyhow::{Result, bail};

/// Represents a square on the chess board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Square {
    pub file: File,
    pub rank: Rank,
}

impl Square {
    pub const A1: Self = Self::new(File::A, Rank::R1);
    pub const A2: Self = Self::new(File::A, Rank::R2);
    pub const A3: Self = Self::new(File::A, Rank::R3);
    pub const A4: Self = Self::new(File::A, Rank::R4);
    pub const A5: Self = Self::new(File::A, Rank::R5);
    pub const A6: Self = Self::new(File::A, Rank::R6);
    pub const A7: Self = Self::new(File::A, Rank::R7);
    pub const A8: Self = Self::new(File::A, Rank::R8);

    pub const B1: Self = Self::new(File::B, Rank::R1);
    pub const B2: Self = Self::new(File::B, Rank::R2);
    pub const B3: Self = Self::new(File::B, Rank::R3);
    pub const B4: Self = Self::new(File::B, Rank::R4);
    pub const B5: Self = Self::new(File::B, Rank::R5);
    pub const B6: Self = Self::new(File::B, Rank::R6);
    pub const B7: Self = Self::new(File::B, Rank::R7);
    pub const B8: Self = Self::new(File::B, Rank::R8);

    pub const C1: Self = Self::new(File::C, Rank::R1);
    pub const C2: Self = Self::new(File::C, Rank::R2);
    pub const C3: Self = Self::new(File::C, Rank::R3);
    pub const C4: Self = Self::new(File::C, Rank::R4);
    pub const C5: Self = Self::new(File::C, Rank::R5);
    pub const C6: Self = Self::new(File::C, Rank::R6);
    pub const C7: Self = Self::new(File::C, Rank::R7);
    pub const C8: Self = Self::new(File::C, Rank::R8);

    pub const D1: Self = Self::new(File::D, Rank::R1);
    pub const D2: Self = Self::new(File::D, Rank::R2);
    pub const D3: Self = Self::new(File::D, Rank::R3);
    pub const D4: Self = Self::new(File::D, Rank::R4);
    pub const D5: Self = Self::new(File::D, Rank::R5);
    pub const D6: Self = Self::new(File::D, Rank::R6);
    pub const D7: Self = Self::new(File::D, Rank::R7);
    pub const D8: Self = Self::new(File::D, Rank::R8);

    pub const E1: Self = Self::new(File::E, Rank::R1);
    pub const E2: Self = Self::new(File::E, Rank::R2);
    pub const E3: Self = Self::new(File::E, Rank::R3);
    pub const E4: Self = Self::new(File::E, Rank::R4);
    pub const E5: Self = Self::new(File::E, Rank::R5);
    pub const E6: Self = Self::new(File::E, Rank::R6);
    pub const E7: Self = Self::new(File::E, Rank::R7);
    pub const E8: Self = Self::new(File::E, Rank::R8);

    pub const F1: Self = Self::new(File::F, Rank::R1);
    pub const F2: Self = Self::new(File::F, Rank::R2);
    pub const F3: Self = Self::new(File::F, Rank::R3);
    pub const F4: Self = Self::new(File::F, Rank::R4);
    pub const F5: Self = Self::new(File::F, Rank::R5);
    pub const F6: Self = Self::new(File::F, Rank::R6);
    pub const F7: Self = Self::new(File::F, Rank::R7);
    pub const F8: Self = Self::new(File::F, Rank::R8);

    pub const G1: Self = Self::new(File::G, Rank::R1);
    pub const G2: Self = Self::new(File::G, Rank::R2);
    pub const G3: Self = Self::new(File::G, Rank::R3);
    pub const G4: Self = Self::new(File::G, Rank::R4);
    pub const G5: Self = Self::new(File::G, Rank::R5);
    pub const G6: Self = Self::new(File::G, Rank::R6);
    pub const G7: Self = Self::new(File::G, Rank::R7);
    pub const G8: Self = Self::new(File::G, Rank::R8);

    pub const H1: Self = Self::new(File::H, Rank::R1);
    pub const H2: Self = Self::new(File::H, Rank::R2);
    pub const H3: Self = Self::new(File::H, Rank::R3);
    pub const H4: Self = Self::new(File::H, Rank::R4);
    pub const H5: Self = Self::new(File::H, Rank::R5);
    pub const H6: Self = Self::new(File::H, Rank::R6);
    pub const H7: Self = Self::new(File::H, Rank::R7);
    pub const H8: Self = Self::new(File::H, Rank::R8);

    pub const MIN: u8 = 0;
    pub const MAX: u8 = 63;

    pub const fn new(file: File, rank: Rank) -> Self {
        Self { file, rank }
    }

    /// Creates a new square from a file character and rank number.
    pub fn from_file_rank(file: char, rank: u8) -> Result<Self> {
        let file = File::try_from(file)?;
        let rank = Rank::try_from(rank)?;
        Ok(Self { file, rank })
    }

    /// Creates a new square from a bitboard.
    ///
    /// This will get the first square from the bitboard and convert it to a [`Square`].
    pub fn from_bitboard(bitboard: &Bitboard) -> Self {
        let sq = bitboard_helpers::next_bit(&mut bitboard.to_owned());
        Self::from_square_index(sq as u8)
    }

    /// Convert to a raw square index (0-63).
    pub const fn to_square_index(&self) -> u8 {
        to_square(self.file as u8, self.rank as u8)
    }

    /// Convert a square index to a [`Square`] object.
    pub const fn from_square_index(square: u8) -> Self {
        assert!(square < NumberOf::SQUARES as u8);
        Self {
            file: File::of(square),
            rank: Rank::of(square),
        }
    }

    /// Offset the square by the given file and rank deltas.
    ///
    /// Returns `None` if the resulting square is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use chess::square::Square;
    /// use chess::rank::Rank;
    /// use chess::file::File;
    ///
    /// let square = Square::try_from("e4").unwrap();
    /// let new_square = square.offset(1, 1).unwrap();
    /// assert_eq!(new_square.file, File::F);
    /// assert_eq!(new_square.rank, Rank::R5);
    ///
    /// let square = Square::try_from("a1").unwrap();
    /// let new_square = square.offset(-1, -1);
    /// assert!(new_square.is_none());
    /// ```
    pub const fn offset(&self, file_delta: i8, rank_delta: i8) -> Option<Self> {
        let new_file = self.file.offset(file_delta);
        let new_rank = self.rank.offset(rank_delta);
        if new_file.is_none() || new_rank.is_none() {
            return None;
        }
        let new_file = new_file.unwrap();
        let new_rank = new_rank.unwrap();
        Some(Self::new(new_file, new_rank))
    }

    /// Get the bitboard representation of the square.
    pub fn bitboard(&self) -> Bitboard {
        Bitboard::from_square(self.to_square_index())
    }

    /// Returns `true` if the square is a dark square.
    pub fn is_dark(&self) -> bool {
        Bitboard::from(DARK_SQUARES) & self.bitboard() != Bitboard::EMPTY
    }

    /// Returns `true` if the square is a light square.
    pub fn is_light(&self) -> bool {
        !self.is_dark()
    }

    /// Returns the color of the square.
    pub fn color(&self) -> Color {
        if self.is_dark() {
            Color::Black
        } else {
            Color::White
        }
    }

    /// Flips the current square and returns a new instance at the flipped location.
    pub fn flip(&self) -> Self {
        let sq = self.to_square_index();
        let flipped_sq = flip(sq);
        Self::from_square_index(flipped_sq)
    }
}

/// Flips the square vertically.
///
/// # Arguments
///
/// - `sq` - The square to flip.
///
/// # Returns
///
/// The flipped square
///
/// # Examples
///
/// ```
/// use chess::square::Square;
/// use chess::square::flip;
/// use chess::file::File;
/// use chess::rank::Rank;
///
/// let sq = Square::new(File::A, Rank::R1);
/// let flipped_sq = flip(sq.to_square_index());
/// assert_eq!(flipped_sq, 56);
/// let new_sq = Square::from_square_index(flipped_sq);
/// assert_eq!(new_sq.file, File::A);
/// assert_eq!(new_sq.rank, Rank::R8);
pub const fn flip(sq: u8) -> u8 {
    sq ^ 56
}

/// Flips the square if the given boolean is `true`.
///
/// This will flip the square vertically if `flip` is `true`.
///
/// # Arguments
///
/// - `flip` - A boolean indicating if the square should be flipped.
/// - `sq` - The square to flip.
///
/// # Returns
///
/// The flipped square or sq if `flip` is `false`.
///
/// # Examples
///
/// ```
/// use chess::square::Square;
/// use chess::square::flip_if;
/// use chess::file::File;
/// use chess::rank::Rank;
///
/// let sq = Square::new(File::A, Rank::R1);
/// let flipped_sq = flip_if(true, sq.to_square_index());
/// assert_eq!(flipped_sq, 56);
/// let new_sq = Square::from_square_index(flipped_sq);
/// assert_eq!(new_sq.file, File::A);
/// assert_eq!(new_sq.rank, Rank::R8);
pub fn flip_if(should_flip: bool, sq: u8) -> u8 {
    if should_flip { flip(sq) } else { sq }
}

impl TryFrom<&str> for Square {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 2 {
            return Err(anyhow::Error::msg(format!(
                "Input square must be at least 2 characters {value}",
            )));
        }

        // file can match directly to a char so we don't alter it
        let file = value.chars().nth(0).unwrap();
        // read the raw rank value (1-8)
        let rank = value.chars().nth(1).unwrap();
        // rank values are 1-8, so we need to convert to 0-7
        let rank_digit = rank.to_digit(10).unwrap() - 1;
        Square::from_file_rank(file, rank_digit as u8)
    }
}

impl TryFrom<u8> for Square {
    type Error = anyhow::Error;

    fn try_from(sq_index: u8) -> Result<Self, Self::Error> {
        if sq_index >= 64 {
            bail!("Invalid square index: {sq_index}");
        }

        Ok(Self::from_square_index(sq_index))
    }
}

/// Converts a file and rank tuple to a square
///
/// # Arguments
///
/// * `file` - The file to convert
/// * `rank` - The rank to convert
///
/// # Returns
///
/// The square corresponding to the given file and rank
pub const fn to_square(file: u8, rank: u8) -> u8 {
    rank * 8 + file
}

/// Converts a file and rank index to a [`Square`] object.
///
/// # Arguments
///
/// * `file` - The file index to convert
/// * `rank` - The rank index to convert
///
/// # Returns
///
/// A [`Square`] object corresponding to the given file and rank indices.
pub fn to_square_object(file: u8, rank: u8) -> Square {
    Square::new(File::try_from(file).unwrap(), Rank::try_from(rank).unwrap())
}

/// Converts a square to a file and rank tuple
///
/// # Arguments
///
/// * `square` - The square to convert
///
/// # Returns
///
/// A tuple containing the file and rank of the given square (file, rank)
pub const fn from_square(square: u8) -> (u8, u8) {
    let rank = square / 8;
    let file = square % 8;
    (file, rank)
}

/// Checks if a given square is on a given rank.
pub const fn is_square_on_rank(square: u8, rank: u8) -> bool {
    let (_, rnk) = from_square(square);
    rnk == rank
}

#[cfg(test)]
mod tests {
    use crate::{
        definitions::{NumberOf, Squares},
        file::File,
        rank::Rank,
        square::{Square, is_square_on_rank, to_square},
    };

    #[test]
    fn check_square_on_rank() {
        assert!(is_square_on_rank(Squares::A1, Rank::R1 as u8));
        assert!(!is_square_on_rank(Squares::A1, Rank::R2 as u8));
        assert!(is_square_on_rank(Squares::C5, Rank::R5 as u8));
    }

    #[test]
    fn parse_square_from_uci_str() {
        let square = Square::try_from("e4").unwrap();
        assert_eq!(square.file, File::E);
        assert_eq!(square.rank, Rank::R4);
    }

    #[test]
    fn offset() {
        let square = Square::try_from("e4").unwrap();
        let new_square = square.offset(1, 1).unwrap();
        assert_eq!(new_square.file, File::F);
        assert_eq!(new_square.rank, Rank::R5);

        let square = Square::try_from("a1").unwrap();
        let new_square = square.offset(-1, -1);
        assert!(new_square.is_none());
    }

    #[test]
    fn flip() {
        for rank in 0..4_u8 {
            for file in 0..NumberOf::FILES as u8 {
                let sq = to_square(file, rank);
                let square = Square::from_square_index(sq);
                let flipped = square.flip();
                assert_eq!(flipped.flip(), square);
                assert_eq!(flipped.file, square.file);
                assert_eq!(flipped.rank.as_number(), (Rank::R8 - square.rank) as u8);
            }
        }
    }
}
