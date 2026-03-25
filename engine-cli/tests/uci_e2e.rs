// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::Duration,
};

struct EngineProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl EngineProcess {
    fn spawn() -> Self {
        // Use the engine target.
        // See https://doc.rust-lang.org/cargo/reference/cargo-targets.html#integration-tests
        let mut child = Command::new(env!("CARGO_BIN_EXE_byte-knight"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn engine binary");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, cmd: &str) {
        writeln!(self.stdin, "{cmd}").unwrap();
        self.stdin.flush().unwrap();
    }

    fn expect_line_containing(&mut self, needle: &str) -> String {
        let deadline = std::time::Instant::now() + Duration::from_secs(30);
        loop {
            let mut line = String::new();
            self.stdout.read_line(&mut line).unwrap();
            let line = line.trim_end().to_string();
            if line.contains(needle) {
                return line;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "timed out waiting for line containing {needle:?}, last line: {line:?}"
            );
        }
    }

    fn quit(mut self) {
        self.send("quit");
        self.child.wait().unwrap();
    }
}

#[test]
fn uci_handshake() {
    let mut e = EngineProcess::spawn();
    e.send("uci");
    e.expect_line_containing("id name");
    e.expect_line_containing("id author");
    e.expect_line_containing("uciok");
    e.quit();
}

#[test]
fn isready_responds_readyok() {
    let mut e = EngineProcess::spawn();
    e.send("isready");
    e.expect_line_containing("readyok");
    e.quit();
}

#[test]
fn go_depth1_returns_bestmove() {
    let mut e = EngineProcess::spawn();
    e.send("ucinewgame");
    e.send("position startpos");
    e.send("go depth 1");
    e.expect_line_containing("bestmove");
    e.quit();
}

#[test]
fn position_with_moves_then_search() {
    let mut e = EngineProcess::spawn();
    e.send("ucinewgame");
    e.send("position startpos moves e2e4 e7e5");
    e.send("go depth 1");
    e.expect_line_containing("bestmove");
    e.quit();
}

#[test]
fn setoption_hash() {
    let mut e = EngineProcess::spawn();
    e.send("setoption name Hash value 32");
    e.send("isready");
    e.expect_line_containing("readyok");
    e.quit();
}

#[test]
fn ucinewgame_resets_state() {
    let mut e = EngineProcess::spawn();
    e.send("ucinewgame");
    e.send("position startpos moves e2e4");
    e.send("ucinewgame");
    e.send("go depth 1");
    e.expect_line_containing("bestmove");
    e.quit();
}

#[test]
fn promotion_position() {
    let mut e = EngineProcess::spawn();
    // White pawn on e7 to promote; Black king far away — bestmove must be a promotion
    e.send("ucinewgame");
    e.send("position fen 8/4P3/8/8/8/8/8/4K2k w - - 0 1");
    e.send("go depth 1");
    let bestmove = e.expect_line_containing("bestmove");
    // Promotion moves end with piece character (q/r/b/n)
    assert!(
        bestmove.ends_with('q')
            || bestmove.ends_with('r')
            || bestmove.ends_with('b')
            || bestmove.ends_with('n'),
        "expected promotion move, got: {bestmove}"
    );
    e.quit();
}
