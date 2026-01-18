// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    io::Write,
    str::FromStr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    thread::JoinHandle,
};

use chess::{board::Board, moves::Move, pieces::SQUARE_NAME};
use uci_parser::{UciMove, UciResponse};

use crate::{
    history_table::HistoryTable,
    log_level::{LogDebug, LogInfo, LogLevel},
    search::{Search, SearchParameters},
    ttable::TranspositionTable,
};

fn square_index_to_uci_square(square: u8) -> uci_parser::Square {
    uci_parser::Square::from_str(SQUARE_NAME[square as usize]).unwrap()
}

fn move_to_uci_move(mv: &Move) -> UciMove {
    let promotion = mv.promotion_piece().map(|p| p.as_char());

    match promotion {
        Some(promotion) => UciMove {
            src: square_index_to_uci_square(mv.from()),
            dst: square_index_to_uci_square(mv.to()),
            promote: Some(uci_parser::Piece::from_str(&promotion.to_string()).unwrap()),
        },
        None => UciMove {
            src: square_index_to_uci_square(mv.from()),
            dst: square_index_to_uci_square(mv.to()),
            promote: None,
        },
    }
}

#[allow(clippy::large_enum_variant)]
pub(crate) enum SearchThreadValue {
    Params(
        Board,
        SearchParameters,
        Arc<Mutex<TranspositionTable>>,
        Arc<Mutex<HistoryTable>>,
        bool,
    ),
    Exit,
}

/// A thread worker that manages the search. It receives search parameters and a board state and
/// sends the best move to the UCI output.
pub(crate) struct SearchThread {
    sender: Sender<SearchThreadValue>,
    handle: Option<JoinHandle<()>>,
    stop_search_flag: Arc<AtomicBool>,
    is_searching: Arc<AtomicBool>,
}

impl SearchThread {
    /// Creates a new [`SearchThread`]. The search thread is responsible for managing the search.
    /// When the search thread is created, the thread loop starts and begins to wait for search parameters.
    pub(crate) fn new() -> SearchThread {
        let (sender, receiver) = mpsc::channel();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let is_searching = Arc::new(AtomicBool::new(false));

        let stop_flag_clone = stop_flag.clone();
        let is_searching_clone = is_searching.clone();

        let handle = std::thread::Builder::new()
            .name("bk-search-thread".to_string())
            .stack_size(8 * 1024 * 1024) // 8 MiB
            .spawn(move || {
                let mut stdout = std::io::stdout();
                'search_loop: loop {
                    let value = receiver.recv().unwrap();
                    match value {
                        SearchThreadValue::Params(mut board, params, ttable, history, is_debug) => {
                            let mut tt = ttable.lock().unwrap();
                            let mut hist_table = history.lock().unwrap();
                            let flag = stop_flag.clone();
                            is_searching.store(true, Ordering::Relaxed);
                            let result = if is_debug {
                                Search::<LogDebug>::new(&params, &mut tt, &mut hist_table)
                                    .search(&mut board, Some(flag))
                            } else {
                                Search::<LogInfo>::new(&params, &mut tt, &mut hist_table)
                                    .search(&mut board, Some(flag))
                            };
                            is_searching.store(false, Ordering::Relaxed);
                            let best_move = result.best_move;
                            let move_output = UciResponse::BestMove {
                                bestmove: best_move
                                    .map(|bot_move| move_to_uci_move(&bot_move).to_string()),
                                ponder: None,
                            };
                            writeln!(
                                stdout,
                                "{move_output}",
                                // TODO: Ponder
                            )
                            .unwrap();
                        }

                        SearchThreadValue::Exit => {
                            break 'search_loop;
                        }
                    }
                }
            })
            .unwrap();

        SearchThread {
            sender,
            handle: Some(handle),
            stop_search_flag: stop_flag_clone,
            is_searching: is_searching_clone,
        }
    }

    /// Exits the search thread. This will stop the search thread and join it.
    pub(crate) fn exit(&mut self) {
        self.stop_search();
        self.sender.send(SearchThreadValue::Exit).unwrap();
        self.handle.take().unwrap().join().unwrap();
    }

    /// Stops the current search if any is in progress.
    pub(crate) fn stop_search(&self) {
        self.stop_search_flag.store(true, Ordering::Relaxed);
    }

    /// Starts a new search with the given parameters and board state.
    pub(crate) fn start_search<Log: LogLevel>(
        &self,
        board: &Board,
        params: SearchParameters,
        ttable: Arc<Mutex<TranspositionTable>>,
        history_table: Arc<Mutex<HistoryTable>>,
    ) {
        self.stop_search_flag.store(false, Ordering::Relaxed);
        self.sender
            .send(SearchThreadValue::Params(
                board.clone(),
                params,
                ttable,
                history_table,
                Log::DEBUG,
            ))
            .unwrap();
    }

    pub(crate) fn is_searching(&self) -> bool {
        self.is_searching.load(Ordering::Relaxed)
    }
}
