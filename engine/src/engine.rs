// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::sync::{Arc, atomic::AtomicBool};

use chess::board::Board;
use chess::move_generation::MoveGenerator;

use crate::{
    history_table::HistoryTable,
    log_level::{LogDebug, LogInfo},
    search::{Search, SearchParameters, SearchResult},
    ttable::{self, TranspositionTable},
};

pub struct Engine {
    board: Board,
    transposition_table: TranspositionTable,
    history_table: HistoryTable,
    move_gen: MoveGenerator,
    debug: bool,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            board: Board::default_board(),
            transposition_table: TranspositionTable::default(),
            history_table: HistoryTable::default(),
            move_gen: MoveGenerator::new(),
            debug: false,
        }
    }

    pub fn new_game(&mut self) {
        self.board = Board::default_board();
        self.transposition_table.clear();
        self.history_table.clear();
    }

    pub fn set_position(&mut self, fen: Option<&str>, moves: &[String]) -> anyhow::Result<()> {
        match fen {
            Some(fen) => {
                self.board = Board::from_fen(fen)?;
            }
            None => {
                self.board = Board::default_board();
            }
        }

        for mv in moves {
            self.board.make_uci_move(mv)?;
        }

        Ok(())
    }

    pub fn set_hash_size(&mut self, mb: usize) -> anyhow::Result<()> {
        if mb < ttable::MIN_TABLE_SIZE_MB {
            anyhow::bail!(
                "Hash size too small. Must be at least {} MB",
                ttable::MIN_TABLE_SIZE_MB
            );
        }
        if mb > ttable::MAX_TABLE_SIZE_MB {
            anyhow::bail!(
                "Hash size too large. Must be at most {} MB",
                ttable::MAX_TABLE_SIZE_MB
            );
        }
        self.transposition_table = TranspositionTable::from_size_in_mb(mb);
        Ok(())
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn search(&mut self, params: SearchParameters, stop_flag: Arc<AtomicBool>) -> SearchResult {
        if self.debug {
            Search::<LogDebug>::new(
                &params,
                &mut self.transposition_table,
                &mut self.history_table,
            )
            .search(&mut self.board, Some(stop_flag))
        } else {
            Search::<LogInfo>::new(
                &params,
                &mut self.transposition_table,
                &mut self.history_table,
            )
            .search(&mut self.board, Some(stop_flag))
        }
    }

    pub fn perft(&mut self, depth: u16) -> u64 {
        chess::perft::perft(&mut self.board, &self.move_gen, depth as usize, false).unwrap()
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn tt_fullness(&self) -> f64 {
        self.transposition_table.fullness()
    }

    pub fn tt_hits(&self) -> usize {
        self.transposition_table.hits
    }

    pub fn tt_accesses(&self) -> usize {
        self.transposition_table.accesses
    }

    pub fn tt_collisions(&self) -> usize {
        self.transposition_table.collisions
    }

    pub fn tt_size(&self) -> usize {
        self.transposition_table.size()
    }

    pub fn history_table(&self) -> &HistoryTable {
        &self.history_table
    }
}

impl Default for Engine {
    fn default() -> Self {
        Engine::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_resets_board() {
        let mut engine = Engine::new();
        engine.set_position(None, &["e2e4".to_string()]).unwrap();
        engine.new_game();
        assert_eq!(engine.board().to_fen(), chess::definitions::DEFAULT_FEN);
    }

    #[test]
    fn set_position_applies_moves() {
        let mut engine = Engine::new();
        engine.set_position(None, &["e2e4".to_string()]).unwrap();
        assert_ne!(engine.board().to_fen(), chess::definitions::DEFAULT_FEN);
    }

    #[test]
    fn set_hash_size_rejects_out_of_bounds() {
        let mut e = Engine::new();
        assert!(e.set_hash_size(0).is_err());
        assert!(e.set_hash_size(99999).is_err());
        assert!(e.set_hash_size(64).is_ok());
    }
}
