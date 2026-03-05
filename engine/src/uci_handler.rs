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
        mpsc::Receiver,
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
    UciMove {
        src: square_index_to_uci_square(mv.from()),
        dst: square_index_to_uci_square(mv.to()),
        promote: mv
            .promotion_piece()
            .map(|p| uci_parser::Piece::from_str(&p.as_char().to_string()).unwrap()),
    }
}

pub struct UciHandler<Writable: Write> {
    engine: Engine,
    stop_flag: Arc<AtomicBool>,
    output: Writable,
}

impl UciHandler<io::Stdout> {
    pub fn new() -> Self {
        Self::with_output(io::stdout())
    }
}

impl<W: Write> UciHandler<W> {
    pub fn with_output(output: W) -> Self {
        Self {
            engine: Engine::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            output,
        }
    }

    /// Run the UCI protocol loop. This blocks until a `quit` command is received.
    pub fn run(&mut self) -> anyhow::Result<()> {
        writeln!(self.output, "{}", About::BANNER)?;
        writeln!(
            self.output,
            "{} {} by {} <{}>",
            About::NAME,
            About::VERSION,
            About::AUTHORS,
            About::EMAIL
        )?;

        let mut input_handler = InputHandler::new(Arc::clone(&self.stop_flag));
        self.dispatch_loop(input_handler.receiver())?;
        input_handler.stop();
        Ok(())
    }

    pub(crate) fn dispatch_loop(
        &mut self,
        receiver: &Receiver<CommandProxy>,
    ) -> anyhow::Result<()> {
        'engine_loop: while let Ok(command) = receiver.recv() {
            match command {
                CommandProxy::Uci(uci_command) => match uci_command {
                    UciCommand::Debug(debug) => {
                        self.engine.set_debug(debug);
                    }
                    UciCommand::Quit => {
                        break 'engine_loop;
                    }
                    UciCommand::IsReady => {
                        writeln!(self.output, "{}", UciResponse::<String>::ReadyOk)?;
                    }
                    UciCommand::Uci => {
                        let name = UciResponse::Name(format!("{} {}", About::NAME, About::VERSION));
                        let authors = UciResponse::Author(About::AUTHORS.to_string());

                        let options = vec![
                            UciOption::<&str, i32>::spin("Hash", 16, 1, 1024),
                            UciOption::<&str, i32>::spin("Threads", 1, 1, 1),
                        ];

                        writeln!(self.output, "{name}")?;
                        writeln!(self.output, "{authors}")?;
                        for option in options {
                            writeln!(self.output, "{}", UciResponse::Option(option))?;
                        }
                        writeln!(self.output, "{}", UciResponse::<String>::UciOk)?;
                    }
                    UciCommand::UciNewGame => {
                        self.engine.new_game();
                    }
                    UciCommand::Position { fen, moves } => {
                        let move_strings: Vec<String> =
                            moves.iter().map(|m| m.to_string()).collect();
                        let result = self.engine.set_position(fen.as_deref(), &move_strings);
                        if let Err(e) = result {
                            eprintln!("Failed to set engine position: {e}");
                        }
                    }
                    UciCommand::Go(search_options) => {
                        let info = UciInfo::default()
                            .string(format!("searching {}", self.engine.board().to_fen()));
                        writeln!(self.output, "{}", UciResponse::info(info))?;

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
                        writeln!(self.output, "{move_output}")?;
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
                        // The input handler thread set stop_flag when it read "stop"; the synchronous
                        // search already polled the flag and returned. Nothing to do here.
                    }
                    _ => {}
                },
                CommandProxy::Engine(engine_command) => match engine_command {
                    EngineCommand::HashInfo => {
                        writeln!(
                            self.output,
                            "full: {:.2}% hits: {} access: {} collisions: {} cap: {}",
                            self.engine.tt_fullness(),
                            self.engine.tt_hits(),
                            self.engine.tt_accesses(),
                            self.engine.tt_collisions(),
                            self.engine.tt_size(),
                        )?;
                    }
                    EngineCommand::History => {
                        self.engine
                            .history_table()
                            .print_for_side(self.engine.board().side_to_move());
                    }
                    EngineCommand::Perft(depth) => {
                        let nodes = self.engine.perft(depth);
                        writeln!(self.output, "info nodes {}", nodes)?;
                    }
                },
            }
        }

        Ok(())
    }
}

impl Default for UciHandler<io::Stdout> {
    fn default() -> Self {
        UciHandler::new()
    }
}
