use chess::moves::Move;

use crate::defs::{MAX_DEPTH, MAX_KILLERS_PER_PLY};

pub struct KillerMovesTable {
    table: [[Option<Move>; MAX_KILLERS_PER_PLY]; MAX_DEPTH as usize],
}

impl KillerMovesTable {
    pub(crate) fn new() -> Self {
        let table = [[None; MAX_KILLERS_PER_PLY]; MAX_DEPTH as usize];

        Self { table }
    }

    pub(crate) fn get(&self, ply: u8) -> &[Option<Move>] {
        assert!(ply < MAX_DEPTH, "Depth is out of bounds");

        &self.table[ply as usize][..]
    }

    fn get_mut(&mut self, ply: u8) -> &mut [Option<Move>] {
        assert!(ply < MAX_DEPTH, "Depth is out of bounds");

        &mut self.table[ply as usize][..]
    }

    pub(crate) fn update(&mut self, ply: u8, mv: Move) {
        assert!(ply < MAX_DEPTH, "Depth is out of bounds");

        let current_killers = self.get_mut(ply);
        if !current_killers[0].is_some_and(|killer_mv| killer_mv == mv) {
            current_killers.swap(0, 1);
            current_killers[0] = Some(mv);
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
    use chess::{board::Board, move_generation::MoveGenerator, move_list::MoveList};

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
        let move_gen = MoveGenerator::default();
        let mut move_list = MoveList::default();
        move_gen.generate_legal_moves(&board, &mut move_list);

        let mv_a = *move_list.at(0).unwrap();
        let mv_b = *move_list.at(1).unwrap();

        kt.update(0, mv_a);
        kt.update(0, mv_b);
        kt.update(0, mv_b); // duplicate — should NOT evict A from slot 1

        assert_eq!(kt.get(0)[0], Some(mv_b));
        assert_eq!(kt.get(0)[1], Some(mv_a)); // A should still be here
    }

    #[test]
    fn killer_update_rotates_slots() {
        let mut kt = KillerMovesTable::new();
        let board = Board::default_board();
        let move_gen = MoveGenerator::default();
        let mut move_list = MoveList::default();
        move_gen.generate_legal_moves(&board, &mut move_list);

        let mv_a = *move_list.at(0).unwrap();
        let mv_b = *move_list.at(1).unwrap();
        let mv_c = *move_list.at(2).unwrap();

        kt.update(0, mv_a);
        assert_eq!(kt.get(0)[0], Some(mv_a));
        assert_eq!(kt.get(0)[1], None);

        kt.update(0, mv_b);
        assert_eq!(kt.get(0)[0], Some(mv_b));
        assert_eq!(kt.get(0)[1], Some(mv_a)); // A rotated to slot 1

        kt.update(0, mv_c);
        assert_eq!(kt.get(0)[0], Some(mv_c));
        assert_eq!(kt.get(0)[1], Some(mv_b)); // B rotated, A evicted
    }
}
