// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::fmt::{Display, Formatter};

use anyhow::{Result, bail};
use chess::{
    attacks,
    bitboard::Bitboard,
    definitions::{BISHOP_BLOCKER_PERMUTATIONS, NumberOf, ROOK_BLOCKER_PERMUTATIONS},
    magics::MagicNumber,
    move_generation,
    pieces::{Piece, SQUARE_NAME},
    square::Square,
};
use rand::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
struct TableFillError {
    message: String,
}

impl Display for TableFillError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Error filling table: {}", self.message)
    }
}

fn generate_random_u64<R: Rng>(rng: &mut R) -> u64 {
    rng.random::<u64>() & rng.random::<u64>() & rng.random::<u64>()
}

fn is_valid_random_number(random_num: u64, relevant_bb: Bitboard) -> bool {
    let test = ((random_num as u128 * relevant_bb.as_number() as u128) & 0xFF00000000000000) as u64;
    test.count_ones() >= 6
}

fn find_magic<R: Rng>(
    piece: Piece,
    relevant_bb: Bitboard,
    rng: &mut R,
    square: Square,
) -> Result<MagicNumber> {
    let mut random_num = generate_random_u64(rng);
    while !is_valid_random_number(random_num, relevant_bb) {
        random_num = generate_random_u64(rng);
    }

    let bits = relevant_bb.as_number().count_ones();
    let magic_entry = MagicNumber::new(relevant_bb, 64 - bits as u8, 0, random_num);

    let blocker_permutations = move_generation::create_blocker_permutations(relevant_bb);
    let total_permutations = 2u64.pow(bits);
    assert_eq!(blocker_permutations.len(), total_permutations as usize);
    try_to_make_table(piece, square, &magic_entry, &blocker_permutations)?;

    Ok(magic_entry)
}

fn try_to_make_table(
    piece: Piece,
    square: Square,
    &magic: &MagicNumber,
    blockers: &[Bitboard],
) -> Result<()> {
    let bit_count = magic.relevant_bits_mask.count_ones();
    let mut table = vec![Bitboard::default(); 1 << bit_count];

    for blocker in blockers {
        let attack = match piece {
            Piece::Rook => attacks::rook(square, *blocker),
            Piece::Bishop => attacks::bishop(square, *blocker),
            _ => panic!("Invalid piece type"),
        };

        let index = magic.index(*blocker);
        if index >= table.len() {
            bail!(TableFillError {
                message: "Index out of bounds".to_string(),
            });
        }

        let entry = &mut table[magic.index(*blocker)];
        if *entry == Bitboard::default() {
            *entry = attack;
        } else if *entry != attack {
            bail!(TableFillError {
                message: "Key collision".to_string(),
            });
        }
    }

    Ok(())
}

fn find_magic_numbers(piece: Piece) -> Vec<MagicNumber> {
    let mut rng = rand::rng();
    let mut magic_numbers = Vec::with_capacity(NumberOf::SQUARES);
    assert!(piece == Piece::Rook || piece == Piece::Bishop);

    let mut offset = 0;

    println!("Finding magic numbers for {piece}");
    for sq in Bitboard::filled() {
        let rook_mask = move_generation::relevant_rook_bits(sq);
        let bishop_mask = move_generation::relevant_bishop_bits(sq);

        let use_mask = if piece == Piece::Rook {
            rook_mask
        } else {
            bishop_mask
        };

        let total_permutations = 2u64.pow(use_mask.as_number().count_ones());

        // flag that stops the loop
        let mut found = false;
        // Seems like we have to use a while loop with a flag here.
        // Using `loop` and `break` didn't work here and caused the loop to break early or never end.

        while !found {
            let magic_result = find_magic(piece, use_mask, &mut rng, sq);
            if let Ok(mut magic) = magic_result {
                found = true;
                // set the offset before saving it
                magic.offset = offset;
                magic_numbers.push(magic);
            }
        }
        println!("{} {}", SQUARE_NAME[sq], magic_numbers.last().unwrap());
        offset += total_permutations;
    }

    // the offset should match the table size
    let total = if piece == Piece::Rook {
        ROOK_BLOCKER_PERMUTATIONS
    } else {
        BISHOP_BLOCKER_PERMUTATIONS
    };
    assert_eq!(offset, total as u64, "Permutations were skipped.");

    magic_numbers
}

fn main() {
    let magic_bishop_numbers = find_magic_numbers(Piece::Bishop);
    let magic_rook_numbers = find_magic_numbers(Piece::Rook);

    println!("pub(crate) const BISHOP_MAGICS: [MagicNumber; NumberOf::SQUARES] = [");
    for magic in magic_bishop_numbers {
        println!(
            "  MagicNumber::new(Bitboard::new({}), {}, {}, {}),",
            magic.relevant_bits_mask, magic.shift, magic.offset, magic.magic_value
        );
    }
    println!("];");

    println!("pub(crate) const ROOK_MAGICS: [MagicNumber; NumberOf::SQUARES] = [");
    for magic in magic_rook_numbers {
        println!(
            "  MagicNumber::new(Bitboard::new({}), {}, {}, {}),",
            magic.relevant_bits_mask, magic.shift, magic.offset, magic.magic_value
        );
    }
    println!("];");
}
