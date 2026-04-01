// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    fmt::Display,
    io::Write,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use anyhow::{Result, bail};
use chess::{
    board::Board,
    definitions::MAX_MOVE_LIST_SIZE,
    move_generation::{self, move_filter::MoveFilter},
    moves::Move,
    pieces::Piece,
};
use uci_parser::{UciInfo, UciResponse, UciScore, UciSearchOptions};

use crate::{
    aspiration_window::AspirationWindow,
    defs::{MAX_DEPTH, MAX_PLY},
    evaluation::ByteKnightEvaluation,
    history_table::{self, HistoryTable},
    killers_table::KillerMovesTable,
    lmr,
    log_level::LogLevel,
    move_picker,
    node_types::{NodeType, NonPvNode, PvNode, RootNode},
    principle_variation::PrincipleVariation,
    score::{LargeScoreType, Score, ScoreType},
    table::Table,
    traits::Eval,
    ttable,
    tuneable::{
        IIR_DEPTH_REDUCTION, IIR_MIN_DEPTH, LMR_MIN_DEPTH, LMR_MIN_MOVES_SEEN, MAX_RFP_DEPTH,
        NMP_DEPTH_REDUCTION, NMP_MIN_DEPTH, RAZORING_OFFSET, RAZORING_SCALING, RFP_MARGIN,
        lmp_max_depth,
    },
};
use ttable::TranspositionTable;

mod params;

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
    killers_table: &'search_lifetime mut KillerMovesTable,
    nodes: u64,
    seldepth: ScoreType,
    parameters: SearchParameters,
    eval: ByteKnightEvaluation,
    stop_flag: Option<Arc<AtomicBool>>,
    lmr_table: Table<f64, 32_000>,
    output: &'search_lifetime mut dyn Write,
    /// Marker for the level of logging to print.
    log: PhantomData<Log>,
}

impl<'a, Log: LogLevel> Search<'a, Log> {
    pub fn new(
        parameters: &SearchParameters,
        ttable: &'a mut TranspositionTable,
        history_table: &'a mut HistoryTable,
        killers_table: &'a mut KillerMovesTable,
        output: &'a mut dyn Write,
    ) -> Self {
        // Initialize our LMR table as a 2D array of our LMR formula for depth and moves played
        let mut table = Table::<f64, 32_000>::new(MAX_DEPTH as usize, MAX_MOVE_LIST_SIZE);
        table.fill(lmr::formula);

        // Clear killers as this is a new position.
        killers_table.clear();

        Self {
            transposition_table: ttable,
            history_table,
            killers_table,
            nodes: 0,
            seldepth: 0,
            parameters: parameters.clone(),
            eval: ByteKnightEvaluation::default(),
            stop_flag: None,
            lmr_table: table,
            output,
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
            self.send_message(format!("starting search for FEN {}", board.to_fen()));
            self.send_message(format!("searching {}", self.parameters));
        }

        let ml = move_generation::legal::generate_moves(board, MoveFilter::All);
        let mut result = match ml.len() {
            0 => {
                // Draw or something else?
                let result = SearchResult {
                    score: if move_generation::is_in_check(board) {
                        -Score::MATE
                    } else {
                        Score::DRAW
                    },
                    best_move: None,
                    nodes: 0,
                    depth: 0,
                    pv: PrincipleVariation::new(),
                };
                self.nodes += 1;
                if Log::DEBUG {
                    self.send_message(
                        format!(
                            "{} has no legal moves available - scored as {}",
                            board.to_fen(),
                            UciScore::from(result.score)
                        )
                        .to_string(),
                    );
                }

                result
            }
            _ => self.iterative_deepening(board),
        };
        if Log::DEBUG {
            self.send_message(format!("search ended after {} nodes", self.nodes));
        }

        // Try to ensure we have a move
        if result.best_move.is_none()
            && let Some(mv) = ml.as_slice().first().copied()
        {
            result.best_move = Some(mv);
            result.score = self.eval.eval(board);
        }

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

    /// Send UCI info to the the output.
    #[allow(clippy::too_many_arguments)]
    fn send_info(
        &mut self,
        depth: u8,
        seldepth: ScoreType,
        nodes: u64,
        score: Score,
        nps: f32,
        time: u64,
        hashfull: u16,
        pv: &PrincipleVariation,
    ) {
        // create UciInfo and print it
        let info = UciInfo::new()
            .depth(depth)
            .seldepth(seldepth)
            .nodes(nodes)
            .score(score)
            .nps(nps.trunc())
            .time(time)
            .hashfull(hashfull)
            .pv(pv.iter().map(|m| m.to_long_algebraic()));
        let message = UciResponse::info(info);
        let _unused = writeln!(self.output, "{message}");
    }

    /// Write a string to the output.
    fn send_message(&mut self, message: String) {
        let info = UciInfo::default().string(message);
        let message = UciResponse::info(info);
        let _unused = writeln!(self.output, "{message}");
    }

    /// Verify that a given [PrincipleVariation] is valid. This is expensive and should only be used for debugging.
    #[allow(clippy::expect_used)]
    fn verify_pv_moves(&self, pv: &PrincipleVariation, board: &Board) -> Result<()> {
        let mut board_cpy = board.clone();
        let all_ok = pv.iter().all(|mv| {
            let mv_ok = board_cpy.make_move(mv);
            mv_ok.is_ok()
        });
        if !all_ok {
            bail!("PV is invalid!")
        }

        Ok(())
    }

    /// Perform [iterative deepening](https://www.chessprogramming.org/Iterative_Deepening) on the search position.
    ///
    /// This is a simple way for the engine to manage it's time. Each iteration we check if we still have time to continue
    /// searching deeper based on the soft timeout. If we do, then we search at depth `d+1`. If we have exceeded the soft timeout,
    /// we stop searching and return the best move found so far.
    fn iterative_deepening(&mut self, board: &mut Board) -> SearchResult {
        // initialize the best result
        let mut best_result = SearchResult::default();

        let move_list = move_generation::legal::generate_moves(board, MoveFilter::All);
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
            // reset seldepth for this iteration
            self.seldepth = 0;

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
                    self.seldepth,
                    self.nodes,
                    best_result.score,
                    (self.nodes as f32 / self.parameters.start_time.elapsed().as_secs_f32())
                        .trunc(),
                    self.parameters.start_time.elapsed().as_millis() as u64,
                    self.transposition_table.hashfull(),
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
                self.seldepth,
                self.nodes,
                best_result.score,
                (self.nodes as f32 / self.parameters.start_time.elapsed().as_secs_f32()).trunc(),
                self.parameters.start_time.elapsed().as_millis() as u64,
                self.transposition_table.hashfull(),
                &best_result.pv,
            );
        }

        // return our best result so far
        best_result
    }

    /// Implements the [Negamax](https://www.chessprogramming.org/Negamax) search algorithm with alpha-beta
    /// pruning and a [fail-soft](https://www.chessprogramming.org/Alpha-Beta#Negamax_Framework) framework.
    ///
    /// This is the core of the search algorithm. It recursively searches the game tree to find the best move.
    fn negamax<Node>(
        &mut self,
        board: &mut Board,
        mut depth: ScoreType,
        ply: ScoreType,
        mut alpha: Score,
        mut beta: Score,
        pv: &mut PrincipleVariation,
    ) -> Score
    where
        Node: NodeType,
    {
        // increment node count
        self.nodes += 1;
        self.seldepth = self.seldepth.max(ply);

        // Ply guard: prevent unbounded recursion
        if ply >= MAX_PLY {
            return self.eval.eval(board);
        }

        if depth <= 0 {
            return self.quiescence::<Node>(board, ply, alpha, beta, pv);
        }

        if !Node::ROOT {
            // Mate Distance Pruning
            // If we have already found a mate, prune nodes where no shorter mate is possible
            alpha = alpha.max(Score::mated_in(ply));
            beta = beta.min(Score::mate_in(ply) + 1);
            if alpha >= beta {
                return alpha;
            }
        }

        let alpha_original = alpha;

        let mut local_pv = PrincipleVariation::new();
        // clear the current PV because this is a new position
        pv.clear();

        // Transposition Table Cutoffs: https://www.chessprogramming.org/Transposition_Table#Transposition_Table_Cutoffs
        // Check if we have a transposition table entry and if we can return early
        let tt_move = match self.transposition_table.probe::<Node>(
            depth,
            ply,
            board.zobrist_hash(),
            alpha,
            beta,
        ) {
            ttable::ProbeResult::CutOff(entry) => {
                // we have a cutoff, so return the score, but only in a non-PV node
                if !Node::PV {
                    return entry.score.ply_relative(ply);
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
        if let Some(score) = self.pruned_score::<Node>(board, depth, ply, beta, alpha) {
            return score;
        }

        // Build move picker. Move generation is lazy (deferred to stage machine).
        let mut picker = move_picker::MovePicker::new(tt_move, self.killers_table, ply as u8);

        // Really "bad" initial score
        let mut best_score = -Score::INF;
        let mut best_move = tt_move;
        let mut moves_seen = 0;
        let static_eval = self.eval.eval(board);

        // Loop through all moves in best-first order.
        while let Some(mv) = picker.next(board, self.history_table) {
            let loop_counter = picker.moves_yielded() - 1;

            // Calculate the LMR reduction and depth which will be used later in FP
            let lmr_table_value = self.lmr_table.at(depth as usize, loop_counter);
            let base_reduction = if let Some(table_val) = lmr_table_value {
                *table_val
            } else {
                1f64
            };

            let lmr_reduction = (1f64 + base_reduction).floor() as i16;
            let is_mated = best_score.mated();
            let is_in_check = picker.in_check();
            let is_root = Node::ROOT;
            let is_pv = Node::PV;
            let is_quiet = board.captured(&mv).is_none() && !mv.is_promotion();
            let piece = board.piece_on_square(mv.from()).map(|(pc, _)| pc).unwrap();

            // Move-loop pruning techniques

            // --------------------------------------------------------------------------------------------------------
            // Futility pruning: https://www.chessprogramming.org/Futility_Pruning
            // If we are at a shallow depth and have already found a good score, we start skipping moves
            // --------------------------------------------------------------------------------------------------------
            if !is_root && !is_pv && !is_in_check && !best_score.mated() {
                let fp_margin = depth * FUTILITY_COEFF + FUTILITY_OFFSET;
                if mv.is_quiet() && depth <= FUTILITY_MAX_DEPTH && static_eval + fp_margin <= alpha
                {
                    continue;
                }
            }

            // LMP - Late Move Pruning
            // We assume our move ordering is just too good, so if we're under a certain depth
            // and have made more than a certain number of moves, we can assume that later moves
            // won't be as good, so we prune them.
            // ---------------------------------------------------------------------------------
            if !is_root
                && !is_pv
                && !is_in_check
                && !is_mated
                && is_quiet
                && depth <= lmp_max_depth() as i16
                && moves_seen > params::late_move_threshold(depth as i32)
            {
                break;
            }

            // local PV is for each node below this one is different when we call negamax recursively
            // so we have to clear it
            local_pv.clear();

            // make the move
            board.make_move_unchecked(&mv).unwrap();
            self.transposition_table.prefetch(board.zobrist_hash());
            let mut score = Score::DRAW;

            // Don't bother searching drawn positions
            if !board.is_draw() {
                score =
                // Principal Variation Search (PVS)
                if moves_seen == 0 {
                    -self.negamax::<Node::Next>(board, depth - 1, ply + 1, -beta, -alpha, &mut local_pv)
                } else {
                    let is_killer = self.killers_table.get(ply as u8).iter().any(|entry|entry.is_some_and(|k|k.matches(mv, piece)));
                    // No LMR reduction for killer moves
                    let reduction = if is_quiet && depth >= LMR_MIN_DEPTH && moves_seen as usize >= LMR_MIN_MOVES_SEEN {
                        if is_killer {
                            // Reduce less if the move is a killer
                            (lmr_reduction-1).max(1)
                        } else {
                            lmr_reduction
                        }
                    } else {
                        1
                    };

                    // Calculate the reduced depth
                    let reduced_depth = depth.saturating_sub(reduction);

                    // Search with a null window at a reduced depth
                    let mut temp_score = -self.negamax::<NonPvNode>(board, reduced_depth, ply + 1, -alpha - 1, -alpha, &mut local_pv);

                    // If the reduced depth failed, verify again at full depth with null window to avoid a more expensive full re-search
                    temp_score = if temp_score > alpha && reduction > 1 {
                        -self.negamax::<NonPvNode>(board, depth - 1, ply + 1, -alpha - 1, -alpha, &mut local_pv)
                    }
                    else {
                        temp_score
                    };

                    // If it fails again, we now know we need to do a full re-search
                    if temp_score > alpha && temp_score < beta {
                        -self.negamax::<PvNode>(board, depth - 1, ply + 1, -beta, -alpha, &mut local_pv)
                    }
                    else {
                        temp_score
                    }
                };
            }

            // undo the move
            board.unmake_move().unwrap();
            moves_seen += 1;

            // check the results
            if score > best_score {
                // we improved, so update the score and best move
                best_score = score;
                best_move = Some(mv);
                if Node::PV {
                    // assert_pv_is_legal(board, mv, &local_pv);
                    pv.extend(mv, &local_pv);
                }

                alpha = alpha.max(best_score);
                // Did we fail high?
                if alpha >= beta {
                    // update history table for quiets
                    if is_quiet {
                        // Update the killers table
                        self.killers_table.update(ply as u8, mv, piece);

                        // calculate history bonus
                        let bonus = history_table::calculate_bonus_for_depth(depth);
                        self.history_table.update(
                            board.side_to_move(),
                            piece,
                            mv.to(),
                            bonus as LargeScoreType,
                        );

                        // Apply a penalty to all quiets searched so far.
                        // The board is already in the parent state (we already unmade the move)
                        // so it's safe to look up the piece on the board using mv.from().
                        for &(prev_mv, prev_piece) in picker.searched_quiets() {
                            if prev_mv == mv {
                                continue;
                            }
                            self.history_table.update(
                                board.side_to_move(),
                                prev_piece,
                                prev_mv.to(),
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

        // No moves were yielded: checkmate or stalemate.
        if picker.moves_yielded() == 0 {
            return if picker.in_check() {
                -Score::MATE + ply
            } else {
                Score::DRAW
            };
        }

        if let Some(bm) = best_move {
            // store the best move in the transposition table
            let flag = if best_score <= alpha_original {
                ttable::EntryFlag::UpperBound
            } else if best_score >= beta {
                ttable::EntryFlag::LowerBound
            } else {
                ttable::EntryFlag::Exact
            };

            self.transposition_table.store_entry(
                board.zobrist_hash(),
                depth as u8,
                best_score.remove_ply_bias(ply),
                flag,
                bm,
            );
        }
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
        alpha: Score,
    ) -> Option<Score> {
        // no pruning if we are in check or if we are in a PV node
        if move_generation::is_in_check(board) || Node::PV {
            return None;
        }

        let static_eval = self.eval.eval(board);

        // Razoring: https://www.chessprogramming.org/Razoring
        // Check if the static eval + margin is less than alpha. For byte-knight, we prune based on qsearch evaluation.
        // If we can't beat alpha with the qsearch score, then we fail-low.
        let razoring_margin = RAZORING_OFFSET + RAZORING_SCALING * depth;
        if static_eval + razoring_margin < alpha {
            let mut brd_cpy = board.clone();
            let mut razor_pv = PrincipleVariation::new();
            let score =
                self.quiescence::<NonPvNode>(&mut brd_cpy, ply, alpha, alpha + 1, &mut razor_pv);
            if score < alpha && !score.is_mate() {
                return Some(score);
            }
        }

        // --------------------------------------------------------------------------------------------------------
        // Reverse futility pruning
        // https://cosmo.tardis.ac/files/2023-02-20-viri-wiki.html
        // https://www.chessprogramming.org/Reverse_Futility_Pruning
        // If the static evaluation is very high and beats beta by a depth-dependent margin, we can prune the move.
        // --------------------------------------------------------------------------------------------------------
        if depth <= MAX_RFP_DEPTH && static_eval - RFP_MARGIN * depth > beta {
            return Some(static_eval);
        }

        // --------------------------------------------------------------------------------
        // Null move pruning
        // https://www.chessprogramming.org/Null_Move_Pruning
        // https://cosmo.tardis.ac/files/2023-02-20-viri-wiki.html
        // Give the opponent a free move. If they cannot improve their position (beat beta)
        // then prune the tree as our advantage is too great to bother searching further.
        // --------------------------------------------------------------------------------
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
            self.transposition_table.prefetch(null_board.zobrist_hash());
            let mut nmp_pv = PrincipleVariation::new();
            let null_score = -self.negamax::<NonPvNode>(
                &mut null_board,
                null_move_depth,
                ply + 1,
                -beta,
                -beta + 1,
                &mut nmp_pv,
            );
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
        ply: ScoreType,
        alpha: Score,
        beta: Score,
        pv: &mut PrincipleVariation,
    ) -> Score {
        // Quiescence search shouldn't be called at root
        debug_assert!(ply > 0);

        self.seldepth = self.seldepth.max(ply);

        // Are we in a draw?
        if ply > 0 && board.is_draw() {
            return Score::DRAW;
        }

        let in_check = move_generation::is_in_check(board);
        let standing_eval = self.eval.eval(board);

        // Have we exceeded max ply?
        if ply >= MAX_PLY {
            return standing_eval;
        }

        // Stand-pat: when not in check we can always choose not to capture.
        // When in check we are forced to move, so stand-pat does not apply.
        let mut alpha_use: Score = if !in_check {
            if standing_eval >= beta {
                return beta;
            }
            alpha.max(standing_eval)
        } else {
            alpha
        };

        // Transposition Table Cutoffs: https://www.chessprogramming.org/Transposition_Table#Transposition_Table_Cutoffs
        // Check if we have a transposition table entry and if we can return early
        let tt_move = match self.transposition_table.probe::<Node>(
            0,
            ply,
            board.zobrist_hash(),
            alpha_use,
            beta,
        ) {
            ttable::ProbeResult::CutOff(entry) => {
                // we have a cutoff, so return the score, but only in a non-PV node
                if !Node::PV {
                    return entry.score.ply_relative(ply);
                };
                Some(entry.board_move)
            }
            ttable::ProbeResult::Hit(entry) => Some(entry.board_move),
            ttable::ProbeResult::Empty => None,
        };

        // When in check we must consider all moves; otherwise tacticals only.
        let mut picker = move_picker::MovePicker::new_qsearch(tt_move, in_check);

        let mut local_pv = PrincipleVariation::new();
        // clear the current PV because this is a new position
        pv.clear();

        // When in check there is no stand-pat floor, so begin from -INF.
        let mut best = if in_check { -Score::INF } else { standing_eval };
        let mut best_move = tt_move;
        let original_alpha = alpha_use;

        while let Some(mv) = picker.next(board, self.history_table) {
            // local PV is for each node below this one is different when we call negamax recursively
            // so we have to clear it
            local_pv.clear();

            board.make_move_unchecked(&mv).unwrap();
            self.transposition_table.prefetch(board.zobrist_hash());

            let score = if board.is_draw() {
                Score::DRAW
            } else {
                let eval =
                    -self.quiescence::<Node>(board, ply + 1, -beta, -alpha_use, &mut local_pv);
                self.nodes += 1;
                eval
            };

            board.unmake_move().unwrap();

            if score > best {
                best = score;
                best_move = Some(mv);

                // extend PV if we're in a PV node
                if Node::PV {
                    // assert_pv_is_legal(board, mv, &local_pv);
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

        // In check with no legal moves: checkmate.
        // When not in check, standing_eval already covers the stand-pat case.
        if picker.moves_yielded() == 0 && in_check {
            return Score::new_mated() + ply;
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

            self.transposition_table.store_entry(
                board.zobrist_hash(),
                0u8,
                best.remove_ply_bias(ply),
                flag,
                bm,
            );
        }

        best
    }
}

#[allow(dead_code)]
fn assert_pv_is_legal(board: &Board, mv: Move, local_pv: &PrincipleVariation) {
    let fen = board.to_fen();
    let mut board_cpy = board.clone();

    for local_mv in [&mv].into_iter().chain(local_pv.iter()) {
        assert!(
            move_generation::is_legal(&board_cpy, local_mv),
            "Illegal PV move {local_mv} after move {local_mv} in position {fen}\nFull PV: {}\nResulting FEN: {}",
            [local_mv]
                .into_iter()
                .chain(local_pv.iter())
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            board_cpy.to_fen()
        );

        let mv_ok = board_cpy.make_move(local_mv);
        assert!(
            mv_ok.is_ok(),
            "Failed to make PV move {local_mv} in position {fen}"
        );
    }
}

#[cfg(test)]
mod tests {
    use std::{io, time::Duration};

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
        let mut killers_table = Default::default();
        let mut sink = io::sink();
        let mut search = Search::<LogDebug>::new(
            &config,
            &mut ttable,
            &mut history_table,
            &mut killers_table,
            &mut sink,
        );

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
        let mut killers_table = Default::default();
        let mut sink = io::sink();
        let mut search = Search::<LogDebug>::new(
            &config,
            &mut ttable,
            &mut history_table,
            &mut killers_table,
            &mut sink,
        );
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
        let mut killers_table = Default::default();
        let mut sink = io::sink();
        let mut search = Search::<LogDebug>::new(
            &config,
            &mut ttable,
            &mut history_table,
            &mut killers_table,
            &mut sink,
        );
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
        let mut killers_table = Default::default();
        let mut sink = io::sink();
        let mut search = Search::<LogDebug>::new(
            &config,
            &mut ttable,
            &mut history_table,
            &mut killers_table,
            &mut sink,
        );
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
        let mut killers_table = Default::default();
        let mut sink = io::sink();
        let mut search = Search::<LogDebug>::new(
            &config,
            &mut ttable,
            &mut history_table,
            &mut killers_table,
            &mut sink,
        );
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
        let mut killers_table = Default::default();
        let mut sink = io::sink();
        let mut search = Search::<LogDebug>::new(
            &config,
            &mut ttable,
            &mut history_table,
            &mut killers_table,
            &mut sink,
        );
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
        let mut killers_table = Default::default();
        let mut sink = io::sink();
        let mut search = Search::<LogDebug>::new(
            &config,
            &mut ttable,
            &mut history_table,
            &mut killers_table,
            &mut sink,
        );
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
            let mut killers_table = Default::default();
            let mut sink = io::sink();
            let mut search = Search::<LogDebug>::new(
                &config,
                &mut ttable,
                &mut history_table,
                &mut killers_table,
                &mut sink,
            );
            let res = search.search(&mut board, None);
            drop(search);

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

    #[test]
    fn pv_going_past_three_fold_repetition() {
        let starting_fen = "rnbqr1k1/pp2bppp/2p1pn2/8/P1BP4/2N1PN2/1P3PPP/R1BQ1RK1 w - - 3 9";
        let uci_moves = [
            "e3e4", "b8d7", "f1e1", "b7b6", "e4e5", "f6d5", "a4a5", "b6a5", "c3e4", "c8b7", "c1g5",
            "d7b6", "c4d5", "d8d5", "g5e7", "e8e7", "a1c1", "d5a2", "e4d6", "h7h6", "b2b3", "a5a4",
            "f3d2", "a2b2", "e1e4", "a8b8", "c1b1", "b2c3", "b3a4", "c6c5", "b1c1", "c3d3", "d6b7",
            "b8b7", "d4c5", "b6d5", "d1c2", "d3a3", "d2b1", "a3b2", "b1d2", "b2c2", "c1c2", "d5b4",
            "c2c1", "e7c7", "e4d4", "b4c6", "d4e4", "b7b2", "d2c4", "b2b4", "c4d6", "b4e4", "d6e4",
            "c6e5", "h2h3", "e5d7", "c5c6", "d7e5", "c1c5", "e5d3", "c5c4", "d3e5", "c4c5", "e5c6",
            "e4d6", "g7g5", "d6c4", "c7c8", "c4d6", "c8c7", "d6c4", "c7c8", "c4d6",
        ];

        let mut board = Board::from_fen(starting_fen).unwrap();
        for mv in uci_moves {
            assert!(board.make_uci_move(mv).is_ok());
        }

        let is_repetiton = board.is_repetition();
        assert!(!is_repetiton, "Expected position to not be a repetition");
        let config = SearchParameters {
            max_depth: 24,
            ..Default::default()
        };

        let mut ttable = Default::default();
        let mut history_table = Default::default();
        let mut killers_table = Default::default();
        let mut sink = io::sink();
        let mut search = Search::<LogDebug>::new(
            &config,
            &mut ttable,
            &mut history_table,
            &mut killers_table,
            &mut sink,
        );
        let res = search.search(&mut board, None);

        assert!(res.best_move.is_some());
        let mv = res.best_move.unwrap();
        println!("{}", mv.to_long_algebraic());
    }

    /// Helper: run a search at the given depth and assert it doesn't panic.
    fn search_position(board: &mut Board, depth: u8) {
        let config = SearchParameters {
            max_depth: depth,
            ..Default::default()
        };
        let mut ttable = TranspositionTable::default();
        let mut history_table = Default::default();
        let mut killers_table = Default::default();
        let mut sink = io::sink();
        let mut search = Search::<LogDebug>::new(
            &config,
            &mut ttable,
            &mut history_table,
            &mut killers_table,
            &mut sink,
        );
        let res = search.search(board, None);
        assert!(res.best_move.is_some(), "Search must return a move");
    }

    /// Regression tests for positions that caused crashes during fastchess play.
    /// Each position is from a game where the engine disconnected.
    /// We search the starting FEN of each crash game at high depth to trigger
    /// the same code paths that caused the crash.
    #[test]
    fn repro_crash_positions() {
        let crash_fens = [
            // Round 7: shortest crash (4 ply before disconnect)
            "r2qkb1r/2p2ppp/p1npbn2/1p2p3/4P3/2P2N2/PPBP1PPP/RNBQR1K1 w kq - 2 9",
            // Round 1: 42 ply crash
            "r1bq1rk1/ppppn1bp/2n3p1/4p2P/4p3/2NP2P1/PPP1NPB1/R1BQK2R w KQ - 0 9",
            // Round 60: 24 ply crash
            "r3k2r/pp1npp1p/1qpp1p1b/5b2/3P4/1NP3P1/PP2PPBP/R2QK1NR w KQkq - 5 9",
            // Round 93: 46 ply crash
            "rnbqkb1r/p3nppp/1p2p3/2ppP3/3P1BP1/2PB1N2/PP3P1P/RN1QK2R w KQkq - 2 9",
        ];

        for fen in &crash_fens {
            println!("Testing: {fen}");
            let mut board = Board::from_fen(fen).unwrap();
            search_position(&mut board, 12);
        }
    }

    /// Integration tests: inject invalid TT entries that simulate hash collisions,
    /// then verify the search completes without crashing.
    ///
    /// Each test stores a poisoned move into the TT at the exact index the
    /// position will probe, forcing the move picker to encounter it as a TT move.
    mod tt_collision_tests {
        use std::io;

        use chess::{
            board::Board,
            definitions::Squares,
            moves::{Move, MoveFlag},
            square::Square,
        };

        use crate::{
            log_level::LogDebug,
            score::Score,
            search::{Search, SearchParameters},
            ttable::{EntryFlag, TranspositionTable},
        };

        /// Inject a poisoned move into the TT and run a search.
        /// The search must complete without panicking.
        fn search_with_poisoned_tt(fen: &str, bad_move: Move, depth: u8) {
            let mut board = Board::from_fen(fen).unwrap();
            let zobrist = board.zobrist_hash();

            let mut ttable = TranspositionTable::default();
            // Store the bad move at the position's zobrist hash so the search
            // will find it on the first TT probe.
            ttable.store_entry(
                zobrist,
                depth + 2, // high depth so the TT entry is trusted
                Score::new(50),
                EntryFlag::Exact,
                bad_move,
            );

            let config = SearchParameters {
                max_depth: depth,
                ..Default::default()
            };
            let mut history_table = Default::default();
            let mut killers_table = Default::default();
            let mut sink = io::sink();
            let mut search = Search::<LogDebug>::new(
                &config,
                &mut ttable,
                &mut history_table,
                &mut killers_table,
                &mut sink,
            );
            let res = search.search(&mut board, None);
            assert!(res.best_move.is_some(), "Search must return a move");
        }

        #[test]
        fn tt_collision_pawn_to_promotion_rank() {
            // Position with White pawn on a7. Inject a7a8 as Standard (no promo flag).
            let bad_move = Move::new(
                Square::from_square_index(Squares::A7),
                Square::from_square_index(Squares::A8),
                MoveFlag::Standard,
            );
            search_with_poisoned_tt("3qk3/P7/8/8/8/8/8/4K3 w - - 0 1", bad_move, 8);
        }

        #[test]
        fn tt_collision_pawn_to_back_rank() {
            // Position with White pawn on b2. Inject b2a1 as Standard.
            let bad_move = Move::new(
                Square::from_square_index(Squares::B2),
                Square::from_square_index(Squares::A1),
                MoveFlag::Standard,
            );
            search_with_poisoned_tt("4k3/8/8/8/8/8/1P6/4K3 w - - 0 1", bad_move, 8);
        }

        #[test]
        fn tt_collision_castle_with_occupied_square() {
            // White can castle kingside but we inject the castle move for a
            // position where f1 is occupied by a bishop — the move should be
            // rejected by is_legal.
            let bad_move = Move::new(
                Square::from_square_index(Squares::E1),
                Square::from_square_index(Squares::G1),
                MoveFlag::CastleK,
            );
            search_with_poisoned_tt("4k3/8/8/8/8/8/8/4KB1R w K - 0 1", bad_move, 8);
        }

        #[test]
        fn tt_collision_en_passant_without_target() {
            // White pawn on e5, no EP square set. Inject EP move e5f6.
            let bad_move = Move::new(
                Square::from_square_index(Squares::E5),
                Square::from_square_index(Squares::F6),
                MoveFlag::EnPassant,
            );
            search_with_poisoned_tt("4k3/8/8/4P3/8/8/8/4K3 w - - 0 1", bad_move, 8);
        }

        #[test]
        fn tt_collision_castle_without_rook() {
            // King on e1, no rook on h1 but has K rights (bogus FEN).
            // Inject kingside castle.
            let bad_move = Move::new(
                Square::from_square_index(Squares::E1),
                Square::from_square_index(Squares::G1),
                MoveFlag::CastleK,
            );
            search_with_poisoned_tt("4k3/8/8/8/8/8/4PPPP/4K3 w K - 0 1", bad_move, 8);
        }

        #[test]
        fn tt_collision_black_pawn_to_back_rank() {
            // Black pawn on g7. Inject g7h8 as Standard (backward to rank 7).
            let bad_move = Move::new(
                Square::from_square_index(Squares::G7),
                Square::from_square_index(Squares::H8),
                MoveFlag::Standard,
            );
            search_with_poisoned_tt("4k3/6p1/8/8/8/8/8/4K3 b - - 0 1", bad_move, 8);
        }
    }
}
