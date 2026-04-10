use std::ops::{Deref, DerefMut};

use crate::score::LargeScoreType;
use chess::definitions::MAX_MOVE_LIST_SIZE;
use chess::move_list::MoveList;
use chess::moves::Move;
use chess::util::ArrayVecList;

#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct ScoredMove {
    pub(crate) mv: Move,
    pub(crate) score: LargeScoreType,
}

pub(crate) struct ScoredMoveList {
    inner: ArrayVecList<ScoredMove, MAX_MOVE_LIST_SIZE>,
}

impl Default for ScoredMoveList {
    fn default() -> Self {
        Self {
            inner: ArrayVecList::new(),
        }
    }
}

impl Deref for ScoredMoveList {
    type Target = ArrayVecList<ScoredMove, MAX_MOVE_LIST_SIZE>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for ScoredMoveList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl ScoredMoveList {
    pub(crate) fn from_move_list(
        move_list: &MoveList,
        scoring_fn: impl Fn(&Move) -> LargeScoreType,
    ) -> Self {
        let mut list = ScoredMoveList::default();

        for mv in move_list.iter() {
            list.push(ScoredMove {
                mv: *mv,
                score: scoring_fn(mv),
            });
        }

        list
    }
}
