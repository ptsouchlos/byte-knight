use chess::{moves::Move, pieces::Piece, side::Side};

use crate::{history::quiet_history::QuietHistory, score::LargeScoreType};

pub mod quiet_history;

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
