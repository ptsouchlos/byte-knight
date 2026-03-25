use crate::tuneable::{lmp_base, lmp_scale};

#[inline]
pub(crate) fn late_move_threshold(depth: i32) -> i32 {
    (lmp_base() + depth * lmp_scale()) / 10
}
