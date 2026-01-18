// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::definitions::NumberOf;

/// A Zobrist hash value.
pub type ZobristHash = u64;

#[derive(Debug)]
pub(crate) struct ZobristRandomValues {
    pub piece_values: [[[u64; NumberOf::SQUARES]; NumberOf::PIECE_TYPES]; NumberOf::SIDES],
    pub castling_values: [u64; NumberOf::CASTLING_OPTIONS],
    pub en_passant_values: [u64; NumberOf::SQUARES + 1],
    pub side_values: [u64; NumberOf::SIDES],
}

impl Clone for ZobristRandomValues {
    fn clone(&self) -> Self {
        Self {
            piece_values: self.piece_values,
            castling_values: self.castling_values,
            en_passant_values: self.en_passant_values,
            side_values: self.side_values,
        }
    }
}

const RANDOM_SEED: [u8; 32] = [115; 32];

impl Default for ZobristRandomValues {
    fn default() -> Self {
        Self::new()
    }
}

impl ZobristRandomValues {
    pub fn new() -> Self {
        let mut random = StdRng::from_seed(RANDOM_SEED);
        // initialize everything to 0
        let mut random_values = Self {
            piece_values: [[[0; NumberOf::SQUARES]; NumberOf::PIECE_TYPES]; NumberOf::SIDES],
            castling_values: [0; NumberOf::CASTLING_OPTIONS],
            en_passant_values: [0; NumberOf::SQUARES + 1],
            side_values: [0; NumberOf::SIDES],
        };

        random_values
            .piece_values
            .iter_mut()
            .for_each(|piece_values| {
                piece_values.iter_mut().for_each(|square_values| {
                    square_values.iter_mut().for_each(|value| {
                        *value = random.random();
                    });
                });
            });

        random_values.castling_values.iter_mut().for_each(|value| {
            *value = random.random();
        });

        random_values
            .en_passant_values
            .iter_mut()
            .for_each(|value| {
                *value = random.random();
            });

        random_values.side_values.iter_mut().for_each(|value| {
            *value = random.random();
        });

        random_values
    }

    /// Returns the Zobrist hash value for the given piece, side, and square.
    pub fn get_piece_value(&self, piece: usize, side: usize, square: usize) -> u64 {
        self.piece_values[side][piece][square]
    }

    /// Returns the Zobrist hash value for the given castling option.
    pub fn get_castling_value(&self, castling: usize) -> u64 {
        self.castling_values[castling]
    }

    /// Returns the Zobrist hash value for the given en passant square.
    pub fn get_en_passant_value(&self, square: Option<u8>) -> u64 {
        match square {
            None => self.en_passant_values[NumberOf::SQUARES],
            Some(square) => self.en_passant_values[square as usize],
        }
    }

    /// Returns the Zobrist hash value for the given side.
    pub fn get_side_value(&self, side: usize) -> u64 {
        self.side_values[side]
    }
}
