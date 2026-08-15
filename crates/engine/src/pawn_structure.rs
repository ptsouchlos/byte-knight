// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{
    bitboard::Bitboard, bitboard_helpers, board::Board, definitions::NumberOf, file::File,
    pieces::Piece, rank::Rank, side::Side, square::Square,
};

pub struct PawnStructure {
    pub passed_pawns: [Bitboard; 2],
    pub doubled_pawns: [Bitboard; 2],
    pub isolated_pawns: [Bitboard; 2],
}

impl PawnStructure {
    fn new(
        white_passed_pawns: Bitboard,
        black_passed_pawns: Bitboard,
        white_doubled_pawns: Bitboard,
        black_doubled_pawns: Bitboard,
        white_isolated_pawns: Bitboard,
        black_isolated_pawns: Bitboard,
    ) -> Self {
        PawnStructure {
            passed_pawns: [white_passed_pawns, black_passed_pawns],
            doubled_pawns: [white_doubled_pawns, black_doubled_pawns],
            isolated_pawns: [white_isolated_pawns, black_isolated_pawns],
        }
    }
}
pub struct PawnEvaluator {
    passed_pawn_masks: [[Bitboard; Square::COUNT]; NumberOf::SIDES],
    adjacent_file_masks: [Bitboard; File::COUNT],
}

impl PawnEvaluator {
    pub fn new() -> Self {
        let mut eval = PawnEvaluator {
            passed_pawn_masks: [[Bitboard::default(); Square::COUNT]; NumberOf::SIDES],
            adjacent_file_masks: [Bitboard::default(); File::COUNT],
        };

        eval.initialize_pawn_masks();
        eval
    }

    pub fn detect_pawn_structure(&self, board: &Board) -> PawnStructure {
        // Get pawn structure from the board
        let white_pawns = board.piece_bitboard(Piece::Pawn, Side::White);
        let black_pawns = board.piece_bitboard(Piece::Pawn, Side::Black);

        // Create masks for the passed pawns mask. We start with the pawns themselves and make a copy.
        let mut white_passed_pawns_mask = white_pawns;
        let mut black_passed_pawns_mask = black_pawns;

        let mut white_isolated_pawns = Bitboard::default();
        let mut black_isolated_pawns = Bitboard::default();

        // Loop through white pawns
        for sq in white_pawns {
            // Add the passed pawn mask for this square
            white_passed_pawns_mask |= self.passed_pawn_mask(Side::White, sq);

            let file = sq.file();
            if (self.adjacent_file_masks[file] & white_pawns).as_number() == 0 {
                white_isolated_pawns |= Bitboard::from(sq);
            }
        }

        // Loop through the black pawns
        for sq in black_pawns {
            // Add the passed pawn mask for this square
            black_passed_pawns_mask |= self.passed_pawn_mask(Side::Black, sq);

            let file = sq.file();
            if (self.adjacent_file_masks[file] & black_pawns).as_number() == 0 {
                black_isolated_pawns |= Bitboard::from(sq);
            }
        }

        // At this points our black/white passed pawns masks have all the squares that would block passed pawns and include the location of the pawns themselves.
        // To find the actual passed pawns, we invert the masks and AND them with our respective pawns.
        // So white passed pawns are where we have an overlap (&) of the inverse of the black pawns mask and the white pawns.
        let mut white_passed_pawns = !black_passed_pawns_mask & white_pawns;
        let mut black_passed_pawns = !white_passed_pawns_mask & black_pawns;

        // Simple doubled pawn detection. Shift the pawns one rank backward (white) or
        // forward (black) and use north/south fill to find all squares behind them.
        let shifted_white_pawns = white_pawns >> 8;
        let shifted_black_pawns = black_pawns << 8;

        let shifted_wp_fill = bitboard_helpers::south_fill(shifted_white_pawns);
        let shifted_bp_fill = bitboard_helpers::north_fill(shifted_black_pawns);

        // Now fill to find all squares behind the shifted pawns and AND with the original pawns to find doubled pawns.
        // Note that only the pawns behind the frontmost pawn on each file will be detected as doubled.
        let white_doubled_pawns = shifted_wp_fill & white_pawns;
        let black_doubled_pawns = shifted_bp_fill & black_pawns;

        // Only the forward most pawn on each file should be considered a passed pawn.
        // Since the doubled pawn mask includes all but the frontmost pawn, we can remove those from the passed pawn set.
        white_passed_pawns &= !white_doubled_pawns;
        black_passed_pawns &= !black_doubled_pawns;

        PawnStructure::new(
            white_passed_pawns,
            black_passed_pawns,
            white_doubled_pawns,
            black_doubled_pawns,
            white_isolated_pawns,
            black_isolated_pawns,
        )
    }

    fn passed_pawn_mask(&self, side: Side, square: Square) -> Bitboard {
        self.passed_pawn_masks[side as usize][square]
    }

    fn initialize_pawn_masks(&mut self) {
        for file in File::iter() {
            let mut mask = Bitboard::default();
            if let Some(fl) = file.offset(-1) {
                mask |= fl.to_bitboard();
            }
            if let Some(fl) = file.offset(1) {
                mask |= fl.to_bitboard();
            }
            self.adjacent_file_masks[file.index()] = mask;
        }

        for sq in Bitboard::filled() {
            let file = sq.file();
            let rank = sq.rank();

            let mut mask_w = Bitboard::default();
            let mut mask_b = Bitboard::default();
            // Mask for white pawns
            // All squares in front of the pawn on the same file and adjacent files
            for r in (rank.inner() + 1)..NumberOf::RANKS as u8 {
                let rnk = Rank::try_from(r).unwrap();
                let rank_bb = rnk.to_bitboard();

                if let Some(fl) = file.offset(-1) {
                    mask_w |= fl.to_bitboard() & rank_bb;
                }

                mask_w |= file.to_bitboard() & rank_bb;

                if let Some(fl) = file.offset(1) {
                    mask_w |= fl.to_bitboard() & rank_bb;
                }
            }

            // Mask for black pawns
            // All squares in front of the pawn on the same file and adjacent files
            for r in 0..rank.inner() {
                let rnk = Rank::try_from(r).unwrap();
                let rank_bb = rnk.to_bitboard();

                if let Some(fl) = file.offset(-1) {
                    mask_b |= fl.to_bitboard() & rank_bb;
                }

                mask_b |= file.to_bitboard() & rank_bb;

                if let Some(fl) = file.offset(1) {
                    mask_b |= fl.to_bitboard() & rank_bb;
                }
            }

            self.passed_pawn_masks[Side::White as usize][sq] = mask_w;
            self.passed_pawn_masks[Side::Black as usize][sq] = mask_b;
        }
    }
}

impl Default for PawnEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use chess::{bitboard::Bitboard, board::Board, side::Side, square::Square};

    use crate::pawn_structure::{PawnEvaluator, PawnStructure};

    #[test]
    fn initialization() {
        let pawn_eval = PawnEvaluator::new();
        let board = chess::board::Board::default_board();
        let structure = pawn_eval.detect_pawn_structure(&board);
        assert_eq!(structure.passed_pawns[0].as_number(), 0);
        assert_eq!(structure.passed_pawns[1].as_number(), 0);
    }

    #[test]
    fn full_test_suite() {
        // Test suite adapted from https://www.stmintz.com/ccc/index.php?id=98943
        let test_suite = &[
            // Simple case, white has passed pawns on c3, d5
            (
                "8/8/8/3P4/8/2P5/8/8 w - - 0 1",
                PawnStructure {
                    passed_pawns: [
                        Bitboard::from(Square::C3) | Square::D5.into(),
                        Bitboard::default(),
                    ],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Default::default(), Default::default()],
                },
            ),
            (
                "4Q3/6pk/2pq4/3p4/1p1P3p/1P1K1P2/1PP3P1/8 b - -",
                PawnStructure {
                    passed_pawns: [Default::default(), Default::default()],
                    doubled_pawns: [Bitboard::from(Square::B2), Default::default()],
                    isolated_pawns: [Default::default(), Default::default()],
                },
            ),
            (
                "8/5pk1/4p3/7Q/8/3q4/KP6/8 b - -",
                PawnStructure {
                    passed_pawns: [
                        Square::B2.into(),
                        Bitboard::from(Square::E6) | Square::F7.into(),
                    ],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Square::B2.into(), Default::default()],
                },
            ),
            (
                "r3bb2/P1q3k1/Q2p3p/2pPp1pP/2B1P3/2B5/6P1/R5K1 w - -",
                PawnStructure {
                    passed_pawns: [Square::A7.into(), Square::C5.into()],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Square::A7.into(), Default::default()],
                },
            ),
            (
                "r1b5/p2k1r1p/3P2pP/1ppR4/2P2p2/2P5/P1B4P/4R1K1 w - -",
                PawnStructure {
                    passed_pawns: [Square::D6.into(), Square::F4.into()],
                    doubled_pawns: [
                        Bitboard::from(Square::C3) | Square::H2.into(),
                        Default::default(),
                    ],
                    isolated_pawns: [
                        Bitboard::from(Square::A2) | Square::H2.into() | Square::H6.into(),
                        Default::default(),
                    ],
                },
            ),
            (
                "6r1/1p3k2/pPp4R/K1P1p1p1/1P2Pp1p/5P1P/6P1/8 w - -",
                PawnStructure {
                    passed_pawns: Default::default(),
                    doubled_pawns: [Bitboard::from(Square::B4), Default::default()],
                    isolated_pawns: [Default::default(), Default::default()],
                },
            ),
            (
                "1k2b3/4bpp1/p2pp1P1/1p3P2/2q1P3/4B3/PPPQN2r/1K1R4 w - -",
                PawnStructure {
                    passed_pawns: Default::default(),
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Default::default(), Default::default()],
                },
            ),
            (
                "2kr3r/ppp1qpp1/2p5/2b2b2/2P1pPP1/1P2P1p1/PBQPB3/RN2K1R1 b Q -",
                PawnStructure {
                    passed_pawns: [Default::default(), Square::G3.into()],
                    doubled_pawns: [
                        Default::default(),
                        Bitboard::from(Square::C7) | Square::G7.into(),
                    ],
                    isolated_pawns: [Default::default(), Default::default()],
                },
            ),
            (
                "6k1/2q3p1/1n2Pp1p/pBp2P2/Pp2P3/1P1Q1KP1/8/8 w - -",
                PawnStructure {
                    passed_pawns: [Square::E6.into(), Default::default()],
                    doubled_pawns: [Square::E4.into(), Default::default()],
                    isolated_pawns: [Default::default(), Default::default()],
                },
            ),
            (
                "5r2/pp1RRrk1/4Qq1p/1PP3p1/8/4B3/1b3P1P/6K1 w - -",
                PawnStructure {
                    passed_pawns: Default::default(),
                    doubled_pawns: Default::default(),
                    isolated_pawns: [
                        Bitboard::from(Square::F2) | Square::H2.into(),
                        Default::default(),
                    ],
                },
            ),
            (
                "6k1/1q2rpp1/p6p/P7/1PB1n3/5Q2/6PP/5R1K w - -",
                PawnStructure {
                    passed_pawns: Default::default(),
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Default::default(), Square::A6.into()],
                },
            ),
            (
                "3r2k1/p6p/b2r2p1/2qPQp2/2P2P2/8/6BP/R4R1K w - -",
                PawnStructure {
                    passed_pawns: [
                        Bitboard::from(Square::C4) | Square::D5.into(),
                        Square::A7.into(),
                    ],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [
                        Bitboard::from(Square::F4) | Square::H2.into(),
                        Square::A7.into(),
                    ],
                },
            ),
            (
                "8/6Bp/6p1/2k1p3/4PPP1/1pb4P/8/2K5 b - -",
                PawnStructure {
                    passed_pawns: [Default::default(), Square::B3.into()],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [
                        Default::default(),
                        Bitboard::from(Square::B3) | Square::E5.into(),
                    ],
                },
            ),
            (
                "2r1rbk1/p1Bq1ppp/Ppn1b3/1Npp4/B7/3P2Q1/1PP2PPP/R4RK1 w - -",
                PawnStructure {
                    passed_pawns: Default::default(),
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Default::default(), Default::default()],
                },
            ),
            (
                "r4rk1/ppq3pp/2p1Pn2/4p1Q1/8/2N5/PP4PP/2KR1R2 w - -",
                PawnStructure {
                    passed_pawns: [Square::E6.into(), Square::E5.into()],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Square::E6.into(), Square::E5.into()],
                },
            ),
            (
                "6k1/p4pp1/Pp2r3/1QPq3p/8/6P1/2P2P1P/1R4K1 w - -",
                PawnStructure {
                    passed_pawns: Default::default(),
                    doubled_pawns: [Square::C2.into(), Default::default()],
                    isolated_pawns: [
                        Bitboard::from(Square::A6) | Square::C2.into() | Square::C5.into(),
                        Default::default(),
                    ],
                },
            ),
            (
                "8/2k5/2p5/2pb2K1/pp4P1/1P1R4/P7/8 b - -",
                PawnStructure {
                    passed_pawns: [Square::G4.into(), Default::default()],
                    doubled_pawns: [Default::default(), Square::C6.into()],
                    isolated_pawns: [Square::G4.into(), Default::default()],
                },
            ),
            (
                // Note: In this position, black has doubled passed pawns on the H file
                "2r5/1r5k/1P3p2/PR2pP1p/4P2p/2p1BP2/1p2n3/4R2K b - -",
                PawnStructure {
                    passed_pawns: [
                        Bitboard::from(Square::B6) | Square::A5.into(),
                        Bitboard::from(Square::H4) | Square::B2.into() | Square::C3.into(),
                    ],
                    doubled_pawns: [Square::F3.into(), Square::H5.into()],
                    isolated_pawns: [
                        Default::default(),
                        Bitboard::from(Square::H4) | Square::H5.into(),
                    ],
                },
            ),
            (
                "8/1R2P3/6k1/3B4/2P2P2/1p2r3/1Kb4p/8 w - -",
                PawnStructure {
                    passed_pawns: [
                        Bitboard::from(Square::C4) | Square::E7.into() | Square::F4.into(),
                        Bitboard::from(Square::B3) | Square::H2.into(),
                    ],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [
                        Square::C4.into(),
                        Bitboard::from(Square::B3) | Square::H2.into(),
                    ],
                },
            ),
            (
                "1q1r3k/3P1pp1/ppBR1n1p/4Q2P/P4P2/8/5PK1/8 w - -",
                PawnStructure {
                    passed_pawns: [Bitboard::from(Square::D7), Default::default()],
                    doubled_pawns: [Square::F2.into(), Default::default()],
                    isolated_pawns: [
                        Bitboard::from(Square::A4)
                            | Square::D7.into()
                            | Square::F2.into()
                            | Square::F4.into()
                            | Square::H5.into(),
                        Default::default(),
                    ],
                },
            ),
            (
                "6k1/5pp1/pb1r3p/8/2q1P3/1p3N1P/1P3PP1/2R1Q1K1 b - -",
                PawnStructure {
                    passed_pawns: [Default::default(), Default::default()],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Square::B2.into(), Default::default()],
                },
            ),
            (
                "8/Bpk5/8/P2K4/8/8/8/8 w - -",
                PawnStructure {
                    passed_pawns: [Default::default(), Default::default()],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Square::A5.into(), Square::B7.into()],
                },
            ),
            (
                "1r6/5k2/p4p1K/5R2/7P/8/6P1/8 w - -",
                PawnStructure {
                    passed_pawns: [Square::H4.into(), Square::A6.into()],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [
                        Default::default(),
                        Bitboard::from(Square::A6) | Square::F6.into(),
                    ],
                },
            ),
            (
                "8/6k1/p4p2/P3q2p/7P/5Q2/5PK1/8 w - -",
                PawnStructure {
                    passed_pawns: [Default::default(), Default::default()],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [
                        Bitboard::from(Square::A5) | Square::F2.into() | Square::H4.into(),
                        Bitboard::from(Square::A6) | Square::F6.into() | Square::H5.into(),
                    ],
                },
            ),
            (
                "8/8/6p1/3Pkp2/4P3/2K5/6P1/n7 w - -",
                PawnStructure {
                    passed_pawns: [Square::D5.into(), Default::default()],
                    doubled_pawns: Default::default(),
                    isolated_pawns: [Square::G2.into(), Default::default()],
                },
            ),
            // Special case, white has passed and double pawns on d4, d5
            (
                "8/r5kp/p2RB1p1/3P4/1p1P4/nP4P1/P3K2P/8 b - - 0 36",
                PawnStructure {
                    passed_pawns: [Square::D5.into(), Default::default()],
                    doubled_pawns: [Square::D4.into(), Default::default()],
                    isolated_pawns: [
                        Bitboard::from(Square::D4) | Square::D5.into(),
                        Default::default(),
                    ],
                },
            ),
        ];

        let pawn_eval = PawnEvaluator::new();

        for (fen, expected_structure) in test_suite {
            let board = Board::from_fen(fen).unwrap();
            let structure = pawn_eval.detect_pawn_structure(&board);

            let expected_pp_w = expected_structure.passed_pawns[Side::White as usize];
            let expected_pp_b = expected_structure.passed_pawns[Side::Black as usize];
            let expected_dp_w = expected_structure.doubled_pawns[Side::White as usize];
            let expected_dp_b = expected_structure.doubled_pawns[Side::Black as usize];
            let expected_ip_w = expected_structure.isolated_pawns[Side::White as usize];
            let expected_ip_b = expected_structure.isolated_pawns[Side::Black as usize];

            assert_eq!(
                structure.passed_pawns[Side::White as usize],
                expected_pp_w,
                "Detected pawn structure for {} did not match expected.",
                Side::White
            );
            assert_eq!(
                structure.passed_pawns[Side::Black as usize],
                expected_pp_b,
                "Detected pawn structure for {} did not match expected.",
                Side::Black
            );

            assert_eq!(
                structure.doubled_pawns[Side::White as usize],
                expected_dp_w,
                "Detected doubled pawn structure for {} did not match expected.",
                Side::White
            );
            assert_eq!(
                structure.doubled_pawns[Side::Black as usize],
                expected_dp_b,
                "Detected doubled pawn structure for {} did not match expected.",
                Side::Black
            );

            assert_eq!(
                structure.isolated_pawns[Side::White as usize],
                expected_ip_w,
                "Detected isolated pawn structure for {} did not match expected.",
                Side::White
            );
            assert_eq!(
                structure.isolated_pawns[Side::Black as usize],
                expected_ip_b,
                "Detected isolated pawn structure for {} did not match expected.",
                Side::Black
            );
        }
    }
}
