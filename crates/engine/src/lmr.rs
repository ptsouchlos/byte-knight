// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::tuneable::{lmr_offset, lmr_scaling_factor};

/// LMR (Late Move Reduction) formula for calculating the reduction factor
/// based on the natural logarithm of the current depth and the number of moves made.
///
/// # Arguments
///
/// - `depth` - The current depth in the search tree.
/// - `move_count` - The number of moves made so far.
///
/// # Returns
///
/// A floating-point value representing the reduction factor.
pub(crate) fn formula(depth: usize, move_count: usize) -> f64 {
    let d_ln = (depth as f64).ln();
    let mvs_ln = (move_count as f64).ln();
    let scaled_lmr_offset = lmr_offset() as f64 / 1000.0;
    let scaled_lmr_scaling_factor = lmr_scaling_factor() as f64 / 1000.0;
    if d_ln.is_finite() && mvs_ln.is_finite() {
        scaled_lmr_offset + (d_ln * mvs_ln) / scaled_lmr_scaling_factor
    } else {
        0_f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const ZERO: f64 = 0.0;

    #[test]
    fn test_lmr_formula() {
        let depth = 5;
        let move_count = 10;
        let result = formula(depth, move_count);
        assert!(result.is_finite());
        assert!(result > ZERO);
    }

    #[test]
    fn test_lmr_formula_zero_depth() {
        let depth = 0;
        let move_count = 10;
        let result = formula(depth, move_count);
        assert_eq!(result, ZERO);
    }

    #[test]
    fn test_lmr_formula_zero_moves() {
        let depth = 5;
        let move_count = 0;
        let result = formula(depth, move_count);
        assert_eq!(result, ZERO);
    }
}
