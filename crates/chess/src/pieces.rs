// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::fmt::Display;

use crate::definitions::NumberOf;

/// Names of squares on the board. The index of the square name corresponds to the square index as represented by the bitboard.
/// See the [crate::bitboard::Bitboard] for more information.
#[rustfmt::skip]
pub const SQUARE_NAME: [&str; NumberOf::SQUARES] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8"
];

/// Fully qualified piece names. Use [Pieces] to index into this array.
pub const PIECE_NAMES: [&str; NumberOf::PIECE_TYPES] =
    ["King", "Queen", "Rook", "Bishop", "Knight", "Pawn"];

/// Short names for pieces. Use [Pieces] to index into this array.
pub const PIECE_SHORT_NAMES: [char; NumberOf::PIECE_TYPES + 1] =
    ['K', 'Q', 'R', 'B', 'N', 'P', ' '];

pub const SLIDER_PIECES: [Piece; 3] = [Piece::Rook, Piece::Bishop, Piece::Queen];
pub const ALL_PIECES: [Piece; 6] = [
    Piece::King,
    Piece::Queen,
    Piece::Rook,
    Piece::Bishop,
    Piece::Knight,
    Piece::Pawn,
];

/// Represents a chess piece.
///
/// **Note**: The ordinal value of the piece is used throughout the
/// code to index into arrays and tables. Changing the value of a piece
/// would likely be catastrophic and result in a number of bugs and possibly crashes.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Piece {
    King = 0,
    Queen = 1,
    Rook = 2,
    Bishop = 3,
    Knight = 4,
    Pawn = 5,
}

impl Piece {
    pub const NONE: u32 = 6;

    /// Returns `true` if the piece is [`King`].
    ///
    /// [`King`]: Piece::King
    #[must_use]
    pub fn is_king(&self) -> bool {
        matches!(self, Self::King)
    }

    /// Returns `true` if the piece is [`Queen`].
    ///
    /// [`Queen`]: Piece::Queen
    #[must_use]
    pub fn is_queen(&self) -> bool {
        matches!(self, Self::Queen)
    }

    /// Returns `true` if the piece is [`Rook`].
    ///
    /// [`Rook`]: Piece::Rook
    #[must_use]
    pub fn is_rook(&self) -> bool {
        matches!(self, Self::Rook)
    }

    /// Returns `true` if the piece is [`Bishop`].
    ///
    /// [`Bishop`]: Piece::Bishop
    #[must_use]
    pub fn is_bishop(&self) -> bool {
        matches!(self, Self::Bishop)
    }

    /// Returns `true` if the piece is [`Knight`].
    ///
    /// [`Knight`]: Piece::Knight
    #[must_use]
    pub fn is_knight(&self) -> bool {
        matches!(self, Self::Knight)
    }

    /// Returns `true` if the piece is [`Pawn`].
    ///
    /// [`Pawn`]: Piece::Pawn
    #[must_use]
    pub fn is_pawn(&self) -> bool {
        matches!(self, Self::Pawn)
    }

    /// Returns `true` if the piece is a slider piece.
    ///
    /// A slider piece is a piece that can move any number of squares in a straight line.
    #[must_use]
    pub fn is_slider(&self) -> bool {
        self.is_rook() || self.is_bishop() || self.is_queen()
    }

    /// Returns the short name of the piece as a lowercase character.
    pub fn as_char(&self) -> char {
        PIECE_SHORT_NAMES[*self as usize].to_ascii_lowercase()
    }

    /// Returns an iterator over all the pieces.
    pub fn iter() -> impl Iterator<Item = Piece> {
        ALL_PIECES.iter().copied()
    }
}

impl Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Piece::King => write!(f, "King"),
            Piece::Queen => write!(f, "Queen"),
            Piece::Rook => write!(f, "Rook"),
            Piece::Bishop => write!(f, "Bishop"),
            Piece::Knight => write!(f, "Knight"),
            Piece::Pawn => write!(f, "Pawn"),
        }
    }
}

impl TryFrom<u8> for Piece {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Piece::King),
            1 => Ok(Piece::Queen),
            2 => Ok(Piece::Rook),
            3 => Ok(Piece::Bishop),
            4 => Ok(Piece::Knight),
            5 => Ok(Piece::Pawn),
            _ => Err(()),
        }
    }
}

impl TryFrom<char> for Piece {
    type Error = ();
    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value.to_ascii_uppercase() {
            'K' => Ok(Piece::King),
            'Q' => Ok(Piece::Queen),
            'R' => Ok(Piece::Rook),
            'B' => Ok(Piece::Bishop),
            'N' => Ok(Piece::Knight),
            'P' => Ok(Piece::Pawn),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_from_u8() {
        assert_eq!(Piece::try_from(0), Ok(Piece::King));
        assert_eq!(Piece::try_from(1), Ok(Piece::Queen));
        assert_eq!(Piece::try_from(2), Ok(Piece::Rook));
        assert_eq!(Piece::try_from(3), Ok(Piece::Bishop));
        assert_eq!(Piece::try_from(4), Ok(Piece::Knight));
        assert_eq!(Piece::try_from(5), Ok(Piece::Pawn));
        assert_eq!(Piece::try_from(7), Err(()));
    }

    #[test]
    fn piece_from_char() {
        assert_eq!(Piece::try_from('K'), Ok(Piece::King));
        assert_eq!(Piece::try_from('Q'), Ok(Piece::Queen));
        assert_eq!(Piece::try_from('R'), Ok(Piece::Rook));
        assert_eq!(Piece::try_from('B'), Ok(Piece::Bishop));
        assert_eq!(Piece::try_from('N'), Ok(Piece::Knight));
        assert_eq!(Piece::try_from('P'), Ok(Piece::Pawn));
        assert_eq!(Piece::try_from(' '), Err(()));
    }

    #[test]
    fn piece_display() {
        assert_eq!(Piece::King.to_string(), "King");
        assert_eq!(Piece::Queen.to_string(), "Queen");
        assert_eq!(Piece::Rook.to_string(), "Rook");
        assert_eq!(Piece::Bishop.to_string(), "Bishop");
        assert_eq!(Piece::Knight.to_string(), "Knight");
        assert_eq!(Piece::Pawn.to_string(), "Pawn");
    }

    #[test]
    fn as_char() {
        assert_eq!(Piece::King.as_char(), 'k');
        assert_eq!(Piece::Queen.as_char(), 'q');
        assert_eq!(Piece::Rook.as_char(), 'r');
        assert_eq!(Piece::Bishop.as_char(), 'b');
        assert_eq!(Piece::Knight.as_char(), 'n');
        assert_eq!(Piece::Pawn.as_char(), 'p');
    }

    #[test]
    fn char_roundtrip() {
        for piece in Piece::iter() {
            let c = piece.as_char();
            let piece2 = Piece::try_from(c).unwrap();
            assert_eq!(piece, piece2);
        }
    }

    #[test]
    fn is_functions() {
        assert!(Piece::King.is_king());
        assert!(!Piece::King.is_queen());
        assert!(!Piece::King.is_rook());
        assert!(!Piece::King.is_bishop());
        assert!(!Piece::King.is_knight());
        assert!(!Piece::King.is_pawn());

        assert!(Piece::Queen.is_queen());
        assert!(!Piece::Queen.is_king());
        assert!(!Piece::Queen.is_rook());
        assert!(!Piece::Queen.is_bishop());
        assert!(!Piece::Queen.is_knight());
        assert!(!Piece::Queen.is_pawn());

        assert!(Piece::Bishop.is_bishop());
        assert!(!Piece::Bishop.is_king());
        assert!(!Piece::Bishop.is_queen());
        assert!(!Piece::Bishop.is_rook());
        assert!(!Piece::Bishop.is_knight());
        assert!(!Piece::Bishop.is_pawn());

        assert!(Piece::Knight.is_knight());
        assert!(!Piece::Knight.is_king());
        assert!(!Piece::Knight.is_queen());
        assert!(!Piece::Knight.is_rook());
        assert!(!Piece::Knight.is_bishop());
        assert!(!Piece::Knight.is_pawn());

        assert!(Piece::Rook.is_rook());
        assert!(!Piece::Rook.is_king());
        assert!(!Piece::Rook.is_queen());
        assert!(!Piece::Rook.is_bishop());
        assert!(!Piece::Rook.is_knight());
        assert!(!Piece::Rook.is_pawn());

        assert!(Piece::Pawn.is_pawn());
        assert!(!Piece::Pawn.is_king());
        assert!(!Piece::Pawn.is_queen());
        assert!(!Piece::Pawn.is_rook());
        assert!(!Piece::Pawn.is_bishop());
        assert!(!Piece::Pawn.is_knight());
    }
}
