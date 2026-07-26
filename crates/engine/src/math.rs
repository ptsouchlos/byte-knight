// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

/// Linearly interpolate between a and b using a factor ([0 - 100]).
/// See also https://en.cppreference.com/cpp/numeric/lerp.
///
/// # Arguments
/// - `a`: The first value.
/// - `b`: The second value.
/// - `factor`: The factor to use to scale between the two values.
pub(crate) fn lerp(a: i32, b: i32, factor: i32) -> i32 {
    debug_assert!(
        (0..=100).contains(&factor),
        "factor must be between 0 and 100: {factor}"
    );

    let a_scale = 100 - factor;
    let b_scale = factor;

    ((a * a_scale) + (b * b_scale)) / 100
}

#[cfg(test)]
mod tests {
    #[test]
    fn lerp_validation() {
        let a = 20i32;
        let b = 40i32;

        let factor = 50;
        let result = super::lerp(a, b, factor);
        assert_eq!(result, 30);
    }
}
