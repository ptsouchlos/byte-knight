// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module provides functionality to retrieve the ray (line of squares) between two squares on a chessboard.

use crate::{
    attacks,
    bitboard::Bitboard,
    definitions::{FILE_BITBOARDS, NumberOf, RANK_BITBOARDS},
    file::File,
    rank::Rank,
};

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

/// Returns a [`Bitboard`] representing the edge squares of the chessboard, excluding the specified file and rank.
///
/// # Arguments
/// - `file`: The file (0-7) to exclude from the edge squares.
/// - `rank`: The rank (0-7) to exclude from the edge squares.
///
/// # Returns
/// - A [`Bitboard`] representing the edge squares of the chessboard, excluding the specified
pub fn edges(file: u8, rank: u8) -> Bitboard {
    let file_bb = FILE_BITBOARDS[file as usize];
    let rank_bb = RANK_BITBOARDS[rank as usize];
    (FILE_BITBOARDS[File::A as usize] & !file_bb)
        | (FILE_BITBOARDS[File::H as usize] & !file_bb)
        | (RANK_BITBOARDS[Rank::R1 as usize] & !rank_bb)
        | (RANK_BITBOARDS[Rank::R8 as usize] & !rank_bb)
}

#[cfg(test)]
mod tests {
    use crate::pieces::SQUARE_NAME;

    #[test]
    fn validate_rays_between() {
        for from in 0..64_u8 {
            for to in 0..64_u8 {
                let bb = super::between(from, to);
                println!(
                    "{} -> {}\n{}",
                    SQUARE_NAME[from as usize], SQUARE_NAME[to as usize], bb
                );
                // Verify symmetry: between(a, b) == between(b, a)
                assert_eq!(bb, super::between(to, from));
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
