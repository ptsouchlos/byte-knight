use crate::tuneable::{
    fp_base, fp_improving_margin, fp_scale, lmp_base, lmp_improvement_divisor, lmp_improvement_max,
    lmp_improvement_min, nmp_depth_divisor, nmp_depth_reduction, nmp_improving_bonus,
};

#[inline]
pub(crate) fn nmp_reduction(depth: i32, improving: bool) -> i32 {
    nmp_depth_reduction()
        + depth / nmp_depth_divisor()
        + if improving { nmp_improving_bonus() } else { 0 }
}

#[inline]
pub(crate) fn late_move_threshold(depth: i32, improvement: i32) -> i32 {
    let clamped_improvement = improvement.clamp(lmp_improvement_min(), lmp_improvement_max());
    let adjustment = clamped_improvement / lmp_improvement_divisor();
    lmp_base() + depth * depth + adjustment
}

#[inline]
pub(crate) fn fp_pruning_margin(depth: i32, improving: bool) -> i32 {
    (fp_base() + fp_scale() * depth) - improving as i32 * fp_improving_margin()
}
