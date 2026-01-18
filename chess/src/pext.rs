// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::bitboard::Bitboard;

/// Simple structure to help with PEXT index generation when computing sliding piece attacks.
#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct Pext {
    pub relevant_bits_mask: u64,
    pub offset: usize,
}

impl Pext {
    /// Create a new [`Pext`] structure with the given relevant bits [`Bitboard`] and an index offset.
    ///
    /// # Arguments
    ///
    /// - relevant_bits: The "relevant" bits for this entry. Typically this is generated as a set of permutations for all the given move possibilities of a sliding piece from a specific board location. Think of rays cast to the edges of the chess board from a given square. This [`Bitboard`] should not include the edges.
    /// - offset: The offset of the index for this PEXT entry to the corresponding attack table.
    ///
    /// # Returns
    ///
    /// - New Pext struct.
    pub(crate) fn new(relavent_bits: Bitboard, offset: usize) -> Self {
        Pext {
            relevant_bits_mask: relavent_bits.as_number(),
            offset,
        }
    }

    /// Compute the index into the attack table for this Pext entry using the PEXT BMI2 instruction.
    ///
    /// # Arguments
    ///
    /// - occupancy: The current chess board occupancy as a [`Bitboard`].
    ///
    /// # Returns
    ///
    /// - A `usize` index that can be used directly in the attack table.
    #[cfg(target_arch = "x86_64")]
    pub(crate) fn index(&self, occupancy: &Bitboard) -> usize {
        unsafe {
            use std::arch::x86_64::_pext_u64;
            let index = _pext_u64(occupancy.as_number(), self.relevant_bits_mask) as usize;
            index + self.offset
        }
    }
}
