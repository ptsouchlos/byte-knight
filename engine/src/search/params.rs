use crate::tuneable::{
    lmp_base, lmp_improvement_divisor, lmp_improvement_max, lmp_improvement_min, nmp_depth_divisor,
    nmp_depth_reduction, nmp_eval_margin_div, nmp_eval_margin_max, nmp_improving_bonus,
};

/// Calculates the NMP reduction for a given depth with the given input parameters.
/// # Arguments
/// - `depth`: The current search depth
/// - `eval_margin`: The current magin between the current eval and the previous eval.
/// - `improving`: Boolean that indicates if the static evaluation is currently improving.
///
/// # Returns
/// The NMP depth reduction amount.
#[inline]
pub(crate) fn nmp_reduction(depth: i32, eval_margin: i32, improving: bool) -> i32 {
    let base = nmp_depth_reduction();
    let improving_bonus = if improving { nmp_improving_bonus() } else { 0 };
    let depth_bonus = depth / nmp_depth_divisor();
    let eval_bonus = (eval_margin / nmp_eval_margin_div()).min(nmp_eval_margin_max());
    base + improving_bonus + depth_bonus + eval_bonus
}

#[inline]
pub(crate) fn late_move_threshold(depth: i32, improvement: i32) -> i32 {
    let clamped_improvement = improvement.clamp(lmp_improvement_min(), lmp_improvement_max());
    let adjustment = clamped_improvement / lmp_improvement_divisor();
    lmp_base() + depth * depth + adjustment
}
