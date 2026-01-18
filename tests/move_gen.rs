// Part of the byte-knight project.
// Author: Paul Tsouchlos (ptsouchlos) (developer.paul.123@gmail.com)
// GNU General Public License v3.0 or later
// https://www.gnu.org/licenses/gpl-3.0-standalone.html

use chess::{board::Board, move_generation::MoveGenerator, move_list::MoveList, moves::MoveType};

#[test]
fn make_promotion_move() {
    let mut board =
        Board::from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8").unwrap();
    let move_gen = MoveGenerator::new();
    let mut move_list = MoveList::new();
    move_gen.generate_moves(&board, &mut move_list, MoveType::All);
    let promotion_move = move_list.iter().find(|m| m.is_promotion()).unwrap();
    board.make_move(promotion_move, &move_gen).unwrap();
    println!("{}", board.to_fen());
    assert_eq!(
        board.to_fen(),
        "rnQq1k1r/pp2bppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R b KQ - 0 8"
    );
}
