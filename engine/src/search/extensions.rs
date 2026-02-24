use chess::board::Board;

use crate::{
    log_level::LogLevel, score::ScoreType, search::Search, tuneable::CHECK_EXTENSION_DEPTH,
};

impl<'a, Log: LogLevel> Search<'a, Log> {
    pub(crate) fn extension_value(&self, board: &Board) -> ScoreType {
        let mut extension = 0 as ScoreType;

        if board.is_in_check(&self.move_gen) {
            extension += CHECK_EXTENSION_DEPTH
        }

        extension
    }
}
