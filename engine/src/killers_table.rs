use chess::{moves::Move, pieces::Piece};

use crate::defs::{MAX_KILLERS_PER_PLY, MAX_PLY};

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

/// Indexed by recursion ply; sized to `MAX_PLY` to match the negamax ply guard.
pub struct KillerMovesTable {
    table: [[Option<KillerEntry>; MAX_KILLERS_PER_PLY]; MAX_PLY as usize],
}

impl KillerMovesTable {
    pub(crate) fn new() -> Self {
        let table = [[None; MAX_KILLERS_PER_PLY]; MAX_PLY as usize];

        Self { table }
    }

    pub(crate) fn get(&self, ply: usize) -> &[Option<KillerEntry>] {
        &self.table[ply][..]
    }

    fn get_mut(&mut self, ply: usize) -> &mut [Option<KillerEntry>] {
        &mut self.table[ply][..]
    }

    pub(crate) fn update(&mut self, ply: usize, mv: Move, piece: Piece) {
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
    use crate::defs::{MAX_KILLERS_PER_PLY, MAX_PLY};
    use chess::{board::Board, move_generation, pieces::Piece};

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
        for ply in 0..(MAX_PLY as usize) {
            let killers = killers_table.get(ply);
            assert_eq!(killers, &[None, None]);
            assert_eq!(killers.len(), MAX_KILLERS_PER_PLY);
        }
    }

    /// Regression test for the 2026-05-12 SPRT crashes: prior to widening
    /// the table from `MAX_DEPTH` (128) to `MAX_PLY` (256), the asserts in
    /// `get`/`update`/`get_mut` panicked at `ply >= 128`. The negamax ply
    /// guard only bounds at `MAX_PLY`, so deep recursion could cause issues/crashes.
    #[test]
    fn high_ply_access_does_not_panic() {
        let mut kt = KillerMovesTable::new();
        let board = Board::default_board();
        let move_list = move_generation::legal::generate_all_moves(&board);
        let mv = *move_list.at(0).unwrap();
        let piece = piece_for_move(&board, &mv);

        for ply in [128usize, 200, (MAX_PLY as usize) - 1] {
            let _ = kt.get(ply);
            kt.update(ply, mv, piece);
            assert!(kt.get(ply)[0].is_some_and(|k| k.matches(mv, piece)));
        }
    }

    #[test]
    fn killer_update_no_duplicate_in_slot0() {
        let mut kt = KillerMovesTable::new();
        let board = Board::default_board();

        let move_list = move_generation::legal::generate_all_moves(&board);

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
        let move_list = move_generation::legal::generate_all_moves(&board);

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
