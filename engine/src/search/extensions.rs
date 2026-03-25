use chess::{board::Board, move_generation};

use crate::{
    log_level::LogLevel, score::ScoreType, search::Search, tuneable::CHECK_EXTENSION_DEPTH,
};

impl<'a, Log: LogLevel> Search<'a, Log> {
    pub(crate) fn extension_value(&self, board: &Board) -> ScoreType {
        let mut extension = 0 as ScoreType;

        // --------------------------------------------------------------------------------------------------------
        // Check extension: If in check, increase the depth we search.
        // --------------------------------------------------------------------------------------------------------
        if move_generation::is_in_check(board) {
            extension += CHECK_EXTENSION_DEPTH
        }

        extension
    }
}
