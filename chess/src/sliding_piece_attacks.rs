// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{
    bitboard::Bitboard,
    definitions::NumberOf,
    magics::{BISHOP_MAGICS, ROOK_MAGICS},
    move_generation::MoveGenerator,
    pext::Pext,
    pieces::{Piece, SQUARE_NAME},
    slider_pieces::SliderPiece,
};

struct PextSlidingPieceAttacks {
    rook_pext: [Pext; NumberOf::SQUARES],
    bishop_pext: [Pext; NumberOf::SQUARES],
    rook_pext_attacks: Vec<Bitboard>,
    bishop_pext_attacks: Vec<Bitboard>,
}

#[cfg(target_arch = "x86_64")]
impl Default for PextSlidingPieceAttacks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "x86_64")]
impl PextSlidingPieceAttacks {
    fn new() -> Self {
        let mut instance = PextSlidingPieceAttacks {
            rook_pext: [Pext::default(); NumberOf::SQUARES],
            bishop_pext: [Pext::default(); NumberOf::SQUARES],
            rook_pext_attacks: vec![Bitboard::default(); 102400], // 2^12 * 64
            bishop_pext_attacks: vec![Bitboard::default(); 5248], // 2^10 * 64
        };

        instance.initialize_pext_tables(SliderPiece::Rook);
        instance.initialize_pext_tables(SliderPiece::Bishop);

        instance
    }

    /// Initialize PEXT tables and the associated attack tables.
    ///
    /// # Arguments
    ///
    /// - `piece` - The piece to generate the attack table for.
    #[cfg(target_arch = "x86_64")]
    fn initialize_pext_tables(&mut self, piece: SliderPiece) {
        let mut offset = 0usize;

        assert!(piece == SliderPiece::Bishop || piece == SliderPiece::Rook);
        let relevant_bits_fn = if piece == SliderPiece::Rook {
            MoveGenerator::relevant_rook_bits
        } else {
            MoveGenerator::relevant_bishop_bits
        };
        let get_attacks_fn = if piece == SliderPiece::Rook {
            MoveGenerator::rook_attacks
        } else {
            MoveGenerator::bishop_attacks
        };

        let mut min_pext = i64::MAX;
        let mut max_pext = i64::MIN;
        for square in 0..NumberOf::SQUARES as u8 {
            let relevant_bits = relevant_bits_fn(square);

            let blocker_bitboards = MoveGenerator::create_blocker_permutations(relevant_bits);
            let attacks = get_attacks_fn(square, &blocker_bitboards);
            let attack_table = if piece == SliderPiece::Rook {
                &mut self.rook_pext_attacks
            } else {
                &mut self.bishop_pext_attacks
            };

            let pext_table = if piece == SliderPiece::Rook {
                &mut self.rook_pext
            } else {
                &mut self.bishop_pext
            };

            let total_permutations = blocker_bitboards.len();

            pext_table[square as usize] = Pext::new(relevant_bits, offset);
            let current_pext = pext_table[square as usize];
            for i in 0..blocker_bitboards.len() {
                let blocker = blocker_bitboards[i];
                let index = current_pext.index(&blocker);
                if (index as i64) < min_pext {
                    min_pext = index as i64;
                }
                if (index as i64) > max_pext {
                    max_pext = index as i64;
                }
                assert!(index < attack_table.len());

                attack_table[index] = attacks[i];
            } // blocker bitboards loop

            offset += total_permutations;
        }
    }

    /// Get sliding piece attacks using PEXT bitboards.
    ///
    /// # Arguments
    ///
    /// - `piece` - The sliding piece to get the moves for.
    /// - `from_square` - The square the moves are generated from.
    /// - `occupancy` - The current occupancy of the board.
    ///
    /// # Returns
    ///
    /// A [`Bitboard`] of sliding piece attacks.
    #[cfg(target_arch = "x86_64")]
    #[inline(always)]
    fn get_attack(&self, piece: SliderPiece, from_square: u8, occupancy: &Bitboard) -> Bitboard {
        match piece {
            SliderPiece::Rook => {
                let pext = self.rook_pext[from_square as usize];
                let index = pext.index(occupancy);
                self.rook_pext_attacks[index]
            }
            SliderPiece::Bishop => {
                let pext = self.bishop_pext[from_square as usize];
                let index = pext.index(occupancy);
                self.bishop_pext_attacks[index]
            }
            SliderPiece::Queen => {
                let rook_index = self.rook_pext[from_square as usize].index(occupancy);
                let bishop_index = self.bishop_pext[from_square as usize].index(occupancy);
                self.rook_pext_attacks[rook_index] ^ self.bishop_pext_attacks[bishop_index]
            }
        }
    }
}

pub struct SlidingPieceAttacks {
    pub(crate) rook_attacks: Vec<Bitboard>,
    pub(crate) bishop_attacks: Vec<Bitboard>,
    #[cfg(target_arch = "x86_64")]
    pext_sliding_attacks: PextSlidingPieceAttacks,
}

impl Default for SlidingPieceAttacks {
    fn default() -> Self {
        Self::new()
    }
}

// Public API
impl SlidingPieceAttacks {
    /// Create a new instance of SlidingPieceAttacks with initialized magic numbers and attack tables.
    pub fn new() -> Self {
        let mut instance = SlidingPieceAttacks {
            rook_attacks: vec![Bitboard::default(); 102400], // 2^12 * 64
            bishop_attacks: vec![Bitboard::default(); 5248], // 2^10 * 64
            #[cfg(target_arch = "x86_64")]
            pext_sliding_attacks: PextSlidingPieceAttacks::new(),
        };

        // Initialize the magic numbers and attack tables.
        instance.initialize_magic_numbers(Piece::Rook);
        instance.initialize_magic_numbers(Piece::Bishop);

        instance
    }

    /// Get the attack bitboard for a sliding piece (rook, bishop, or queen) from a given square,
    /// considering the current occupancy of the board.
    ///
    /// # Arguments
    ///
    /// - `piece` - The type of sliding piece (rook, bishop, or queen).
    /// - `from_square` - The square from which the piece is attacking (0-63).
    /// - `occupancy` - The current occupancy bitboard of the board.
    ///
    /// # Returns
    ///
    /// - A Bitboard representing the attack squares for the given piece from the specified square.
    #[inline(always)]
    pub fn get_slider_attack(
        &self,
        piece: SliderPiece,
        from_square: u8,
        occupancy: &Bitboard,
    ) -> Bitboard {
        #[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
        {
            self.pext_sliding_attacks
                .get_attack(piece, from_square, occupancy)
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            self.get_attack(piece, from_square, occupancy)
        }
    }
}

// Private functions
impl SlidingPieceAttacks {
    /// Initialize the magic numbers and the associated attack boards.
    #[allow(clippy::panic)]
    fn initialize_magic_numbers(&mut self, piece: Piece) {
        assert!(piece == Piece::Rook || piece == Piece::Bishop);
        let mut offset = 0;

        for square in 0..NumberOf::SQUARES as u8 {
            let rook_relevant_bits = MoveGenerator::relevant_rook_bits(square);
            let bishop_relevant_bits = MoveGenerator::relevant_bishop_bits(square);
            let use_mask = if piece == Piece::Rook {
                rook_relevant_bits
            } else {
                bishop_relevant_bits
            };

            let bit_count = use_mask.as_number().count_ones();
            let total_permutations = 2u64.pow(bit_count);
            let end = offset + total_permutations - 1u64;
            let blocker_bitboards = MoveGenerator::create_blocker_permutations(use_mask);
            assert_eq!(blocker_bitboards.len(), total_permutations as usize);

            let rook_attacks = MoveGenerator::rook_attacks(square, &blocker_bitboards);
            let bishop_attacks = MoveGenerator::bishop_attacks(square, &blocker_bitboards);
            let attacks = if piece == Piece::Rook {
                rook_attacks
            } else {
                bishop_attacks
            };

            let magics = if piece == Piece::Rook {
                ROOK_MAGICS
            } else {
                BISHOP_MAGICS
            };

            let attack_table = if piece == Piece::Rook {
                &mut self.rook_attacks
            } else {
                &mut self.bishop_attacks
            };

            for i in 0..blocker_bitboards.len() {
                let blocker = blocker_bitboards[i];
                let index = magics[square as usize].index(blocker);

                if attack_table[index] == Bitboard::default() || attack_table[index] == attacks[i] {
                    // did we fail high or low index wise? (out of bounds)
                    assert!(
                        index as u64 >= offset && index as u64 <= end,
                        "index out of bounds"
                    );
                    attack_table[index] = attacks[i];
                } else {
                    panic!(
                        "Collision detected while initializing attack tables for piece {:?} and square {} - \n\t{}",
                        piece, SQUARE_NAME[square as usize], magics[square as usize]
                    );
                }
            }

            // update the offset for the next square
            offset += total_permutations;
        }
    }

    /// Get sliding piece attacks using so-called "magic" bitboards.
    ///
    /// # Arguments
    ///
    /// - `piece` - The sliding piece to get the moves for.
    /// - `from_square` - The square the moves are generated from.
    /// - `occupancy` - The current occupancy of the board.
    ///
    /// # Returns
    ///
    /// A [`Bitboard`] of sliding piece attacks.
    #[inline(always)]
    #[allow(dead_code)]
    fn get_attack(&self, piece: SliderPiece, from_square: u8, occupancy: &Bitboard) -> Bitboard {
        match piece {
            SliderPiece::Rook => {
                let index = ROOK_MAGICS[from_square as usize].index(*occupancy);
                self.rook_attacks[index]
            }
            SliderPiece::Bishop => {
                let index = BISHOP_MAGICS[from_square as usize].index(*occupancy);
                self.bishop_attacks[index]
            }
            SliderPiece::Queen => {
                let rook_index = ROOK_MAGICS[from_square as usize].index(*occupancy);
                let bishop_index = BISHOP_MAGICS[from_square as usize].index(*occupancy);
                self.rook_attacks[rook_index] ^ self.bishop_attacks[bishop_index]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bitboard::Bitboard,
        definitions::{FILE_BITBOARDS, NumberOf, RANK_BITBOARDS, Squares},
        file::File,
        move_generation::MoveGenerator,
        rank::Rank,
        slider_pieces::SliderPiece,
    };

    #[test]
    fn check_rook_attacks() {
        for square in 0..NumberOf::SQUARES {
            let rook_bb = MoveGenerator::relevant_rook_bits(square as u8);
            let blockers = MoveGenerator::create_blocker_permutations(rook_bb);
            let edges = MoveGenerator::edges_from_square(square as u8);
            let rook_bb_with_edges = rook_bb | edges;

            let attacks = MoveGenerator::rook_attacks(square as u8, &blockers);
            assert!(attacks.len() <= blockers.len());

            for attack in attacks {
                // attack should be a subset of the rook bitboard with edges
                // blockers does not include the edges
                // but attacks do include them
                assert_eq!(attack & !rook_bb_with_edges, 0);
            }
        }
    }

    #[test]
    fn check_bishop_attacks() {
        for square in 0..1 {
            let bishop_bb = MoveGenerator::relevant_bishop_bits(square as u8);
            let blockers = MoveGenerator::create_blocker_permutations(bishop_bb);
            let edges = MoveGenerator::edges_from_square(square as u8);
            let bishop_bb_with_edges = bishop_bb | edges;

            let attacks = MoveGenerator::bishop_attacks(square as u8, &blockers);
            assert!(attacks.len() <= blockers.len());

            for attack in attacks {
                println!("attack: \n{attack}");
                // attack should be a subset of the bishop bitboard
                assert_eq!(attack & !bishop_bb_with_edges, 0);
            }
        }
    }

    #[test]
    fn check_queen_attacks() {
        let square = Squares::D8;
        let bishop_bb = MoveGenerator::relevant_bishop_bits(square);
        let rook_bb = MoveGenerator::relevant_rook_bits(square);
        let queen_bb = bishop_bb | rook_bb;

        let sliding_piece_attacks = super::SlidingPieceAttacks::new();
        let queen_attacks =
            sliding_piece_attacks.get_attack(SliderPiece::Queen, square, &Bitboard::default());
        println!("queen attacks: \n{queen_attacks}");
        println!("queen bb: \n{queen_bb}");

        let attacks_without_edges = queen_attacks
            & !FILE_BITBOARDS[File::A as usize]
            & !FILE_BITBOARDS[File::H as usize]
            & !RANK_BITBOARDS[Rank::R1 as usize];

        println!("attacks without edges: \n{attacks_without_edges}");
        assert_eq!(attacks_without_edges, queen_bb);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn check_queen_attacks_pext() {
        let square = Squares::D8;
        let bishop_bb = MoveGenerator::relevant_bishop_bits(square);
        let rook_bb = MoveGenerator::relevant_rook_bits(square);
        let queen_bb = bishop_bb | rook_bb;

        let sliding_piece_attacks = super::PextSlidingPieceAttacks::new();
        let queen_attacks =
            sliding_piece_attacks.get_attack(SliderPiece::Queen, square, &Bitboard::default());
        println!("queen attacks: \n{queen_attacks}");
        println!("queen bb: \n{queen_bb}");

        let attacks_without_edges = queen_attacks
            & !FILE_BITBOARDS[File::A as usize]
            & !FILE_BITBOARDS[File::H as usize]
            & !RANK_BITBOARDS[Rank::R1 as usize];

        println!("attacks without edges: \n{attacks_without_edges}");
        assert_eq!(attacks_without_edges, queen_bb);
    }
}
