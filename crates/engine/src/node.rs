use std::ops::{Index, IndexMut};

use chess::bitboard::Bitboard;

use crate::{defs::MAX_PLY, score::Score};

#[derive(Copy, Clone)]
pub struct Node {
    pub static_eval: i32,
    pub threats: Bitboard,
}

/// Represents the variation in the search tree currently being searched. Is updated every time the
/// node currently being searched changes.
pub struct NodeStack {
    data: [Node; (MAX_PLY + 8) as usize],
}

impl Default for NodeStack {
    fn default() -> Self {
        NodeStack {
            data: [Node {
                static_eval: -Score::INF.0 as i32,
                threats: Bitboard::default(),
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
