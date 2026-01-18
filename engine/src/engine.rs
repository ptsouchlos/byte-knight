// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use chess::board::Board;
use uci_parser::{UciCommand, UciInfo, UciOption, UciResponse};

use crate::{
    defs::About,
    history_table::HistoryTable,
    input_handler::{CommandProxy, EngineCommand, InputHandler},
    log_level::{LogDebug, LogInfo, LogLevel},
    search::SearchParameters,
    search_thread::SearchThread,
    ttable::{self, TranspositionTable},
};

pub struct ByteKnight {
    input_handler: InputHandler,
    search_thread: SearchThread,
    transposition_table: Arc<Mutex<TranspositionTable>>,
    history_table: Arc<Mutex<HistoryTable>>,
    debug: bool,
}

impl ByteKnight {
    pub fn new() -> ByteKnight {
        ByteKnight {
            input_handler: InputHandler::new(),
            search_thread: SearchThread::new(),
            transposition_table: Default::default(),
            history_table: Default::default(),
            debug: false,
        }
    }

    fn clear_hash_tables(&mut self) {
        if let Ok(tt) = self.transposition_table.lock().as_mut() {
            tt.clear();
        }

        if let Ok(ht) = self.history_table.lock().as_mut() {
            ht.clear();
        }
    }

    /// Run the engine loop. This will block until the engine is told to quit by the input handler.
    pub fn run(&mut self) -> anyhow::Result<()> {
        println!("{}", About::BANNER);
        println!(
            "{} {} by {} <{}>",
            About::NAME,
            About::VERSION,
            About::AUTHORS,
            About::EMAIL
        );
        let stdout: io::Stdout = io::stdout();
        let mut board = Board::default_board();
        let move_gen = chess::move_generation::MoveGenerator::new();
        'engine_loop: while let Ok(command) = &self.input_handler.receiver().recv() {
            let mut stdout = stdout.lock();

            match command {
                CommandProxy::Uci(uci_command) => match uci_command {
                    UciCommand::Debug(debug) => {
                        self.debug = *debug;
                    }
                    UciCommand::Quit => {
                        // clean up
                        self.search_thread.exit();
                        self.input_handler.exit();
                        break 'engine_loop;
                    }
                    UciCommand::IsReady => {
                        writeln!(stdout, "{}", UciResponse::<String>::ReadyOk).unwrap();
                    }
                    UciCommand::Uci => {
                        let name = UciResponse::Name(format!("{} {}", About::NAME, About::VERSION));
                        let authors = UciResponse::Author(About::AUTHORS.to_string());

                        let options = vec![
                            UciOption::<&str, i32>::spin("Hash", 16, 1, 1024),
                            UciOption::<&str, i32>::spin("Threads", 1, 1, 1),
                        ];

                        for option in options {
                            writeln!(stdout, "{}", UciResponse::Option(option)).unwrap();
                        }
                        writeln!(stdout, "{name}").unwrap();
                        writeln!(stdout, "{authors}").unwrap();
                        writeln!(stdout, "{}", UciResponse::<String>::UciOk).unwrap();
                    }
                    UciCommand::UciNewGame => {
                        board = Board::default_board();
                        self.clear_hash_tables();
                    }
                    UciCommand::Position { fen, moves } => {
                        match fen {
                            None => {
                                board = Board::default_board();
                            }
                            Some(fen) => {
                                board = Board::from_fen(fen.as_str()).unwrap();
                            }
                        }

                        for mv in moves {
                            board.make_uci_move(&mv.to_string()).unwrap();
                        }
                    }
                    UciCommand::Go(search_options) => {
                        if self.search_thread.is_searching() {
                            eprintln!("Attempting to start a search while already searching");
                            self.search_thread.stop_search();
                        }

                        let info =
                            UciInfo::default().string(format!("searching {}", board.to_fen()));
                        writeln!(stdout, "{}", UciResponse::info(info)).unwrap();

                        // create the search parameters
                        let search_params = SearchParameters::new(search_options, &board);
                        if self.debug {
                            self.start_search::<LogDebug>(board.clone(), search_params);
                        } else {
                            self.start_search::<LogInfo>(board.clone(), search_params);
                        }
                    }
                    UciCommand::SetOption { name, value } => {
                        if name.to_lowercase() == "hash"
                            && let Some(val) = value
                        {
                            // set the hash size, making sure it is within the bounds we have set.
                            if let Ok(hash_size) = val.parse::<usize>() {
                                if hash_size < ttable::MIN_TABLE_SIZE_MB {
                                    eprintln!(
                                        "Hash size too small. Must be at least {} MB",
                                        ttable::MIN_TABLE_SIZE_MB
                                    );
                                    continue;
                                } else if hash_size > ttable::MAX_TABLE_SIZE_MB {
                                    eprintln!(
                                        "Hash size too large. Must be at most {} MB",
                                        ttable::MAX_TABLE_SIZE_MB
                                    );
                                    continue;
                                }

                                self.transposition_table = Arc::new(Mutex::new(
                                    TranspositionTable::from_size_in_mb(hash_size),
                                ));
                            }
                        }
                    }
                    UciCommand::Stop => {
                        self.search_thread.stop_search();
                    }
                    _ => {}
                },
                CommandProxy::Engine(engine_command) => match engine_command {
                    EngineCommand::HashInfo => {
                        if let Ok(tt) = self.transposition_table.lock() {
                            writeln!(
                                stdout,
                                "full: {:.2}% hits: {} access: {} collisions: {} cap: {}",
                                tt.fullness(),
                                tt.hits,
                                tt.accesses,
                                tt.collisions,
                                tt.size(),
                            )
                            .unwrap();
                        }
                    }
                    EngineCommand::History => {
                        if let Ok(ht) = self.history_table.lock() {
                            ht.print_for_side(board.side_to_move());
                        }
                    }
                    EngineCommand::Perft(depth) => {
                        let nodes =
                            chess::perft::perft(&mut board, &move_gen, *depth as usize, false)
                                .unwrap();
                        writeln!(stdout, "info nodes {}", nodes).unwrap();
                    }
                },
            }
        }

        Ok(())
    }

    fn start_search<Log: LogLevel>(&self, board: Board, params: SearchParameters) {
        // send them and the current board to the search thread
        self.search_thread.start_search::<Log>(
            &board,
            params,
            Arc::clone(&self.transposition_table),
            Arc::clone(&self.history_table),
        );
    }
}

impl Default for ByteKnight {
    fn default() -> Self {
        ByteKnight::new()
    }
}
