// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use arrayvec::ArrayVec;
use chess::{
    board::Board,
    definitions::MAX_MOVE_LIST_SIZE,
    move_generation::{self, move_filter::MoveFilter},
    move_list::MoveList,
    moves::{Move, MoveFlag},
    pieces::Piece,
};

use crate::{
    evaluation::Evaluation,
    hce_values::ByteKnightValues,
    history_table::HistoryTable,
    killers_table::{KillerEntry, KillerMovesTable},
    score::{LargeScoreType, Score},
};

/// Bonus applied to killer moves in the quiet scoring stage so they sort
/// above all history-scored quiets (history is clamped to MAX_HISTORY).
const KILLER_BONUS: LargeScoreType = Score::MAX_HISTORY + 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    TtMove,
    ScoreTacticals,
    YieldTacticals,
    ScoreQuiets,
    YieldQuiets,
    Done,
}

/// A staged move picker that yields moves in best-first order.
///
/// Main search stages:
///   `TtMove → ScoreTacticals → YieldTacticals → ScoreQuiets → YieldQuiets → Done`
///
/// For quiescence search (`new_qsearch`), the quiet stages are skipped when not
/// in check (no quiets are generated).
pub(crate) struct MovePicker {
    stage: Stage,
    moves: MoveList,
    /// Scores parallel to `moves`. Populated per stage.
    scores: [LargeScoreType; MAX_MOVE_LIST_SIZE],
    tt_move: Option<Move>,
    killers: [Option<KillerEntry>; 2],
    /// End index (exclusive) of the tactical partition: moves[0..tactical_end).
    tactical_end: usize,
    /// End index (exclusive) of the quiet partition: moves[tactical_end..quiet_end).
    quiet_end: usize,
    /// Monotonically increasing selection-sort cursor (absolute index).
    pick_index: usize,
    /// Total moves yielded so far. Caller uses `moves_yielded() - 1` as the loop counter.
    moves_yielded: usize,
    /// Quiet (non-promotion, non-capture) moves yielded so far, with their moving piece.
    /// Used by the caller for history penalty on a beta cutoff.
    searched_quiets: ArrayVec<(Move, Piece), MAX_MOVE_LIST_SIZE>,
}

/// Returns true if `mv` is tactical: a capture or a queen push-promotion.
fn is_tactical(board: &Board, mv: Move) -> bool {
    board.captured(&mv).is_some() || mv.flag() == MoveFlag::PromotionQueen
}

/// Partitions `moves` in-place so tacticals come first.
/// Returns `(tactical_end, quiet_end)` where `quiet_end == moves.len()`.
fn partition(board: &Board, moves: &mut MoveList) -> (usize, usize) {
    let total = moves.len();
    let mut tactical_end = 0;
    for i in 0..total {
        if is_tactical(board, moves.as_slice()[i]) {
            moves.as_mut_slice().swap(i, tactical_end);
            tactical_end += 1;
        }
    }
    (tactical_end, total)
}

impl MovePicker {
    /// Creates a `MovePicker` for the main negamax search.
    ///
    /// Generates all legal moves, partitions them (tacticals first), and sets up
    /// the stage sequence for full move ordering.
    pub(crate) fn new(
        board: &Board,
        tt_move: Option<Move>,
        killers_table: &KillerMovesTable,
        ply: u8,
    ) -> Self {
        let mut moves = move_generation::legal::generate_moves(board, MoveFilter::All);
        let (tactical_end, quiet_end) = partition(board, &mut moves);

        let killer_slice = killers_table.get(ply);
        let killers = [
            killer_slice.first().copied().unwrap_or(None),
            killer_slice.get(1).copied().unwrap_or(None),
        ];

        Self {
            stage: if tt_move.is_some() {
                Stage::TtMove
            } else {
                Stage::ScoreTacticals
            },
            moves,
            scores: [0; MAX_MOVE_LIST_SIZE],
            tt_move,
            killers,
            tactical_end,
            quiet_end,
            pick_index: 0,
            moves_yielded: 0,
            searched_quiets: ArrayVec::new(),
        }
    }

    /// Creates a `MovePicker` for quiescence search.
    ///
    /// When not in check, only tacticals are generated and the quiet stages are
    /// skipped. When in check, all legal moves are generated and all stages run
    /// (since king evasions include quiet moves).
    pub(crate) fn new_qsearch(board: &Board, tt_move: Option<Move>, in_check: bool) -> Self {
        if in_check {
            let mut moves = move_generation::legal::generate_moves(board, MoveFilter::All);
            let (tactical_end, quiet_end) = partition(board, &mut moves);
            Self {
                stage: if tt_move.is_some() {
                    Stage::TtMove
                } else {
                    Stage::ScoreTacticals
                },
                moves,
                scores: [0; MAX_MOVE_LIST_SIZE],
                tt_move,
                killers: [None; 2],
                tactical_end,
                quiet_end,
                pick_index: 0,
                moves_yielded: 0,
                searched_quiets: ArrayVec::new(),
            }
        } else {
            // Only tacticals (captures + queen promotions).
            // Set quiet_end == tactical_end so ScoreQuiets/YieldQuiets are no-ops.
            let moves = move_generation::legal::generate_moves(board, MoveFilter::Tacticals);
            let n = moves.len();
            Self {
                stage: if tt_move.is_some() {
                    Stage::TtMove
                } else {
                    Stage::ScoreTacticals
                },
                moves,
                scores: [0; MAX_MOVE_LIST_SIZE],
                tt_move,
                killers: [None; 2],
                tactical_end: n,
                quiet_end: n,
                pick_index: 0,
                moves_yielded: 0,
                searched_quiets: ArrayVec::new(),
            }
        }
    }

    /// Returns `true` if no legal moves were generated (checkmate or stalemate).
    pub(crate) fn is_empty(&self) -> bool {
        self.moves.is_empty()
    }

    /// Returns the number of moves yielded so far.
    /// The caller can compute `loop_counter = moves_yielded() - 1` (0-based) after each `next()`.
    pub(crate) fn moves_yielded(&self) -> usize {
        self.moves_yielded
    }

    /// Returns the quiet moves yielded so far (move + piece pairs).
    /// Used by the caller to apply history penalties on a beta cutoff.
    pub(crate) fn searched_quiets(&self) -> &[(Move, Piece)] {
        self.searched_quiets.as_slice()
    }

    /// Returns the next best move in staged order, or `None` when exhausted.
    #[allow(clippy::expect_used)]
    pub(crate) fn next(&mut self, board: &Board, history_table: &HistoryTable) -> Option<Move> {
        loop {
            match self.stage {
                Stage::TtMove => {
                    self.stage = Stage::ScoreTacticals;
                    if let Some(tt_mv) = self.tt_move
                        && self.remove_from_partition(tt_mv)
                    {
                        // Track as a searched quiet if this is a quiet move.
                        if !is_tactical(board, tt_mv) && !tt_mv.is_promotion() {
                            let piece = board
                                .piece_on_square(tt_mv.from())
                                .map(|(pc, _)| pc)
                                .expect("TT move from-square must have a piece");
                            let _ = self.searched_quiets.try_push((tt_mv, piece));
                        }
                        self.moves_yielded += 1;
                        return Some(tt_mv);
                    }
                    // TT move not in legal list (or no TT move); fall through.
                    continue;
                }

                Stage::ScoreTacticals => {
                    for i in 0..self.tactical_end {
                        let mv = self.moves.as_slice()[i];
                        let piece = board
                            .piece_on_square(mv.from())
                            .map(|(pc, _)| pc)
                            .expect("Move from-square must have a piece");
                        // Queen push-promotions have no captured piece; score them as
                        // if capturing a pawn with a pawn (lowest MVV-LVA), which still
                        // places them above all quiets.
                        let victim = board.captured(&mv).unwrap_or(Piece::Pawn);
                        self.scores[i] = Evaluation::<ByteKnightValues>::mvv_lva(victim, piece);
                    }
                    self.stage = Stage::YieldTacticals;
                    continue;
                }

                Stage::YieldTacticals => {
                    if self.pick_index < self.tactical_end {
                        let mv = self.selection_sort_pick(self.tactical_end);
                        self.moves_yielded += 1;
                        return Some(mv);
                    }
                    self.stage = Stage::ScoreQuiets;
                    continue;
                }

                Stage::ScoreQuiets => {
                    let stm = board.side_to_move();
                    for i in self.tactical_end..self.quiet_end {
                        let mv = self.moves.as_slice()[i];
                        let piece = board
                            .piece_on_square(mv.from())
                            .map(|(pc, _)| pc)
                            .expect("Move from-square must have a piece");
                        let is_killer = self
                            .killers
                            .iter()
                            .any(|k| k.is_some_and(|k| k.matches(mv, piece)));
                        self.scores[i] = if is_killer {
                            KILLER_BONUS
                        } else {
                            history_table.get(stm, piece, mv.to())
                        };
                    }
                    self.stage = Stage::YieldQuiets;
                    continue;
                }

                Stage::YieldQuiets => {
                    if self.pick_index < self.quiet_end {
                        let mv = self.selection_sort_pick(self.quiet_end);
                        let piece = board
                            .piece_on_square(mv.from())
                            .map(|(pc, _)| pc)
                            .expect("Move from-square must have a piece");
                        // Only track truly quiet moves (not underpromotions) for history penalty.
                        if !mv.is_promotion() {
                            let _ = self.searched_quiets.try_push((mv, piece));
                        }
                        self.moves_yielded += 1;
                        return Some(mv);
                    }
                    self.stage = Stage::Done;
                    continue;
                }

                Stage::Done => return None,
            }
        }
    }

    /// Selection sort: find the highest-scored move in `moves[pick_index..range_end]`,
    /// swap it to `pick_index`, advance `pick_index`, and return the move.
    fn selection_sort_pick(&mut self, range_end: usize) -> Move {
        let mut best_idx = self.pick_index;
        for i in (self.pick_index + 1)..range_end {
            if self.scores[i] > self.scores[best_idx] {
                best_idx = i;
            }
        }

        if best_idx != self.pick_index {
            self.scores.swap(self.pick_index, best_idx);
            self.moves.as_mut_slice().swap(self.pick_index, best_idx);
        }

        let mv = self.moves.as_slice()[self.pick_index];
        self.pick_index += 1;
        mv
    }

    /// Finds `target` in the partition and removes it by adjusting the range boundaries.
    /// Returns `true` if the move was found and removed.
    ///
    /// When found in the tactical range, the move is swapped out of both the tactical
    /// and quiet ranges to keep the boundaries consistent.
    fn remove_from_partition(&mut self, target: Move) -> bool {
        // Search tacticals [0..tactical_end).
        for i in 0..self.tactical_end {
            if self.moves.as_slice()[i] == target {
                let last_tactical = self.tactical_end - 1;
                self.moves.as_mut_slice().swap(i, last_tactical);
                // The target is now at index `last_tactical` (= new tactical_end after decrement).
                self.tactical_end -= 1;
                // Swap it out of the quiet range (which now begins at tactical_end).
                let last_quiet = self.quiet_end - 1;
                self.moves.as_mut_slice().swap(self.tactical_end, last_quiet);
                self.quiet_end -= 1;
                return true;
            }
        }

        // Search quiets [tactical_end..quiet_end).
        for i in self.tactical_end..self.quiet_end {
            if self.moves.as_slice()[i] == target {
                let last_quiet = self.quiet_end - 1;
                self.moves.as_mut_slice().swap(i, last_quiet);
                self.quiet_end -= 1;
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use chess::{board::Board, move_generation};

    use crate::{
        history_table::HistoryTable,
        killers_table::KillerMovesTable,
        move_picker::MovePicker,
        score::Score,
        ttable::{EntryFlag, TranspositionTable},
    };

    fn piece_for_move(board: &Board, mv: &chess::moves::Move) -> chess::pieces::Piece {
        board
            .piece_on_square(mv.from())
            .map(|(pc, _)| pc)
            .expect("From piece must exist")
    }

    fn collect_all(
        picker: &mut MovePicker,
        board: &Board,
        history: &HistoryTable,
    ) -> Vec<chess::moves::Move> {
        let mut out = Vec::new();
        while let Some(mv) = picker.next(board, history) {
            out.push(mv);
        }
        out
    }

    // ---- helpers ----

    /// FEN with multiple captures available:
    ///   White queen on d5 can capture black rook on d8 (QxR).
    ///   White pawn on e5 can capture black pawn on d6 (PxP).
    ///   Many quiet moves also available.
    const CAPTURES_FEN: &str = "3r4/8/3p4/3QP3/8/8/8/4K1k1 w - - 0 1";

    /// Starting position — no captures, no promotions.
    const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    /// Position with captures of different values:
    ///   White rook on a1, white pawn on e4, black queen on d5, black pawn on d6.
    ///   Pawn can capture queen (PxQ) and pawn (PxP); rook can capture queen (RxQ).
    ///   Multiple captures with different MVV-LVA values.
    const MULTI_CAPTURE_FEN: &str = "8/8/3p4/3q4/3PP3/8/8/R3K1k1 w - - 0 1";

    // ---- tests ----

    #[test]
    fn tt_move_comes_first() {
        let board = Board::from_fen(STARTING_FEN).unwrap();
        let mut tt = TranspositionTable::from_capacity(16);
        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();

        let all_moves = move_generation::legal::generate_all_moves(&board);
        let chosen = *all_moves.at(5).unwrap();

        tt.store_entry(
            board.zobrist_hash(),
            3,
            Score::new(100),
            EntryFlag::Exact,
            chosen,
        );

        let tt_entry = tt.get_entry(board.zobrist_hash()).unwrap();
        let tt_move = Some(tt_entry.board_move);

        let mut picker = MovePicker::new(&board, tt_move, &killers, 0);
        let first = picker.next(&board, &history).expect("must have a move");
        assert_eq!(first, chosen, "TT move must be yielded first");
    }

    #[test]
    fn tacticals_before_quiets() {
        let board = Board::from_fen(CAPTURES_FEN).unwrap();
        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();
        let mut picker = MovePicker::new(&board, None, &killers, 0);

        let mut seen_quiet = false;
        while let Some(mv) = picker.next(&board, &history) {
            let is_capture = board.captured(&mv).is_some();
            let is_queen_promo = mv.flag() == chess::moves::MoveFlag::PromotionQueen;
            let is_tactical = is_capture || is_queen_promo;
            if seen_quiet {
                assert!(
                    !is_tactical,
                    "tactical move {:?} yielded after a quiet move",
                    mv.to_long_algebraic()
                );
            }
            if !is_tactical {
                seen_quiet = true;
            }
        }
    }

    #[test]
    fn mvv_lva_ordering_within_tacticals() {
        // PxQ (pawn captures queen) has higher MVV-LVA than PxP (pawn captures pawn),
        // and RxQ (rook captures queen) also scores highly.
        // Expected MVV-LVA ordering (highest first):
        //   PxQ (victim=queen, attacker=pawn): 25*5 - 1 = 124
        //   RxQ (victim=queen, attacker=rook): 25*5 - 4 = 121
        //   PxP (victim=pawn, attacker=pawn):  25*1 - 1 = 24
        let board = Board::from_fen(MULTI_CAPTURE_FEN).unwrap();
        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();
        let mut picker = MovePicker::new(&board, None, &killers, 0);

        let mut captures: Vec<chess::moves::Move> = Vec::new();
        while let Some(mv) = picker.next(&board, &history) {
            if board.captured(&mv).is_some() {
                captures.push(mv);
            } else {
                // First quiet signals end of tactical stage
                break;
            }
        }

        assert!(!captures.is_empty(), "Expected at least one capture");
        // Verify that each capture is at least as valuable as the next one (descending order)
        for pair in captures.windows(2) {
            let a = pair[0];
            let b = pair[1];
            let victim_a = board.captured(&a).unwrap();
            let piece_a = piece_for_move(&board, &a);
            let victim_b = board.captured(&b).unwrap();
            let piece_b = piece_for_move(&board, &b);
            let score_a = crate::evaluation::Evaluation::<crate::hce_values::ByteKnightValues>::mvv_lva(victim_a, piece_a);
            let score_b = crate::evaluation::Evaluation::<crate::hce_values::ByteKnightValues>::mvv_lva(victim_b, piece_b);
            assert!(
                score_a >= score_b,
                "Captures not in MVV-LVA order: {:?} ({score_a}) before {:?} ({score_b})",
                a.to_long_algebraic(),
                b.to_long_algebraic()
            );
        }
    }

    #[test]
    fn killers_sort_to_top_of_quiets() {
        let board = Board::from_fen(STARTING_FEN).unwrap();
        let history = HistoryTable::new();
        let mut killers = KillerMovesTable::new();

        let all_moves = move_generation::legal::generate_all_moves(&board);
        // Pick two quiet moves as killers (last two in the list)
        let n = all_moves.len();
        let killer_mv = *all_moves.at(n - 1).unwrap();
        let killer_piece = piece_for_move(&board, &killer_mv);
        killers.update(0, killer_mv, killer_piece);

        let mut picker = MovePicker::new(&board, None, &killers, 0);
        // Starting position has no captures; all moves are quiet.
        let moves = collect_all(&mut picker, &board, &history);

        // The killer should be the first quiet move yielded.
        assert_eq!(
            moves[0], killer_mv,
            "Killer move must be first among quiets"
        );
    }

    #[test]
    fn history_ordering_among_quiets() {
        let board = Board::from_fen(STARTING_FEN).unwrap();
        let mut history = HistoryTable::new();
        let killers = KillerMovesTable::new();

        let all_moves = move_generation::legal::generate_all_moves(&board);
        // Give a high history score to the move at index 3
        let favored_mv = *all_moves.at(3).unwrap();
        let favored_piece = piece_for_move(&board, &favored_mv);
        history.update(board.side_to_move(), favored_piece, favored_mv.to(), Score::MAX_HISTORY);

        let mut picker = MovePicker::new(&board, None, &killers, 0);
        let moves = collect_all(&mut picker, &board, &history);

        // The favored move should be yielded first (highest history score).
        assert_eq!(
            moves[0], favored_mv,
            "Highest-history move must come first among quiets"
        );
    }

    #[test]
    fn tt_move_not_duplicated_when_capture() {
        // Use a position where a capture is available so the TT move is a capture.
        let board = Board::from_fen(CAPTURES_FEN).unwrap();
        let mut tt = TranspositionTable::from_capacity(16);
        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();

        // Find a capture move to use as TT move.
        let all_moves = move_generation::legal::generate_all_moves(&board);
        let capture_mv = all_moves
            .iter()
            .find(|mv| board.captured(mv).is_some())
            .copied()
            .expect("Need a capture move");

        tt.store_entry(
            board.zobrist_hash(),
            3,
            Score::new(500),
            EntryFlag::Exact,
            capture_mv,
        );

        let tt_entry = tt.get_entry(board.zobrist_hash()).unwrap();
        let tt_move = Some(tt_entry.board_move);
        let mut picker = MovePicker::new(&board, tt_move, &killers, 0);
        let moves = collect_all(&mut picker, &board, &history);

        let count = moves.iter().filter(|&&m| m == capture_mv).count();
        assert_eq!(count, 1, "TT capture move must appear exactly once");
    }

    #[test]
    fn new_qsearch_yields_only_tacticals_when_not_in_check() {
        let board = Board::from_fen(CAPTURES_FEN).unwrap();
        let history = HistoryTable::new();
        let mut picker = MovePicker::new_qsearch(&board, None, false);

        while let Some(mv) = picker.next(&board, &history) {
            let is_capture = board.captured(&mv).is_some();
            let is_queen_promo = mv.flag() == chess::moves::MoveFlag::PromotionQueen;
            assert!(
                is_capture || is_queen_promo,
                "QSearch (not in check) yielded a non-tactical move: {}",
                mv.to_long_algebraic()
            );
        }
    }

    #[test]
    fn is_empty_when_no_tacticals_in_qsearch() {
        // Starting position has no captures — qsearch picker should be empty.
        let board = Board::from_fen(STARTING_FEN).unwrap();
        let picker = MovePicker::new_qsearch(&board, None, false);
        assert!(picker.is_empty(), "QSearch picker must be empty with no captures");
    }

    #[test]
    fn moves_yielded_matches_loop_counter() {
        let board = Board::from_fen(STARTING_FEN).unwrap();
        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();
        let mut picker = MovePicker::new(&board, None, &killers, 0);

        let mut expected_counter = 0usize;
        while let Some(_mv) = picker.next(&board, &history) {
            let loop_counter = picker.moves_yielded() - 1;
            assert_eq!(loop_counter, expected_counter);
            expected_counter += 1;
        }
        assert_eq!(picker.moves_yielded(), expected_counter);
    }
}
