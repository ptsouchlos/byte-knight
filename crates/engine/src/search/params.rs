use crate::tuneable::{
    cap_hist_bonus_offset, cap_hist_bonus_scale, cap_hist_malus_offset, cap_hist_malus_scale,
    cont_hist_1_bonus_offset, cont_hist_1_bonus_scale, cont_hist_1_malus_offset,
    cont_hist_1_malus_scale, cont_hist_2_bonus_offset, cont_hist_2_bonus_scale,
    cont_hist_2_malus_offset, cont_hist_2_malus_scale, lmp_base, lmp_improvement_divisor,
    lmp_improvement_max, lmp_improvement_min, nmp_depth_divisor, nmp_depth_reduction,
    nmp_improving_bonus, quiet_hist_bonus_offset, quiet_hist_bonus_scale, quiet_hist_malus_offset,
    quiet_hist_malus_scale,
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

// Defines a history bonus function using the given tuneable values
macro_rules! history_bonus {
    (
        $bonus_fn:ident ($bonus_scale:ident, $bonus_offset:ident)
    ) => {
        #[inline]
        pub(crate) fn $bonus_fn(depth: i16) -> i16 {
            depth
                .saturating_mul($bonus_scale() as i16)
                .saturating_sub($bonus_offset() as i16) as i16
        }
    };
}

// Defines a history malus function using the given tuneable values
macro_rules! history_malus {
    (
        $malus_fn:ident ($malus_scale:ident, $malus_offset:ident)
    ) => {
        #[inline]
        pub(crate) fn $malus_fn(depth: i16) -> i16 {
            -(depth
                .saturating_mul($malus_scale() as i16)
                .saturating_sub($malus_offset() as i16)) as i16
        }
    };
}

history_bonus!(quiet_history_bonus(
    quiet_hist_bonus_scale,
    quiet_hist_bonus_offset
));

history_malus!(quiet_history_malus(
    quiet_hist_malus_scale,
    quiet_hist_malus_offset
));

history_bonus!(capture_history_bonus(
    cap_hist_bonus_scale,
    cap_hist_bonus_offset
));

history_malus!(capture_history_malus(
    cap_hist_malus_scale,
    cap_hist_malus_offset
));

history_bonus!(cont_hist_1_bonus(
    cont_hist_1_bonus_scale,
    cont_hist_1_bonus_offset
));

history_malus!(cont_hist_1_malus(
    cont_hist_1_malus_scale,
    cont_hist_1_malus_offset
));

history_bonus!(cont_hist_2_bonus(
    cont_hist_2_bonus_scale,
    cont_hist_2_bonus_offset
));

history_malus!(cont_hist_2_malus(
    cont_hist_2_malus_scale,
    cont_hist_2_malus_offset
));

#[cfg(test)]
mod tests {
    use crate::{
        defs::MAX_DEPTH,
        search::params::{
            capture_history_bonus, capture_history_malus, cont_hist_1_bonus, cont_hist_1_malus,
            cont_hist_2_bonus, cont_hist_2_malus, quiet_history_bonus, quiet_history_malus,
        },
    };

    #[test]
    fn history_bonus_malus_sign() {
        for depth in 1..MAX_DEPTH as i16 {
            for f in [
                quiet_history_bonus,
                capture_history_bonus,
                cont_hist_1_bonus,
                cont_hist_2_bonus,
            ] {
                assert!(f(depth) > 0);
            }
            for f in [
                quiet_history_malus,
                capture_history_malus,
                cont_hist_1_malus,
                cont_hist_2_malus,
            ] {
                assert!(f(depth) < 0);
            }
        }
    }
}
