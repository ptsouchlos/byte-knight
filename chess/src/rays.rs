// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module provides functionality to retrieve the ray (line of squares) between two squares on a chessboard.

use crate::{attacks, bitboard::Bitboard, definitions::NumberOf};

#[allow(long_running_const_eval)]
static RAYS_BETWEEN: [[Bitboard; NumberOf::SQUARES]; NumberOf::SQUARES] = initialize_rays_between();

/// Initializes the rays between all pairs of squares on the chessboard.
///
/// Returns
/// - A 2D array where each entry [from][to] contains a Bitboard representing the squares between `from` and `to`.
const fn initialize_rays_between() -> [[Bitboard; NumberOf::SQUARES]; NumberOf::SQUARES] {
    let mut rays_between: [[Bitboard; NumberOf::SQUARES]; NumberOf::SQUARES] =
        [[Bitboard::default(); NumberOf::SQUARES]; NumberOf::SQUARES];
    let mut from = 0u8;
    let mut to = 0u8;
    while from < NumberOf::SQUARES as u8 {
        while to < NumberOf::SQUARES as u8 {
            if attacks::rook(from, Bitboard::default()).intersects(Bitboard::from_square(to)) {
                rays_between[from as usize][to as usize] = Bitboard::new(
                    attacks::rook(from, Bitboard::from_square(to)).as_number()
                        & attacks::rook(to, Bitboard::from_square(from)).as_number(),
                );
            }

            if attacks::bishop(from, Bitboard::default()).intersects(Bitboard::from_square(to)) {
                rays_between[from as usize][to as usize] = Bitboard::new(
                    attacks::bishop(from, Bitboard::from_square(to)).as_number()
                        & attacks::bishop(to, Bitboard::from_square(from)).as_number(),
                );
            }

            to += 1;
        }

        from += 1;
        to = 0;
    }
    rays_between
}

/// Returns the [`Bitboard`] representing the ray between two squares.
///
/// # Arguments
/// - `from`: The starting square (0-63).
/// - `to`: The ending square (0-63).
///
/// # Returns
/// - A [`Bitboard`] representing the squares between `from` and `to`.
pub fn between(from: u8, to: u8) -> Bitboard {
    RAYS_BETWEEN[from as usize][to as usize]
}

#[cfg(test)]
mod tests {
    use crate::{move_generation::MoveGenerator, pieces::SQUARE_NAME, square::Square};

    #[test]
    fn validate_rays_between() {
        let move_gen = MoveGenerator::new();
        for from in 0..64_u8 {
            for to in 0..64_u8 {
                let bb = super::between(from, to);
                let move_gen_bb = move_gen.ray_between(
                    Square::from_square_index(from),
                    Square::from_square_index(to),
                );
                println!(
                    "{} -> {}\n{}\n{}",
                    SQUARE_NAME[from as usize], SQUARE_NAME[to as usize], bb, move_gen_bb
                );

                assert_eq!(bb, move_gen_bb);
            }
        }
    }

    #[test]
    fn test_initialize_rays_between() {
        let rays_between = super::initialize_rays_between();
        for from in 0..64_u8 {
            for to in 0..64_u8 {
                let bb = rays_between[from as usize][to as usize];
                let expected_bb = super::between(from, to);
                assert_eq!(
                    bb, expected_bb,
                    "Rays between {} and {} do not match.",
                    SQUARE_NAME[from as usize], SQUARE_NAME[to as usize]
                );

                let bb_rev = rays_between[to as usize][from as usize];
                let expected_bb_rev = super::between(to, from);
                assert_eq!(
                    bb_rev, expected_bb_rev,
                    "Rays between {} and {} do not match.",
                    SQUARE_NAME[to as usize], SQUARE_NAME[from as usize]
                );
            }
        }
    }
}
