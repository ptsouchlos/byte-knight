// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use arrayvec::ArrayVec;
use chess::{
    board::Board,
    definitions::MAX_MOVE_LIST_SIZE,
    move_generation::{
        self, legal::generate_moves_with_metadata, metadata::CheckPinMetadata,
        move_filter::MoveFilter,
    },
    moves::Move,
    pieces::Piece,
};

use crate::{
    evaluation::Evaluation,
    hce_values::ByteKnightValues,
    history_table::HistoryTable,
    killers_table::{KillerEntry, KillerMovesTable},
    score::{LargeScoreType, Score},
    scored_move::ScoredMove,
    scored_move_list::ScoredMoveList,
};

/// Bonus applied to killer moves in the quiet scoring stage so they sort
/// above all history-scored quiets (history is clamped to MAX_HISTORY).
const KILLER_BONUS: LargeScoreType = Score::MAX_HISTORY + 1;

/// Score tier for queen capture-promotions — above queen push-promos.
const QUEEN_CAPTURE_PROMO_BONUS: LargeScoreType = 30_000;
/// Score tier for queen push-promotions — above all regular captures.
/// Max MVV-LVA (PxQ) is 124, so 20_000 comfortably clears it.
const QUEEN_PUSH_PROMO_BONUS: LargeScoreType = 20_000;

/// MVV-LVA score for move ordering without the `<< 16` shift used by
/// `Evaluation::mvv_lva`. Range: 20..=124 for non-king captures.
fn mvv_lva(victim: Piece, attacker: Piece) -> LargeScoreType {
    let can_capture = victim != Piece::King;
    (can_capture as LargeScoreType)
        * (25 * Evaluation::<ByteKnightValues>::piece_value(victim)
            - Evaluation::<ByteKnightValues>::piece_value(attacker))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    TtMove,
    GenerateTacticals,
    Tacticals,
    GenerateQuiets,
    Quiets,
    Done,
}

/// A staged move picker that yields moves in best-first order.
///
/// Main search stages:
///   `TtMove → GenerateTacticals → ScoreTacticals → YieldTacticals → GenerateQuiets → ScoreQuiets → YieldQuiets → Done`
///
/// For quiescence search (`new_qsearch`), the quiet stages are skipped when not
/// in check (`skip_quiets = true`).
///
/// Move generation is lazy: tacticals are generated only when entering
/// `GenerateTacticals`, and quiets are generated only when entering
/// `GenerateQuiets`. If a beta cutoff occurs on a tactical, quiets are
/// never generated.
pub(crate) struct MovePicker {
    stage: Stage,
    moves: ScoredMoveList,
    tt_move: Option<Move>,
    /// True after the TT move has been yielded, so yield stages can skip it.
    tt_move_yielded: bool,
    killers: [Option<KillerEntry>; 2],
    /// Cached check/pin metadata computed in GenerateTacticals; reused in GenerateQuiets.
    metadata: Option<CheckPinMetadata>,
    /// Monotonically increasing selection-sort cursor (absolute index).
    pick_index: usize,
    /// Total moves yielded so far. Caller uses `moves_yielded() - 1` as the loop counter.
    moves_yielded: usize,
    /// Quiet (non-promotion, non-capture) moves yielded so far, with their moving piece.
    /// Used by the caller for history penalty on a beta cutoff.
    searched_quiets: ArrayVec<(Move, Piece), MAX_MOVE_LIST_SIZE>,
    /// When true (qsearch not-in-check), skip GenerateQuiets and go directly to Done.
    skip_quiets: bool,
}

impl MovePicker {
    /// Creates a `MovePicker` for the main negamax search.
    ///
    /// No moves are generated upfront; generation is deferred to the stage machine.
    pub(crate) fn new(tt_move: Option<Move>, killers_table: &KillerMovesTable, ply: u8) -> Self {
        let killer_slice = killers_table.get(ply);
        let killers = [
            killer_slice.first().copied().unwrap_or(None),
            killer_slice.get(1).copied().unwrap_or(None),
        ];

        Self {
            stage: if tt_move.is_some() {
                Stage::TtMove
            } else {
                Stage::GenerateTacticals
            },
            moves: ScoredMoveList::new(),
            tt_move,
            tt_move_yielded: false,
            killers,
            metadata: None,
            pick_index: 0,
            moves_yielded: 0,
            searched_quiets: ArrayVec::new(),
            skip_quiets: false,
        }
    }

    /// Creates a `MovePicker` for quiescence search.
    ///
    /// When not in check, quiet generation is skipped entirely. When in check,
    /// all moves including quiets are generated (king evasions may be quiet).
    pub(crate) fn new_qsearch(tt_move: Option<Move>, in_check: bool) -> Self {
        Self {
            stage: if tt_move.is_some() {
                Stage::TtMove
            } else {
                Stage::GenerateTacticals
            },
            moves: ScoredMoveList::new(),
            tt_move,
            tt_move_yielded: false,
            killers: [None; 2],
            metadata: None,
            pick_index: 0,
            moves_yielded: 0,
            searched_quiets: ArrayVec::new(),
            skip_quiets: !in_check,
        }
    }

    /// Returns true if the side to move is in check.
    ///
    /// Only valid after `GenerateTacticals` has run (i.e., after `next()` has been
    /// called at least once past the TT move stage).
    pub(crate) fn in_check(&self) -> bool {
        self.metadata.as_ref().is_some_and(|m| m.in_check())
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

    fn score_move(&self, board: &Board, mv: &Move, history_table: &HistoryTable) -> LargeScoreType {
        let maybe_victim = board.captured(mv);
        if let Some(victim) = maybe_victim {
            let piece = board
                .piece_on_square(mv.from())
                .map(|(pc, _)| pc)
                .expect("Move from-square must have a piece");

            let base = if mv.is_promote_to_queen() {
                QUEEN_CAPTURE_PROMO_BONUS
            } else {
                0
            };

            base + mvv_lva(victim, piece)
        } else if mv.is_promotion() {
            if mv.is_promote_to_queen() {
                QUEEN_PUSH_PROMO_BONUS
            } else {
                Evaluation::<ByteKnightValues>::piece_value(mv.promotion_piece().unwrap())
            }
        } else {
            let piece = board
                .piece_on_square(mv.from())
                .map(|(pc, _)| pc)
                .expect("Move from-square must have a piece");
            let is_killer = self
                .killers
                .iter()
                .any(|k| k.is_some_and(|k| k.matches(*mv, piece)));
            if is_killer {
                KILLER_BONUS
            } else {
                history_table.get(board.side_to_move(), piece, mv.to())
            }
        }
    }

    /// Returns the next best move in staged order, or `None` when exhausted.
    #[allow(clippy::expect_used, clippy::panic)]
    pub(crate) fn next(&mut self, board: &Board, history_table: &HistoryTable) -> Option<Move> {
        if self.stage == Stage::TtMove {
            self.stage = Stage::GenerateTacticals;
            // Compute metadata now so it can be reused by GenerateTacticals/GenerateQuiets.
            let meta = move_generation::metadata::compute(board);
            let tt_legal = self.tt_move.is_some_and(|tt_mv| {
                chess::legal::is_pseudo_legal(board, &tt_mv)
                    && chess::legal::is_legal_with_metadata(board, &tt_mv, &meta)
            });
            self.metadata = Some(meta);
            if let Some(tt_mv) = self.tt_move.filter(|_| tt_legal) {
                // Track as a searched quiet if this is a quiet move.
                if board.captured(&tt_mv).is_none() && !tt_mv.is_promotion() {
                    let piece = board
                        .piece_on_square(tt_mv.from())
                        .map(|(pc, _)| pc)
                        .expect("TT move from-square must have a piece");
                    let _ = self.searched_quiets.try_push((tt_mv, piece));
                }
                self.tt_move_yielded = true;
                self.moves_yielded += 1;
                return Some(tt_mv);
            }
            // TT move not legal (or no TT move); fall through.
        }

        if self.stage == Stage::GenerateTacticals {
            self.pick_index = 0;
            // Reuse metadata computed in TtMove stage if available.
            let meta = self
                .metadata
                .get_or_insert_with(|| move_generation::metadata::compute(board))
                .clone();

            let moves = generate_moves_with_metadata(board, MoveFilter::Tacticals, &meta);
            self.moves.clear();

            for mv in moves.iter() {
                // TODO: Score move, then push it to the correct list depending on SEE
                self.moves.push(ScoredMove {
                    mv: *mv,
                    score: self.score_move(board, mv, history_table),
                });
            }
            self.stage = Stage::Tacticals;
        }

        if self.stage == Stage::Tacticals {
            while self.pick_index < self.moves.len() {
                let mv = self.selection_sort_pick();
                // Skip the TT move if it was already yielded.
                if self.tt_move_yielded && self.tt_move == Some(mv) {
                    continue;
                }
                self.moves_yielded += 1;
                return Some(mv);
            }
            self.stage = Stage::GenerateQuiets;
        }

        if self.stage == Stage::GenerateQuiets {
            self.pick_index = 0;
            if self.skip_quiets {
                self.stage = Stage::Done;
            } else {
                // Reuse cached metadata to avoid recomputing.
                let meta = self
                    .metadata
                    .as_ref()
                    .expect("metadata must be set after GenerateTacticals");
                let moves = generate_moves_with_metadata(board, MoveFilter::Quiets, meta);
                self.moves.clear();

                for mv in moves.iter() {
                    self.moves.push(ScoredMove {
                        mv: *mv,
                        score: self.score_move(board, mv, history_table),
                    });
                }

                self.stage = Stage::Quiets;
            }
        }

        if self.stage == Stage::Quiets {
            while self.pick_index < self.moves.len() {
                let mv = self.selection_sort_pick();
                // Skip the TT move if it was already yielded.
                if self.tt_move_yielded && self.tt_move == Some(mv) {
                    continue;
                }
                let piece = board
                    .piece_on_square(mv.from())
                    .map(|(pc, _)| pc)
                    .unwrap_or_else(|| {
                        panic!(
                            "Move from-square must have a piece: {} {}",
                            board.to_fen(),
                            mv.to_long_algebraic()
                        )
                    });
                // Only track truly quiet moves (not underpromotions) for history penalty.
                if !mv.is_promotion() {
                    let _ = self.searched_quiets.try_push((mv, piece));
                }
                self.moves_yielded += 1;
                return Some(mv);
            }
            self.stage = Stage::Done;
        }

        None
    }

    /// Selection sort: find the highest-scored move in `moves[pick_index..range_end]`,
    /// swap it to `pick_index`, advance `pick_index`, and return the move.
    fn selection_sort_pick(&mut self) -> Move {
        let mut best_idx = self.pick_index;
        for i in (self.pick_index + 1)..self.moves.len() {
            if self.moves.as_slice()[i].score > self.moves.as_slice()[best_idx].score {
                best_idx = i;
            }
        }

        if best_idx != self.pick_index {
            self.moves.as_mut_slice().swap(self.pick_index, best_idx);
        }

        let scored_mv = self.moves.as_slice()[self.pick_index];
        self.pick_index += 1;
        scored_mv.mv
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use chess::{board::Board, move_generation, move_generation::move_filter::MoveFilter};

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

        let mut picker = MovePicker::new(tt_move, &killers, 0);
        let first = picker.next(&board, &history).expect("must have a move");
        assert_eq!(first, chosen, "TT move must be yielded first");
    }

    #[test]
    fn tacticals_before_quiets() {
        let board = Board::from_fen(CAPTURES_FEN).unwrap();
        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();
        let mut picker = MovePicker::new(None, &killers, 0);

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
        let mut picker = MovePicker::new(None, &killers, 0);

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
            let score_a = super::mvv_lva(victim_a, piece_a);
            let score_b = super::mvv_lva(victim_b, piece_b);
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

        let mut picker = MovePicker::new(None, &killers, 0);
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
        history.update(
            board.side_to_move(),
            favored_piece,
            favored_mv.to(),
            Score::MAX_HISTORY,
        );

        let mut picker = MovePicker::new(None, &killers, 0);
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
        let mut picker = MovePicker::new(tt_move, &killers, 0);
        let moves = collect_all(&mut picker, &board, &history);

        let count = moves.iter().filter(|&&m| m == capture_mv).count();
        assert_eq!(count, 1, "TT capture move must appear exactly once");
    }

    #[test]
    fn new_qsearch_yields_only_tacticals_when_not_in_check() {
        let board = Board::from_fen(CAPTURES_FEN).unwrap();
        let history = HistoryTable::new();
        let mut picker = MovePicker::new_qsearch(None, false);

        while let Some(mv) = picker.next(&board, &history) {
            let is_capture = board.captured(&mv).is_some();
            let is_queen_promo = mv.flag() == chess::moves::MoveFlag::PromotionQueen;
            assert!(
                is_capture || is_queen_promo,
                "QSearch (not in check) yielded a non-tactical move: {}\n{}\n{}",
                mv.to_long_algebraic(),
                CAPTURES_FEN,
                board
            );
        }
    }

    #[test]
    fn no_moves_yielded_when_no_tacticals_in_qsearch() {
        // Starting position has no captures — qsearch picker should yield nothing.
        let board = Board::from_fen(STARTING_FEN).unwrap();
        let history = HistoryTable::new();
        let mut picker = MovePicker::new_qsearch(None, false);
        assert_eq!(
            picker.moves_yielded(),
            0,
            "QSearch picker must yield 0 moves when no captures available"
        );
        // Exhaust the picker
        while picker.next(&board, &history).is_some() {}
        assert_eq!(
            picker.moves_yielded(),
            0,
            "QSearch picker must yield 0 moves with no captures"
        );
    }

    #[test]
    fn moves_yielded_matches_loop_counter() {
        let board = Board::from_fen(STARTING_FEN).unwrap();
        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();
        let mut picker = MovePicker::new(None, &killers, 0);

        let mut expected_counter = 0usize;
        while let Some(_mv) = picker.next(&board, &history) {
            let loop_counter = picker.moves_yielded() - 1;
            assert_eq!(loop_counter, expected_counter);
            expected_counter += 1;
        }
        assert_eq!(picker.moves_yielded(), expected_counter);
    }

    #[test]
    fn no_queen_push_promo_duplicates() {
        // Position with a pawn that can queen-promote (push, no capture available).
        // The queen push-promo should appear exactly once across all yielded moves.
        let board = Board::from_fen("8/P3k3/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();
        let mut picker = MovePicker::new(None, &killers, 0);
        let moves = collect_all(&mut picker, &board, &history);

        let queen_push_promos: Vec<_> = moves
            .iter()
            .filter(|mv| {
                mv.flag() == chess::moves::MoveFlag::PromotionQueen && board.captured(mv).is_none()
            })
            .collect();

        assert_eq!(
            queen_push_promos.len(),
            1,
            "Queen push-promotion must appear exactly once, found {}",
            queen_push_promos.len()
        );
    }

    #[test]
    fn lazy_gen_produces_same_moves_as_eager_gen() {
        let positions = [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            // check position
            "4k3/8/8/8/8/8/4r3/4K3 w - - 0 1",
            // push-promotion position
            "8/P3k3/8/8/8/8/8/4K3 w - - 0 1",
        ];

        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();

        for fen in &positions {
            let board = Board::from_fen(fen).unwrap();
            println!("{}\n{}", board.to_fen(), board);
            // Eager: generate all moves at once
            let eager_moves: Vec<_> =
                move_generation::legal::generate_moves(&board, MoveFilter::All)
                    .iter()
                    .cloned()
                    .collect();

            // Lazy: collect via picker
            let mut picker = MovePicker::new(None, &killers, 0);
            let lazy_moves = collect_all(&mut picker, &board, &history);
            assert_eq!(
                lazy_moves.len(),
                picker.moves_yielded(),
                "Moves yielded count doesn't match: list={}, picker={}",
                lazy_moves.len(),
                picker.moves_yielded()
            );
            assert_eq!(
                eager_moves.len(),
                lazy_moves.len(),
                "Move count mismatch for {fen}: eager={}, lazy={}",
                eager_moves.len(),
                lazy_moves.len()
            );
            for mv in &eager_moves {
                assert!(
                    lazy_moves.iter().any(|m| m == mv),
                    "Move {} from eager gen not in lazy gen for {fen}",
                    mv.to_long_algebraic()
                );
            }
        }
    }

    #[test]
    fn in_check_returns_correct_value() {
        // Position where white king is in check
        let board_in_check = Board::from_fen("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1").unwrap();
        let history = HistoryTable::new();
        let killers = KillerMovesTable::new();
        let mut picker = MovePicker::new(None, &killers, 0);
        // Drive past TtMove stage by calling next() once
        let _ = picker.next(&board_in_check, &history);
        assert!(picker.in_check(), "in_check() should return true");

        // Position where white king is NOT in check
        let board_safe = Board::from_fen(STARTING_FEN).unwrap();
        let mut picker2 = MovePicker::new(None, &killers, 0);
        let _ = picker2.next(&board_safe, &history);
        assert!(!picker2.in_check(), "in_check() should return false");
    }
}
