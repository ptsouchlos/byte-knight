// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    fmt::Display,
    hash::Hash,
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
        ShrAssign,
    },
};

use crate::square::Square;

/// Bitboard representation of a chess board.
/// LSB (bit 0) is a1, MSB (bit 63) is h8.
/// The board is represented as a 64-bit integer.
/// A bit is set if the corresponding square is occupied.
///
/// v-a8 (bit 56)
/// 0 0 0 0 0 0 0 0 <- h8 (bit 63)
/// 0 0 0 0 0 0 0 0
/// 0 0 0 0 0 0 0 0
/// 0 0 0 0 0 0 0 0
/// 0 0 0 0 1 0 0 0
/// 0 0 0 0 0 0 0 0 <- h3 (bit 23)
/// 0 0 0 0 0 0 0 0
/// 0 0 0 0 0 0 0 0 <- h1 (bit 7)
/// ^-a1 (bit 0)
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard {
    data: u64,
}

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard::new(0);
    pub const FULL: Bitboard = Bitboard::new(0xFFFFFFFFFFFFFFFF);

    /// Create a new Bitboard with the given data.
    pub const fn new(data: u64) -> Self {
        Bitboard { data }
    }

    /// Create an empty Bitboard.
    pub const fn default() -> Self {
        Bitboard { data: 0 }
    }

    /// Create a bitboard from the given square index.
    pub const fn from_square(square: u8) -> Self {
        Bitboard { data: 1 << square }
    }

    /// Create a filled Bitboard.
    pub const fn filled() -> Self {
        Bitboard { data: u64::MAX }
    }

    /// Check if a square is occupied.
    pub fn is_square_occupied(&self, square: u8) -> bool {
        self.data & (1 << square) != 0
    }

    /// Mark a square as occupied.
    pub const fn set_square(&mut self, square: u8) {
        self.clear_square(square);
        self.data |= 1 << square;
    }

    /// Clear a given square.
    pub const fn clear_square(&mut self, square: u8) {
        self.data &= !(1 << square);
    }

    /// Get the number of occupied squares on the board.
    pub const fn number_of_occupied_squares(&self) -> u32 {
        self.data.count_ones()
    }

    /// Convert to a 64-bit unsigned integer.
    pub const fn as_number(&self) -> u64 {
        self.data
    }

    /// Check if the bitboard intersects with another bitboard.
    ///
    /// # Arguments
    ///
    /// - `other` - The other bitboard to check for intersection.
    ///
    /// # Returns
    ///
    /// - `bool` - True if the bitboards intersect, false otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use chess::bitboard::Bitboard;
    ///
    /// let bb1 = Bitboard::new(0x8000000000000001);
    /// let bb2 = Bitboard::new(0x0000000000000001);
    /// assert!(bb1.intersects(bb2));
    /// ```
    pub const fn intersects(&self, other: Bitboard) -> bool {
        Bitboard::new(self.as_number() & other.as_number()).number_of_occupied_squares() > 0
    }
}

impl PartialOrd<u64> for Bitboard {
    fn partial_cmp(&self, other: &u64) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(other)
    }
}

impl PartialEq<u64> for Bitboard {
    fn eq(&self, other: &u64) -> bool {
        self.data == *other
    }
}

impl BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard {
            data: self.data & rhs.data,
        }
    }
}

impl BitAnd<u64> for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: u64) -> Self::Output {
        Bitboard {
            data: self.data & rhs,
        }
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.data &= rhs.data;
    }
}

impl BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard {
            data: self.data | rhs.data,
        }
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.data |= rhs.data;
    }
}

impl BitXor for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard {
            data: self.data ^ rhs.data,
        }
    }
}

impl BitXor<u64> for Bitboard {
    type Output = Self;
    fn bitxor(self, rhs: u64) -> Self::Output {
        Bitboard {
            data: self.data ^ rhs,
        }
    }
}

impl BitXorAssign<u64> for Bitboard {
    fn bitxor_assign(&mut self, rhs: u64) {
        self.data ^= rhs;
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.data ^= rhs.data;
    }
}

impl Not for Bitboard {
    type Output = Self;
    fn not(self) -> Self::Output {
        Bitboard { data: !self.data }
    }
}

impl Shl for Bitboard {
    type Output = Self;
    fn shl(self, rhs: Bitboard) -> Self::Output {
        Bitboard {
            data: self.data << rhs.data,
        }
    }
}

impl Shl<u64> for Bitboard {
    type Output = Self;
    fn shl(self, rhs: u64) -> Self::Output {
        Bitboard {
            data: self.data << rhs,
        }
    }
}

impl ShlAssign for Bitboard {
    fn shl_assign(&mut self, rhs: Bitboard) {
        self.data <<= rhs.data;
    }
}

impl ShlAssign<u64> for Bitboard {
    fn shl_assign(&mut self, rhs: u64) {
        self.data <<= rhs;
    }
}

impl Shr for Bitboard {
    type Output = Self;
    fn shr(self, rhs: Bitboard) -> Self::Output {
        Bitboard {
            data: self.data >> rhs.data,
        }
    }
}

impl Shr<u64> for Bitboard {
    type Output = Self;
    fn shr(self, rhs: u64) -> Self::Output {
        Bitboard {
            data: self.data >> rhs,
        }
    }
}

impl ShrAssign for Bitboard {
    fn shr_assign(&mut self, rhs: Bitboard) {
        self.data >>= rhs.data;
    }
}

impl ShrAssign<u64> for Bitboard {
    fn shr_assign(&mut self, rhs: u64) {
        self.data >>= rhs;
    }
}

impl Hash for Bitboard {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl From<u64> for Bitboard {
    fn from(data: u64) -> Self {
        Bitboard { data }
    }
}

impl From<Square> for Bitboard {
    fn from(square: Square) -> Self {
        Bitboard::from_square(square.to_square_index())
    }
}

impl From<u8> for Bitboard {
    fn from(square: u8) -> Self {
        Bitboard::from_square(square)
    }
}

impl Default for Bitboard {
    fn default() -> Self {
        Bitboard::default()
    }
}

// Allow printing the Bitboard
impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const LAST_BIT: u64 = 63;
        for rank in 0..8 {
            for file in (0..8).rev() {
                let mask = 1u64 << (LAST_BIT - (rank * 8) - file);
                let symbol = if self.data & mask != 0 { 'x' } else { '-' };
                write!(f, "{symbol} ")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{bitboard_helpers, definitions::Squares};

    use super::*;

    #[test]
    fn bitboard_new() {
        let bb = Bitboard::new(0x8000000000000001);
        assert_eq!(bb.data, 0x8000000000000001);
        println!("{bb}");
    }

    #[test]
    fn is_square_occupied() {
        let bb = Bitboard::new(0x8000000000000001);
        assert!(bb.is_square_occupied(0));
        assert!(bb.is_square_occupied(63));
        assert!(!bb.is_square_occupied(1));
        assert!(!bb.is_square_occupied(62));
    }

    #[test]
    fn set_square() {
        let mut bb = Bitboard::new(0);
        bb.set_square(0);
        bb.set_square(63);
        assert_eq!(bb.data, 0x8000000000000001);

        bb = Bitboard::new(0);
        bb.set_square(28);
        assert_eq!(bb.data, 0x10000000);
    }

    #[test]
    fn clear_square() {
        let mut bb = Bitboard::new(0xFFFFFFFFFFFFFFFF);
        bb.clear_square(0);
        bb.clear_square(63);
        assert_eq!(bb.data, 0x7FFFFFFFFFFFFFFE);
    }

    #[test]
    fn bitwise_operations() {
        let bb1 = Bitboard::new(0xF0F0F0F0F0F0F0F0);
        let bb2 = Bitboard::new(0x0F0F0F0F0F0F0F0F);

        // AND
        assert_eq!((bb1 & bb2), 0);

        // OR
        assert_eq!((bb1 | bb2), 0xFFFFFFFFFFFFFFFF);

        // XOR
        assert_eq!((bb1 ^ bb2), 0xFFFFFFFFFFFFFFFF);
        assert_eq!(bb1 ^ 0x0F0F0F0F0F0F0F0F, 0xFFFFFFFFFFFFFFFF);
        let mut bb_xor = bb1;
        bb_xor ^= bb2;
        assert_eq!(bb_xor, 0xFFFFFFFFFFFFFFFF);

        // NOT
        assert_eq!((!bb1), 0x0F0F0F0F0F0F0F0F);
    }

    #[test]
    fn shifts() {
        let bb = Bitboard::new(0x8000000000000001);

        assert_eq!(bb << 1, Bitboard::new(0x2));
        assert_eq!(bb << 2, Bitboard::new(0x4));
        assert_eq!(bb << Bitboard::new(1), Bitboard::new(0x2));
        assert_eq!(bb << Bitboard::new(2), Bitboard::new(0x4));

        assert_eq!(bb >> 1, Bitboard::new(0x4000000000000000));
        assert_eq!(bb >> 2, Bitboard::new(0x2000000000000000));
        assert_eq!(bb >> Bitboard::new(1), Bitboard::new(0x4000000000000000));
        assert_eq!(bb >> Bitboard::new(2), Bitboard::new(0x2000000000000000));

        let mut bb_left = bb;
        bb_left <<= 1;
        assert_eq!(bb_left, Bitboard::new(0x2));
        bb_left = bb;
        bb_left <<= Bitboard::new(1);
        assert_eq!(bb_left, Bitboard::new(0x2));

        let mut bb_right = bb;
        bb_right >>= 1;
        assert_eq!(bb_right, Bitboard::new(0x4000000000000000));
        bb_right = bb;
        bb_right >>= Bitboard::new(1);
        assert_eq!(bb_right, Bitboard::new(0x4000000000000000));
    }

    #[test]
    fn from_square() {
        let bb_a8 = Bitboard::from_square(Squares::A8);
        let bb_g8 = Bitboard::from_square(Squares::G8);
        let bb_h8 = Bitboard::from_square(Squares::H8);
        let bb = Bitboard::from_square(Squares::D5);

        assert_eq!(bb_a8.data, 72057594037927936);
        assert_eq!(bb_g8.data, 4611686018427387904);
        assert_eq!(bb_h8.data, 9223372036854775808);
        assert_eq!(bb.data, 34359738368);
    }

    #[test]
    fn square_shifting() {
        let mut bb = Bitboard::from_square(Squares::B4);
        let mut bb_front = bb << 8;
        let mut bb_back = bb >> 8;
        println!("{bb}\n{bb_front}\n{bb_back}");

        let original_square = bitboard_helpers::next_bit(&mut bb) as u8;
        let front_square = bitboard_helpers::next_bit(&mut bb_front) as u8;
        let back_square = bitboard_helpers::next_bit(&mut bb_back) as u8;

        assert_eq!(original_square, Squares::B4);
        assert_eq!(front_square, Squares::B5);
        assert_eq!(back_square, Squares::B3);
    }

    #[test]
    fn hash() {
        use std::collections::hash_map::HashMap;
        let mut map = HashMap::new();
        for sq in 0..64 {
            let bb = Bitboard::from_square(sq);
            map.insert(bb, sq);
        }

        for sq in 0..64 {
            let bb = Bitboard::from_square(sq);
            assert_eq!(map.get(&bb), Some(&sq));
        }
    }

    #[test]
    fn filled() {
        let bb = Bitboard::filled();
        assert_eq!(bb.data, u64::MAX);
    }
}
