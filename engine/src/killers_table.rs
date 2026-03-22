use chess::{moves::Move, pieces::Piece};

use crate::defs::{MAX_DEPTH, MAX_KILLERS_PER_PLY};

/// A killer move entry stores both the move and the piece that was moving.
/// The piece is needed because with 16-bit moves, the piece type is no longer
/// encoded in the move itself, and killer comparisons across sibling branches
/// must verify the piece matches to avoid false positives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct KillerEntry {
    pub mv: Move,
    pub piece: Piece,
}

impl KillerEntry {
    pub fn new(mv: Move, piece: Piece) -> Self {
        Self { mv, piece }
    }

    /// Returns true if this killer entry matches the given move and piece.
    pub fn matches(&self, mv: Move, piece: Piece) -> bool {
        self.mv == mv && self.piece == piece
    }
}

pub struct KillerMovesTable {
    table: [[Option<KillerEntry>; MAX_KILLERS_PER_PLY]; MAX_DEPTH as usize],
}

impl KillerMovesTable {
    pub(crate) fn new() -> Self {
        let table = [[None; MAX_KILLERS_PER_PLY]; MAX_DEPTH as usize];

        Self { table }
    }

    pub(crate) fn get(&self, ply: u8) -> &[Option<KillerEntry>] {
        assert!(ply < MAX_DEPTH, "Depth is out of bounds");

        &self.table[ply as usize][..]
    }

    fn get_mut(&mut self, ply: u8) -> &mut [Option<KillerEntry>] {
        assert!(ply < MAX_DEPTH, "Depth is out of bounds");

        &mut self.table[ply as usize][..]
    }

    pub(crate) fn update(&mut self, ply: u8, mv: Move, piece: Piece) {
        assert!(ply < MAX_DEPTH, "Depth is out of bounds");

        let entry = KillerEntry::new(mv, piece);
        let current_killers = self.get_mut(ply);
        if !current_killers[0].is_some_and(|k| k == entry) {
            current_killers.swap(0, 1);
            current_killers[0] = Some(entry);
        }
    }

    pub(crate) fn clear(&mut self) {
        for item in self.table.as_flattened_mut() {
            *item = None;
        }
    }
}

impl Default for KillerMovesTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::KillerMovesTable;
    use crate::defs::{MAX_DEPTH, MAX_KILLERS_PER_PLY};
    use chess::{board::Board, move_generation, moves::MoveType, pieces::Piece};

    #[allow(clippy::expect_used)]
    fn piece_for_move(board: &Board, mv: &chess::moves::Move) -> Piece {
        board
            .piece_on_square(mv.from())
            .map(|(pc, _)| pc)
            .expect("From piece must exist.")
    }

    #[test]
    fn initialize_killers_table() {
        let killers_table: KillerMovesTable = Default::default();
        for i in 0..MAX_DEPTH {
            let killers = killers_table.get(i);
            assert_eq!(killers, &[None, None]);
            assert_eq!(killers.len(), MAX_KILLERS_PER_PLY);
        }
    }

    #[test]
    fn killer_update_no_duplicate_in_slot0() {
        let mut kt = KillerMovesTable::new();
        let board = Board::default_board();

        let move_list = move_generation::generate_legal_moves(&board, MoveType::All);

        let mv_a = *move_list.at(0).unwrap();
        let mv_b = *move_list.at(1).unwrap();
        let piece_a = piece_for_move(&board, &mv_a);
        let piece_b = piece_for_move(&board, &mv_b);

        kt.update(0, mv_a, piece_a);
        kt.update(0, mv_b, piece_b);
        kt.update(0, mv_b, piece_b); // duplicate — should NOT evict A from slot 1

        assert!(kt.get(0)[0].unwrap().matches(mv_b, piece_b));
        assert!(kt.get(0)[1].unwrap().matches(mv_a, piece_a)); // A should still be here
    }

    #[test]
    fn killer_update_rotates_slots() {
        let mut kt = KillerMovesTable::new();
        let board = Board::default_board();
        let move_list = move_generation::generate_legal_moves(&board, MoveType::All);

        let mv_a = *move_list.at(0).unwrap();
        let mv_b = *move_list.at(1).unwrap();
        let mv_c = *move_list.at(2).unwrap();
        let piece_a = piece_for_move(&board, &mv_a);
        let piece_b = piece_for_move(&board, &mv_b);
        let piece_c = piece_for_move(&board, &mv_c);

        kt.update(0, mv_a, piece_a);
        assert!(kt.get(0)[0].unwrap().matches(mv_a, piece_a));
        assert_eq!(kt.get(0)[1], None);

        kt.update(0, mv_b, piece_b);
        assert!(kt.get(0)[0].unwrap().matches(mv_b, piece_b));
        assert!(kt.get(0)[1].unwrap().matches(mv_a, piece_a)); // A rotated to slot 1

        kt.update(0, mv_c, piece_c);
        assert!(kt.get(0)[0].unwrap().matches(mv_c, piece_c));
        assert!(kt.get(0)[1].unwrap().matches(mv_b, piece_b)); // B rotated, A evicted
    }
}
