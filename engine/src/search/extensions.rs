use crate::{
    log_level::LogLevel, score::ScoreType, search::Search, tuneable::CHECK_EXTENSION_DEPTH,
};

impl<'a, Log: LogLevel> Search<'a, Log> {
    pub(crate) fn extension_value(&self, is_in_check: bool) -> ScoreType {
        let mut extension = 0 as ScoreType;

        // --------------------------------------------------------------------------------------------------------
        // Check extension: If in check, increase the depth we search.
        // --------------------------------------------------------------------------------------------------------
        if is_in_check {
            extension += CHECK_EXTENSION_DEPTH
        }

        extension
    }
}
