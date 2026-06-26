// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

//! This module contains all history tables and consolidates them into a Histories object.

use chess::{moves::Move, pieces::Piece, side::Side};

use crate::{history::quiet_history::QuietHistory, score::LargeScoreType};

pub mod quiet_history;

/// Holds all history tables for the engine.
/// Credit to the author of [hobbes](https://github.com/kelseyde/hobbes-chess-engine) for this setup (kelseyde)
#[derive(Default)]
pub struct Histories {
    pub quiet_history: QuietHistory,
}

impl Histories {
    pub(crate) fn get(&self, side: Side, piece: Piece, mv: Move) -> LargeScoreType {
        self.quiet_history.get(side, piece, mv)
    }

    pub(crate) fn update(&mut self, side: Side, piece: Piece, mv: Move, bonus: LargeScoreType) {
        self.quiet_history.update(side, piece, mv, bonus)
    }

    pub fn clear(&mut self) {
        self.quiet_history.clear();
    }
}
