// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

pub mod keys;
pub mod values;

use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{board::Board, definitions::NumberOf};

/// A Zobrist hash value.
pub type ZobristHash = u64;

#[derive(Debug, Default)]
struct Hashes {
    board: u64,
    pawn: u64,
}

impl Hashes {
    pub fn new(board: &Board) -> Self {
        Hashes {
            board: keys::get_hash(board),
            pawn: keys::get_pawn_hash(board),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ZobristRandomValues {
    pub piece_values: [[[u64; NumberOf::SQUARES]; NumberOf::PIECE_TYPES]; NumberOf::SIDES],
    pub castling_values: [u64; NumberOf::CASTLING_OPTIONS],
    pub en_passant_values: [u64; NumberOf::SQUARES + 1],
    pub side_values: [u64; NumberOf::SIDES],
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
                        *value = random.next_u64();
                    });
                });
            });

        random_values.castling_values.iter_mut().for_each(|value| {
            *value = random.next_u64();
        });

        random_values
            .en_passant_values
            .iter_mut()
            .for_each(|value| {
                *value = random.next_u64();
            });

        random_values.side_values.iter_mut().for_each(|value| {
            *value = random.next_u64();
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

#[cfg(test)]
mod tests {
    use crate::{definitions::NumberOf, pieces::Piece, side::Side, zobrist::ZobristRandomValues};

    #[test]
    fn print_params() {
        let values = ZobristRandomValues::default();
        println!(
            "const PIECE_VALUES: [[[u64; NumberOf::SQUARES]; NumberOf::PIECE_TYPES]; NumberOf::SIDES] = ["
        );
        for side in Side::iter() {
            println!("  // {side}");
            println!("  [");
            for piece in Piece::iter() {
                println!("    // {piece}");
                println!("    [");
                for sq in 0..NumberOf::SQUARES {
                    print!(
                        "{}, ",
                        values.get_piece_value(piece as usize, side as usize, sq)
                    );
                }
                println!("    ],");
            }
            println!("  ],");
        }
        println!("];");

        println!("const CASTLING_VALUES: [u64; NumberOf::CASTLING_OPTIONS] = [");
        println!("  ");
        for castling in 0..NumberOf::CASTLING_OPTIONS {
            print!("{}, ", values.get_castling_value(castling));
        }
        println!("];");

        println!("const EN_PASSANT_VALUES: [u64; NumberOf::SQUARES + 1] = [");
        println!("   ");
        for ep in 0..NumberOf::SQUARES + 1 {
            print!("{}, ", values.get_en_passant_value(Some(ep as u8)));
        }
        println!("];");

        println!("const SIDE_VALUES: [u64; NumberOf::SIDES] = [");
        for sd in 0..NumberOf::SIDES {
            print!("{}, ", values.get_side_value(sd));
        }
        println!();
        println!("];");
    }
}
