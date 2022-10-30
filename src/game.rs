use std::io::{self, Write};
use std::time::Instant;

use crate::board::Board;

pub fn play_game() {
    let mut board = Board::new(5, 3);
    board.show();

    let legal_moves = board.legal_moves();
    println!("{:?}", legal_moves);

    // let square_string = String::from("A1");
    // board.place_stone(&square_string);

    while !board.is_game_over() {
        let mut square_string = String::new();
        loop {
            square_string.clear();

            print!("\nYour move: ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut square_string)
                .expect("Failed to read line");
            square_string = square_string.trim().to_string();

            if let Ok(()) = board.place_stone(&square_string) {
                break;
            } else {
                println!("{square_string} is not a valid move.");
            }
        }
        board.show();
    }
}

pub fn play_random_game() {
    // let mut outcomes = vec![];
    let mut board = Board::new(15, 5);
    let now = Instant::now();
    for _ in 0..100 {
        board.reset();
        while !board.is_game_over() {
            board.place_stone_at_random();
        }

        // outcomes.push(board.outcome);
    }

    let elapsed_s = now.elapsed().as_secs_f32();
    println!("Games per second: {}", 100.0 / elapsed_s)
}
