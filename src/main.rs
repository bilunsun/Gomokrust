mod board;
use crate::board::Board;

fn main() {
    let mut board = Board::new(15);
    board.show();
}
