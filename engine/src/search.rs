// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    fmt::Display,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::{Result, bail};
use arrayvec::ArrayVec;
use chess::{
    board::Board, definitions::MAX_MOVE_LIST_SIZE, move_generation::MoveGenerator,
    move_list::MoveList, moves::Move, pieces::Piece,
};
use uci_parser::{UciInfo, UciResponse, UciSearchOptions};

use crate::{
    aspiration_window::AspirationWindow,
    defs::MAX_DEPTH,
    evaluation::ByteKnightEvaluation,
    history_table::{self, HistoryTable},
    inplace_incremental_sort::InplaceIncrementalSort,
    lmr,
    log_level::LogLevel,
    move_order::MoveOrder,
    node_types::{NodeType, NonPvNode, PvNode, RootNode},
    principle_variation::PrincipleVariation,
    score::{LargeScoreType, Score, ScoreType},
    table::Table,
    traits::Eval,
    ttable::{self, TranspositionTableEntry},
    tuneable::{
        IIR_DEPTH_REDUCTION, IIR_MIN_DEPTH, LMP_MIN_THRESHOLD_DEPTH, MAX_RFP_DEPTH,
        NMP_DEPTH_REDUCTION, NMP_MIN_DEPTH, RFP_MARGIN,
    },
};
use ttable::TranspositionTable;

/// Result for a search.
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub score: Score,
    pub best_move: Option<Move>,
    pub nodes: u64,
    pub depth: u8,
    pub pv: PrincipleVariation,
}

impl Default for SearchResult {
    fn default() -> SearchResult {
        SearchResult {
            score: -Score::INF,
            best_move: None,
            nodes: 0,
            depth: 1,
            pv: PrincipleVariation::new(),
        }
    }
}

impl Display for SearchResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "score {} nodes {} depth {} bestmove {}",
            self.score,
            self.nodes,
            self.depth,
            self.best_move
                .map(|m| m.to_long_algebraic())
                .unwrap_or_else(|| "none".to_string())
        )
    }
}

/// Input parameters for the search.
#[derive(Clone, Debug)]
pub struct SearchParameters {
    pub max_depth: u8,
    pub start_time: Instant,
    pub soft_timeout: Duration,
    pub hard_timeout: Duration,
    pub max_nodes: u64,
}

impl Default for SearchParameters {
    fn default() -> Self {
        SearchParameters {
            max_depth: MAX_DEPTH,
            start_time: Instant::now(),
            soft_timeout: Duration::MAX,
            hard_timeout: Duration::MAX,
            max_nodes: u64::MAX,
        }
    }
}

impl SearchParameters {
    /// Creates a new set of search parameters from the UCI options and the current board.
    pub fn new(uci_options: &UciSearchOptions, board: &Board) -> Self {
        let mut params = Self::default();
        if let Some(depth) = uci_options.depth {
            params.max_depth = depth as u8;
        }

        if let Some(nodes) = uci_options.nodes {
            params.max_nodes = nodes as u64;
        }

        if let Some(time) = uci_options.movetime {
            params.soft_timeout = time;
            params.hard_timeout = time;
        } else {
            let (time, increment) = if board.side_to_move().is_white() {
                (uci_options.wtime, uci_options.winc)
            } else {
                (uci_options.btime, uci_options.binc)
            };

            // do we have valid time
            if let Some(time) = time {
                // TODO: How can we tune these params?
                let inc = increment.unwrap_or(Duration::ZERO) / 2;
                params.soft_timeout = time / 20 + inc;
                params.hard_timeout = time / 5 + inc;
            }
        }

        params
    }
}

impl Display for SearchParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "max depth {} start_time {:?} soft_timeout {:?} hard_timeout {:?}",
            self.max_depth, self.start_time, self.soft_timeout, self.hard_timeout
        )
    }
}

pub struct Search<'search_lifetime, Log> {
    transposition_table: &'search_lifetime mut TranspositionTable,
    history_table: &'search_lifetime mut HistoryTable,
    move_gen: MoveGenerator,
    nodes: u64,
    parameters: SearchParameters,
    eval: ByteKnightEvaluation,
    stop_flag: Option<Arc<AtomicBool>>,
    lmr_table: Table<f64, 32_000>,
    /// Marker for the level of logging to print.
    log: PhantomData<Log>,
}

impl<'a, Log: LogLevel> Search<'a, Log> {
    pub fn new(
        parameters: &SearchParameters,
        ttable: &'a mut TranspositionTable,
        history_table: &'a mut HistoryTable,
    ) -> Self {
        // Initialize our LMR table as a 2D array of our LMR formula for depth and moves played
        let mut table = Table::<f64, 32_000>::new(MAX_DEPTH as usize, MAX_MOVE_LIST_SIZE);
        table.fill(lmr::formula);

        Self {
            transposition_table: ttable,
            history_table,
            move_gen: MoveGenerator::new(),
            nodes: 0,
            parameters: parameters.clone(),
            eval: ByteKnightEvaluation::default(),
            stop_flag: None,
            lmr_table: table,
            log: PhantomData,
        }
    }

    /// Search for the best move in the given board state. This will output
    /// UCI info lines as it searches.
    ///
    /// # Arguments
    ///
    /// - `board` - The current board state.
    /// - `stop_flag` - An optional flag to stop the search.
    ///
    /// # Returns
    ///
    /// The best move found.
    pub fn search(
        &mut self,
        board: &mut Board,
        stop_flag: Option<Arc<AtomicBool>>,
    ) -> SearchResult {
        self.stop_flag = stop_flag;

        if Log::DEBUG {
            let info = UciInfo::default().string(format!("searching {}", self.parameters));
            let message = UciResponse::info(info);
            println!("{message}");
        }

        let result = self.iterative_deepening(board);
        // search ended, reset our node count
        self.nodes = 0;
        result
    }

    fn should_stop_searching(&self) -> bool {
        self.parameters.start_time.elapsed() >= self.parameters.hard_timeout // hard timeout
            || self.nodes >= self.parameters.max_nodes // node limit reached
            || self.stop_flag.as_ref().is_some_and(|f| f.load(Ordering::Relaxed))
        // stop flag set
    }

    fn send_info(
        &self,
        depth: u8,
        nodes: u64,
        score: Score,
        nps: f32,
        time: u64,
        pv: &PrincipleVariation,
    ) {
        // create UciInfo and print it
        let info = UciInfo::new()
            .depth(depth)
            .nodes(nodes)
            .score(score)
            .nps(nps.trunc())
            .time(time)
            .pv(pv.iter().map(|m| m.to_long_algebraic()));
        let message = UciResponse::info(info);
        println!("{message}");
    }

    /// Verify that a given [PrincipleVariation] is valid. This is expensive and should only be used for debugging.
    #[allow(clippy::expect_used)]
    fn verify_pv_moves(&self, pv: &PrincipleVariation, board: &Board) -> Result<()> {
        let mut board_cpy = board.clone();
        let all_ok = pv.iter().all(|mv| {
            let mv_ok = board_cpy.make_move(mv, &self.move_gen);
            mv_ok.is_ok()
        });
        if !all_ok {
            bail!("PV is invalid!")
        }

        Ok(())
    }

    fn iterative_deepening(&mut self, board: &mut Board) -> SearchResult {
        // initialize the best result
        let mut best_result = SearchResult::default();
        let mut move_list = MoveList::new();

        self.move_gen.generate_legal_moves(board, &mut move_list);
        if !move_list.is_empty() {
            best_result.best_move = Some(*move_list.at(0).unwrap())
        }

        'deepening: while self.parameters.start_time.elapsed() <= self.parameters.soft_timeout
            && best_result.depth <= self.parameters.max_depth
            && !self
                .stop_flag
                .as_ref()
                .is_some_and(|f| f.load(Ordering::Relaxed))
        {
            // create an aspiration window around the best result so far
            let mut aspiration_window =
                AspirationWindow::around(best_result.score, best_result.depth as ScoreType);
            let mut pv = PrincipleVariation::new();

            let mut score: Score;
            'aspiration_window: loop {
                // search the tree, starting at the current depth (starts at 1)
                score = self.negamax::<RootNode>(
                    board,
                    best_result.depth as ScoreType,
                    0,
                    aspiration_window.alpha(),
                    aspiration_window.beta(),
                    &mut pv,
                );

                if aspiration_window.failed_low(score) {
                    // fail low, widen the window
                    aspiration_window.widen_down(score, best_result.depth as ScoreType);
                } else if aspiration_window.failed_high(score) {
                    // fail high, widen the window
                    aspiration_window.widen_up(score, best_result.depth as ScoreType);
                } else {
                    // we have a valid score, break the loop
                    break 'aspiration_window;
                }

                // check stop conditions
                if self.should_stop_searching() {
                    // we have to stop searching now, use the best result we have
                    // no score update
                    break 'deepening;
                }
            }

            // update the best result
            best_result.score = score;
            best_result.best_move = self
                .transposition_table
                .get_entry(board.zobrist_hash())
                .map(|e| e.board_move);
            best_result.pv = pv;

            // verify the PV as a sanity check, but only in debug
            debug_assert!(
                self.verify_pv_moves(&best_result.pv, board).is_ok(),
                "PV invalid"
            );

            if Log::INFO {
                // send UCI info
                self.send_info(
                    best_result.depth,
                    self.nodes,
                    best_result.score,
                    (self.nodes as f32 / self.parameters.start_time.elapsed().as_secs_f32())
                        .trunc(),
                    self.parameters.start_time.elapsed().as_millis() as u64,
                    &best_result.pv,
                );
            }

            // increment depth for next iteration
            best_result.depth += 1;
        }

        // update total nodes for the current search
        best_result.nodes = self.nodes;

        if Log::INFO {
            // Send one last info line with the final result
            // send UCI info
            self.send_info(
                best_result.depth,
                self.nodes,
                best_result.score,
                (self.nodes as f32 / self.parameters.start_time.elapsed().as_secs_f32()).trunc(),
                self.parameters.start_time.elapsed().as_millis() as u64,
                &best_result.pv,
            );
        }

        // return our best result so far
        best_result
    }

    fn negamax<Node>(
        &mut self,
        board: &mut Board,
        mut depth: ScoreType,
        ply: ScoreType,
        alpha: Score,
        beta: Score,
        pv: &mut PrincipleVariation,
    ) -> Score
    where
        Node: NodeType,
    {
        // increment node count
        self.nodes += 1;
        let alpha_original = alpha;
        let mut alpha_use = alpha;

        if depth <= 0 {
            return self.quiescence::<Node>(board, alpha, beta, pv);
        }

        let mut local_pv = PrincipleVariation::new();
        // clear the current PV because this is a new position
        pv.clear();

        // Transposition Table Cutoffs: https://www.chessprogramming.org/Transposition_Table#Transposition_Table_Cutoffs
        // Check if we have a transposition table entry and if we can return early
        let tt_move =
            match self
                .transposition_table
                .probe::<Node>(depth, board.zobrist_hash(), alpha, beta)
            {
                ttable::ProbeResult::CutOff(entry) => {
                    // we have a cutoff, so return the score, but only in a non-PV node
                    self.nodes += 1;
                    if !Node::PV {
                        return entry.score;
                    }
                    Some(entry.board_move)
                }
                ttable::ProbeResult::Hit(entry) => Some(entry.board_move),
                ttable::ProbeResult::Empty => None,
            };

        // Internal Iterative Reductions: https://www.chessprogramming.org/Internal_Iterative_Reductions
        // If no tt entry was found, searching it will be very costly, so we reduce the depth. This is
        // working under the assumption that the position is likely not important.
        if tt_move.is_none() && depth >= IIR_MIN_DEPTH {
            depth -= IIR_DEPTH_REDUCTION;
        }

        // can we prune the current node with something other than TT?
        if let Some(score) = self.pruned_score::<Node>(board, depth, ply, beta, &mut local_pv) {
            return score;
        }

        // get all legal moves
        let mut move_list = MoveList::new();
        let mut order_list = ArrayVec::<MoveOrder, MAX_MOVE_LIST_SIZE>::new();
        self.move_gen.generate_legal_moves(board, &mut move_list);

        // do we have moves?
        if move_list.is_empty() {
            return if board.is_in_check(&self.move_gen) {
                -Score::MATE + ply
            } else {
                Score::DRAW
            };
        }

        let classify_res = MoveOrder::classify_all(
            board.side_to_move(),
            move_list.as_slice(),
            &tt_move,
            self.history_table,
            &mut order_list,
        );

        // TODO(PT): Should we log a message to the CLI or a log?
        assert!(classify_res.is_ok());

        // sort moves by MVV/LVA
        let move_iter = InplaceIncrementalSort::new(move_list.as_mut_slice(), &mut order_list);

        // Really "bad" initial score
        let mut best_score = -Score::INF;
        let mut best_move = tt_move;

        // Loop through all moves
        for (i, mv) in move_iter.into_iter().enumerate() {
            // Calculate the LMR reduction and depth which will be used later in FP
            let lmr_table_value = self.lmr_table.at(depth as usize, i);
            let base_reduction = if let Some(table_val) = lmr_table_value {
                *table_val
            } else {
                1f64
            };
            let lmr_reduction = (1f64 + base_reduction).floor() as i16;
            let _lmr_depth = depth.saturating_sub(lmr_reduction);
            let is_in_check = board.is_in_check(&self.move_gen);
            let is_root = Node::ROOT;
            let is_pv = Node::PV;

            // Move-loop pruning techniques

            // LMP - Late Move Pruning
            // We assume our move ordering is just too good, so if we're under a certain depth
            // and have made more than a certain number of moves, we can assume that later moves
            // won't be as good, so we prune them.
            if !is_root && !is_pv && !is_in_check && !best_score.mated() {
                let min_lmp_moves =
                    LMP_MIN_THRESHOLD_DEPTH as usize + depth as usize * depth as usize;
                if i >= min_lmp_moves {
                    break;
                }
            }

            // local PV is for each node below this one is different when we call negamax recursively
            // so we have to clear it
            local_pv.clear();

            // make the move
            board.make_move_unchecked(&mv).unwrap();
            let mut score = Score::DRAW;

            // Don't bother searching drawn positions
            if !board.is_draw() {
                score =
                // Principal Variation Search (PVS)
                if Node::PV && i == 0 {
                    -self.negamax::<PvNode>(board, depth - 1, ply + 1, -beta, -alpha_use, &mut local_pv)
                } else {
                    let reduction = if mv.is_quiet() &&  depth >= 3 && board.full_move_number() >= 3 {
                        lmr_reduction
                    } else {
                        1
                    };
                    // search with a null window
                    let temp_score = -self.negamax::<NonPvNode>(board, depth - reduction, ply + 1, -alpha_use - 1, -alpha_use, &mut local_pv);
                    // if it fails, we need to do a full re-search
                    if temp_score > alpha_use && temp_score < beta {
                        -self.negamax::<NonPvNode>(board, depth - 1, ply + 1, -beta, -alpha_use, &mut local_pv)
                    }
                    else {
                        temp_score
                    }
                };
            }

            // undo the move
            board.unmake_move().unwrap();

            // check the results
            if score > best_score {
                // we improved, so update the score and best move
                best_score = score;
                best_move = Some(mv);
                if Node::PV {
                    pv.extend(mv, &local_pv);
                }

                alpha_use = alpha_use.max(best_score);
                // Did we fail high?
                if alpha_use >= beta {
                    // update history table for quiets
                    if mv.is_quiet() {
                        // calculate history bonus
                        let bonus = history_table::calculate_bonus_for_depth(depth);
                        self.history_table.update(
                            board.side_to_move(),
                            mv.piece(),
                            mv.to(),
                            bonus as LargeScoreType,
                        );

                        // apply a penalty to all quiets searched so far
                        for mv in move_list.iter().take(i).filter(|mv| mv.is_quiet()) {
                            self.history_table.update(
                                board.side_to_move(),
                                mv.piece(),
                                mv.to(),
                                -bonus as LargeScoreType,
                            );
                        }
                    }
                    break;
                }
            }

            // do we need to stop searching?
            if self.should_stop_searching() {
                break;
            }
        }

        // store the best move in the transposition table
        let flag = if best_score <= alpha_original {
            ttable::EntryFlag::UpperBound
        } else if best_score >= beta {
            ttable::EntryFlag::LowerBound
        } else {
            ttable::EntryFlag::Exact
        };

        self.transposition_table
            .store_entry(TranspositionTableEntry::new(
                board.zobrist_hash(),
                depth as u8,
                best_score,
                flag,
                best_move.unwrap(),
            ));

        best_score
    }

    /// Checks to see if the current node can be pruned. If it can, returns the score. Otherwise returns None.
    ///
    /// # Arguments
    ///
    /// - `board` - The current board state.
    /// - `depth` - The current depth.
    /// - `beta` - The current beta value.
    ///
    /// # Returns
    ///
    /// The score of the position if it can be pruned, otherwise None.
    fn pruned_score<Node: NodeType>(
        &mut self,
        board: &Board,
        depth: ScoreType,
        ply: ScoreType,
        beta: Score,
        local_pv: &mut PrincipleVariation,
    ) -> Option<Score> {
        // no pruning if we are in check or if we are in a PV node
        if board.is_in_check(&self.move_gen) || Node::PV {
            return None;
        }

        let static_eval = self.eval.eval(board);
        // Reverse futility pruning
        // https://cosmo.tardis.ac/files/2023-02-20-viri-wiki.html
        // https://www.chessprogramming.org/Reverse_Futility_Pruning
        // If the static evaluation is very high and beats beta by a depth-dependent margin, we can prune the move.
        if depth <= MAX_RFP_DEPTH && static_eval - RFP_MARGIN * depth > beta {
            return Some(static_eval);
        }

        /*
        Null move pruning
        https://www.chessprogramming.org/Null_Move_Pruning
        https://cosmo.tardis.ac/files/2023-02-20-viri-wiki.html
        Give the opponent a free move. If they cannot improve their position (beat beta)
        then prune the tree as our advantage is too great to bother searching further.
        */

        // Are we left with more than just kings and pawns?
        let sufficient_material = (board.all_pieces()
            ^ board.piece_kind_bitboard(Piece::King)
            ^ board.piece_kind_bitboard(Piece::Pawn))
        .number_of_occupied_squares()
            > 0;
        // was the last move null?
        let last_move_was_null = board.last_move().is_some_and(|mv| mv.is_null_move());

        if !last_move_was_null
            && depth >= NMP_MIN_DEPTH
            && static_eval >= beta
            && sufficient_material
        {
            let null_move_depth = depth - NMP_DEPTH_REDUCTION - 1;
            let mut null_board = board.clone();
            null_board.null_move();
            let null_score = -self.negamax::<Node>(
                &mut null_board,
                null_move_depth,
                ply + 1,
                -beta,
                -beta + 1,
                local_pv,
            );
            null_board.unmake_move().unwrap();
            if null_score >= beta {
                return Some(null_score);
            }
        }

        None
    }

    /// Implements [quiescence search](https://www.chessprogramming.org/Quiescence_Search).
    /// We use this to avoid the horizon effect. The idea is to evaluate quiet moves where there are no tactical moves to make.
    ///
    /// # Arguments
    ///
    /// - `board` - The current board state.
    /// - `ply` - The current ply.
    /// - `alpha` - The current alpha value.
    /// - `beta` - The current beta value.
    ///
    /// # Returns
    ///
    /// The score of the position.
    ///
    fn quiescence<Node: NodeType>(
        &mut self,
        board: &mut Board,
        alpha: Score,
        beta: Score,
        pv: &mut PrincipleVariation,
    ) -> Score {
        let standing_eval = self.eval.eval(board);
        if standing_eval >= beta {
            return beta;
        }
        let mut alpha_use: Score = alpha.max(standing_eval);

        let mut move_list = MoveList::new();
        let mut move_order_list = ArrayVec::<MoveOrder, MAX_MOVE_LIST_SIZE>::new();
        self.move_gen.generate_legal_moves(board, &mut move_list);

        let mut local_pv = PrincipleVariation::new();
        // clear the current PV because this is a new position
        pv.clear();

        // we only want captures here
        let mut captures = move_list
            .iter()
            .filter(|mv| mv.captured_piece().is_some())
            .copied()
            .collect::<Vec<_>>();

        // no captures
        if captures.is_empty() {
            return standing_eval;
        }

        // Transposition Table Cutoffs: https://www.chessprogramming.org/Transposition_Table#Transposition_Table_Cutoffs
        // Check if we have a transposition table entry and if we can return early
        let tt_move =
            match self
                .transposition_table
                .probe::<Node>(0, board.zobrist_hash(), alpha_use, beta)
            {
                ttable::ProbeResult::CutOff(entry) => {
                    // we have a cutoff, so return the score, but only in a non-PV node
                    if !Node::PV {
                        return entry.score;
                    }
                    Some(entry.board_move)
                }
                ttable::ProbeResult::Hit(entry) => Some(entry.board_move),
                ttable::ProbeResult::Empty => None,
            };

        // sort moves by MVV/LVA
        let classify_res = MoveOrder::classify_all(
            board.side_to_move(),
            captures.as_slice(),
            &tt_move,
            self.history_table,
            &mut move_order_list,
        );
        // TODO(PT): Should we log a message to the CLI or a log?
        assert!(classify_res.is_ok());

        let captures_slice = captures.as_mut_slice();
        let move_iter = InplaceIncrementalSort::new(captures_slice, &mut move_order_list);

        let mut best = standing_eval;
        let mut best_move = tt_move;
        let original_alpha = alpha_use;

        for mv in move_iter.into_iter() {
            // local PV is for each node below this one is different when we call negamax recursively
            // so we have to clear it
            local_pv.clear();

            board.make_move_unchecked(&mv).unwrap();
            let score = if board.is_draw() {
                Score::DRAW
            } else {
                let eval = -self.quiescence::<Node>(board, -beta, -alpha_use, &mut local_pv);
                self.nodes += 1;
                eval
            };
            board.unmake_move().unwrap();

            if score > best {
                best = score;
                best_move = Some(mv);

                // extend PV if we're in a PV node
                if Node::PV {
                    pv.extend(mv, &local_pv);
                }

                if score >= beta {
                    break;
                }
                if score > alpha_use {
                    alpha_use = score;
                }
            }

            if self.should_stop_searching() {
                break;
            }
        }

        if let Some(bm) = best_move {
            // store the best move in the transposition table
            let flag = if best <= original_alpha {
                ttable::EntryFlag::UpperBound
            } else if best >= beta {
                ttable::EntryFlag::LowerBound
            } else {
                ttable::EntryFlag::Exact
            };

            self.transposition_table
                .store_entry(TranspositionTableEntry::new(
                    board.zobrist_hash(),
                    0u8,
                    best,
                    flag,
                    bm,
                ));
        }

        best
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chess::{board::Board, pieces::ALL_PIECES};

    use crate::{
        evaluation::ByteKnightEvaluation,
        log_level::LogDebug,
        score::Score,
        search::{Search, SearchParameters},
        ttable::TranspositionTable,
    };

    use super::LargeScoreType;

    fn run_search_tests(test_pairs: &[(&str, &str)], config: SearchParameters) {
        let mut ttable = TranspositionTable::default();
        let mut history_table = Default::default();
        let mut search = Search::<LogDebug>::new(&config, &mut ttable, &mut history_table);

        for (fen, expected_move) in test_pairs {
            let mut board = Board::from_fen(fen).unwrap();
            let result = search.search(&mut board, None);
            assert_eq!(
                result.best_move.unwrap().to_long_algebraic(),
                *expected_move
            );
        }
    }

    #[test]
    fn white_mate_in_1() {
        let fen = "k7/8/KQ6/8/8/8/8/8 w - - 0 1";
        let board = Board::from_fen(fen).unwrap();
        let config = SearchParameters {
            max_depth: 2,
            ..Default::default()
        };

        let mut ttable = TranspositionTable::default();
        let mut history_table = Default::default();
        let mut search = Search::<LogDebug>::new(&config, &mut ttable, &mut history_table);
        let res = search.search(&mut board.clone(), None);
        // b6a7
        assert_eq!(
            res.best_move.unwrap().to_long_algebraic(),
            "b6a7".to_string()
        );
    }

    #[test]
    fn black_mated_in_1() {
        let fen = "1k6/8/KQ6/2Q5/8/8/8/8 b - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        let config = SearchParameters {
            max_depth: 3,
            ..Default::default()
        };

        let mut ttable = Default::default();
        let mut history_table = Default::default();
        let mut search = Search::<LogDebug>::new(&config, &mut ttable, &mut history_table);
        let res = search.search(&mut board, None);

        assert_eq!(res.best_move.unwrap().to_long_algebraic(), "b8a8")
    }

    #[test]
    fn mate_in_one() {
        // taken from Toad: https://github.com/dannyhammer/toad/blob/a84ea4c01c8bb036a132ff0e0f3d283029854289/src/search.rs#L1820
        let tests = [
            ("6k1/R7/6K1/8/8/8/8/8 w - - 0 1", "a7a8"),
            ("8/8/8/8/8/6k1/r7/6K1 b - - 0 1", "a2a1"),
            ("6k1/4R3/6K1/q7/8/8/8/8 w - - 0 1", "e7e8"),
            ("8/8/8/8/Q7/6k1/4r3/6K1 b - - 0 1", "e2e1"),
            ("6k1/8/6K1/q3R3/8/8/8/8 w - - 0 1", "e5e8"),
            ("8/8/8/8/Q3r3/6k1/8/6K1 b - - 0 1", "e4e1"),
            ("k7/6R1/5R1P/8/8/8/8/K7 w - - 0 1", "f6f8"),
            ("k7/8/8/8/8/5r1p/6r1/K7 b - - 0 1", "f3f1"),
        ];

        let params = SearchParameters {
            max_depth: 3,
            ..Default::default()
        };
        run_search_tests(&tests, params);
    }

    #[test]
    fn obvious_captures() {
        let tests = [
            ("5k2/8/8/b7/2N5/r7/8/5K2 w - - 0 1", "c4a3"),
            ("5k2/8/8/B7/2n5/R7/8/5K2 b - - 0 1", "c4a3"),
            ("5k2/8/8/b7/2N5/r7/8/5K2 w - - 0 1", "c4a3"),
            ("5k2/8/8/B7/2n5/R7/8/5K2 b - - 0 1", "c4a3"),
            ("4k3/8/8/1n1p4/2P5/8/8/4K3 w - - 0 1", "c4b5"),
            ("4k3/8/8/2p5/1N1P4/8/8/4K3 b - - 0 1", "c5b4"),
        ];

        let params = SearchParameters {
            max_depth: 3,
            ..Default::default()
        };
        run_search_tests(&tests, params);
    }

    #[test]
    fn stalemate() {
        let fen = "k7/8/KQ6/8/8/8/8/8 b - - 0 1";
        let mut board = Board::from_fen(fen).unwrap();
        let config = SearchParameters::default();

        let mut ttable = Default::default();
        let mut history_table = Default::default();
        let mut search = Search::<LogDebug>::new(&config, &mut ttable, &mut history_table);
        let res = search.search(&mut board, None);
        assert!(res.best_move.is_none());
        assert_eq!(res.score, Score::DRAW);
    }

    #[test]
    #[ignore = "Timing on this is not consistent when instrumentation is enabled"]
    fn do_not_exceed_time() {
        let mut board = Board::default_board();
        let config = SearchParameters {
            soft_timeout: Duration::from_millis(100),
            hard_timeout: Duration::from_millis(1000),
            ..Default::default()
        };

        let mut ttable = Default::default();
        let mut history_table = Default::default();
        let mut search = Search::<LogDebug>::new(&config, &mut ttable, &mut history_table);
        let res = search.search(&mut board, None);

        assert!(res.best_move.is_some());
        assert!(config.start_time.elapsed() <= config.hard_timeout);
    }

    #[test]
    fn starting_position() {
        let mut board = Board::default_board();
        let config = SearchParameters {
            max_depth: 8,
            ..Default::default()
        };

        let mut ttable = Default::default();
        let mut history_table = Default::default();
        let mut search = Search::<LogDebug>::new(&config, &mut ttable, &mut history_table);
        let res = search.search(&mut board, None);
        assert!(res.best_move.is_some());
        println!("{}", res.best_move.unwrap().to_long_algebraic());
    }

    #[test]
    fn no_time() {
        let mut board = Board::from_fen("8/7p/5p2/2K1qp2/7P/8/6k1/4q3 w - - 1 2").unwrap();
        let config = SearchParameters {
            soft_timeout: Duration::from_millis(0),
            hard_timeout: Duration::from_millis(0),
            ..Default::default()
        };

        let mut ttable = Default::default();
        let mut history_table = Default::default();
        let mut search = Search::<LogDebug>::new(&config, &mut ttable, &mut history_table);
        let res = search.search(&mut board, None);
        assert!(res.best_move.is_some());
        println!("{}", res.best_move.unwrap().to_long_algebraic());
    }

    const TEST_FENS: [&str; 25] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "4k3/8/8/8/8/8/8/4K2R w K - 0 1",
        "4k3/8/8/8/8/8/8/R3K3 w Q - 0 1",
        "4k2r/8/8/8/8/8/8/4K3 w k - 0 1",
        "r3k3/8/8/8/8/8/8/4K3 w q - 0 1",
        "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1",
        "r3k2r/8/8/8/8/8/8/4K3 w kq - 0 1",
        "8/8/8/8/8/8/6k1/4K2R w K - 0 1",
        "8/8/8/8/8/8/1k6/R3K3 w Q - 0 1",
        "4k2r/6K1/8/8/8/8/8/8 w k - 0 1",
        "r3k3/1K6/8/8/8/8/8/8 w q - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "r3k2r/8/8/8/8/8/8/1R2K2R w Kkq - 0 1",
        "r3k2r/8/8/8/8/8/8/2R1K2R w Kkq - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K1R1 w Qkq - 0 1",
        "1r2k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1",
        "2r1k2r/8/8/8/8/8/8/R3K2R w KQk - 0 1",
        "r3k1r1/8/8/8/8/8/8/R3K2R w KQq - 0 1",
        "4k3/8/8/8/8/8/8/4K2R b K - 0 1",
        "4k3/8/8/8/8/8/8/R3K3 b Q - 0 1",
        "4k2r/8/8/8/8/8/8/4K3 b k - 0 1",
        "r3k3/8/8/8/8/8/8/4K3 b q - 0 1",
        "4k3/8/8/8/8/8/8/R3K2R b KQ - 0 1",
        "r3k2r/8/8/8/8/8/8/4K3 b kq - 0 1",
    ];

    #[test]
    fn quiets_ordered_after_captures() {
        let config = SearchParameters {
            max_depth: 6,
            ..Default::default()
        };

        let mut min_mvv_lva = LargeScoreType::MAX;
        let mut max_mvv_lva = LargeScoreType::MIN;
        for capturing in ALL_PIECES {
            for captured in ALL_PIECES.iter().filter(|p| !p.is_king()) {
                let mvv_lva = ByteKnightEvaluation::mvv_lva(*captured, capturing);
                if mvv_lva < min_mvv_lva {
                    min_mvv_lva = mvv_lva;
                }
                if mvv_lva > max_mvv_lva {
                    max_mvv_lva = mvv_lva;
                }
            }
        }

        for fen in TEST_FENS {
            let mut board = Board::from_fen(fen).unwrap();

            let mut ttable = Default::default();
            let mut history_table = Default::default();
            let mut search = Search::<LogDebug>::new(&config, &mut ttable, &mut history_table);
            let res = search.search(&mut board, None);

            assert!(res.best_move.is_some());

            let side = board.side_to_move();
            let mut max_history = LargeScoreType::MIN;
            for piece in ALL_PIECES {
                for square in 0..64 {
                    let score = history_table.get(side, piece, square);
                    if score > max_history {
                        max_history = score;
                    }
                }
            }

            println!("max history: {max_history:5}");
            println!("min/max mvv-lva: {min_mvv_lva}, {max_mvv_lva}");
            assert!(max_history < min_mvv_lva);
        }
    }
}
