// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

#[rustfmt::skip]
const BANNER: &str = r#"
 _         _           _        _      _   _
| |__ _  _| |_ ___ ___| |___ _ (_)__ _| |_| |_
| '_ \ || |  _/ -_)___| / / ' \| / _` | ' \  _|
|_.__/\_, |\__\___|   |_\_\_||_|_\__, |_||_\__|
      |__/                       |___/
"#;

pub struct About;
impl About {
    pub const NAME: &'static str = "byte-knight";
    pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    pub const EMAIL: &'static str = "developer.paul.123@gmail.com";
    pub const SHORT_DESCRIPTION: &'static str = "byte-knight is a UCI compliant chess engine.";
    pub const AUTHORS: &'static str = "Paul T. (ptsouchlos)";
    pub const BANNER: &'static str = BANNER;
}

pub(crate) const MAX_DEPTH: u8 = 128;
