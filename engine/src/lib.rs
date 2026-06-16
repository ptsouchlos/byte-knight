// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

#![deny(clippy::unused_result_ok)]
#![deny(clippy::panic)]
#![deny(clippy::expect_used)]
#![cfg_attr(
    all(nightly, target_arch = "aarch64"),
    feature(stdarch_aarch64_prefetch)
)]

pub mod aspiration_window;
pub mod defs;
pub mod engine;
pub mod evaluation;
pub mod hce_values;
pub mod history_table;
pub mod killers_table;
mod lmr;
pub mod log_level;
mod move_picker;
pub(crate) mod node;
pub(crate) mod node_types;
pub mod pawn_structure;
pub mod phased_score;
pub(crate) mod principle_variation;
pub mod score;
mod scored_move;
mod scored_move_list;
pub mod search;
pub mod see;
pub mod structure;
pub(crate) mod table;
pub mod thread_data;
pub mod traits;
pub mod ttable;
pub mod tuneable;
pub(crate) mod utils;
