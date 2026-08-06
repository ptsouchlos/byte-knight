/// Applies the standard history "gravity" formula: moves `current` toward `bonus`, weighted by
/// how close `current` already is to `max`, so repeated updates saturate instead of overflowing.
pub(crate) fn gravity(current: i32, bonus: i32, max: i32) -> i32 {
    current + bonus - current * bonus.abs() / max
}
