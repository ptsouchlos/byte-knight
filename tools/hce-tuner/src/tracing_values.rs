// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

use std::cell::{Cell, RefCell};

use chess::{pieces::Piece, side::Side};
use engine::{
    hce_values::{GAME_PHASE_INC, GAME_PHASE_MAX},
    phased_score::PhasedScore,
    traits::EvalValues,
};

use crate::offsets::Offsets;

/// An `EvalValues` implementation that records which parameter indices are accessed
/// during evaluation. Used to extract feature index lists for HCE tuning without
/// re-implementing feature detection separately from the main engine evaluation.
pub(crate) struct TracingValues {
    white_indexes: RefCell<Vec<usize>>,
    black_indexes: RefCell<Vec<usize>>,
    game_phase: Cell<i32>,
}

impl TracingValues {
    pub(crate) fn new() -> Self {
        Self {
            white_indexes: RefCell::new(Vec::new()),
            black_indexes: RefCell::new(Vec::new()),
            game_phase: Cell::new(0),
        }
    }

    fn record(&self, side: Side, idx: usize) {
        match side {
            Side::White => self.white_indexes.borrow_mut().push(idx),
            Side::Black => self.black_indexes.borrow_mut().push(idx),
        }
    }

    /// Consume the tracer and return `(white_indexes, black_indexes, scaled_phase)`.
    pub(crate) fn into_trace(self) -> (Vec<usize>, Vec<usize>, f64) {
        let scaled = self.game_phase.get() as f64 / GAME_PHASE_MAX as f64;
        (
            self.white_indexes.into_inner(),
            self.black_indexes.into_inner(),
            scaled,
        )
    }
}

impl EvalValues for TracingValues {
    type ReturnScore = PhasedScore;

    fn psqt(&self, square: u8, piece: Piece, side: Side) -> PhasedScore {
        let idx = Offsets::offset_for_piece_and_square(square as usize, piece, side);
        self.record(side, idx);
        self.game_phase
            .set(self.game_phase.get() + GAME_PHASE_INC[piece as usize] as i32);
        PhasedScore::default()
    }

    fn passed_pawn_bonus(&self, square: u8, side: Side) -> PhasedScore {
        let idx = Offsets::offset_for_passed_pawn(square as usize, side);
        self.record(side, idx);
        PhasedScore::default()
    }

    fn doubled_pawn_value(&self, square: u8, side: Side) -> PhasedScore {
        let idx = Offsets::offset_for_doubled_pawn(square as usize, side);
        self.record(side, idx);
        PhasedScore::default()
    }

    fn isolated_pawn_value(&self, square: u8, side: Side) -> PhasedScore {
        let idx = Offsets::offset_for_isolated_pawn(square as usize, side);
        self.record(side, idx);
        PhasedScore::default()
    }

    fn bishop_pair_bonus_value(&self, side: Side) -> PhasedScore {
        let idx = Offsets::offset_for_bishop_pair();
        self.record(side, idx);
        PhasedScore::default()
    }

    fn king_safety_value(&self, piece: Piece, side: Side) -> PhasedScore {
        let idx = Offsets::offset_for_king_safety(piece);
        self.record(side, idx);
        PhasedScore::default()
    }

    fn threat_value(&self, piece: Piece, attacked_piece: Piece, side: Side) -> PhasedScore {
        let idx = Offsets::offset_for_threat(piece, attacked_piece);
        self.record(side, idx);
        PhasedScore::default()
    }
}
