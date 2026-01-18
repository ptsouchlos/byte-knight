// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::{board_state::BoardState, definitions::MAX_MOVES};

/// A struct that holds the history of the board states
#[derive(Debug)]
pub(crate) struct BoardHistory {
    board_states: Vec<BoardState>,
}

impl Clone for BoardHistory {
    fn clone(&self) -> Self {
        Self {
            board_states: self.board_states.clone(),
        }
    }
}

impl BoardHistory {
    pub fn new() -> Self {
        BoardHistory {
            board_states: Vec::with_capacity(MAX_MOVES),
        }
    }

    /// Push a board state to the history list
    pub fn push(&mut self, board_state: BoardState) {
        self.board_states.push(board_state);
    }

    /// Pop a board state from the history list
    pub fn pop(&mut self) -> Option<BoardState> {
        self.board_states.pop()
    }

    /// Get an iterator to the board history
    pub fn iter(&self) -> std::slice::Iter<'_, BoardState> {
        self.board_states.iter()
    }
}
