// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{bitboard::Bitboard, moves::Move, side::Side};

use crate::{
    history::{
        threat_bucket::{ThreatBucket, ThreatIndex},
        types::{self, FromToHistory},
        util::gravity,
    },
    score::{LargeScoreType, Score},
};

/// A single quiet-history cell, split into a threat-agnostic `factoriser` and a `bucket` indexed
/// by whether the move's `from`/`to` squares are currently attacked by the opponent.
///
/// The factorizer is updated on every touch regardless of threat state, carrying a dense
/// threat-agnostic baseline; each bucket cell only has to learn the *delta* for its specific
/// threat configuration. A bare `[2][2]` split without this baseline would update each cell a
/// quarter as often, producing a sparse/noisy table.
#[derive(Default, Copy, Clone, Debug, PartialEq)]
struct QuietHistoryEntry {
    factorizer: LargeScoreType,
    bucket: ThreatBucket<i32>,
}

impl QuietHistoryEntry {
    fn score(&self, threat_index: ThreatIndex) -> LargeScoreType {
        self.factorizer + self.bucket[threat_index.from()][threat_index.to()]
    }

    fn update(
        &mut self,
        threat_index: ThreatIndex,
        bonus: LargeScoreType,
        factorizer_bonus: LargeScoreType,
    ) {
        // Clamp defensively regardless of what the caller passes in - gravity() multiplies
        // `current * bonus.abs()`, which overflows i32 for a large-enough unclamped bonus.
        let factorizer_bonus =
            factorizer_bonus.clamp(-Score::FACTORIZER_MAX, Score::FACTORIZER_MAX);
        self.factorizer = gravity(self.factorizer, factorizer_bonus, Score::FACTORIZER_MAX);

        let bonus = bonus.clamp(-Score::BUCKET_MAX, Score::BUCKET_MAX);
        let cell = &mut self.bucket[threat_index.from()][threat_index.to()];
        *cell = gravity(*cell, bonus, Score::BUCKET_MAX);
    }
}

/// History table for all quiet moves, indexed by from-square -> to-square -> per side (butterfly
/// history), with each entry further split into threat buckets (see [`QuietHistoryEntry`]).
pub struct QuietHistory {
    from_to_entries: [FromToHistory<QuietHistoryEntry>; Side::COUNT],
}

/// Safe calculation of the bonus applied to quiet moves that are inserted into the history table.
/// This uses `wrappinag_mul` and `wrapping_sub` to safely calculate the value.
///
/// # Arguments
/// - `depth`: The current depth
///
/// # Returns
/// The calculated history score.
pub(crate) fn calculate_bonus_for_depth(depth: i16) -> i16 {
    depth
        .saturating_mul(Score::HISTORY_MULT)
        .saturating_sub(Score::HISTORY_OFFSET)
}

impl QuietHistory {
    pub(crate) fn new() -> Self {
        let from_to_entries = [types::default_from_to_history(); Side::COUNT];
        Self { from_to_entries }
    }

    pub(crate) fn get(&self, side: Side, mv: Move, threats: Bitboard) -> LargeScoreType {
        let idx = ThreatIndex::new(&mv, threats);
        self.from_to_entries[side][mv.from()][mv.to()].score(idx)
    }

    pub(crate) fn update(
        &mut self,
        side: Side,
        mv: Move,
        threats: Bitboard,
        bonus: LargeScoreType,
        factorizer_bonus: LargeScoreType,
    ) {
        let idx = ThreatIndex::new(&mv, threats);
        self.from_to_entries[side][mv.from()][mv.to()].update(idx, bonus, factorizer_bonus);
    }

    pub(crate) fn clear(&mut self) {
        self.from_to_entries = [types::default_from_to_history(); Side::COUNT];
    }
}

impl Default for QuietHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::defs::MAX_DEPTH;

    use super::{QuietHistory, calculate_bonus_for_depth};
    use chess::{bitboard::Bitboard, moves::Move, side::Side, square::Square};

    #[test]
    fn initialize_history_table() {
        let history_table = QuietHistory::new();
        // loop through all sides, from-squares, and to-squares
        for side in 0..2 {
            for from in 0..64 {
                for to in 0..64 {
                    assert_eq!(
                        history_table.from_to_entries[side][from][to],
                        Default::default()
                    );
                }
            }
        }
    }

    #[test]
    fn store_and_read() {
        // Values are the sum of a factoriser and a bucket cell (see `QuietHistoryEntry`), so
        // repeated positive updates must strictly increase the read-back score - but the exact
        // magnitude is an internal accounting detail (bonus split, gravity truncation), not
        // something worth hardcoding here.
        let mut history_table = QuietHistory::new();
        let mv = Move::new(Square::B1, Square::A1, chess::moves::MoveFlag::Standard);
        let side = Side::Black;
        let score = 37;
        let no_threats = Bitboard::default();

        assert_eq!(history_table.get(side, mv, no_threats), 0);
        history_table.update(side, mv, no_threats, score, score);
        let after_first = history_table.get(side, mv, no_threats);
        assert!(
            after_first > 0,
            "a positive bonus must raise the score above zero"
        );

        history_table.update(side, mv, no_threats, score, score);
        let after_second = history_table.get(side, mv, no_threats);
        assert!(
            after_second > after_first,
            "a second positive bonus must raise the score further"
        );
    }

    #[test]
    fn threat_buckets_are_independent_of_untouched_buckets() {
        let mut history_table = QuietHistory::new();
        let mv = Move::new(Square::B1, Square::A1, chess::moves::MoveFlag::Standard);
        let side = Side::Black;

        let no_threats = Bitboard::default();
        let from_only_threatened = Bitboard::from(Square::B1);
        let both_threatened = Bitboard::from(Square::B1) | Bitboard::from(Square::A1);

        // Only ever update the "both squares threatened" bucket.
        history_table.update(side, mv, both_threatened, 1000, 1000);

        let untouched = history_table.get(side, mv, no_threats);
        let touched = history_table.get(side, mv, both_threatened);
        assert!(
            untouched > 0,
            "the factoriser baseline should move on every update, regardless of threat state"
        );
        assert!(
            touched > untouched,
            "the touched bucket should score higher than a bucket that never saw the bonus"
        );

        let from_only = history_table.get(side, mv, from_only_threatened);
        assert_eq!(
            from_only, untouched,
            "a bucket that was never updated should equal the other untouched buckets"
        );
    }

    #[test]
    fn calculate_bonus_for_any_depth() {
        for depth in 1..MAX_DEPTH {
            let bonus = calculate_bonus_for_depth(depth as i16);
            assert!(bonus > 0);
            assert!(bonus as i32 <= i16::MAX.into());
        }
    }
}
