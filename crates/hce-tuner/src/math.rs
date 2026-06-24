// Part of the byte-knight project.
// Tuner adapted from jw1912/hce-tuner (https://github.com/jw1912/hce-tuner)

pub(crate) fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + f64::exp(-x))
}
