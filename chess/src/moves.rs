// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::fmt::Display;

use anyhow::{Result, bail};

use crate::{
    pieces::{PIECE_SHORT_NAMES, Piece, SQUARE_NAME},
    square::Square,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MoveFlag {
    Standard = 0,
    DoublePush = 1,
    EnPassant = 2,
    CastleK = 3,
    CastleQ = 4,
    PromotionQueen = 5,
    PromotionRook = 6,
    PromotionBishop = 7,
    PromotionKnight = 8,
}

impl MoveFlag {
    pub fn is_promotion(&self) -> bool {
        matches!(
            self,
            MoveFlag::PromotionQueen
                | MoveFlag::PromotionRook
                | MoveFlag::PromotionBishop
                | MoveFlag::PromotionKnight
        )
    }

    pub fn promotion_piece(&self) -> Option<Piece> {
        match self {
            MoveFlag::PromotionQueen => Some(Piece::Queen),
            MoveFlag::PromotionRook => Some(Piece::Rook),
            MoveFlag::PromotionBishop => Some(Piece::Bishop),
            MoveFlag::PromotionKnight => Some(Piece::Knight),
            _ => None,
        }
    }

    #[allow(clippy::panic)]
    pub fn from_promotion_piece(piece: Piece) -> Self {
        match piece {
            Piece::Queen => MoveFlag::PromotionQueen,
            Piece::Rook => MoveFlag::PromotionRook,
            Piece::Bishop => MoveFlag::PromotionBishop,
            Piece::Knight => MoveFlag::PromotionKnight,
            _ => panic!(
                "Invalid promotion piece: {:?} cannot be used for promotion",
                piece
            ),
        }
    }

    pub fn validate(&self, moving_piece: Piece) -> Result<()> {
        match self {
            MoveFlag::PromotionBishop
            | MoveFlag::PromotionKnight
            | MoveFlag::PromotionQueen
            | MoveFlag::PromotionRook => {
                if moving_piece != Piece::Pawn {
                    bail!(
                        "Invalid move flag: {:?} cannot be used with moving piece {:?}",
                        self,
                        moving_piece
                    );
                }
            }
            MoveFlag::DoublePush => {
                if moving_piece != Piece::Pawn {
                    bail!(
                        "Invalid move flag: {:?} cannot be used with moving piece {:?}",
                        self,
                        moving_piece
                    );
                }
            }
            MoveFlag::EnPassant => {
                if moving_piece != Piece::Pawn {
                    bail!(
                        "Invalid move flag: {:?} cannot be used with moving piece {:?}",
                        self,
                        moving_piece
                    );
                }
            }
            MoveFlag::CastleK | MoveFlag::CastleQ => {
                if moving_piece != Piece::King {
                    bail!(
                        "Invalid move flag: {:?} cannot be used with moving piece {:?}",
                        self,
                        moving_piece
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveType {
    Quiet,
    Capture,
    All,
}

// Bit masks for move information
const MOVE_INFO_TO_MASK: u16 = 0b111111;
const MOVE_INFO_FROM_MASK: u16 = 0b111111000000;
const MOVE_INFO_DESCRIPTOR_MASK: u16 = 0b1111000000000000;

// Shifts for move information
const MOVE_INFO_TO_SHIFT: u16 = 0;
const MOVE_INFO_FROM_SHIFT: u16 = 6;
const MOVE_INFO_MOVE_DESCRIPTOR_SHIFT: u16 = 12;

/// Compact 16-bit move representation. Inspired by Hobbes and Carp.
///
/// ```ignore
///      -------- ----------- ------------
///     |  Type  |   From    |     To     |
///      --------|-----------|------------
///     | 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 |
/// MSB  --------|-----------|------------ LSB
/// ```
#[derive(Default, Debug, Clone, Copy)]
pub struct Move {
    /// The move information, from LSB to MSB:
    /// - The first 6 bits represent the from square (0-63).
    /// - Next 6 bits represent the to square (0-63).
    /// - Final 4 bits represent the move descriptor (0-15).
    move_info: u16,
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, type: {}",
            self.to_long_algebraic(),
            self.flag() as u8,
        )
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        self.move_info == other.move_info
    }
}

impl PartialOrd for Move {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.move_info.partial_cmp(&other.move_info)
    }
}

impl Move {
    /// Creates a new [`Move`].
    pub fn new(from: Square, to: Square, descriptor: MoveFlag) -> Self {
        Self {
            move_info: (to.to_square_index() as u16)
                | ((from.to_square_index() as u16) << MOVE_INFO_FROM_SHIFT)
                | ((descriptor as u16) << MOVE_INFO_MOVE_DESCRIPTOR_SHIFT),
        }
    }

    /// Checks if the underlying move information is valid (i.e. non-zero).
    pub fn is_valid(&self) -> bool {
        !self.is_null_move()
    }

    /// Create a new castle move
    pub fn new_castle(king_from: Square, king_to: Square) -> Self {
        let flag = if king_to.file > king_from.file {
            MoveFlag::CastleK
        } else {
            MoveFlag::CastleQ
        };

        Self::new(king_from, king_to, flag)
    }

    /// Returns the from [`Square`] of the move.
    pub fn from(&self) -> u8 {
        ((self.move_info & MOVE_INFO_FROM_MASK) >> MOVE_INFO_FROM_SHIFT) as u8
    }

    /// Returns the to [`Square`] of the move.
    pub fn to(&self) -> u8 {
        ((self.move_info & MOVE_INFO_TO_MASK) >> MOVE_INFO_TO_SHIFT) as u8
    }

    pub fn flag(&self) -> MoveFlag {
        match (self.move_info & MOVE_INFO_DESCRIPTOR_MASK) >> MOVE_INFO_MOVE_DESCRIPTOR_SHIFT {
            0 => MoveFlag::Standard,
            1 => MoveFlag::DoublePush,
            2 => MoveFlag::EnPassant,
            3 => MoveFlag::CastleK,
            4 => MoveFlag::CastleQ,
            5 => MoveFlag::PromotionQueen,
            6 => MoveFlag::PromotionRook,
            7 => MoveFlag::PromotionBishop,
            8 => MoveFlag::PromotionKnight,
            _ => MoveFlag::Standard, // default to standard if somehow invalid
        }
    }

    /// Checks if the move is an en passant capture.
    pub fn is_en_passant_capture(&self) -> bool {
        self.flag() == MoveFlag::EnPassant
    }

    /// Checks if the move is a castle move.
    pub fn is_castle(&self) -> bool {
        self.flag() == MoveFlag::CastleK || self.flag() == MoveFlag::CastleQ
    }

    /// Checks if the move is a pawn two up move.
    pub fn is_pawn_two_up(&self) -> bool {
        self.flag() == MoveFlag::DoublePush
    }

    /// Checks if the move is a promotion move and promotes to a queen.
    pub fn is_promote_to_queen(&self) -> bool {
        self.flag() == MoveFlag::PromotionQueen
    }

    /// Checks if the move is a promotion move and promotes to a knight.
    pub fn is_promote_to_knight(&self) -> bool {
        self.flag() == MoveFlag::PromotionKnight
    }

    /// Checks if the move is a promotion move and promotes to a rook.
    pub fn is_promote_to_rook(&self) -> bool {
        self.flag() == MoveFlag::PromotionRook
    }

    /// Checks if the move is a promotion move and promotes to a bishop.
    pub fn is_promote_to_bishop(&self) -> bool {
        self.flag() == MoveFlag::PromotionBishop
    }

    /// Checks if the move is a promotion move.
    pub fn is_promotion(&self) -> bool {
        self.flag().is_promotion()
    }

    /// Returns the [`Piece`] that the move promotes to if any. Can be `None`.
    pub fn promotion_piece(&self) -> Option<Piece> {
        match self.flag() {
            MoveFlag::PromotionQueen => Some(Piece::Queen),
            MoveFlag::PromotionRook => Some(Piece::Rook),
            MoveFlag::PromotionBishop => Some(Piece::Bishop),
            MoveFlag::PromotionKnight => Some(Piece::Knight),
            _ => None,
        }
    }

    /// Return true if the move is a null move
    pub fn is_null_move(&self) -> bool {
        // this is the default value, and should be interpreted as a null move
        // the reason for this is that a move at a minimum should always have a to and from square
        // and a piece. So if there is no information about the move, it is a null move
        self.move_info == 0
    }

    pub fn to_long_algebraic(&self) -> String {
        let from = SQUARE_NAME[self.from() as usize];
        let to = SQUARE_NAME[self.to() as usize];
        // handle promotion too
        let promotion_piece = self.promotion_piece().map_or(Piece::NONE, |p| p as u32);
        format!(
            "{}{}{}",
            from,
            to,
            PIECE_SHORT_NAMES[promotion_piece as usize].to_ascii_lowercase()
        )
        .trim()
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::file::File;
    use crate::moves::{Move, MoveFlag};
    use crate::pieces::Piece;
    use crate::rank::Rank;
    use crate::square::Square;
    #[test]
    fn new_move() {
        {
            let from = Square::new(File::B, Rank::R1);
            let to = Square::new(File::C, Rank::R2);
            let m = Move::new(from, to, MoveFlag::Standard);
            assert_eq!(m.from(), 1);
            assert_eq!(m.to(), 10);
            assert!(!m.is_promotion());
        }

        {
            let from = Square::new(File::H, Rank::R8);
            let to = Square::new(File::A, Rank::R8);
            let m = Move::new(from, to, MoveFlag::Standard);
            assert_eq!(m.from(), 63);
            assert_eq!(m.to(), 56);
            assert!(!m.is_promotion());
        }

        {
            let from = Square::new(File::F, Rank::R4);
            let to = Square::new(File::E, Rank::R6);
            let m = Move::new(from, to, MoveFlag::EnPassant);
            assert_eq!(m.from(), from.to_square_index());
            assert_eq!(m.to(), to.to_square_index());
            assert!(!m.is_pawn_two_up());
            assert!(!m.is_castle());
            assert!(m.is_en_passant_capture());
        }
        {
            let from = Square::new(File::A, Rank::R2);
            let to = Square::new(File::A, Rank::R4);
            let m = Move::new(from, to, MoveFlag::DoublePush);
            assert_eq!(m.from(), 8);
            assert_eq!(m.to(), 24);
            assert!(!m.is_castle());
            assert!(!m.is_en_passant_capture());
            assert!(m.is_pawn_two_up());
        }
        {
            let from = Square::new(File::A, Rank::R7);
            let to = Square::new(File::A, Rank::R8);
            let m = Move::new(from, to, MoveFlag::PromotionQueen);
            assert_eq!(m.from(), 48);
            assert_eq!(m.to(), 56);
            assert!(m.is_promote_to_queen());
            assert!(m.is_promotion());
            assert_eq!(m.promotion_piece().unwrap(), Piece::Queen);
        }
        {
            let from = Square::new(File::A, Rank::R7);
            let to = Square::new(File::A, Rank::R8);
            let m = Move::new(from, to, MoveFlag::PromotionKnight);
            assert_eq!(m.from(), 48);
            assert_eq!(m.to(), 56);
            assert!(m.is_promote_to_knight());
            assert!(m.is_promotion());
            assert_eq!(m.promotion_piece().unwrap(), Piece::Knight);
        }
        {
            let from = Square::new(File::A, Rank::R7);
            let to = Square::new(File::A, Rank::R8);
            let m = Move::new(from, to, MoveFlag::PromotionRook);
            assert_eq!(m.from(), 48);
            assert_eq!(m.to(), 56);
            assert!(m.is_promote_to_rook());
            assert!(m.is_promotion());
            assert_eq!(m.promotion_piece().unwrap(), Piece::Rook);
        }
        {
            let from = Square::new(File::A, Rank::R7);
            let to = Square::new(File::A, Rank::R8);
            let m = Move::new(from, to, MoveFlag::PromotionBishop);
            assert_eq!(m.from(), 48);
            assert_eq!(m.to(), 56);
            assert!(m.is_promote_to_bishop());
            assert!(!m.is_promote_to_rook());
            assert!(!m.is_promote_to_queen());
            assert!(!m.is_promote_to_knight());
            assert!(m.is_promotion());
            assert_eq!(m.promotion_piece().unwrap(), Piece::Bishop);
        }
    }

    #[test]
    fn move_types() {
        let from = Square::new(File::A, Rank::R2);
        let to = Square::new(File::A, Rank::R4);

        let mut mv = Move::new(from, to, MoveFlag::Standard);

        assert!(!mv.is_en_passant_capture());
        assert!(!mv.is_pawn_two_up());
        assert!(!mv.is_castle());
        assert!(!mv.is_promotion());
        assert!(!mv.is_null_move());
        assert_eq!(mv.from(), from.to_square_index());
        assert_eq!(mv.to(), to.to_square_index());

        mv = Move::new(from, to, MoveFlag::DoublePush);

        assert!(!mv.is_en_passant_capture());
        assert!(mv.is_pawn_two_up());
        assert!(!mv.is_castle());
        assert!(!mv.is_promotion());
        assert!(!mv.is_null_move());
        assert!(mv.flag() == MoveFlag::DoublePush);
        assert_eq!(mv.from(), from.to_square_index());
        assert_eq!(mv.to(), to.to_square_index());
    }
}
