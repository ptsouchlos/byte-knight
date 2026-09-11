use std::ops::{Index, IndexMut};

use chess::{bitboard::Bitboard, moves::Move, pieces::Piece};

use crate::{defs::MAX_PLY, score::Score};

#[derive(Copy, Clone)]
pub struct Node {
    pub static_eval: i32,
    pub threats: Bitboard,
    pub mv: Option<Move>,
    pub piece: Option<Piece>,
}

/// Represents the variation in the search tree currently being searched. Is updated every time the
/// node currently being searched changes.
pub struct NodeStack {
    data: [Node; (MAX_PLY + 8) as usize],
}

impl NodeStack {
    /// Record a move/piece pair for the given ply.
    pub(crate) fn record_move(&mut self, mv: Move, pc: Piece, ply: usize) {
        self.data[ply].mv = Some(mv);
        self.data[ply].piece = Some(pc);
    }

    /// Clear the move/piece pair for the given ply.
    pub(crate) fn clear_move(&mut self, ply: usize) {
        self.data[ply].mv = None;
        self.data[ply].piece = None;
    }
}

impl Default for NodeStack {
    fn default() -> Self {
        NodeStack {
            data: [Node {
                static_eval: -Score::INF.0 as i32,
                threats: Bitboard::default(),
                mv: None,
                piece: None,
            }; (MAX_PLY + 8) as usize],
        }
    }
}

impl Index<usize> for NodeStack {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        unsafe { self.data.get_unchecked(index) }
    }
}

impl IndexMut<usize> for NodeStack {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { self.data.get_unchecked_mut(index) }
    }
}
