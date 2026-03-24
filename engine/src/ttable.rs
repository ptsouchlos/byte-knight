// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::i32;

use chess::moves::Move;

use crate::{
    node_types::NodeType,
    score::{Score, ScoreType},
};

const BYTES_PER_MB: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[repr(u8)]
pub enum EntryFlag {
    #[default]
    None,
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Clone, Debug, Default, Copy)]
pub struct Flags {
    data: u8,
}

const TT_FLAG_MASK: u8 = 0b11;
const TT_AGE_MASK: u8 = 0b1111100;

const TT_FLAG_SHIFT: u8 = 0;
const TT_AGE_SHIFT: u8 = 2;

impl Flags {
    pub fn new(tt_flag: EntryFlag, age: u8) -> Self {
        Self {
            data: (tt_flag as u8) | (age << TT_AGE_SHIFT),
        }
    }

    pub fn flag(&self) -> EntryFlag {
        unsafe { std::mem::transmute(self.data & TT_FLAG_MASK) }
    }

    pub fn age(&self) -> u8 {
        (self.data & TT_AGE_MASK) >> TT_AGE_SHIFT
    }
}

/// A transposition table entry.
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub(crate) struct TranspositionTableEntry {
    pub zobrist_key: u16,
    pub score: Score,
    pub board_move: Move,
    pub depth: u8,
    pub flags: Flags,
}

const _: () = assert!(std::mem::size_of::<TranspositionTableEntry>() == 8);

impl TranspositionTableEntry {
    pub fn is_empty(&self) -> bool {
        self.flags.flag() == EntryFlag::None
    }

    #[allow(dead_code)]
    pub fn new(
        zobrist: u64,
        depth: u8,
        score: Score,
        mv: Move,
        flag: EntryFlag,
        age: u8,
    ) -> TranspositionTableEntry {
        TranspositionTableEntry {
            zobrist_key: zobrist as u16,
            depth,
            score,
            board_move: mv,
            flags: Flags::new(flag, age),
        }
    }

    pub const fn validate_key(&self, zobrist: u64) -> bool {
        self.zobrist_key == (zobrist & 0xFFFF) as u16
    }

    pub fn relative_age(&self, age: u8) -> u8 {
        (age - self.flags.age()) & 0x3F
    }

    pub fn flag(&self) -> EntryFlag {
        self.flags.flag()
    }

    pub fn age(&self) -> u8 {
        self.flags.age()
    }
}

const ENTRIES_PER_BUCKET: usize = 4;

// The size of a Bucket should divide the size of a cache line for best performance (prefetch).
#[derive(Default, Clone)]
#[repr(align(32))]
struct Bucket {
    entries: [TranspositionTableEntry; ENTRIES_PER_BUCKET],
}

/// A transposition table used to store the results of previous searches.
pub struct TranspositionTable {
    table: Vec<Bucket>,
    pub(crate) collisions: usize,
    pub(crate) accesses: usize,
    pub(crate) hits: usize,
    pub(crate) age: u8,
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
            table: vec![Bucket::default(); capacity],
            collisions: 0,
            accesses: 0,
            hits: 0,
            age: 0,
        }
    }

    pub(crate) fn from_size_in_mb(mb: usize) -> Self {
        let capacity = mb * BYTES_PER_MB / std::mem::size_of::<Bucket>();
        Self::from_capacity(capacity)
    }

    fn get_index(&self, zobrist: u64) -> usize {
        fast_range_64(zobrist, self.table.len() as u64) as usize
    }

    pub(crate) fn get_entry(&self, zobrist: u64) -> Option<TranspositionTableEntry> {
        let index = self.get_index(zobrist);
        let bucket = &self.table[index];
        bucket
            .entries
            .iter()
            .find(|&entry| entry.validate_key(zobrist))
            .map(|&ent| ent)
    }

    pub(crate) fn store_entry(
        &mut self,
        zobrist: u64,
        depth: u8,
        score: Score,
        flag: EntryFlag,
        mv: Move,
    ) {
        debug_assert_ne!(
            flag,
            EntryFlag::None,
            "cannot store an entry with EntryFlag::None"
        );
        let index = self.get_index(zobrist);

        let tt_age = self.age;
        let bucket = &mut self.table[index];
        let mut replace_index = 0;
        for (i, entry) in bucket.entries.iter_mut().enumerate() {
            // Replace the entry if the keys match or the entry is empty
            if entry.validate_key(zobrist) || entry.is_empty() {
                replace_index = i;
                break;
            }

            let mut min_quality = i32::MAX;
            let quality = entry.depth as i32 - 4 * entry.relative_age(tt_age) as i32;
            if quality < min_quality {
                min_quality = quality;
                replace_index = i;
            }
        }

        // let new_entry = TranspositionTableEntry::new(zobrist, depth, score, mv, flag, self.age);
        let entry = &mut bucket.entries[replace_index];
        if !(entry.validate_key(zobrist)
            || flag == EntryFlag::Exact
            || depth as i32 + 4 > entry.depth as i32
            || entry.flags.age() != tt_age)
        {
            return;
        }

        entry.zobrist_key = zobrist as u16;
        entry.depth = depth;
        entry.score = score;
        entry.flags = Flags::new(flag, tt_age);
        entry.board_move = mv;
    }

    pub(crate) fn clear(&mut self) {
        self.table.iter_mut().for_each(|element| {
            *element = Bucket::default();
        });

        // reset stats as well
        self.collisions = 0;
        self.accesses = 0;
        self.hits = 0;
        self.age = 0;
    }

    pub(crate) fn fullness(&self) -> f64 {
        (self
            .table
            .iter()
            .map(|bucket| bucket.entries.iter().filter(|entry| entry.is_empty()))
            .count() as f64
            / self.table.len() as f64)
            * 100_f64
    }

    pub(crate) fn hashfull(&self) -> u16 {
        let mut fill = 0;
        for bucket in self.table.iter().take(1000 / ENTRIES_PER_BUCKET) {
            for entry in &bucket.entries {
                if entry.flags.flag() == EntryFlag::None {
                    fill += 1;
                }
            }
        }
        fill
    }

    pub(crate) fn size(&self) -> usize {
        self.table.len()
    }

    pub(crate) fn increment_age(&mut self) {
        self.age += 1;
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
        depth: ScoreType,
        ply: ScoreType,
        zobrist: u64,
        alpha: Score,
        beta: Score,
    ) -> ProbeResult {
        if let Some(entry) = self.get_entry(zobrist) {
            self.accesses += 1;
            // verify the partial zobrist key to detect collisions
            if entry.validate_key(zobrist) {
                self.hits += 1;
                if entry.depth >= depth as u8 {
                    // can we cut off?
                    // cutoff can only happen if the entry depth >= current depth and 1 of the following:
                    // - the entry type is exact
                    // - the entry type is lower bound and the score >= beta
                    // - the entry type is upper bound and the score <= alpha
                    // see https://www.chessprogramming.org/Transposition_Table#Transposition_Table_Cutoffs
                    let ply_relative_score = entry.score.ply_relative(ply);
                    if entry.flag() == EntryFlag::Exact
                        || ((entry.flag() == EntryFlag::LowerBound && ply_relative_score >= beta)
                            || (entry.flag() == EntryFlag::UpperBound
                                && ply_relative_score <= alpha))
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
    use crate::{score::Score, ttable::Flags};
    use chess::{
        moves::{Move, MoveFlag},
        square::Square,
    };
    use itertools::Itertools;
    use rand::Rng;
    use std::collections::HashMap;

    #[test]
    fn entry_size() {
        assert_eq!(std::mem::size_of::<TranspositionTableEntry>(), 8);
    }

    #[test]
    fn get_index() {
        let tt = TranspositionTable::from_size_in_mb(32);
        let mut rng = rand::rng();
        let random_numbers: Vec<u64> = (0..tt.size()).map(|_| rng.next_u64()).collect();
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
        // Use hashes with non-zero upper 16 bits (realistic zobrist hash values).
        let hash1 = 0xDEAD_BEEF_1234_5678_u64;
        let hash2 = 0xCAFE_BABE_ABCD_EF01_u64;
        let hash3 = 0x1234_ABCD_5678_EF01_u64;
        let mv1 = Move::new(
            Square::from_square_index(3),
            Square::from_square_index(4),
            MoveFlag::Standard,
        );
        let mv2 = Move::new(
            Square::from_square_index(7),
            Square::from_square_index(10),
            MoveFlag::Standard,
        );
        let mv3 = Move::new(
            Square::from_square_index(7),
            Square::from_square_index(11),
            MoveFlag::Standard,
        );

        // our tt implementation always overwrites, so let's make sure that's the case.
        tt.store_entry(hash1, 3, Score::new(-123), EntryFlag::Exact, mv1);

        let stored_entry1 = tt.get_entry(hash1);
        assert!(stored_entry1.is_some());
        assert_eq!(stored_entry1.unwrap().board_move, mv1);

        tt.store_entry(hash2, 3, Score::new(123), EntryFlag::Exact, mv2);

        let stored_entry2 = tt.get_entry(hash2);
        assert!(stored_entry2.is_some());
        assert_eq!(stored_entry2.unwrap().board_move, mv2);

        tt.store_entry(hash3, 3, Score::new(123), EntryFlag::Exact, mv3);

        let stored_entry3 = tt.get_entry(hash3);
        assert!(stored_entry3.is_some());
        assert_eq!(stored_entry3.unwrap().board_move, mv3);
    }

    #[test]
    fn capacity() {
        let tt = TranspositionTable::from_size_in_mb(16);
        // Measured emperically. If the TT entry size changes, this test will fail.
        assert_eq!(tt.size(), 2097152);
        println!("{} entries", tt.size());
    }

    #[test]
    fn flags() {
        let mut flags = Flags::new(EntryFlag::None, 10);
        assert_eq!(flags.flag(), EntryFlag::None);
        assert_eq!(flags.age(), 10);

        flags = Flags::new(EntryFlag::Exact, 83);
        assert_eq!(flags.flag(), EntryFlag::Exact);
        assert_eq!(flags.age(), 83);

        flags = Flags::new(EntryFlag::LowerBound, 121);
        assert_eq!(flags.flag(), EntryFlag::LowerBound);
        assert_eq!(flags.age(), 121);
    }
}
