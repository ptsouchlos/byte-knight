// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::tunable_params;

#[rustfmt::skip]
tunable_params!(
    lmp_max_depth                = 5, 1..=10,            true;
    lmp_base                     = 1, 1..=10,            true;
    lmp_improvement_min          = -182, -300..=0,       true;
    lmp_improvement_max          = 223, 50..=300,        true;
    lmp_improvement_divisor      = 12, 2..=24,           true;
    see_pawn                     = 126, 25..=150,        true;
    see_knight                   = 219, 100..=300,       true;
    see_bishop                   = 267, 150..=450,       true;
    see_rook                     = 421, 250..=750,       true;
    see_queen                    = 701, 450..=1350,      true;
    qs_see_threshold             = -112, -160..=0,       true;
    qs_delta_margin              = 291, 120..=360,       true;
    see_tacticals_max_depth      = 9, 4..=10,            true;
    see_tacticals_margin         = 40, 0..=100,          true;
    fp_base                      = 105, 50..=150,        true;
    fp_scale                     = 56, 40..=120,         true;
    fp_max_depth                 = 6, 1..=6,             true;
    rfp_max_depth                = 4, 1..=10,            true;
    rfp_margin                   = 71, 40..=120,         true;
    rfp_improving_margin         = 31, 0..=60,           true;
    nmp_min_depth                = 2, 1..=6,             true;
    nmp_depth_reduction          = 2, 0..=6,             false;
    nmp_improving_bonus          = 1, 0..=4,             false;
    nmp_depth_divisor            = 4, 2..=8,             false;
    min_aspiration_depth         = 1, 1..=12,            false;
    aspiration_window            = 20, 10..=50,          true;
    iir_min_depth                = 5, 1..=10,            true;
    iir_depth_reduction          = 1, 1..=6,             false;
    razoring_scaling             = 397, 150..=550,       true;
    razoring_offset              = 451, 250..=750,       true;
    lmr_min_depth                = 1, 1..=6,             true;
    lmr_min_moves_seen           = 1, 1..=6,             true;
    cap_hist_bonus_scale         = 323, 150..=450,       true;
    cap_hist_bonus_offset        = 220, 100..=400,       true;
    cap_hist_malus_scale         = 338, 150..=450,       true;
    cap_hist_malus_offset        = 283, 100..=400,       true;
    movepick_mvv_scale           = 1605, 1024..=3072,    true;
    quiet_hist_bonus_scale       = 334, 150..=450,       true;
    quiet_hist_bonus_offset      = 247, 100..=400,       true;
    quiet_hist_malus_scale       = 394, 150..=450,       true;
    quiet_hist_malus_offset      = 245, 100..=400,       true;
    cont_hist_1_bonus_scale      = 268, 150..=450,       true;
    cont_hist_1_bonus_offset     = 226, 100..=400,       true;
    cont_hist_1_malus_scale      = 300, 150..=450,       true;
    cont_hist_1_malus_offset     = 187, 100..=400,       true;
    cont_hist_2_bonus_scale      = 252, 150..=450,       true;
    cont_hist_2_bonus_offset     = 251, 100..=400,       true;
    cont_hist_2_malus_scale      = 330, 150..=450,       true;
    cont_hist_2_malus_offset     = 220, 100..=400,       true;
    score_stability_threshold    = 11, 4..=24,           true;
    tm_node_stability_offset     = 14955, 12000..=16000, true;
    tm_node_stability_scale      = 12701, 12000..=16000, true;
    tm_eval_stability_offset     = 12430, 10000..=14000, true;
    tm_eval_stability_scale      = 477, 200..=600,       true;
    tm_eval_stability_min_scale  = 8709, 7000..=10000,   true;
    tm_best_move_offset          = 10558, 9000..=13000,  true;
    tm_best_move_scale           = 460, 300..=800,       true;
    tm_best_move_min_scale       = 5044, 4000..=6000,    true;
    lmr_offset                   = 247, 50..=350,        true;
    lmr_scaling_factor           = 1673, 1000..=3000,    true;
    cap_hist_max                 = 17215, 8192..=24576,  true;
    cont_hist_max                = 18803, 8192..=24576,  true;
    quiet_hist_factorizer_max    = 4830, 2048..=6144,    true;
    quiet_hist_bucket_max        = 12956, 8192..=16384,  true;
    soft_tm_scale                = 57, 20..=80,          true;
    soft_tm_inc_scale            = 582, 250..=750,       true;
    hard_tm_scale                = 230, 50..=350,        true;
    hard_tm_inc_scale            = 492, 250..=750,       true;
);
