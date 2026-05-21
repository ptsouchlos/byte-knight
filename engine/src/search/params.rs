use crate::tuneable::{
    lmp_base, lmp_improvement_divisor, lmp_improvement_max, lmp_improvement_min,
    nmp_depth_reduction, nmp_improving_bonus, nmp_not_improving_margin,
};

#[inline]
pub(crate) fn nmp_reduction(improving: bool) -> i32 {
    nmp_depth_reduction() + if improving { nmp_improving_bonus() } else { 0 }
}

pub(crate) fn nmp_margin(improving: bool) -> i32 {
    if improving {
        0
    } else {
        nmp_not_improving_margin()
    }
}

#[inline]
pub(crate) fn late_move_threshold(depth: i32, improvement: i32) -> i32 {
    let clamped_improvement = improvement.clamp(lmp_improvement_min(), lmp_improvement_max());
    let adjustment = clamped_improvement / lmp_improvement_divisor();
    lmp_base() + depth * depth + adjustment
}
