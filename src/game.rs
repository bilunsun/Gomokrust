use indexmap::IndexSet;
use rand::Rng;
use std::io::{self, Write};
use std::time::Instant;

use crate::board::{Action, Board, Outcome, Player};

fn get_random_square_index(legal_moves_indices_indexset: &IndexSet<usize>) -> usize {
    let random_index = rand::thread_rng().gen_range(0..legal_moves_indices_indexset.len());

    *legal_moves_indices_indexset
        .get_index(random_index)
        .expect("The random index should be in the IndexSet.")
}

pub fn play_game() {
    let mut board = Board::new(3, 3);
    println!("{board}");

    while !board.is_game_over() {
        let mut square_string = String::new();
        loop {
            square_string.clear();
            println!("{:?}", board.legal_moves_as_strings());

            print!("\nYour move: ");
            io::stdout().flush().unwrap();
            io::stdin()
                .read_line(&mut square_string)
                .expect("Failed to read line");
            square_string = square_string.trim().to_string();

            let action = board.parse_string_to_action(&square_string);
            if action.is_ok() && board.make_action(action.unwrap()).is_ok() {
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
        let random_square_index = get_random_square_index(&board.legal_moves());
        board
            .make_action(Action(random_square_index))
            .expect("Randomly selected move from the legal moves should not result in an error.");

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
            let random_square_index = get_random_square_index(&board.legal_moves());
            board.make_action(Action(random_square_index)).expect(
                "Randomly selected move from the legal moves should not result in an error.",
            );
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
            let random_square_index = get_random_square_index(&board.legal_moves());
            board.make_action(Action(random_square_index)).expect(
                "Randomly selected move from the legal moves should not result in an error.",
            );
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
