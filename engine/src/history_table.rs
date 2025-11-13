// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{
    definitions::NumberOf,
    pieces::{PIECE_NAMES, Piece},
    side::Side,
};

use crate::score::{LargeScoreType, Score};

pub struct HistoryTable {
    table: [[[LargeScoreType; NumberOf::SQUARES]; NumberOf::SQUARES]; NumberOf::SIDES],
}

/// Safe calculation of the bonus applied to quiet moves that are inserted into the history table.
/// This uses `wrappinag_mul` and `wrapping_sub` to safely calculate the value.
///
/// # Arguments
///
/// - depth: The current depth
///
/// # Returns
///
/// The calculated history score.
fn calculate_bonus_for_depth(depth: i16) -> i16 {
    depth
        .saturating_mul(Score::HISTORY_MULT)
        .saturating_sub(Score::HISTORY_OFFSET)
}

fn gravity_update(current_value: i32, clamped_bonus: i32) -> i32 {
    current_value + clamped_bonus - current_value * clamped_bonus.abs() / Score::MAX_HISTORY
}
pub(crate) enum HistoryUpdateType {
    Bonus,
    Malus,
}

impl HistoryTable {
    pub(crate) fn new() -> Self {
        let table = [[[Default::default(); NumberOf::SQUARES]; NumberOf::SQUARES]; NumberOf::SIDES];
        Self { table }
    }

    pub(crate) fn get(&self, side: Side, from: Square, to: Square) -> LargeScoreType {
        self.table[side as usize][from.to_square_index() as usize][to.to_square_index() as usize]
    }

    fn set(&mut self, side: Side, from: Square, to: Square, value: LargeScoreType) {
        self.table[side as usize][from.to_square_index() as usize][to.to_square_index() as usize] =
            value;
    }

    pub(crate) fn update(
        &mut self,
        depth: i16,
        side: Side,
        from: Square,
        to: Square,
        update_type: HistoryUpdateType,
    ) {
        let bonus = match update_type {
            HistoryUpdateType::Bonus => calculate_bonus_for_depth(depth) as LargeScoreType,
            HistoryUpdateType::Malus => -calculate_bonus_for_depth(depth) as LargeScoreType,
        };

        let current_value = self.get(side, from, to);
        let clamped_bonus = bonus.clamp(-Score::MAX_HISTORY, Score::MAX_HISTORY);
        let new_value = gravity_update(current_value, clamped_bonus);
        self.set(side, from, to, new_value);
    }

    pub(crate) fn clear(&mut self) {
        for side in 0..NumberOf::SIDES {
            for piece_type in 0..NumberOf::PIECE_TYPES {
                for square in 0..NumberOf::SQUARES {
                    self.table[side][piece_type][square] = Default::default();
                }
            }
        }
    }

    pub(crate) fn print_for_side(&self, side: Side) {
        for (piece_type, piece_name) in PIECE_NAMES.iter().enumerate() {
            println!("{piece_name} - {side}");
            // print from white's perspective
            for rank in (0..=NumberOf::RANKS - 1).rev() {
                print!("|");
                for file in 0..NumberOf::FILES {
                    let square = file + rank * NumberOf::FILES;
                    print!("{:5} ", self.table[side as usize][piece_type][square]);
                }
                println!("|");
            }
        }
    }
}

impl Default for HistoryTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        defs::MAX_DEPTH,
        history_table::{HistoryUpdateType, gravity_update},
        score::LargeScoreType,
    };

    use super::{HistoryTable, calculate_bonus_for_depth};
    use chess::{definitions::Squares, side::Side, square::Square};

    #[test]
    fn initialize_history_table() {
        let history_table = HistoryTable::new();
        // loop through all sides, piece types, and squares
        for side in 0..2 {
            for piece_type in 0..6 {
                for square in 0..64 {
                    assert_eq!(
                        history_table.table[side][piece_type][square],
                        Default::default()
                    );
                }
            }
        }
    }

    #[test]
    fn store_and_read() {
        let mut history_table = HistoryTable::new();
        let side = Side::Black;
        let from: Square = Squares::B2.into();
        let to: Square = Squares::C3.into();
        let depth = 5;
        let score = calculate_bonus_for_depth(depth) as LargeScoreType;
        history_table.update(depth, side, from, to, HistoryUpdateType::Bonus);

        assert_eq!(history_table.get(side, from, to), score);
        history_table.update(depth, side, from, to, HistoryUpdateType::Bonus);
        assert_eq!(
            history_table.get(side, from, to),
            gravity_update(score, score)
        );
        let current_value = history_table.get(side, from, to);
        history_table.update(depth, side, from, to, HistoryUpdateType::Malus);
        assert!(history_table.get(side, from, to) < score + score);
        assert_eq!(
            history_table.get(side, from, to),
            gravity_update(current_value, -score)
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
