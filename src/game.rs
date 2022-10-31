use std::io::{self, Write};
use std::time::Instant;

use crate::board::{Board, Outcome, Player};

pub fn play_game() {
    let mut board = Board::new(3, 3);
    println!("{board}");

    while !board.is_game_over() {
        let mut square_string = String::new();
        loop {
            square_string.clear();
            println!("{:?}", board.legal_moves());

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
        println!("{board}");
    }
}

pub fn play_random_game() {
    let mut board = Board::new(3, 3);
    while !board.is_game_over() {
        board.place_stone_at_random();
        println!("{board}");
    }
}

pub fn benchmark() {
    let n_games = 10_000;
    let mut board = Board::new(15, 5);
    let now = Instant::now();
    for _ in 0..n_games {
        board.reset();
        while !board.is_game_over() {
            board
                .place_stone_at_random()
                .expect("The game has ended and should have an outcome.");
        }
    }

    let elapsed_s = now.elapsed().as_secs_f32();
    println!(
        "Games per second: {}",
        (n_games as f32 / elapsed_s) as usize
    );
}

pub fn check_stats() {
    let n_games = 10_000;
    let mut board = Board::new(15, 5);

    let mut black_wins = 0;
    let mut white_wins = 0;
    let mut draws = 0;
    let mut num_stones_placed = 0;
    for _ in 0..n_games {
        board.reset();
        while !board.is_game_over() {
            board.place_stone_at_random().unwrap();
        }

        match board.outcome {
            Some(outcome) => match outcome {
                Outcome::Winner(player) => match player {
                    Player::Black => black_wins += 1,
                    Player::White => white_wins += 1,
                },
                Outcome::Draw => draws += 1,
            },
            None => panic!("The game has ended and should have an outcome."),
        }

        num_stones_placed += board.num_stones_placed;
    }

    println!(
        "Black wins: {:.1}%",
        black_wins as f32 / n_games as f32 * 100.0
    );
    println!(
        "White wins: {:.1}%",
        white_wins as f32 / n_games as f32 * 100.0
    );
    println!("Draws: {:.1}%", draws as f32 / n_games as f32 * 100.0);
    println!("Stones placed: {:}", num_stones_placed / n_games);
}
