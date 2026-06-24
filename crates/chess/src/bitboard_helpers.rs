// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{bitboard::Bitboard, file::File};

/// Returns the index of the next bit set to 1 in the bitboard and sets it to 0.
///
/// # Arguments
///
/// * `bitboard` - The bitboard to get the next bit from.
///
/// # Returns
///
/// The index of the next bit set to 1 in the bitboard.
///
/// # Examples
///
/// ```
/// use chess::bitboard::Bitboard;
/// use chess::bitboard_helpers::next_bit;
///
/// let mut bb = Bitboard::new(0x8000000000000001);
/// assert_eq!(next_bit(&mut bb), 0);
/// assert_eq!(next_bit(&mut bb), 63);
/// assert_eq!(bb.as_number(), 0);
///
/// ```
///
/// ```
/// use chess::bitboard::Bitboard;
/// use chess::bitboard_helpers::next_bit;
///
/// let mut bb = Bitboard::new(0xFFFFFFFFFFFFFFFF);
/// for i in 0..64 {
///    assert_eq!(next_bit(&mut bb), i);
/// }
///
/// ```
///
pub fn next_bit(bitboard: &mut Bitboard) -> usize {
    let square = bitboard.as_number().trailing_zeros();
    *bitboard ^= 1u64 << square;
    square as usize
}

/// Fills all bits to the north of the given bitboard.
/// # Arguments
/// * `bitboard` - The bitboard to fill to the north.
///
/// # Returns
/// A new bitboard with all bits to the north filled.
///
/// # Examples
///
/// ```
/// use chess::bitboard::Bitboard;
/// use chess::bitboard_helpers::north_fill;
/// use chess::definitions::Squares;
/// let bb = Bitboard::from_square(Squares::A2) | Bitboard::from_square(Squares::H2) | Bitboard::from_square(Squares::D4);
/// let filled = north_fill(bb);
/// // Should look like this:
/// // x - - x - - - x
/// // x - - x - - - x
/// // x - - x - - - x
/// // x - - x - - - x
/// // x - - x - - - x
/// // x - - - - - - x
/// // x - - - - - - x
/// // - - - - - - - -
/// assert_eq!(filled.as_number(), 0x8989898989818100);
/// ```
pub const fn north_fill(bitboard: Bitboard) -> Bitboard {
    let mut b = bitboard.as_number();
    b |= b << 8;
    b |= b << 16;
    b |= b << 32;
    Bitboard::new(b)
}

/// Fills all bits to the south of the given bitboard.
/// # Arguments
/// * `bitboard` - The bitboard to fill to the south.
/// # Returns
/// A new bitboard with all bits to the south filled.
/// # Examples
/// ```
/// use chess::bitboard::Bitboard;
/// use chess::bitboard_helpers::south_fill;
/// use chess::definitions::Squares;
/// let bb = Bitboard::from_square(Squares::A7) | Bitboard::from_square(Squares::H7) | Bitboard::from_square(Squares::D5);
/// let filled = south_fill(bb);
/// // // Should look like this:
/// // - - - - - - - -
/// // x - - - - - - x
/// // x - - - - - - x
/// // x - - x - - - x
/// // x - - x - - - x
/// // x - - x - - - x
/// // x - - x - - - x
/// // x - - x - - - x
/// assert_eq!(filled.as_number(), 0x81818989898989);
/// ```
pub const fn south_fill(bitboard: Bitboard) -> Bitboard {
    let mut b = bitboard.as_number();
    b |= b >> 8;
    b |= b >> 16;
    b |= b >> 32;
    Bitboard::new(b)
}

/// Fills all bits to the east of the given bitboard without wrap-around.
/// # Arguments
/// * `bitboard` - The bitboard to fill to the east.
/// # Returns
/// A new bitboard with all bits to the east filled.
/// # Examples
/// ```
/// use chess::bitboard::Bitboard;
/// use chess::bitboard_helpers::east_fill;
/// use chess::definitions::Squares;
/// let bb = Bitboard::from_square(Squares::A1) | Bitboard::from_square(Squares::C3);
/// let filled = east_fill(bb);
/// // Should fill eastward on each rank without wrapping to the next rank
/// assert_eq!(filled.as_number(), 0x00000000000000FF | 0x0000000000FC0000);
/// ```
pub const fn east_fill(bitboard: Bitboard) -> Bitboard {
    // Mask to exclude H-file to prevent wrap-around
    let not_h_file = !(File::H.to_bitboard().as_number());
    let mut b = bitboard.as_number();
    b |= (b & not_h_file) << 1;
    b |= (b & not_h_file) << 1;
    b |= (b & not_h_file) << 1;
    b |= (b & not_h_file) << 1;
    b |= (b & not_h_file) << 1;
    b |= (b & not_h_file) << 1;
    b |= (b & not_h_file) << 1;
    Bitboard::new(b)
}

/// Fills all bits to the west of the given bitboard without wrap-around.
/// # Arguments
/// * `bitboard` - The bitboard to fill to the west.
/// # Returns
/// A new bitboard with all bits to the west filled.
/// # Examples
/// ```
/// use chess::bitboard::Bitboard;
/// use chess::bitboard_helpers::west_fill;
/// use chess::definitions::Squares;
/// let bb = Bitboard::from_square(Squares::H2) | Bitboard::from_square(Squares::D4);
/// let filled = west_fill(bb);
/// // Should fill eastward on each rank without wrapping to the next rank
/// assert_eq!(filled.as_number(), 0x000000000000FF00 | 0x000000000F000000);
/// ```
pub const fn west_fill(bitboard: Bitboard) -> Bitboard {
    let not_a_file = !(File::A.to_bitboard().as_number());
    let mut b = bitboard.as_number();
    b |= (b & not_a_file) >> 1;
    b |= (b & not_a_file) >> 1;
    b |= (b & not_a_file) >> 1;
    b |= (b & not_a_file) >> 1;
    b |= (b & not_a_file) >> 1;
    b |= (b & not_a_file) >> 1;
    b |= (b & not_a_file) >> 1;
    Bitboard::new(b)
}

const NORTH: u64 = 8;
const SOUTH: u64 = 8;
const WEST: u64 = 1;
const EAST: u64 = 1;
const NORTH_EAST: u64 = 9;
const NORTH_WEST: u64 = 7;
const SOUTH_EAST: u64 = 7;
const SOUTH_WEST: u64 = 9;

/// Shifts the [`Bitboard`] one square to the north.
///
/// # Arguments
/// - `bitboard` - The [`Bitboard`] to shift.
///
/// # Returns
/// A new [`Bitboard`] shifted one square to the north.
pub fn north(bitboard: Bitboard) -> Bitboard {
    bitboard << NORTH
}

/// Shifts the [`Bitboard`] one square to the south.
///
/// # Arguments
/// - `bitboard` - The [`Bitboard`] to shift.
///
/// # Returns
/// A new [`Bitboard`] shifted one square to the south.
pub fn south(bitboard: Bitboard) -> Bitboard {
    bitboard >> SOUTH
}

/// Shifts the [`Bitboard`] one square to the west without wrap-around.
///
/// # Arguments
/// - `bitboard` - The [`Bitboard`] to shift.
///
/// # Returns
/// A new [`Bitboard`] shifted one square to the west without wrap-around.
pub fn west(bitboard: Bitboard) -> Bitboard {
    bitboard >> WEST & !File::H.to_bitboard()
}

/// Shifts the [`Bitboard`] one square to the east without wrap-around.
///
/// # Arguments
/// - `bitboard` - The [`Bitboard`] to shift.
///
/// # Returns
/// A new [`Bitboard`] shifted one square to the east without wrap-around.
pub fn east(bitboard: Bitboard) -> Bitboard {
    bitboard << EAST & !File::A.to_bitboard()
}

pub fn north_west(bitboard: Bitboard) -> Bitboard {
    bitboard << NORTH_WEST & !File::H.to_bitboard()
}

/// Shifts the [`Bitboard`] one square to the north-east without wrap-around.
///
/// # Arguments
/// - `bitboard` - The [`Bitboard`] to shift.
///
/// # Returns
/// A new [`Bitboard`] shifted one square to the north-east without wrap-around.
pub fn north_east(bitboard: Bitboard) -> Bitboard {
    bitboard << NORTH_EAST & !File::A.to_bitboard()
}

/// Shifts the [`Bitboard`] one square to the south-west without wrap-around.
///
/// # Arguments
/// - `bitboard` - The [`Bitboard`] to shift.
///
/// # Returns
/// A new [`Bitboard`] shifted one square to the south-west without wrap-around.
pub fn south_west(bitboard: Bitboard) -> Bitboard {
    bitboard >> SOUTH_WEST & !File::H.to_bitboard()
}

/// Shifts the bitboard one square to the south-east without wrap-around.
///
/// # Arguments
/// - `bitboard` - The bitboard to shift.
///
/// # Returns
/// A new bitboard shifted one square to the south-east without wrap-around.
pub fn south_east(bitboard: Bitboard) -> Bitboard {
    bitboard >> SOUTH_EAST & !File::A.to_bitboard()
}

#[cfg(test)]
mod tests {
    use crate::{
        bitboard::Bitboard,
        bitboard_helpers::{
            self, east, north, north_east, north_west, south, south_east, south_west, west,
        },
        definitions::{RANK_BITBOARDS, Squares},
        rank::Rank,
    };

    #[test]
    fn test_next_bit() {
        use super::*;
        let mut bb = Bitboard::new(0x8000000000000001);
        assert_eq!(next_bit(&mut bb), 0);
        assert_eq!(next_bit(&mut bb), 63);
        assert_eq!(bb.as_number(), 0);

        {
            let mut bb = Bitboard::new(0xFFFFFFFFFFFFFFFF);
            for i in 0..64 {
                assert_eq!(next_bit(&mut bb), i);
            }
        }
    }

    #[test]
    fn test_north_fill() {
        use super::*;
        let bb = Bitboard::new(0x00000000000000FF);

        let test_bbs = [
            Bitboard::new(0x00000000000000FF),
            Bitboard::new(0x000000000000FFFF),
            Bitboard::new(0x00000000FFFFFFFF),
            Bitboard::new(0xFFFFFFFFFFFFFFFF),
            Bitboard::from_square(Squares::C3) | Bitboard::from_square(Squares::E6),
        ];

        for bb in test_bbs {
            let filled = north_fill(bb);
            println!("{}", bb);
            println!("filled");
            println!("{}", filled);
            println!("+-------+");
        }

        println!("{}", bb);
        let filled = north_fill(bb);
        println!("+-------+");
        println!("{}", filled);
        assert_eq!(filled.as_number(), 0xFFFFFFFFFFFFFFFF);
    }

    #[test]
    fn test_south_fill() {}

    #[test]
    fn test_east_fill() {
        // Test with a single bit first
        let bb = Bitboard::from_square(Squares::A1);
        let filled = bitboard_helpers::east_fill(bb);
        assert_eq!(filled, RANK_BITBOARDS[Rank::R1 as usize]);

        // Test with multiple bits
        let bb = Bitboard::from_square(Squares::A3)
            | Bitboard::from_square(Squares::C4)
            | Bitboard::from_square(Squares::D5);
        let filled = bitboard_helpers::east_fill(bb);
        assert_eq!(filled, Bitboard::new(0x000000F8FCFF0000));
    }

    #[test]
    fn test_west_fill() {
        let bb = Bitboard::from_square(Squares::H8)
            | Bitboard::from_square(Squares::G7)
            | Bitboard::from_square(Squares::F6)
            | Bitboard::from_square(Squares::E5)
            | Bitboard::from_square(Squares::D4)
            | Bitboard::from_square(Squares::C3)
            | Bitboard::from_square(Squares::B2)
            | Bitboard::from_square(Squares::A1);
        let filled = bitboard_helpers::west_fill(bb);
        assert_eq!(filled, Bitboard::new(0xFF7F3F1F0F070301));

        let single_sq_bb = Bitboard::from_square(Squares::H1);
        let filled_bb = bitboard_helpers::west_fill(single_sq_bb);
        assert_eq!(filled_bb, RANK_BITBOARDS[Rank::R1 as usize]);
    }

    #[test]
    fn test_direction_shifts() {
        let bb = Bitboard::from_square(Squares::D4);
        assert_eq!(north(bb), Bitboard::from_square(Squares::D5));
        assert_eq!(south(bb), Bitboard::from_square(Squares::D3));
        assert_eq!(east(bb), Bitboard::from_square(Squares::E4));
        assert_eq!(west(bb), Bitboard::from_square(Squares::C4));

        // Do the same for diagonals
        assert_eq!(north_east(bb), Bitboard::from_square(Squares::E5));
        assert_eq!(north_west(bb), Bitboard::from_square(Squares::C5));
        assert_eq!(south_east(bb), Bitboard::from_square(Squares::E3));
        assert_eq!(south_west(bb), Bitboard::from_square(Squares::C3));

        // Repeat for edge cases
        // Use top right corner
        let bb = Bitboard::from_square(Squares::H8);
        assert_eq!(north(bb), Bitboard::default());
        assert_eq!(east(bb), Bitboard::default());
        assert_eq!(west(bb), Bitboard::from_square(Squares::G8));
        assert_eq!(south(bb), Bitboard::from_square(Squares::H7));
        assert_eq!(north_east(bb), Bitboard::default());
        assert_eq!(north_west(bb), Bitboard::default());
        assert_eq!(south_east(bb), Bitboard::default());
        assert_eq!(south_west(bb), Bitboard::from_square(Squares::G7));
    }
}
