// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    io::{BufRead, stdin},
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread::JoinHandle,
};
use uci_parser::UciCommand;

#[derive(Debug)]
pub(crate) enum EngineCommand {
    HashInfo,
    History,
    Perft(u16),
}

fn split_args(s: &str) -> Vec<String> {
    s.split_whitespace()
        .map(|part| part.trim().to_string())
        .collect()
}

impl FromStr for EngineCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let args = split_args(s);
        if args.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }
        let cmd = args[0].as_str();
        let depth = if args.len() > 1 {
            args[1].parse::<u16>().unwrap_or(4)
        } else {
            4
        };
        match cmd {
            "hash" => Ok(EngineCommand::HashInfo),
            "history" => Ok(EngineCommand::History),
            "perft" => Ok(EngineCommand::Perft(depth)),
            _ => Err(anyhow::anyhow!("Invalid engine command")),
        }
    }
}

pub(crate) enum CommandProxy {
    Uci(UciCommand),
    Engine(EngineCommand),
}

#[derive(Debug)]
pub(crate) struct InputHandler {
    handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
    receiver: Receiver<CommandProxy>,
}

impl InputHandler {
    /// Creates a new [`InputHandler`]. The input handler reads from stdin and sends the parsed UCI
    /// commands to the receiver end of the channel via the sender. Creating a new [`InputHandler`]
    /// spawns a new worker thread. The thread starts upon creation.
    ///
    /// # Panics
    ///
    /// Panics if there is an error spawning the worker thread.
    ///
    /// # Returns
    ///
    /// A new [`InputHandler`] instance.
    ///
    pub(crate) fn new() -> InputHandler {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let stdin = stdin();
            let mut input = stdin.lock().lines();
            while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(Ok(line)) = input.next() {
                    let engine_command = EngineCommand::from_str(line.as_str());

                    if let Ok(engine_command) = engine_command {
                        sender.send(CommandProxy::Engine(engine_command)).unwrap();
                    } else {
                        let command = UciCommand::from_str(line.as_str());
                        if let Ok(command) = command {
                            let cmd = command.clone();
                            sender.send(CommandProxy::Uci(cmd)).unwrap();
                            // manually break the loop if the command is "quit"
                            if command == UciCommand::Quit {
                                break;
                            }
                        } else {
                            eprintln!("info error: invalid command: {line}");
                        }
                    }
                } else {
                    eprintln!("info error: failed to read from stdin");
                }
            }
        });
        InputHandler {
            handle: Some(worker),
            stop_flag: stop_flag_clone,
            receiver,
        }
    }

    /// Stops the worker thread. This method blocks until the worker thread has stopped.
    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }

    pub fn receiver(&self) -> &Receiver<CommandProxy> {
        &self.receiver
    }

    /// Signal to the worker thread that it should stop.
    pub(crate) fn exit(&mut self) {
        self.stop();
    }
}
