// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use crate::score::ScoreType;

// Credit to Akimbo author for the implementation
#[macro_export]
macro_rules! tunable_params {
    ($($name:ident = $val:expr, $min:expr, $max:expr, $step:expr, $spsa:expr;)*) => {
        #[cfg(feature = "tuning")]
        use std::sync::atomic::Ordering;

        #[cfg(feature = "tuning")]
        pub fn list_params() {
            $(
                println!(
                    "option name {} type spin default {} min {} max {}",
                    stringify!($name),
                    $name(),
                    $min,
                    $max,
                );
            )*
        }

        #[cfg(feature = "tuning")]
        pub fn set_param(name: &str, val: i32) {
            match name {
                $(
                    stringify!($name) => vals::$name.store(val, Ordering::Relaxed),
                )*
                _ => println!("info error unknown option"),
            }
        }

        #[cfg(feature = "tuning")]
        pub fn print_params_ob() {
            $(
                if $spsa {
                    let step = ($max - $min) / 20;
                    println!(
                        "{}, int, {}.0, {}.0, {}.0, {}, 0.002",
                        stringify!($name),
                        $name(),
                        $min,
                        $max,
                        step,
                    );
                }
            )*
        }

        #[cfg(feature = "tuning")]
        mod vals {
            use std::sync::atomic::AtomicI32;
            $(
            #[allow(non_upper_case_globals)]
            pub static $name: AtomicI32 = AtomicI32::new($val);
            )*
        }

        $(
        #[cfg(feature = "tuning")]
        #[inline]
        pub fn $name() -> i32 {
            vals::$name.load(Ordering::Relaxed)
        }

        #[cfg(not(feature = "tuning"))]
        #[inline]
        pub fn $name() -> i32 {
            $val
        }
        )*
    };
}

#[rustfmt::skip]
tunable_params!(
    lmp_max_depth           = 6, 6, 10, 1,           false;
    lmp_base                = 6, 1, 10, 1,           false;
    see_value_pawn          = 100, 50, 150, 1,       false;
    see_value_knight        = 200, 180, 300, 1,      false;
    see_value_bishop        = 300, 250, 250, 1,      false;
    see_value_rook          = 500, 400, 600, 1,      false;
    see_value_queen         = 900, 800, 1100, 1,     false;
    qs_see_threshold        = -100, -100, 100, 1,       false;
    qs_delta_margin         = 200, 50, 300, 1,       false;
);

pub(crate) const MIN_ASPIRATION_DEPTH: ScoreType = 1;
pub(crate) const ASPIRATION_WINDOW: ScoreType = 50;

pub(crate) const MAX_RFP_DEPTH: ScoreType = 4;
pub(crate) const RFP_MARGIN: ScoreType = 82;

pub(crate) const IIR_MIN_DEPTH: ScoreType = 4;
pub(crate) const IIR_DEPTH_REDUCTION: ScoreType = 1;

pub(crate) const NMP_MIN_DEPTH: ScoreType = 3;
pub(crate) const NMP_DEPTH_REDUCTION: ScoreType = 2;

pub(crate) const LMR_OFFSET: f64 = 0.2;
pub(crate) const LMR_SCALING_FACTOR: f64 = 2.0;
pub(crate) const LMR_MIN_DEPTH: i16 = 3;
pub(crate) const LMR_MIN_MOVES_SEEN: usize = 3;

pub(crate) const RAZORING_SCALING: ScoreType = 400;
pub(crate) const RAZORING_OFFSET: ScoreType = 500;
