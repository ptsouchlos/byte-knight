// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::pieces::Piece;

/// Representation of non-slider pieces only.
#[derive(Debug, PartialEq, Eq, PartialOrd, Copy, Clone)]
pub enum NonSliderPiece {
    King,
    Knight,
    Pawn,
}

impl NonSliderPiece {
    /// Returns the piece type of the non-slider piece.
    pub fn piece_type(&self) -> Piece {
        match self {
            NonSliderPiece::King => Piece::King,
            NonSliderPiece::Knight => Piece::Knight,
            NonSliderPiece::Pawn => Piece::Pawn,
        }
    }
}
impl TryFrom<Piece> for NonSliderPiece {
    type Error = ();

    fn try_from(value: Piece) -> Result<Self, Self::Error> {
        match value {
            Piece::King => Ok(NonSliderPiece::King),
            Piece::Knight => Ok(NonSliderPiece::Knight),
            Piece::Pawn => Ok(NonSliderPiece::Pawn),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NonSliderPiece;
    use crate::pieces::Piece;

    #[test]
    fn try_from() {
        for piece in [Piece::King, Piece::Knight, Piece::Pawn] {
            let try_piece = NonSliderPiece::try_from(piece);
            assert!(try_piece.is_ok());
            let non_slider_piece = try_piece.unwrap();
            assert_eq!(non_slider_piece.piece_type(), piece);
        }

        for piece in [Piece::Queen, Piece::Bishop, Piece::Rook] {
            let try_piece = NonSliderPiece::try_from(piece);
            assert!(try_piece.is_err());
        }
    }
}
