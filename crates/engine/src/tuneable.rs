// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::tunable_params;

#[rustfmt::skip]
tunable_params!(
    lmp_max_depth                = 6, 1..=10,            true;
    lmp_base                     = 6, 1..=10,            true;
    lmp_improvement_min          = -138, -300..=0,       true;
    lmp_improvement_max          = 178, 50..=300,        true;
    lmp_improvement_divisor      = 8, 2..=24,            true;
    see_value_pawn               = 101, 25..=150,        true;
    see_value_knight             = 205, 100..=300,       true;
    see_value_bishop             = 288, 150..=450,       true;
    see_value_rook               = 517, 250..=750,       true;
    see_value_queen              = 865, 450..=1350,      true;
    qs_see_threshold             = -80, -160..=0,        true;
    qs_delta_margin              = 241, 120..=360,       true;
    see_tacticals_max_depth      = 6, 4..=10,            true;
    see_tacticals_margin         = 61, 0..=100,          true;
    fp_base                      = 94, 50..=150,         true;
    fp_scale                     = 74, 40..=120,         true;
    fp_max_depth                 = 3, 1..=6,             true;
    rfp_max_depth                = 4, 1..=10,            true;
    rfp_margin                   = 75, 40..=120,         true;
    rfp_improving_margin         = 26, 0..=60,           true;
    nmp_min_depth                = 3, 1..=6,             true;
    nmp_depth_reduction          = 2, 0..=6,             false;
    nmp_improving_bonus          = 1, 0..=4,             false;
    nmp_depth_divisor            = 4, 2..=8,             false;
    min_aspiration_depth         = 1, 1..=12,            false;
    aspiration_window            = 27, 10..=50,          true;
    iir_min_depth                = 4, 1..=10,            true;
    iir_depth_reduction          = 1, 1..=6,             false;
    razoring_scaling             = 344, 150..=550,       true;
    razoring_offset              = 511, 250..=750,       true;
    lmr_min_depth                = 3, 1..=6,             true;
    lmr_min_moves_seen           = 3, 1..=6,             true;
    cap_hist_bonus_scale         = 300, 150..=450,       true;
    cap_hist_bonus_offset        = 250, 100..=400,       true;
    cap_hist_malus_scale         = 300, 150..=450,       true;
    cap_hist_malus_offset        = 250, 100..=400,       true;
    movepick_mvv_scale           = 2048, 1024..=3072,    true;
    quiet_hist_bonus_scale       = 300, 150..=450,       true;
    quiet_hist_bonus_offset      = 250, 100..=400,       true;
    quiet_hist_malus_scale       = 300, 150..=450,       true;
    quiet_hist_malus_offset      = 250, 100..=400,       true;
    cont_hist_1_bonus_scale      = 300, 150..=450,       true;
    cont_hist_1_bonus_offset     = 250, 100..=400,       true;
    cont_hist_1_malus_scale      = 300, 150..=450,       true;
    cont_hist_1_malus_offset     = 250, 100..=400,       true;
    cont_hist_2_bonus_scale      = 300, 150..=450,       true;
    cont_hist_2_bonus_offset     = 250, 100..=400,       true;
    cont_hist_2_malus_scale      = 300, 150..=450,       true;
    cont_hist_2_malus_offset     = 250, 100..=400,       true;
    score_stability_threshold    = 12, 4..=24,           true;
    tm_node_stability_offset     = 14500, 12000..=16000, true;
    tm_node_stability_scale      = 13500, 12000..=16000, true;
    tm_eval_stability_offset     = 12664, 10000..=14000, true;
    tm_eval_stability_scale      = 416, 200..=600,       true;
    tm_eval_stability_min_scale  = 8642, 7000..=10000,   true;
    tm_best_move_offset          = 11000, 9000..=13000,  true;
    tm_best_move_scale           = 500, 300..=800,       true;
    tm_best_move_min_scale       = 5000, 4000..=6000,    true;
    lmr_offset                   = 200, 50..=350,        true;
    lmr_scaling_factor           = 2000, 1000..=3000,    true;
);
