// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

#![deny(clippy::unused_result_ok)]
#![deny(clippy::panic)]
#![deny(clippy::expect_used)]

pub mod attacks;
pub mod bitboard;
pub mod bitboard_helpers;
pub mod board;
pub mod board_state;
pub mod color;
pub mod definitions;
pub mod fen;
pub mod file;
pub mod legal_move_generation;
pub mod magics;
pub mod move_generation;
pub mod move_history;
pub mod move_list;
pub mod move_making;
pub mod moves;
pub mod non_slider_piece;
pub mod perft;
pub mod pext;
pub mod piece_category;
pub mod pieces;
pub mod rank;
pub mod rays;
pub mod side;
pub mod slider_pieces;
pub mod sliding_piece_attacks;
pub mod square;
pub mod zobrist;
