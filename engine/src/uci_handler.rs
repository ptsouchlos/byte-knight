// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    io::{self, Write},
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use chess::{moves::Move, pieces::SQUARE_NAME};
use uci_parser::{UciCommand, UciInfo, UciMove, UciOption, UciResponse};

use crate::{
    defs::About,
    engine::Engine,
    input_handler::{CommandProxy, EngineCommand, InputHandler},
    search::SearchParameters,
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

pub struct UciHandler {
    engine: Engine,
    stop_flag: Arc<AtomicBool>,
}

impl UciHandler {
    pub fn new() -> Self {
        Self {
            engine: Engine::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Run the UCI protocol loop. This blocks until a `quit` command is received.
    pub fn run(&mut self) -> anyhow::Result<()> {
        println!("{}", About::BANNER);
        println!(
            "{} {} by {} <{}>",
            About::NAME,
            About::VERSION,
            About::AUTHORS,
            About::EMAIL
        );

        let mut input_handler = InputHandler::new(Arc::clone(&self.stop_flag));
        let stdout_handle = io::stdout();

        'engine_loop: while let Ok(command) = input_handler.receiver().recv() {
            let mut stdout = stdout_handle.lock();

            match command {
                CommandProxy::Uci(uci_command) => match uci_command {
                    UciCommand::Debug(debug) => {
                        self.engine.set_debug(debug);
                    }
                    UciCommand::Quit => {
                        input_handler.stop();
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
                        self.engine.new_game();
                    }
                    UciCommand::Position { fen, moves } => {
                        let move_strings: Vec<String> =
                            moves.iter().map(|m| m.to_string()).collect();
                        self.engine.set_position(fen.as_deref(), &move_strings);
                    }
                    UciCommand::Go(search_options) => {
                        let info = UciInfo::default()
                            .string(format!("searching {}", self.engine.board().to_fen()));
                        writeln!(stdout, "{}", UciResponse::info(info)).unwrap();
                        // Drop stdout lock before search — search prints UCI info lines directly
                        drop(stdout);

                        let search_params =
                            SearchParameters::new(&search_options, self.engine.board());
                        // Reset the stop flag before starting a new search
                        self.stop_flag.store(false, Ordering::Relaxed);
                        let result = self
                            .engine
                            .search(search_params, Arc::clone(&self.stop_flag));

                        let best_move = result.best_move;
                        let move_output = UciResponse::BestMove {
                            bestmove: best_move
                                .map(|bot_move| move_to_uci_move(&bot_move).to_string()),
                            ponder: None,
                        };
                        let mut stdout = stdout_handle.lock();
                        writeln!(stdout, "{move_output}").unwrap();
                    }
                    UciCommand::SetOption { name, value } => {
                        if name.to_lowercase() == "hash"
                            && let Some(val) = value
                            && let Ok(hash_size) = val.parse::<usize>()
                            && let Err(e) = self.engine.set_hash_size(hash_size)
                        {
                            eprintln!("{e}");
                        }
                    }
                    UciCommand::Stop => {
                        // Stop flag already set by input handler; search already returned.
                    }
                    _ => {}
                },
                CommandProxy::Engine(engine_command) => match engine_command {
                    EngineCommand::HashInfo => {
                        writeln!(
                            stdout,
                            "full: {:.2}% hits: {} access: {} collisions: {} cap: {}",
                            self.engine.tt_fullness(),
                            self.engine.tt_hits(),
                            self.engine.tt_accesses(),
                            self.engine.tt_collisions(),
                            self.engine.tt_size(),
                        )
                        .unwrap();
                    }
                    EngineCommand::History => {
                        self.engine
                            .history_table()
                            .print_for_side(self.engine.board().side_to_move());
                    }
                    EngineCommand::Perft(depth) => {
                        let nodes = self.engine.perft(depth);
                        writeln!(stdout, "info nodes {}", nodes).unwrap();
                    }
                },
            }
        }

        Ok(())
    }
}

impl Default for UciHandler {
    fn default() -> Self {
        UciHandler::new()
    }
}
