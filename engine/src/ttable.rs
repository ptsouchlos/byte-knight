// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::moves::Move;

use crate::{node_types::NodeType, score::Score};

const BYTES_PER_MB: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EntryFlag {
    Exact,
    LowerBound,
    UpperBound,
}

/// A transposition table entry.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TranspositionTableEntry {
    pub zobrist: u64,
    pub score: Score,
    pub board_move: Move,
    pub depth: u8,
    pub flag: EntryFlag,
}

impl TranspositionTableEntry {
    #[allow(dead_code)]
    pub fn new(
        zobrist: u64,
        depth: u8,
        score: Score,
        flag: EntryFlag,
        mv: Move,
    ) -> TranspositionTableEntry {
        TranspositionTableEntry {
            zobrist,
            depth,
            score,
            flag,
            board_move: mv,
        }
    }
}

/// A transposition table used to store the results of previous searches.
pub struct TranspositionTable {
    table: Vec<Option<TranspositionTableEntry>>,
    pub(crate) collisions: usize,
    pub(crate) accesses: usize,
    pub(crate) hits: usize,
}

pub const MAX_TABLE_SIZE_MB: usize = 1024;
pub const MIN_TABLE_SIZE_MB: usize = 16;
const DEFAULT_TABLE_SIZE_MB: usize = MIN_TABLE_SIZE_MB;

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::from_size_in_mb(DEFAULT_TABLE_SIZE_MB)
    }
}

/// Given "word", produce an integer in the range [0, p) without division.
/// Alternative to modulo operation.
/// See <https://github.com/ozgrakkurt/fastrange-rs/blob/master/src/lib.rs>
const fn fast_range_64(word: u64, p: u64) -> u64 {
    ((word as u128 * p as u128) >> 64) as u64
}

#[derive(Debug, Clone)]
pub(crate) enum ProbeResult {
    CutOff(TranspositionTableEntry),
    Hit(TranspositionTableEntry),
    Empty,
}

impl TranspositionTable {
    pub(crate) fn from_capacity(capacity: usize) -> Self {
        Self {
            table: vec![None; capacity],
            collisions: 0,
            accesses: 0,
            hits: 0,
        }
    }

    pub(crate) fn from_size_in_mb(mb: usize) -> Self {
        let capacity = mb * BYTES_PER_MB / std::mem::size_of::<TranspositionTableEntry>();
        Self::from_capacity(capacity)
    }

    fn get_index(&self, zobrist: u64) -> usize {
        fast_range_64(zobrist, self.table.len() as u64) as usize
    }

    pub(crate) fn get_entry(&self, zobrist: u64) -> Option<TranspositionTableEntry> {
        let index = self.get_index(zobrist);
        self.table[index]
    }

    pub(crate) fn store_entry(&mut self, entry: TranspositionTableEntry) {
        let index = self.get_index(entry.zobrist);
        self.table[index] = Some(entry);
    }

    pub(crate) fn clear(&mut self) {
        self.table.iter_mut().for_each(|element| {
            *element = None;
        });

        // reset stats as well
        self.collisions = 0;
        self.accesses = 0;
        self.hits = 0;
    }

    pub(crate) fn fullness(&self) -> f64 {
        (self.table.iter().filter(|entry| entry.is_some()).count() as f64 / self.table.len() as f64)
            * 100_f64
    }

    pub(crate) fn size(&self) -> usize {
        self.table.len()
    }

    /// Probes the transposition table for a potential entry/cutoff.
    ///
    /// # Arguments
    ///
    /// - `depth` - The depth of the search.
    /// - `zobrist` - The zobrist hash of the position.
    /// - `alpha` - The alpha value of the search.
    /// - `beta` - The beta value of the search.
    ///
    /// # Returns
    ///
    /// - `ProbeResult` - The result of the probe.
    pub(crate) fn probe<Node: NodeType>(
        &mut self,
        depth: i16,
        zobrist: u64,
        alpha: Score,
        beta: Score,
    ) -> ProbeResult {
        if let Some(entry) = self.get_entry(zobrist) {
            self.accesses += 1;
            // verify the zobrist hash as we could have collisions due to using modulo as a hash function
            // and the fact that we are using a fixed size table.
            if entry.zobrist == zobrist {
                self.hits += 1;
                if entry.depth >= depth as u8 {
                    // can we cut off?
                    // cutoff can only happen if the entry depth >= current depth and 1 of the following:
                    // - the entry type is exact
                    // - the entry type is lower bound and the score >= beta
                    // - the entry type is upper bound and the score <= alpha
                    // see https://www.chessprogramming.org/Transposition_Table#Transposition_Table_Cutoffs
                    if entry.flag == EntryFlag::Exact
                        || ((entry.flag == EntryFlag::LowerBound && entry.score >= beta)
                            || (entry.flag == EntryFlag::UpperBound && entry.score <= alpha))
                    {
                        return ProbeResult::CutOff(entry);
                    }
                }
                return ProbeResult::Hit(entry);
            } else {
                // collision
                self.collisions += 1;
            }
        }

        ProbeResult::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::{EntryFlag, TranspositionTable, TranspositionTableEntry};
    use crate::score::Score;
    use chess::{
        moves::{Move, MoveDescriptor},
        pieces::Piece,
        square::Square,
    };
    use itertools::Itertools;
    use rand::Rng;
    use std::collections::HashMap;

    #[test]
    fn get_index() {
        let tt = TranspositionTable::from_size_in_mb(32);
        let mut rng = rand::rng();
        let random_numbers: Vec<u64> = (0..tt.size()).map(|_| rng.random::<u64>()).collect();
        let min = random_numbers.iter().min().unwrap();
        let max = random_numbers.iter().max().unwrap();
        println!("min/max random number: {min}/{max}");
        println!("Table size: {}", tt.size());
        let mut index_histogram: HashMap<usize, usize> = HashMap::new();
        random_numbers.iter().for_each(|&num| {
            let index = tt.get_index(num);
            assert!(index < tt.size());
            *index_histogram.entry(index).or_insert(0) += 1;
        });

        // make sure that the distribution is roughly uniform
        let min = index_histogram.values().min().unwrap();
        let max = index_histogram.values().max().unwrap();
        let mean = index_histogram.values().sum::<usize>() as f64 / index_histogram.len() as f64;
        let count = index_histogram.len();

        println!("Min: {min}, Max: {max}, Mean: {mean}, Len: {count}");
        let unique_keys = random_numbers.iter().unique().count();
        println!("Unique keys: {unique_keys}");
        let collision_rate = (1.0 - (count as f64 / unique_keys as f64)) * 100.0;
        println!("Collision rate: {collision_rate}");
    }

    #[test]
    fn store_and_retrieve() {
        let mut tt = TranspositionTable::from_size_in_mb(16);
        let hash1 = 1234512341999_u64;
        let hash2 = 2423498723999_u64;
        let hash3 = 2423623733999_u64;
        let mv1 = Move::new(
            &Square::from_square_index(3),
            &Square::from_square_index(4),
            MoveDescriptor::None,
            Piece::Knight,
            None,
            None,
        );
        let mv2 = Move::new(
            &Square::from_square_index(7),
            &Square::from_square_index(10),
            MoveDescriptor::None,
            Piece::Knight,
            None,
            None,
        );
        let mv3 = Move::new(
            &Square::from_square_index(7),
            &Square::from_square_index(11),
            MoveDescriptor::None,
            Piece::Bishop,
            Some(Piece::Pawn),
            None,
        );

        // our tt implementation always overwrites, so let's make sure that's the case.
        tt.store_entry(TranspositionTableEntry::new(
            hash1,
            3,
            Score::new(-123),
            EntryFlag::Exact,
            mv1,
        ));

        let stored_entry1 = tt.get_entry(hash1);
        assert!(stored_entry1.is_some());
        assert_eq!(stored_entry1.unwrap().board_move, mv1);

        tt.store_entry(TranspositionTableEntry::new(
            hash2,
            3,
            Score::new(123),
            EntryFlag::Exact,
            mv2,
        ));

        let stored_entry2 = tt.get_entry(hash2);
        assert!(stored_entry2.is_some());
        assert_eq!(stored_entry2.unwrap().board_move, mv2);

        tt.store_entry(TranspositionTableEntry::new(
            hash3,
            3,
            Score::new(123),
            EntryFlag::Exact,
            mv3,
        ));

        let stored_entry3 = tt.get_entry(hash3);
        assert!(stored_entry3.is_some());
        assert_eq!(stored_entry3.unwrap().board_move, mv3);
    }
}
