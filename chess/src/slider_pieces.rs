// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::pieces::Piece;
use anyhow::Result;

/// Enum to represent sliding pieces only.
#[derive(Debug, PartialEq, Eq, PartialOrd, Copy, Clone)]
pub enum SliderPiece {
    Rook,
    Bishop,
    Queen,
}

impl SliderPiece {
    /// Returns the piece type of the slider piece.
    pub fn piece_type(&self) -> Piece {
        match self {
            SliderPiece::Rook => Piece::Rook,
            SliderPiece::Bishop => Piece::Bishop,
            SliderPiece::Queen => Piece::Queen,
        }
    }
}

impl TryFrom<Piece> for SliderPiece {
    type Error = ();

    fn try_from(value: Piece) -> Result<Self, Self::Error> {
        match value {
            Piece::Rook => Ok(SliderPiece::Rook),
            Piece::Bishop => Ok(SliderPiece::Bishop),
            Piece::Queen => Ok(SliderPiece::Queen),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SliderPiece;
    use crate::pieces::Piece;

    #[test]
    fn try_from() {
        for piece in [Piece::Rook, Piece::Bishop, Piece::Queen] {
            let try_result = SliderPiece::try_from(piece);
            assert!(try_result.is_ok());
            let slider_piece = try_result.unwrap();
            assert_eq!(slider_piece.piece_type(), piece);
        }

        for piece in [Piece::Pawn, Piece::King, Piece::Knight] {
            let try_result = SliderPiece::try_from(piece);
            assert!(try_result.is_err());
        }
    }
}
