use std::io::{self, Write};

use crate::board::Board;

pub fn play_game() {
    let mut board = Board::new(15);
    board.show();

    let legal_moves = board.legal_moves();
    println!("{:?}", legal_moves);

    for _ in 0..3 {
        let mut square_string = String::new();
        loop {
            square_string.clear();

            print!("\nYour move: ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut square_string)
                .expect("Failed to read line");
            square_string = square_string.trim().to_string();

            if board.place_stone(&square_string).is_some() {
                break;
            } else {
                println!("{square_string} is not a valid move.");
            }
        }
        board.show();
    }
}
