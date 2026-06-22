// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::tunable_params;

#[rustfmt::skip]
tunable_params!(
    lmp_max_depth           = 6, 1, 10, 1,           false;
    lmp_base                = 6, 1, 10, 1,           false;
    lmp_improvement_min     = -138, -300, 0, 25,     true;
    lmp_improvement_max     = 178, 1, 500, 25,       true;
    lmp_improvement_divisor = 8, 2, 128, 4,          true;
    see_value_pawn          = 101, 25, 150, 1,       true;
    see_value_knight        = 205, 100, 300, 1,      true;
    see_value_bishop        = 288, 150, 450, 1,      true;
    see_value_rook          = 517, 250, 750, 1,      true;
    see_value_queen         = 865, 450, 1350, 1,     true;
    qs_see_threshold        = -80, -200, 200, 1,     true;
    qs_delta_margin         = 241, 50, 350, 1,       true;
    see_tacticals_max_depth = 6, 4, 10, 1,           false;
    see_tacticals_margin    = 61, 0, 100, 1,         true;
    fp_base                 = 94, 50, 150, 1,        true;
    fp_scale                = 74, 40, 120, 1,        true;
    fp_max_depth            = 3, 1, 8, 1,            false;
    fp_improving_margin     = 100, 10, 200, 2,        true;
    rfp_max_depth           = 4, 1, 10, 1,           false;
    rfp_margin              = 75, 40, 120, 1,        true;
    rfp_improving_margin    = 26, 10, 70, 1,         true;
    nmp_min_depth           = 3, 1, 8, 1,            false;
    nmp_depth_reduction     = 2, 0, 6, 1,            false;
    nmp_improving_bonus     = 1, 0, 4, 1,            false;
    nmp_depth_divisor       = 4, 2, 8, 1,            false;
    min_aspiration_depth    = 1, 1, 12, 1,           false;
    aspiration_window       = 27, 25, 150, 1,        true;
    iir_min_depth           = 4, 1, 10, 1,           false;
    iir_depth_reduction     = 1, 1, 6, 1,            false;
    razoring_scaling        = 344, 100, 800, 10,     true;
    razoring_offset         = 511, 250, 1000, 10,    true;
    lmr_min_depth           = 3, 1, 6, 1,            false;
    lmr_min_moves_seen      = 3, 1, 6, 1,            false;
);

pub(crate) const LMR_OFFSET: f64 = 0.2;
pub(crate) const LMR_SCALING_FACTOR: f64 = 2.0;
