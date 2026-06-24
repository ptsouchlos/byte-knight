use chess::{definitions::NumberOf, pieces::Piece, side::Side};
use rand::{Rng, SeedableRng, rngs::StdRng};

#[derive(Debug, clap::Parser)]
pub(crate) struct GenerateArgs {}

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

pub(crate) fn execute(_args: GenerateArgs) -> anyhow::Result<()> {
    let values = ZobristRandomValues::default();
    println!("#[rustfmt::skip]");
    println!(
        "pub(crate) const PIECE_VALUES: [[[u64; NumberOf::SQUARES]; NumberOf::PIECE_TYPES]; NumberOf::SIDES] = ["
    );
    for side in Side::iter() {
        println!("  // {side}");
        println!("  [");
        for piece in Piece::iter() {
            println!("    // {piece}");
            println!("    [");
            for sq in 0..NumberOf::SQUARES {
                if (sq % 8) == 0 {
                    print!("      ");
                }
                print!(
                    "{:<24}, ",
                    values.get_piece_value(piece as usize, side as usize, sq)
                );
                if (sq + 1) % 8 == 0 {
                    println!();
                }
            }
            println!("    ],");
        }
        println!("  ],");
    }
    println!("];");
    println!();

    println!("pub(crate) const CASTLING_VALUES: [u64; NumberOf::CASTLING_OPTIONS] = [");
    for castling in 0..NumberOf::CASTLING_OPTIONS {
        println!("  {:<}, ", values.get_castling_value(castling));
    }
    println!("];");
    println!();

    println!("pub(crate) const EN_PASSANT_VALUES: [u64; NumberOf::SQUARES + 1] = [");
    for ep in 0..NumberOf::SQUARES + 1 {
        println!("  {:<}, ", values.get_en_passant_value(Some(ep as u8)));
    }
    println!("];");
    println!();

    println!("pub(crate) const SIDE_VALUES: [u64; NumberOf::SIDES] = [");
    for sd in 0..NumberOf::SIDES {
        println!("  {:<}, ", values.get_side_value(sd));
    }
    println!("];");
    Ok(())
}
