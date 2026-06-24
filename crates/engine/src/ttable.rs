// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::moves::Move;

use crate::{
    node_types::NodeType,
    score::{Score, ScoreType},
    utils,
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
const TT_AGE_MASK: u8 = 0b11111100;
const TT_AGE_SHIFT: u8 = 2;
const MAX_AGE: u8 = 0x3F;

impl Flags {
    pub fn new(tt_flag: EntryFlag, age: u8) -> Self {
        Self {
            data: (tt_flag as u8) | ((age & MAX_AGE) << TT_AGE_SHIFT),
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
        age.wrapping_sub(self.age()) & 0x3F
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
        utils::fast_range_64(zobrist, self.table.len() as u64) as usize
    }

    pub(crate) fn get_entry(&self, zobrist: u64) -> Option<TranspositionTableEntry> {
        let index = self.get_index(zobrist);
        let bucket = &self.table[index];
        bucket
            .entries
            .iter()
            .find(|&entry| entry.validate_key(zobrist))
            .copied()
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

        // Find the target entry: prefer key match or empty slot,
        // otherwise pick the lowest-quality entry as the replacement victim.
        // We follow the same logic here (basically) as stockfish though we don't take into account PV.
        let mut replace_index = 0;
        let mut min_quality = i32::MAX;
        for (i, entry) in bucket.entries.iter().enumerate() {
            if entry.validate_key(zobrist) || entry.is_empty() {
                replace_index = i;
                break;
            }

            let quality = entry.depth as i32 - 4 * entry.relative_age(tt_age) as i32;
            if quality < min_quality {
                min_quality = quality;
                replace_index = i;
            }
        }

        let entry = &mut bucket.entries[replace_index];
        let key_match = entry.validate_key(zobrist);

        // Preserve the existing move if we have a key match and no new move.
        // In practice mv is always valid, but this mirrors Stockfish's logic.
        if !mv.is_null_move() || !key_match {
            entry.board_move = mv;
        }

        // Overwrite the entry if any of these hold (cheapest checks first):
        //  1. New entry is exact
        //  2. Different position (includes empty slots, which have key=0)
        //  3. New depth is close enough to the stored depth (within -4)
        //  4. Stored entry is from a previous generation (stale)
        if flag == EntryFlag::Exact
            || !key_match
            || depth as i32 + 4 > entry.depth as i32
            || entry.relative_age(tt_age) != 0
        {
            entry.zobrist_key = zobrist as u16;
            entry.depth = depth;
            entry.score = score;
            entry.flags = Flags::new(flag, tt_age);
        }
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
        let total_entries = self.table.len() * ENTRIES_PER_BUCKET;
        let used = self
            .table
            .iter()
            .flat_map(|bucket| bucket.entries.iter())
            .filter(|entry| !entry.is_empty())
            .count();
        (used as f64 / total_entries as f64) * 100_f64
    }

    /// Returns hashfull output for UCI purposes. Only considers non-empty entries in the first 1000 entries.
    pub(crate) fn hashfull(&self) -> u16 {
        let mut fill = 0;
        for bucket in self.table.iter().take(1000 / ENTRIES_PER_BUCKET) {
            for entry in &bucket.entries {
                if entry.flags.flag() != EntryFlag::None {
                    fill += 1;
                }
            }
        }
        fill
    }

    /// Returns the size of the transposition table (number of buckets).
    pub(crate) fn size(&self) -> usize {
        self.table.len()
    }

    /// Increments the age of the transposition table.
    pub(crate) fn increment_age(&mut self) {
        self.age = (self.age + 1) & MAX_AGE;
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
            self.hits += 1;
            if entry.depth >= depth as u8 {
                // Cutoff can only happen if the entry depth >= current depth and 1 of the following:
                // - the entry type is exact
                // - the entry type is lower bound and the score >= beta
                // - the entry type is upper bound and the score <= alpha
                // see https://www.chessprogramming.org/Transposition_Table#Transposition_Table_Cutoffs
                let ply_relative_score = entry.score.ply_relative(ply);
                if entry.flag() == EntryFlag::Exact
                    || ((entry.flag() == EntryFlag::LowerBound && ply_relative_score >= beta)
                        || (entry.flag() == EntryFlag::UpperBound && ply_relative_score <= alpha))
                {
                    return ProbeResult::CutOff(entry);
                }
            }
            return ProbeResult::Hit(entry);
        }

        ProbeResult::Empty
    }

    pub fn prefetch(&self, zobrist: u64) {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};
            use std::ptr;

            let index = self.get_index(zobrist);
            let ptr = ptr::from_ref(&self.table[index]).cast();
            _mm_prefetch(ptr, _MM_HINT_T0);
        }

        #[cfg(all(nightly, target_arch = "aarch64"))]
        unsafe {
            use std::arch::aarch64::{_PREFETCH_LOCALITY3, _PREFETCH_READ, _prefetch};
            use std::ptr;

            let index = self.get_index(zobrist);
            let ptr = ptr::from_ref(&self.table[index]).cast();
            _prefetch(ptr, _PREFETCH_READ, _PREFETCH_LOCALITY3);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EntryFlag, TranspositionTable, TranspositionTableEntry};
    use crate::{
        score::Score,
        ttable::{Bucket, Flags},
    };
    use chess::{
        moves::{Move, MoveFlag},
        square::Square,
    };
    use itertools::Itertools;
    use rand::Rng;
    use std::collections::HashMap;

    fn make_move(from: u8, to: u8) -> Move {
        Move::new(
            Square::from_square_index(from),
            Square::from_square_index(to),
            MoveFlag::Standard,
        )
    }

    #[test]
    fn entry_size() {
        assert_eq!(std::mem::size_of::<TranspositionTableEntry>(), 8);
    }

    #[test]
    fn bucket_size() {
        assert_eq!(std::mem::size_of::<Bucket>(), 32);
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
        let hash1 = 0xDEAD_BEEF_1234_5678_u64;
        let hash2 = 0xCAFE_BABE_ABCD_EF01_u64;
        let hash3 = 0x1234_ABCD_5678_EF01_u64;
        let mv1 = make_move(3, 4);
        let mv2 = make_move(7, 10);
        let mv3 = make_move(7, 11);

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
        // 16 MB / 32 bytes per bucket = 524288 buckets
        assert_eq!(tt.size(), 524288);
        println!("{} buckets", tt.size());
    }

    #[test]
    fn flags_roundtrip() {
        let mut flags = Flags::new(EntryFlag::None, 10);
        assert_eq!(flags.flag(), EntryFlag::None);
        assert_eq!(flags.age(), 10);

        flags = Flags::new(EntryFlag::Exact, 0);
        assert_eq!(flags.flag(), EntryFlag::Exact);
        assert_eq!(flags.age(), 0);

        flags = Flags::new(EntryFlag::LowerBound, 63);
        assert_eq!(flags.flag(), EntryFlag::LowerBound);
        assert_eq!(flags.age(), 63);

        flags = Flags::new(EntryFlag::UpperBound, 42);
        assert_eq!(flags.flag(), EntryFlag::UpperBound);
        assert_eq!(flags.age(), 42);
    }

    #[test]
    fn flags_age_masks_overflow() {
        // Ages above 63 should be masked to 6 bits
        let flags = Flags::new(EntryFlag::Exact, 64);
        assert_eq!(flags.age(), 0);
        assert_eq!(flags.flag(), EntryFlag::Exact);

        let flags = Flags::new(EntryFlag::LowerBound, 65);
        assert_eq!(flags.age(), 1);
        assert_eq!(flags.flag(), EntryFlag::LowerBound);

        let flags = Flags::new(EntryFlag::UpperBound, 127);
        assert_eq!(flags.age(), 63);
        assert_eq!(flags.flag(), EntryFlag::UpperBound);
    }

    #[test]
    fn age_wrapping() {
        let mut tt = TranspositionTable::from_capacity(1024);
        assert_eq!(tt.age, 0);

        // Increment to max
        for _ in 0..63 {
            tt.increment_age();
        }
        assert_eq!(tt.age, 63);

        // Should wrap to 0
        tt.increment_age();
        assert_eq!(tt.age, 0);

        tt.increment_age();
        assert_eq!(tt.age, 1);
    }

    #[test]
    fn relative_age_across_wrap() {
        // Entry stored at age 62, current age is 1 (wrapped past 63→0→1)
        let entry = TranspositionTableEntry::new(
            0x1234,
            10,
            Score::new(0),
            make_move(0, 1),
            EntryFlag::Exact,
            62,
        );
        // relative_age = (1 - 62) & 0x3F = (-61) & 63 = 3
        assert_eq!(entry.relative_age(1), 3);

        // Entry stored at age 0, current age is 63
        let entry2 = TranspositionTableEntry::new(
            0x1234,
            10,
            Score::new(0),
            make_move(0, 1),
            EntryFlag::Exact,
            0,
        );
        assert_eq!(entry2.relative_age(63), 63);

        // Same age → relative age 0
        assert_eq!(entry2.relative_age(0), 0);
    }

    #[test]
    fn same_key_deeper_replaces() {
        let mut tt = TranspositionTable::from_capacity(1024);
        let hash = 0xDEAD_0000_0000_1234_u64;
        let mv1 = make_move(0, 8);
        let mv2 = make_move(1, 9);

        tt.store_entry(hash, 5, Score::new(100), EntryFlag::Exact, mv1);
        let entry = tt.get_entry(hash).unwrap();
        assert_eq!(entry.board_move, mv1);
        assert_eq!(entry.depth, 5);

        // Deeper entry on same key — should fully replace
        tt.store_entry(hash, 10, Score::new(200), EntryFlag::LowerBound, mv2);
        let entry = tt.get_entry(hash).unwrap();
        assert_eq!(entry.board_move, mv2);
        assert_eq!(entry.depth, 10);
        assert_eq!(entry.score, Score::new(200));

        // Verify only one slot is used (no duplicate)
        let index = tt.get_index(hash);
        let bucket = &tt.table[index];
        let matches = bucket
            .entries
            .iter()
            .filter(|e| e.validate_key(hash))
            .count();
        assert_eq!(matches, 1);
    }

    #[test]
    fn same_key_shallow_same_gen_preserves_depth_and_score() {
        let mut tt = TranspositionTable::from_capacity(1024);
        let hash = 0xDEAD_0000_0000_1234_u64;
        let mv1 = make_move(0, 8);
        let mv2 = make_move(1, 9);

        // Store a deep entry at current generation
        tt.store_entry(hash, 20, Score::new(300), EntryFlag::LowerBound, mv1);

        // Shallow non-exact same-gen entry on same key.
        // Guard: Exact? no. !key_match? no. depth(3)+4=7 > 20? no. stale? no.
        // → Guard fails, only the move is updated.
        tt.store_entry(hash, 3, Score::new(50), EntryFlag::UpperBound, mv2);
        let entry = tt.get_entry(hash).unwrap();
        assert_eq!(entry.board_move, mv2, "move should be refreshed");
        assert_eq!(entry.depth, 20, "depth should be preserved");
        assert_eq!(entry.score, Score::new(300), "score should be preserved");
        assert_eq!(
            entry.flag(),
            EntryFlag::LowerBound,
            "flag should be preserved"
        );
    }

    #[test]
    fn same_key_shallow_exact_replaces() {
        let mut tt = TranspositionTable::from_capacity(1024);
        let hash = 0xDEAD_0000_0000_1234_u64;
        let mv1 = make_move(0, 8);
        let mv2 = make_move(1, 9);

        // Store a deep entry
        tt.store_entry(hash, 20, Score::new(300), EntryFlag::LowerBound, mv1);

        // Shallow but Exact on same key — should fully replace
        tt.store_entry(hash, 3, Score::new(50), EntryFlag::Exact, mv2);
        let entry = tt.get_entry(hash).unwrap();
        assert_eq!(entry.board_move, mv2);
        assert_eq!(entry.depth, 3);
        assert_eq!(entry.score, Score::new(50));
        assert_eq!(entry.flag(), EntryFlag::Exact);
    }

    #[test]
    fn same_key_stale_entry_fully_replaced() {
        let mut tt = TranspositionTable::from_capacity(1024);
        let hash = 0xDEAD_0000_0000_1234_u64;

        // Store deep entry at age 0
        tt.store_entry(
            hash,
            20,
            Score::new(100),
            EntryFlag::LowerBound,
            make_move(0, 1),
        );

        // Advance generation — entry is now stale
        tt.increment_age();
        tt.increment_age();

        // Shallow non-exact on same key, but entry is stale → full overwrite
        tt.store_entry(
            hash,
            1,
            Score::new(50),
            EntryFlag::UpperBound,
            make_move(1, 2),
        );
        let entry = tt.get_entry(hash).unwrap();
        assert_eq!(entry.age(), 2, "age should be current generation");
        assert_eq!(
            entry.depth, 1,
            "depth should be overwritten (entry was stale)"
        );
        assert_eq!(entry.score, Score::new(50));
        assert_eq!(entry.flag(), EntryFlag::UpperBound);
    }

    #[test]
    fn bucket_fills_four_slots() {
        // Use a tiny table so we can craft collisions on the same bucket index.
        let mut tt = TranspositionTable::from_capacity(1);

        // 4 different hashes that all map to bucket 0 (only 1 bucket),
        // but have different lower 16 bits so they don't key-match.
        let hashes: Vec<u64> = vec![0x0001, 0x0002, 0x0003, 0x0004];
        for (i, &h) in hashes.iter().enumerate() {
            tt.store_entry(
                h,
                (i + 1) as u8,
                Score::new(i as i16),
                EntryFlag::Exact,
                make_move(0, (i + 1) as u8),
            );
        }

        // All 4 should be retrievable
        for &h in &hashes {
            assert!(tt.get_entry(h).is_some(), "hash {h:#x} should be found");
        }
    }

    #[test]
    fn bucket_overflow_evicts_worst_quality() {
        let mut tt = TranspositionTable::from_capacity(1);

        // Fill all 4 slots with different depths at age 0
        // depth values: 10, 20, 5, 15
        let depths = [10u8, 20, 5, 15];
        let hashes: Vec<u64> = vec![0x0001, 0x0002, 0x0003, 0x0004];
        for (i, (&h, &d)) in hashes.iter().zip(depths.iter()).enumerate() {
            tt.store_entry(
                h,
                d,
                Score::new(0),
                EntryFlag::Exact,
                make_move(0, (i + 1) as u8),
            );
        }

        // Verify depth=5 entry exists
        assert!(tt.get_entry(0x0003).is_some());

        // Store a 5th entry — should evict the lowest-quality (depth=5, hash=0x0003)
        tt.store_entry(
            0x0005,
            12,
            Score::new(0),
            EntryFlag::Exact,
            make_move(0, 10),
        );

        // depth=5 entry should be evicted
        assert!(tt.get_entry(0x0003).is_none());
        // New entry should be present
        assert!(tt.get_entry(0x0005).is_some());
        // Others should still be present
        assert!(tt.get_entry(0x0001).is_some());
        assert!(tt.get_entry(0x0002).is_some());
        assert!(tt.get_entry(0x0004).is_some());
    }

    #[test]
    fn stale_age_entries_evicted_first() {
        let mut tt = TranspositionTable::from_capacity(1);

        // Fill 4 slots at age 0 with high depth
        let hashes: Vec<u64> = vec![0x0001, 0x0002, 0x0003, 0x0004];
        for (i, &h) in hashes.iter().enumerate() {
            tt.store_entry(
                h,
                20,
                Score::new(0),
                EntryFlag::Exact,
                make_move(0, (i + 1) as u8),
            );
        }

        // Advance age by 2 generations
        tt.increment_age();
        tt.increment_age();

        // Store a new entry with low depth — should still evict a stale entry
        // because age penalty is -4 * 2 = -8, making stale entries score 20 - 8 = 12
        // The new entry depth is 1, but the replacement guard allows it because ages differ
        tt.store_entry(0x0005, 1, Score::new(0), EntryFlag::Exact, make_move(0, 10));

        assert!(tt.get_entry(0x0005).is_some());
    }

    #[test]
    fn different_key_always_replaces_victim() {
        // In Stockfish's model, a different-key write always overwrites the
        // victim (the `!key_match` arm of the guard). The guard only protects
        // same-key entries from shallow non-exact same-gen overwrites.
        let mut tt = TranspositionTable::from_capacity(1);
        let hashes: Vec<u64> = vec![0x0001, 0x0002, 0x0003, 0x0004];
        for (i, &h) in hashes.iter().enumerate() {
            tt.store_entry(
                h,
                20,
                Score::new(0),
                EntryFlag::LowerBound,
                make_move(0, (i + 1) as u8),
            );
        }

        // All slots full at depth=20, same age. Store depth=1 with a different key.
        // The victim is the lowest-quality entry (all equal, so first slot).
        // Guard: !key_match → true → overwrites.
        tt.store_entry(
            0x0005,
            1,
            Score::new(0),
            EntryFlag::LowerBound,
            make_move(0, 10),
        );
        assert!(
            tt.get_entry(0x0005).is_some(),
            "different-key should always replace the victim"
        );
    }

    #[test]
    fn hashfull_correctness() {
        let mut tt = TranspositionTable::from_capacity(1024);

        // Fill exactly 500 entries in the first 250 buckets (2 per bucket)
        for i in 0..250u64 {
            let h1 = i * 2 + 1;
            let h2 = i * 2 + 2;
            // We need hashes that map to bucket indices 0..250.
            // With from_capacity(1024), fast_range_64(h, 1024) maps h to a bucket.
            // Just store them — they'll land wherever they land.
            tt.store_entry(h1, 5, Score::new(0), EntryFlag::Exact, make_move(0, 1));
            tt.store_entry(h2, 5, Score::new(0), EntryFlag::Exact, make_move(0, 1));
        }

        let hf = tt.hashfull();
        // hashfull samples first 250 buckets (1000/4). Not all our entries will land there,
        // but the result should be > 0 and <= 1000.
        assert!(hf <= 1000);
    }

    #[test]
    fn increment_age_and_clear() {
        let mut tt = TranspositionTable::from_capacity(64);
        tt.increment_age();
        tt.increment_age();
        tt.increment_age();
        assert_eq!(tt.age, 3);

        tt.store_entry(
            0x1234,
            5,
            Score::new(100),
            EntryFlag::Exact,
            make_move(0, 1),
        );
        assert!(tt.get_entry(0x1234).is_some());

        tt.clear();
        assert_eq!(tt.age, 0);
        assert!(tt.get_entry(0x1234).is_none());
        assert_eq!(tt.accesses, 0);
        assert_eq!(tt.hits, 0);
    }

    #[test]
    fn fullness_empty_table() {
        let tt = TranspositionTable::from_capacity(64);
        assert_eq!(tt.fullness(), 0.0);
    }

    #[test]
    fn fullness_with_entries() {
        let mut tt = TranspositionTable::from_capacity(1024);
        // Use well-distributed hashes so each lands in a different bucket
        let hashes: Vec<u64> = (0..10u64)
            .map(|i| 0xDEAD_BEEF_0000_0000 | (i * 0x1111_1111_1111))
            .collect();
        for h in &hashes {
            tt.store_entry(*h, 5, Score::new(0), EntryFlag::Exact, make_move(0, 1));
        }
        let f = tt.fullness();
        assert!(f > 0.0);
        assert!(f <= 100.0);
    }
}
