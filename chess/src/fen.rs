// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use std::{
    fmt::{Display, Formatter},
    vec,
};

use thiserror::Error;

use crate::{
    board::Board,
    definitions::{CastlingAvailability, DASH, EM_DASH},
    pieces::{PIECE_SHORT_NAMES, Piece, SQUARE_NAME},
    side::Side,
    square::to_square,
};

/// Represents the 6 parts of a FEN string.
#[derive(Debug, PartialEq)]
pub enum FenPart {
    PiecePlacement = 1,
    ActiveColor = 2,
    CastlingAvailability = 3,
    EnPassantTargetSquare = 4,
    HalfmoveClock = 5,
    FullmoveNumber = 6,
}

impl Display for FenPart {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            FenPart::PiecePlacement => write!(f, "Piece Placement"),
            FenPart::ActiveColor => write!(f, "Active Color"),
            FenPart::CastlingAvailability => write!(f, "Castling Availability"),
            FenPart::EnPassantTargetSquare => write!(f, "En Passant Target Square"),
            FenPart::HalfmoveClock => write!(f, "Halfmove Clock"),
            FenPart::FullmoveNumber => write!(f, "Fullmove Number"),
        }
    }
}

/// Represents an error that occurred while parsing a FEN string.
#[derive(Error, Debug)]
pub struct FenError {
    offending_parts: Option<Vec<FenPart>>,
    message: String,
}

impl FenError {
    pub fn new(message: &str) -> FenError {
        FenError {
            offending_parts: None,
            message: message.to_string(),
        }
    }

    pub fn with_offending_parts(message: &str, offending_parts: Vec<FenPart>) -> FenError {
        FenError {
            offending_parts: Some(offending_parts),
            message: message.to_string(),
        }
    }
}

impl Display for FenError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(parts) = &self.offending_parts {
            write!(f, " Offending parts: ")?;
            for part in parts {
                write!(f, "{part} ")?;
            }
        }
        Ok(())
    }
}

pub type FenResult = Result<(), FenError>;
pub type SplitFenStringResult = Result<Vec<String>, FenError>;
pub(crate) type FenPartParser = fn(board: &mut Board, part: &str) -> Result<(), FenError>;

pub(crate) const FEN_PART_PARSERS: [FenPartParser; 6] = [
    parse_piece_placement,
    parse_active_color,
    parse_castling_availability,
    parse_en_passant_target_square,
    parse_halfmove_clock,
    parse_fullmove_number,
];

/// Splits a FEN string into its 6 parts or returns an error if the FEN string is invalid.
///
/// # Errors
/// There are many possible errors that can occur when splitting a FEN string. Some of the most common
/// errors include:
/// - The FEN string is empty.
/// - The FEN string does not have 6 parts (4 parts are allowed if the last 2 parts are omitted).
/// - The FEN string has an invalid character in the piece placement part.
/// - The FEN string has an extra / in the piece placement part.
pub fn split_fen_string(fen: &str) -> SplitFenStringResult {
    if fen.is_empty() {
        return Err(FenError::new("FEN string is empty"));
    }

    let mut parts = fen
        .replace(EM_DASH, DASH.encode_utf8(&mut [0; 4]))
        .split_whitespace()
        .map(String::from)
        .collect::<Vec<String>>();

    if parts.len() == 4 {
        parts.append(&mut vec![String::from("0"), String::from("1")]);
    }

    if parts.len() != 6 {
        return Err(FenError::new("FEN string does not have 6 parts"));
    }

    Ok(parts)
}

/// Parses the piece placement part of a FEN string and updates the board accordingly.
fn parse_piece_placement(board: &mut Board, part: &str) -> FenResult {
    let mut rank = 7;
    let mut file = 0;

    for c in part.chars() {
        match c {
            '/' => {
                if rank == 0 {
                    return Err(FenError::with_offending_parts(
                        &format!("Extra / found in FEN part {}", FenPart::PiecePlacement,),
                        vec![FenPart::PiecePlacement],
                    ));
                }
                rank -= 1;
                file = 0;
            }
            c if c.is_ascii_digit() => {
                file += c.to_digit(10).unwrap() as usize;
            }
            'P' | 'N' | 'B' | 'R' | 'Q' | 'K' | 'p' | 'n' | 'b' | 'r' | 'q' | 'k' => {
                let piece_res = Piece::try_from(c);
                if piece_res.is_err() {
                    return Err(FenError::with_offending_parts(
                        &format!("Could not parse piece from char {c}"),
                        vec![FenPart::PiecePlacement],
                    ));
                }
                let piece = piece_res.unwrap();

                let side = if c.is_ascii_uppercase() {
                    Side::White
                } else {
                    Side::Black
                };

                let square = to_square(file as u8, rank);
                board.set_piece_square(piece as usize, side as usize, square);

                file += 1;
            }
            _ => {
                return Err(FenError::with_offending_parts(
                    &format!(
                        "Invalid character {} in FEN part {}",
                        c,
                        FenPart::PiecePlacement,
                    ),
                    vec![FenPart::PiecePlacement],
                ));
            }
        }
    }

    Ok(())
}

/// Parses the piece placement part of a FEN string and updates the board accordingly.
pub(crate) fn piece_placement_to_fen(board: &Board) -> String {
    let mut fen = String::new();

    for rank in (0..8).rev() {
        let mut empty_squares = 0;
        for file in 0..8 {
            let square = to_square(file, rank);
            let maybe_piece = board.piece_on_square(square);
            if let Some(piece) = maybe_piece {
                if empty_squares > 0 {
                    fen.push_str(&empty_squares.to_string());
                    empty_squares = 0;
                }
                let p = piece.0;
                let color = piece.1;

                let symbol = PIECE_SHORT_NAMES[p as usize];

                // If the piece is white, the symbol is uppercase.
                // If the piece is black, the symbol is lowercase.
                let side = if color == Side::White {
                    symbol
                } else {
                    symbol.to_ascii_lowercase()
                };

                fen.push(side);
            } else {
                empty_squares += 1;
            }
        }

        if empty_squares > 0 {
            fen.push_str(&empty_squares.to_string());
        }

        if rank > 0 {
            fen.push('/');
        }
    }

    fen
}

/// Parses the active color part of a FEN string and updates the board accordingly.
fn parse_active_color(board: &mut Board, part: &str) -> FenResult {
    if part.len() != 1 {
        return Err(FenError::with_offending_parts(
            &format!(
                "Active color length is invalid in FEN part {}",
                FenPart::ActiveColor,
            ),
            vec![FenPart::ActiveColor],
        ));
    }

    // we have validated that the string has a length of 1
    if let Some(val) = part.trim().chars().next() {
        if let Ok(side) = Side::try_from(val) {
            board.set_side_to_move(side);
        } else {
            return Err(FenError::with_offending_parts(
                &format!("Could not parse side from char {val}"),
                vec![FenPart::ActiveColor],
            ));
        }
    } else {
        return Err(FenError::with_offending_parts(
            &format!("Invalid active color string: {part}"),
            vec![FenPart::ActiveColor],
        ));
    }

    Ok(())
}

/// Converts the active color of a board to a FEN string.
pub(crate) fn active_color_to_fen(board: &Board) -> String {
    match board.side_to_move() {
        Side::White => "w".to_string(),
        Side::Black => "b".to_string(),
    }
}

/// Parses the en passant target square (if any) part of a FEN string and updates the board accordingly.
fn parse_en_passant_target_square(board: &mut Board, part: &str) -> FenResult {
    let part_length = part.len();

    // any dash present was previously converted to DASH
    if part_length == 1 && part.chars().next().unwrap() == DASH {
        board.set_en_passant_square(None);
        return Ok(());
    }

    if part_length != 2 {
        return Err(FenError::new(&format!(
            "Invalid en passant target square length in FEN part {}",
            FenPart::EnPassantTargetSquare,
        )));
    }

    let search_part = part.to_lowercase();
    if SQUARE_NAME.contains(&search_part.trim()) {
        let index = SQUARE_NAME
            .iter()
            .position(|&r| r == part.trim().to_lowercase())
            .unwrap();
        board.set_en_passant_square(Some(index as u8));
        return Ok(());
    }

    Err(FenError::new(&format!(
        "Invalid en passant target square found in FEN part {}",
        FenPart::EnPassantTargetSquare,
    )))
}

/// Converts the en passant target square of a board to a FEN string.
pub(crate) fn en_passant_target_square_to_fen(board: &Board) -> String {
    match board.en_passant_square() {
        Some(square) => SQUARE_NAME[square as usize].to_string(),
        None => "-".to_string(),
    }
}

/// Parses the castling availability part of a FEN string and updates the board accordingly.
fn parse_castling_availability(board: &mut Board, part: &str) -> FenResult {
    if part.is_empty() {
        return Err(FenError::with_offending_parts(
            &format!(
                "Empty castling availability found in FEN part {}",
                FenPart::CastlingAvailability,
            ),
            vec![FenPart::CastlingAvailability],
        ));
    }

    if part.len() == 1 && part.trim().chars().next().unwrap() == DASH {
        return Ok(());
    }

    let mut castle_rights = CastlingAvailability::NONE;

    for c in part.chars() {
        match c {
            'K' => castle_rights |= CastlingAvailability::WHITE_KINGSIDE,
            'Q' => castle_rights |= CastlingAvailability::WHITE_QUEENSIDE,
            'k' => castle_rights |= CastlingAvailability::BLACK_KINGSIDE,
            'q' => castle_rights |= CastlingAvailability::BLACK_QUEENSIDE,
            _ => {
                return Err(FenError::with_offending_parts(
                    &format!(
                        "Invalid castling availability found in FEN part {}",
                        FenPart::CastlingAvailability,
                    ),
                    vec![FenPart::CastlingAvailability],
                ));
            }
        }
    }

    board.set_castling_rights(castle_rights);

    Ok(())
}

/// Converts the castling availability of a board to a FEN string.
pub(crate) fn castling_availability_to_fen(board: &Board) -> String {
    let mut fen = String::new();

    if board.castling_rights() == CastlingAvailability::NONE {
        return "-".to_string();
    }

    if board.castling_rights() & CastlingAvailability::WHITE_KINGSIDE != 0 {
        fen.push('K');
    }
    if board.castling_rights() & CastlingAvailability::WHITE_QUEENSIDE != 0 {
        fen.push('Q');
    }
    if board.castling_rights() & CastlingAvailability::BLACK_KINGSIDE != 0 {
        fen.push('k');
    }
    if board.castling_rights() & CastlingAvailability::BLACK_QUEENSIDE != 0 {
        fen.push('q');
    }

    fen
}

/// Parses the halfmove clock part of a FEN string and updates the board accordingly.
fn parse_halfmove_clock(board: &mut Board, part: &str) -> FenResult {
    let halfmove_clock = part.trim().parse::<u32>().unwrap();
    board.set_half_move_clock(halfmove_clock);
    Ok(())
}

/// Converts the halfmove clock of a board to a FEN string.
pub(crate) fn halfmove_clock_to_fen(board: &Board) -> String {
    board.half_move_clock().to_string()
}

/// Parses the fullmove number part of a FEN string and updates the board accordingly.
fn parse_fullmove_number(board: &mut Board, part: &str) -> FenResult {
    let fullmove_number = part.trim().parse::<u32>().unwrap();
    board.set_full_move_number(fullmove_number);
    Ok(())
}

/// Converts the fullmove number of a board to a FEN string.
pub(crate) fn fullmove_number_to_fen(board: &Board) -> String {
    board.full_move_number().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{board::Board, file::File, rank::Rank, square::Square};

    #[test]
    fn format_fen_part() {
        for part in [
            FenPart::PiecePlacement,
            FenPart::ActiveColor,
            FenPart::CastlingAvailability,
            FenPart::EnPassantTargetSquare,
            FenPart::HalfmoveClock,
            FenPart::FullmoveNumber,
        ] {
            assert_eq!(format!("{part}"), part.to_string());
        }
    }

    #[test]
    fn create_fen_error() {
        let error = FenError::new("Test error");
        assert_eq!(error.message, "Test error");
        assert!(error.offending_parts.is_none());
    }

    #[test]
    fn create_fen_error_with_offending_parts() {
        let error = FenError::with_offending_parts("Test error", vec![FenPart::PiecePlacement]);
        assert_eq!(error.message, "Test error");
        assert!(error.offending_parts.is_some());
        assert!(
            error
                .offending_parts
                .is_some_and(|parts| parts.len() == 1 && parts[0] == FenPart::PiecePlacement),
        );
    }

    #[test]
    fn display_fen_error() {
        let error = FenError::new("Test error");
        assert_eq!(format!("{error}"), "Test error".to_string());

        let error_with_parts = FenError::with_offending_parts(
            "Test error",
            vec![FenPart::PiecePlacement, FenPart::ActiveColor],
        );
        assert_eq!(
            format!("{error_with_parts}"),
            "Test error Offending parts: Piece Placement Active Color ".to_string()
        );
    }

    #[test]
    fn split_empty_fen() {
        let result = split_fen_string("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().message, "FEN string is empty");
    }

    #[test]
    fn split_invalid_fen() {
        // only 3 parts
        let result = split_fen_string("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R w KQkq");
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.offending_parts.is_none());
    }

    #[test]
    fn test_parse_piece_placement() {
        let result = split_fen_string("rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R w KQkq -");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts.len(), 6);

        let is_valid_error = |error: &FenError| -> bool {
            error
                .offending_parts
                .as_ref()
                .is_some_and(|parts| !parts.is_empty() && parts[0] == FenPart::PiecePlacement)
        };

        let mut board: Board = Default::default();
        let piece_placement_part = parts[0].clone() + "//";
        let result = parse_piece_placement(&mut board, &piece_placement_part);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(is_valid_error(&error));

        // invalid chars
        let result =
            parse_piece_placement(&mut board, "rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNBQKB1R//");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(is_valid_error(&err));

        let result =
            parse_piece_placement(&mut board, "rnbqkb1r/pppppppp/8/8/8/8/PPPPPPPP/RNZQKB1R");
        let err = result.unwrap_err();
        assert!(is_valid_error(&err));
    }

    #[test]
    fn test_parse_en_passant_square() {
        let sample_ep_square = "e3";
        let mut board: Board = Default::default();
        let result = parse_en_passant_target_square(&mut board, sample_ep_square);
        assert!(result.is_ok());

        let ep_square = board.en_passant_square();
        assert!(ep_square.is_some());
        let expected_sq = Square::from_file_rank(File::E.to_char(), Rank::R3.as_number())
            .unwrap()
            .to_square_index();
        assert_eq!(ep_square.unwrap(), expected_sq);

        let bad_eq_square = "e99";
        let result = parse_en_passant_target_square(&mut board, bad_eq_square);
        assert!(result.is_err());

        let bad_eq_square2 = "z1";
        let result = parse_en_passant_target_square(&mut board, bad_eq_square2);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_castling() {
        // empty
        let mut board = Board::default_board();
        let mut result = parse_castling_availability(&mut board, "");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.offending_parts
                .is_some_and(|parts| !parts.is_empty() && parts[0] == FenPart::CastlingAvailability)
        );

        // invalid chars
        let castling_part = "KQkz";
        result = parse_castling_availability(&mut board, castling_part);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.offending_parts
                .is_some_and(|parts| !parts.is_empty() && parts[0] == FenPart::CastlingAvailability)
        );
    }

    #[test]
    fn test_parse_active_color() {
        let mut board = Board::default_board();

        let validate_errors = |error: &FenError| -> bool {
            error
                .offending_parts
                .as_ref()
                .is_some_and(|parts| !parts.is_empty() && parts[0] == FenPart::ActiveColor)
        };

        // empty string
        let mut result = parse_active_color(&mut board, "");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(validate_errors(&err));

        // invalid string (wrong size)
        result = parse_active_color(&mut board, "wb");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(validate_errors(&err));

        // valid string length but invalid content
        result = parse_active_color(&mut board, "q");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(validate_errors(&err));
    }
}
