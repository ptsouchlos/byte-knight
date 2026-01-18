// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::cmp::Ordering;

use anyhow::{Ok, Result};
use arrayvec::ArrayVec;
use chess::{definitions::MAX_MOVE_LIST_SIZE, moves::Move, pieces::Piece, side::Side};

use crate::{
    evaluation::Evaluation, hce_values::ByteKnightValues, history_table, score::LargeScoreType,
};

#[derive(PartialEq, Eq, Copy, Clone, Debug, Default)]
pub enum MoveOrder {
    #[default]
    TtMove,
    Capture(Piece, Piece),
    Quiet(LargeScoreType),
}

impl PartialOrd for MoveOrder {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MoveOrder {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            // move from the transposition table comes first
            (MoveOrder::TtMove, MoveOrder::TtMove) => Ordering::Equal,
            (MoveOrder::TtMove, _) => Ordering::Less,
            (_, MoveOrder::TtMove) => Ordering::Greater,

            // captures come next, according to MVV/LVA
            (
                MoveOrder::Capture(left_victim, left_attacker),
                MoveOrder::Capture(right_victim, right_attacker),
            ) => {
                let left_value =
                    Evaluation::<ByteKnightValues>::mvv_lva(*left_victim, *left_attacker);
                let right_value =
                    Evaluation::<ByteKnightValues>::mvv_lva(*right_victim, *right_attacker);
                right_value.cmp(&left_value)
            }
            (MoveOrder::Capture(_, _), _) => Ordering::Less,
            (_, MoveOrder::Capture(_, _)) => Ordering::Greater,

            // quiet moves come last, according to their score
            (MoveOrder::Quiet(left_score), MoveOrder::Quiet(right_score)) => {
                right_score.cmp(left_score)
            }
        }
    }
}

impl MoveOrder {
    /// Classify moves for move ordering  purposes.
    #[allow(clippy::expect_used)]
    pub fn classify(
        stm: Side,
        mv: &Move,
        tt_move: &Option<Move>,
        history_table: &history_table::HistoryTable,
    ) -> Self {
        if tt_move.is_some_and(|tt| *mv == tt) {
            return Self::TtMove;
        }

        if mv.is_capture() {
            let victim = mv.captured_piece().expect("Capture move without victim");
            let attacker = mv.piece();
            return Self::Capture(victim, attacker);
        }

        let score = history_table.get(stm, mv.piece(), mv.to());
        Self::Quiet(score)
    }

    pub fn classify_all(
        stm: Side,
        moves: &[Move],
        tt_move: &Option<Move>,
        history_table: &history_table::HistoryTable,
        move_order: &mut ArrayVec<MoveOrder, MAX_MOVE_LIST_SIZE>,
    ) -> Result<()> {
        move_order.clear();

        for mv in moves.iter() {
            move_order.try_push(Self::classify(stm, mv, tt_move, history_table))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chess::{board::Board, move_generation::MoveGenerator, move_list::MoveList, moves::Move};
    use itertools::Itertools;

    use crate::{
        move_order::MoveOrder,
        score::Score,
        ttable::{EntryFlag, TranspositionTable, TranspositionTableEntry},
    };

    #[test]
    fn verify_move_ordering() {
        let mut tt = TranspositionTable::from_capacity(10);
        let mut history_table = crate::history_table::HistoryTable::new();

        let move_gen = MoveGenerator::new();
        let mut move_list = MoveList::new();
        let board =
            Board::from_fen("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R w KQkq - 0 1").unwrap();
        move_gen.generate_legal_moves(&board, &mut move_list);

        assert!(move_list.len() >= 6);
        let depth = 3i32;
        let first_mv = move_list.at(4).unwrap();
        tt.store_entry(TranspositionTableEntry::new(
            board.zobrist_hash(),
            depth as u8,
            Score::new(1234),
            EntryFlag::Exact,
            *first_mv,
        ));

        let second_mv = move_list.at(2).unwrap();
        history_table.update(
            board.side_to_move(),
            second_mv.piece(),
            second_mv.to(),
            300 * depth - 250,
        );
        let tt_entry = tt.get_entry(board.zobrist_hash()).unwrap();
        let tt_move = tt_entry.board_move;
        // sort the moves
        let moves = move_list
            .iter()
            .sorted_by_key(|mv| {
                MoveOrder::classify(board.side_to_move(), mv, &Some(tt_move), &history_table)
            })
            .collect::<Vec<&Move>>();

        assert!(moves.len() >= 6);
        assert_eq!(moves[0], &tt_move);
        // check the order of the moves
    }

    // TODO(PT): Re-enable benchmark when bench is stablized (if ever)
    // #[bench]
    // fn bench_move_ordering(b: &mut Bencher) {
    //     // benchmark the worst case scenario
    //     // 218 possible moves
    //     let fen_pos = "R6R/3Q4/1Q4Q1/4Q3/2Q4Q/Q4Q2/pp1Q4/kBNN1KB1 w - - 0 1";
    //     let board = Board::from_fen(fen_pos).unwrap();
    //     let move_generation = MoveGenerator::new();
    //     let mut move_list = MoveList::new();
    //     let mut tt = TranspositionTable::from_capacity(10);
    //     let history_table = history_table::HistoryTable::new();

    //     move_generation.generate_legal_moves(&board, &mut move_list);
    //     assert!(move_list.len() == 218);

    //     // create a transposition table entry
    //     let king_from = Square::from_file_rank(File::F.to_char(), Rank::R1.as_number()).unwrap();
    //     let king_to = Square::from_file_rank(File::F.to_char(), Rank::R2.as_number()).unwrap();

    //     tt.store_entry(TranspositionTableEntry::new(
    //         board.zobrist_hash(),
    //         3,
    //         Score::new(1234),
    //         EntryFlag::Exact,
    //         Move::new_king_move(&king_from, &king_to, None),
    //     ));

    //     let mut move_order = ArrayVec::<MoveOrder, MAX_MOVE_LIST_SIZE>::new();
    //     let tt_move = tt.get_entry(board.zobrist_hash()).unwrap().board_move;

    //     // now classify the moves and bench
    //     b.iter(|| {
    //         MoveOrder::classify_all(
    //             board.side_to_move(),
    //             move_list.as_slice(),
    //             &Some(tt_move),
    //             &history_table,
    //             &mut move_order,
    //         )
    //         .unwrap();
    //     });
    // }
}
