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
    /// The history table is a 5D array indexed by [is_from_attacked][is_to_attacked][side][piece_type][square (to)]
    table: [[[[[LargeScoreType; NumberOf::SQUARES]; NumberOf::PIECE_TYPES]; NumberOf::SIDES];
        NumberOf::SIDES]; NumberOf::SIDES],
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
pub(crate) fn calculate_bonus_for_depth(depth: i16) -> i16 {
    depth
        .saturating_mul(Score::HISTORY_MULT)
        .saturating_sub(Score::HISTORY_OFFSET)
}

impl HistoryTable {
    pub(crate) fn new() -> Self {
        let table = [[[[[Default::default(); NumberOf::SQUARES]; NumberOf::PIECE_TYPES];
            NumberOf::SIDES]; NumberOf::SIDES]; NumberOf::SIDES];
        Self { table }
    }

    pub(crate) fn get(
        &self,
        side: Side,
        piece: Piece,
        square: u8,
        is_from_attacked: bool,
        is_to_attacked: bool,
    ) -> LargeScoreType {
        self.table[is_from_attacked as usize][is_to_attacked as usize][side as usize]
            [piece as usize][square as usize]
    }

    fn get_mut(
        &mut self,
        side: Side,
        piece: Piece,
        square: u8,
        is_from_attacked: bool,
        is_to_attacked: bool,
    ) -> &mut LargeScoreType {
        &mut self.table[is_from_attacked as usize][is_to_attacked as usize][side as usize]
            [piece as usize][square as usize]
    }

    pub(crate) fn update(
        &mut self,
        side: Side,
        piece: Piece,
        square: u8,
        is_from_attacked: bool,
        is_to_attacked: bool,
        bonus: LargeScoreType,
    ) {
        let current_value = self.get(side, piece, square, is_from_attacked, is_to_attacked);
        let clamped_bonus = bonus.clamp(-Score::MAX_HISTORY, Score::MAX_HISTORY);
        let new_value = current_value + clamped_bonus
            - current_value * clamped_bonus.abs() / Score::MAX_HISTORY;
        *self.get_mut(side, piece, square, is_from_attacked, is_to_attacked) = new_value;
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::new();
    }

    pub(crate) fn print_for_side(&self, side: Side) {
        for (piece_type, piece_name) in PIECE_NAMES.iter().enumerate() {
            println!("{piece_name} - {side}");
            // print from white's perspective
            for rank in (0..=NumberOf::RANKS - 1).rev() {
                print!("|");
                for file in 0..NumberOf::FILES {
                    let square = file + rank * NumberOf::FILES;
                    let grid = self.table[side as usize][piece_type][square];
                    print!(
                        "{:5} | {:5}\n------\n[{:5} | {:5} ]",
                        grid[0][0], grid[0][1], grid[1][0], grid[1][1]
                    );
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
    use crate::defs::MAX_DEPTH;

    use super::{HistoryTable, calculate_bonus_for_depth};
    use chess::{definitions::Squares, pieces::Piece, side::Side};

    #[test]
    fn initialize_history_table() {
        let history_table = HistoryTable::new();
        // loop through all sides, piece types, and squares
        for side in 0..2u8 {
            for piece_type in 0..6 {
                for square in 0..64 {
                    for is_from_attacked in 0..2 {
                        for is_to_attacked in 0..2 {
                            assert_eq!(
                                history_table.get(
                                    Side::try_from(side).unwrap(),
                                    Piece::try_from(piece_type).unwrap(),
                                    square as u8,
                                    is_from_attacked != 0,
                                    is_to_attacked != 0,
                                ),
                                Default::default()
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn store_and_read() {
        let mut history_table = HistoryTable::new();
        let side = Side::Black;
        let piece = Piece::Pawn;
        let square = Squares::A1;
        let score = 37;
        let is_from_attacked = true;
        let is_to_attacked = false;
        history_table.update(side, piece, square, is_from_attacked, is_to_attacked, score);
        assert_eq!(
            history_table.get(side, piece, square, is_from_attacked, is_to_attacked),
            score
        );
        history_table.update(side, piece, square, is_from_attacked, is_to_attacked, score);
        assert_eq!(
            history_table.get(side, piece, square, is_from_attacked, is_to_attacked),
            score + score
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

    #[test]
    fn update_and_then_clear() {
        let mut history_table = HistoryTable::new();
        let side = Side::White;
        let piece = Piece::Knight;
        let square = Squares::E4;
        let is_from_attacked = false;
        let is_to_attacked = true;
        let bonus = 50;

        history_table.update(side, piece, square, is_from_attacked, is_to_attacked, bonus);
        let stored_value = history_table.get(side, piece, square, is_from_attacked, is_to_attacked);
        assert_eq!(stored_value, bonus);

        history_table.clear();
        let cleared_value =
            history_table.get(side, piece, square, is_from_attacked, is_to_attacked);
        assert_eq!(cleared_value, 0);
    }
}
