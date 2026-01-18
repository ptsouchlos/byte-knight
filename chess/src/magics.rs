// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::{bitboard::Bitboard, definitions::NumberOf};

#[allow(unused)]
pub(crate) const BISHOP_MAGICS: [MagicNumber; NumberOf::SQUARES] = [
    MagicNumber::new(Bitboard::new(18049651735527936), 58, 0, 2603089664189407746),
    MagicNumber::new(Bitboard::new(70506452091904), 59, 64, 86711888284551185),
    MagicNumber::new(Bitboard::new(275415828992), 59, 96, 15278462302932713476),
    MagicNumber::new(Bitboard::new(1075975168), 59, 128, 144749060841279488),
    MagicNumber::new(Bitboard::new(38021120), 59, 160, 6927101445162401824),
    MagicNumber::new(Bitboard::new(8657588224), 59, 192, 1261572013983273280),
    MagicNumber::new(Bitboard::new(2216338399232), 59, 224, 13853221991812694016),
    MagicNumber::new(Bitboard::new(567382630219776), 58, 256, 9259683410520049688),
    MagicNumber::new(Bitboard::new(9024825867763712), 59, 320, 7041581735740426),
    MagicNumber::new(
        Bitboard::new(18049651735527424),
        59,
        352,
        4611691555311517952,
    ),
    MagicNumber::new(Bitboard::new(70506452221952), 59, 384, 13983822120825257985),
    MagicNumber::new(Bitboard::new(275449643008), 59, 416, 15573456601896490056),
    MagicNumber::new(Bitboard::new(9733406720), 59, 448, 4508015428370432),
    MagicNumber::new(Bitboard::new(2216342585344), 59, 480, 2234394152962),
    MagicNumber::new(Bitboard::new(567382630203392), 59, 512, 2306151424507527169),
    MagicNumber::new(Bitboard::new(1134765260406784), 59, 544, 576461310783489036),
    MagicNumber::new(Bitboard::new(4512412933816832), 59, 576, 18014965583709217),
    MagicNumber::new(
        Bitboard::new(9024825867633664),
        59,
        608,
        9260535667331704064,
    ),
    MagicNumber::new(
        Bitboard::new(18049651768822272),
        57,
        640,
        292734048860774467,
    ),
    MagicNumber::new(Bitboard::new(70515108615168), 57, 768, 6955000401163223040),
    MagicNumber::new(Bitboard::new(2491752130560), 57, 896, 14073766093127749),
    MagicNumber::new(
        Bitboard::new(567383701868544),
        57,
        1024,
        1441187795308716064,
    ),
    MagicNumber::new(Bitboard::new(1134765256220672), 59, 1152, 28429118685192512),
    MagicNumber::new(Bitboard::new(2269530512441344), 59, 1184, 72127963336280084),
    MagicNumber::new(Bitboard::new(2256206450263040), 59, 1216, 73430196933922816),
    MagicNumber::new(
        Bitboard::new(4512412900526080),
        59,
        1248,
        577025921144587296,
    ),
    MagicNumber::new(Bitboard::new(9024834391117824), 57, 1280, 71605963260416),
    MagicNumber::new(
        Bitboard::new(18051867805491712),
        55,
        1408,
        6919807215895322656,
    ),
    MagicNumber::new(
        Bitboard::new(637888545440768),
        55,
        1920,
        9223517173002018820,
    ),
    MagicNumber::new(
        Bitboard::new(1135039602493440),
        57,
        2432,
        217446016595542592,
    ),
    MagicNumber::new(Bitboard::new(2269529440784384), 59, 2560, 563499781064840),
    MagicNumber::new(
        Bitboard::new(4539058881568768),
        59,
        2592,
        3026984132931518976,
    ),
    MagicNumber::new(
        Bitboard::new(1128098963916800),
        59,
        2624,
        149749790033453066,
    ),
    MagicNumber::new(
        Bitboard::new(2256197927833600),
        59,
        2656,
        145268713762329216,
    ),
    MagicNumber::new(
        Bitboard::new(4514594912477184),
        57,
        2688,
        9593230742550216736,
    ),
    MagicNumber::new(
        Bitboard::new(9592139778506752),
        55,
        2816,
        11565279577752732160,
    ),
    MagicNumber::new(
        Bitboard::new(19184279556981248),
        55,
        3328,
        2893564976789721344,
    ),
    MagicNumber::new(
        Bitboard::new(2339762086609920),
        57,
        3840,
        144502495343935744,
    ),
    MagicNumber::new(Bitboard::new(4538784537380864), 59, 3968, 9293110932703232),
    MagicNumber::new(Bitboard::new(9077569074761728), 59, 4000, 281758444588576),
    MagicNumber::new(Bitboard::new(562958610993152), 59, 4032, 864973840653557760),
    MagicNumber::new(
        Bitboard::new(1125917221986304),
        59,
        4064,
        577024321942273156,
    ),
    MagicNumber::new(
        Bitboard::new(2814792987328512),
        57,
        4096,
        297246647719561220,
    ),
    MagicNumber::new(
        Bitboard::new(5629586008178688),
        57,
        4224,
        4611901806250035200,
    ),
    MagicNumber::new(
        Bitboard::new(11259172008099840),
        57,
        4352,
        10381404075873797122,
    ),
    MagicNumber::new(
        Bitboard::new(22518341868716544),
        57,
        4480,
        2884697433095283200,
    ),
    MagicNumber::new(
        Bitboard::new(9007336962655232),
        59,
        4608,
        4614298528989216904,
    ),
    MagicNumber::new(Bitboard::new(18014673925310464), 59, 4640, 4575086682068740),
    MagicNumber::new(Bitboard::new(2216338399232), 59, 4672, 77689297159327744),
    MagicNumber::new(Bitboard::new(4432676798464), 59, 4704, 10768642671186176),
    MagicNumber::new(
        Bitboard::new(11064376819712),
        59,
        4736,
        11673330517747506216,
    ),
    MagicNumber::new(Bitboard::new(22137335185408), 59, 4768, 2598576986100203664),
    MagicNumber::new(Bitboard::new(44272556441600), 59, 4800, 36028874867476480),
    MagicNumber::new(Bitboard::new(87995357200384), 59, 4832, 189160119089533184),
    MagicNumber::new(Bitboard::new(35253226045952), 59, 4864, 1411807424610305),
    MagicNumber::new(Bitboard::new(70506452091904), 59, 4896, 4505945015590914),
    MagicNumber::new(Bitboard::new(567382630219776), 58, 4928, 41096447739043849),
    MagicNumber::new(
        Bitboard::new(1134765260406784),
        59,
        4992,
        9295852019518693378,
    ),
    MagicNumber::new(
        Bitboard::new(2832480465846272),
        59,
        5024,
        9259400838173241346,
    ),
    MagicNumber::new(Bitboard::new(5667157807464448), 59, 5056, 1407435021750529),
    MagicNumber::new(
        Bitboard::new(11333774449049600),
        59,
        5088,
        36345735139042338,
    ),
    MagicNumber::new(Bitboard::new(22526811443298304), 59, 5120, 1196290398618112),
    MagicNumber::new(Bitboard::new(9024825867763712), 59, 5152, 86008541186883712),
    MagicNumber::new(
        Bitboard::new(18049651735527936),
        58,
        5184,
        189153452646268944,
    ),
];

#[allow(unused)]
pub(crate) const ROOK_MAGICS: [MagicNumber; NumberOf::SQUARES] = [
    MagicNumber::new(Bitboard::new(282578800148862), 52, 0, 2341871961128845312),
    MagicNumber::new(Bitboard::new(565157600297596), 53, 4096, 162129724031632076),
    MagicNumber::new(
        Bitboard::new(1130315200595066),
        53,
        6144,
        324276767523078408,
    ),
    MagicNumber::new(
        Bitboard::new(2260630401190006),
        53,
        8192,
        1188954734040453120,
    ),
    MagicNumber::new(
        Bitboard::new(4521260802379886),
        53,
        10240,
        432363190806922272,
    ),
    MagicNumber::new(
        Bitboard::new(9042521604759646),
        53,
        12288,
        1369095389488614400,
    ),
    MagicNumber::new(
        Bitboard::new(18085043209519166),
        53,
        14336,
        288232609551753360,
    ),
    MagicNumber::new(
        Bitboard::new(36170086419038334),
        52,
        16384,
        4647715050671309056,
    ),
    MagicNumber::new(
        Bitboard::new(282578800180736),
        53,
        20480,
        180566202928627744,
    ),
    MagicNumber::new(Bitboard::new(565157600328704), 54, 22528, 37436446797758496),
    MagicNumber::new(Bitboard::new(1130315200625152), 54, 23552, 563027271229504),
    MagicNumber::new(
        Bitboard::new(2260630401218048),
        54,
        24576,
        2350883405698302016,
    ),
    MagicNumber::new(
        Bitboard::new(4521260802403840),
        54,
        25600,
        20829165601292304,
    ),
    MagicNumber::new(
        Bitboard::new(9042521604775424),
        54,
        26624,
        641904236325504000,
    ),
    MagicNumber::new(
        Bitboard::new(18085043209518592),
        54,
        27648,
        9570153637609984,
    ),
    MagicNumber::new(
        Bitboard::new(36170086419037696),
        53,
        28672,
        4630263369205809249,
    ),
    MagicNumber::new(
        Bitboard::new(282578808340736),
        53,
        30720,
        4647715091398008832,
    ),
    MagicNumber::new(
        Bitboard::new(565157608292864),
        54,
        32768,
        585614736897884160,
    ),
    MagicNumber::new(
        Bitboard::new(1130315208328192),
        54,
        33792,
        1171079389652197376,
    ),
    MagicNumber::new(Bitboard::new(2260630408398848), 54, 34816, 582741301264416),
    MagicNumber::new(
        Bitboard::new(4521260808540160),
        54,
        35840,
        2378041890562902016,
    ),
    MagicNumber::new(
        Bitboard::new(9042521608822784),
        54,
        36864,
        55169370346766592,
    ),
    MagicNumber::new(
        Bitboard::new(18085043209388032),
        54,
        37888,
        954767520678678600,
    ),
    MagicNumber::new(
        Bitboard::new(36170086418907136),
        53,
        38912,
        11542673253106500,
    ),
    MagicNumber::new(
        Bitboard::new(282580897300736),
        53,
        40960,
        441423134374002720,
    ),
    MagicNumber::new(Bitboard::new(565159647117824), 54, 43008, 633338027049088),
    MagicNumber::new(
        Bitboard::new(1130317180306432),
        54,
        44032,
        400820511254513665,
    ),
    MagicNumber::new(
        Bitboard::new(2260632246683648),
        54,
        45056,
        2310346679708549248,
    ),
    MagicNumber::new(
        Bitboard::new(4521262379438080),
        54,
        46080,
        9224497971255607360,
    ),
    MagicNumber::new(
        Bitboard::new(9042522644946944),
        54,
        47104,
        20547686185041928,
    ),
    MagicNumber::new(
        Bitboard::new(18085043175964672),
        54,
        48128,
        11540478340366848,
    ),
    MagicNumber::new(
        Bitboard::new(36170086385483776),
        53,
        49152,
        2252117641822484,
    ),
    MagicNumber::new(Bitboard::new(283115671060736), 53, 51200, 36169809393614880),
    MagicNumber::new(
        Bitboard::new(565681586307584),
        54,
        53248,
        2598867806371987460,
    ),
    MagicNumber::new(
        Bitboard::new(1130822006735872),
        54,
        54272,
        9227893230824071168,
    ),
    MagicNumber::new(Bitboard::new(2261102847592448), 54, 55296, 9024793596659712),
    MagicNumber::new(
        Bitboard::new(4521664529305600),
        54,
        56320,
        2377905003454007296,
    ),
    MagicNumber::new(
        Bitboard::new(9042787892731904),
        54,
        57344,
        2882305962696246272,
    ),
    MagicNumber::new(
        Bitboard::new(18085034619584512),
        54,
        58368,
        2260600268784208,
    ),
    MagicNumber::new(Bitboard::new(36170077829103616), 53, 59392, 281421022235),
    MagicNumber::new(
        Bitboard::new(420017753620736),
        53,
        61440,
        288371388520103944,
    ),
    MagicNumber::new(Bitboard::new(699298018886144), 54, 63488, 36310555472035884),
    MagicNumber::new(Bitboard::new(1260057572672512), 54, 64512, 17731806167105),
    MagicNumber::new(
        Bitboard::new(2381576680245248),
        54,
        65536,
        5764625158174277664,
    ),
    MagicNumber::new(
        Bitboard::new(4624614895390720),
        54,
        66560,
        2308094847683198980,
    ),
    MagicNumber::new(
        Bitboard::new(9110691325681664),
        54,
        67584,
        4611688217517785216,
    ),
    MagicNumber::new(
        Bitboard::new(18082844186263552),
        54,
        68608,
        290483284201439248,
    ),
    MagicNumber::new(
        Bitboard::new(36167887395782656),
        53,
        69632,
        648518913281359873,
    ),
    MagicNumber::new(Bitboard::new(35466950888980736), 53, 71680, 70370891669632),
    MagicNumber::new(
        Bitboard::new(34905104758997504),
        54,
        73728,
        18014949475354240,
    ),
    MagicNumber::new(
        Bitboard::new(34344362452452352),
        54,
        74752,
        9232380404342882944,
    ),
    MagicNumber::new(
        Bitboard::new(33222877839362048),
        54,
        75776,
        11677871286054306304,
    ),
    MagicNumber::new(
        Bitboard::new(30979908613181440),
        54,
        76800,
        288239172311875712,
    ),
    MagicNumber::new(
        Bitboard::new(26493970160820224),
        54,
        77824,
        145241096589426752,
    ),
    MagicNumber::new(
        Bitboard::new(17522093256097792),
        54,
        78848,
        2814962439815680,
    ),
    MagicNumber::new(
        Bitboard::new(35607136465616896),
        53,
        79872,
        99501955163947520,
    ),
    MagicNumber::new(
        Bitboard::new(9079539427579068672),
        52,
        81920,
        54607245534593042,
    ),
    MagicNumber::new(
        Bitboard::new(8935706818303361536),
        53,
        86016,
        492583424909569,
    ),
    MagicNumber::new(
        Bitboard::new(8792156787827803136),
        53,
        88064,
        10033060450369,
    ),
    MagicNumber::new(
        Bitboard::new(8505056726876686336),
        53,
        90112,
        1271173149163913,
    ),
    MagicNumber::new(
        Bitboard::new(7930856604974452736),
        53,
        92160,
        4900479638222931970,
    ),
    MagicNumber::new(
        Bitboard::new(6782456361169985536),
        53,
        94208,
        63331880634941505,
    ),
    MagicNumber::new(
        Bitboard::new(4485655873561051136),
        53,
        96256,
        4574591544602628,
    ),
    MagicNumber::new(
        Bitboard::new(9115426935197958144),
        52,
        98304,
        144274076650064898,
    ),
];

/// "Magic" number used for fancy bitboard operations.
#[derive(Serialize, Default, Deserialize, Debug, Clone, Copy)]
pub struct MagicNumber {
    pub relevant_bits_mask: u64,
    pub shift: u8,
    pub offset: u64,
    pub magic_value: u64,
}

impl MagicNumber {
    pub const fn new(
        relevant_bits_mask: Bitboard,
        shift: u8,
        offset: u64,
        magic_value: u64,
    ) -> Self {
        MagicNumber {
            relevant_bits_mask: relevant_bits_mask.as_number(),
            shift,
            offset,
            magic_value,
        }
    }

    /// Returns the index of the magic number in the table.
    ///
    /// Calculated by multiplying the blocker number with the magic number and shifting it to the right by the shift value.
    pub const fn index(&self, occupancy: Bitboard) -> usize {
        let blockers = occupancy.as_number() & self.relevant_bits_mask;
        // need to shift
        let hash = blockers.wrapping_mul(self.magic_value);
        ((hash >> self.shift) + self.offset) as usize
    }
}

impl Display for MagicNumber {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "bb {:24} shift {:4} offset {:6} magic {:24}",
            self.relevant_bits_mask, self.shift, self.offset, self.magic_value
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{definitions::Squares, move_generation::MoveGenerator};

    use super::*;

    #[test]
    fn test_magic_number_index() {
        // test a1 for the rook
        let relevant_bits = MoveGenerator::relevant_rook_bits(Squares::A1);
        let blockers = MoveGenerator::create_blocker_permutations(relevant_bits);
        let magic_value = 684547693657194778;
        let magic = MagicNumber::new(
            relevant_bits,
            (64 - relevant_bits.as_number().count_ones()) as u8,
            0,
            magic_value,
        );
        let blocker = blockers[0];
        assert_eq!(magic.index(blocker), 0);

        let mut indexes = Vec::with_capacity(blockers.len());
        for blocker in blockers {
            let index = magic.index(blocker);
            indexes.push(index);
        }
    }

    #[test]
    fn magic_number_display() {
        // test a1 for the rook
        let relevant_bits = MoveGenerator::relevant_rook_bits(Squares::A1);
        let magic_value = 684547693657194778;
        let magic = MagicNumber::new(
            relevant_bits,
            (64 - relevant_bits.as_number().count_ones()) as u8,
            0,
            magic_value,
        );

        assert_eq!(
            format!("{magic}"),
            "bb          282578800148862 shift   52 offset      0 magic       684547693657194778"
        );

        // test c4 for the bishop
        let magic = BISHOP_MAGICS[Squares::C4 as usize];
        assert_eq!(
            format!("{magic}"),
            "bb         9024834391117824 shift   57 offset   1280 magic           71605963260416"
        );
    }
}
