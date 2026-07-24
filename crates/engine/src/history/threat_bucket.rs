// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module contains a threat bucket and index type that is used as part of storage entries in certain history tables.

use chess::{bitboard::Bitboard, moves::Move};

/// Index type for a [`ThreatBucket`].
pub(crate) struct ThreatIndex {
    from_attacked: bool,
    to_attacked: bool,
}

impl ThreatIndex {
    /// Create a new [`ThreatIndex`] from a given move and threat bitboard.
    /// # Arguments
    /// - `mv`: The move to test.
    /// - `threats: A [`Bitboard`] of all threatened squares.
    pub(crate) fn new(mv: &Move, threats: Bitboard) -> Self {
        ThreatIndex {
            from_attacked: threats.is_square_occupied(mv.from()),
            to_attacked: threats.is_square_occupied(mv.to()),
        }
    }

    pub(crate) fn from(&self) -> usize {
        self.from_attacked as usize
    }

    pub(crate) fn to(&self) -> usize {
        self.to_attacked as usize
    }
}

/// Bucket of values to indicate whether source and destination squares are threatened.
/// Should be indexed as `threat_bucket[from][to]`. See [`ThreatIndex`] for more.
pub(crate) type ThreatBucket<T> = [[T; 2]; 2];
