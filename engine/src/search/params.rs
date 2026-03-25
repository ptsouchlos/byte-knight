use crate::tuneable::lmp_base;
#[inline]
pub(crate) fn late_move_threshold(depth: i32) -> i32 {
    lmp_base() + depth * depth
}
