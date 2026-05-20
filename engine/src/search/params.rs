use crate::tuneable::{
    lmp_base, lmp_improvement_divisor, lmp_improvement_max, lmp_improvement_min,
};
#[inline]
pub(crate) fn late_move_threshold(depth: i32, improvement: i32) -> i32 {
    let scaled_improvement = improvement / lmp_improvement_divisor();
    let adjustment = scaled_improvement.clamp(lmp_improvement_min(), lmp_improvement_max());
    lmp_base() + depth * depth + adjustment
}
