use std::io::{self, Write};
use std::time::Instant;

extern crate serde_json;
use serde_json::{json, Value};

extern crate uuid;
use uuid::Uuid;

extern crate rayon;
use rayon::prelude::*;

use crate::board::{show, Action, Board, Outcome, Player};
use crate::mcts::MCTS;
use crate::utils::{get_random_action, get_torchjit_model};

pub fn play_game() {
    let mut board = Board::new(3, 3);
    show(&board);

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
        show(&board);
    }
}

pub fn play_random_game() {
    let mut board = Board::new(3, 3);
    while !board.is_game_over() {
        let random_action = get_random_action(&board.legal_actions());
        board.make_action(random_action).expect(
            "Randomly selected action from the legal actions should not result in an error.",
        );

        show(&board);
    }
}

pub fn benchmark() {
    let n_games = 1_000;
    let mut board = Board::new(15, 5);
    let now = Instant::now();
    for _ in 0..n_games {
        board.reset();
        while !board.is_game_over() {
            let random_action = get_random_action(&board.legal_actions());
            board.make_action(random_action).expect(
                "Randomly selected action from the legal actions should not result in an error.",
            );
        }
    }

    let elapsed_s = now.elapsed().as_secs_f32();
    println!(
        "Games per second: {}",
        (n_games as f32 / elapsed_s) as usize
    );
}

pub fn random_against_random() {
    let n_games = 10_000;
    let mut board = Board::new(3, 3);

    let mut black_wins = 0;
    let mut white_wins = 0;
    let mut draws = 0;
    let mut num_stones_placed = 0;
    for _ in 0..n_games {
        board.reset();
        while !board.is_game_over() {
            let random_action = get_random_action(&board.legal_actions());
            board.make_action(random_action).expect(
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

pub fn get_player_action(board: &Board) -> Action {
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
        if action.is_ok() && board.legal_actions().contains(&action.unwrap()) {
            return action.unwrap();
        } else {
            println!("{square_string} is not a valid move.");
        }
    }
}

pub fn play_game_against_mcts() {
    let model = get_torchjit_model("old.pt");
    let mut board = Board::new(3, 3);
    show(&board);

    while !board.is_game_over() {
        let action: Action;
        if board.turn == Player::White {
            action = get_player_action(&mut board);
        } else {
            let mut mcts = MCTS::new(&board, 400);
            action = mcts.get_best_action(&model, false);
        }
        board.make_action(action).ok();
        show(&board);
    }

    dbg!(&board.outcome);
}

// pub fn random_against_mcts() {
//     let n_games = 100;
//     let mcts_player = Player::White;
//     let mut mcts_wins = 0;
//     let mut random_wins = 0;
//     let mut draws = 0;
//     for i in 0..n_games {
//         println!("{}", i);
//         let mut board = Board::new(10, 5);

//         while !board.is_game_over() {
//             let action: Action;
//             if board.turn == mcts_player {
//                 let mut mcts = MCTS::new(&board, 800);
//                 action = mcts.get_best_action();
//             } else {
//                 action = get_random_action(&board.legal_actions());
//             }
//             board.make_action(action).ok();
//         }

//         match board.outcome {
//             Some(outcome) => match outcome {
//                 Outcome::Winner(winner) => {
//                     if winner == mcts_player {
//                         mcts_wins += 1;
//                     } else {
//                         random_wins += 1;
//                     }
//                 }
//                 Outcome::Draw => draws += 1,
//             },
//             None => panic!("The game has ended and should have an outcome."),
//         }
//     }
//     println!(
//         "MCTS wins: {:.1}%",
//         mcts_wins as f32 / n_games as f32 * 100.0
//     );
//     println!(
//         "Random wins: {:.1}%",
//         random_wins as f32 / n_games as f32 * 100.0
//     );
//     println!("Draws: {:.1}%", draws as f32 / n_games as f32 * 100.0);
// }

pub fn self_play_single_game(size: usize, n_in_a_row: usize, n_mcts_simulations: usize) {
    let model = get_torchjit_model("test.pt");
    let mut board = Board::new(size, n_in_a_row);

    let mut policies = Vec::new();
    let mut board_vecs = Vec::new();

    while !board.is_game_over() {
        let mut mcts = MCTS::new(&board, n_mcts_simulations);
        let action = mcts.get_best_action(&model, true);

        policies.push(mcts.get_flat_policy());
        board_vecs.push(board.to_flat_vec());

        board.make_action(action).ok();
    }

    // Create values
    let value = match board
        .outcome
        .expect("The game has ended and should have an outcome.")
    {
        Outcome::Winner(winner) => match winner {
            Player::Black => 1.0,
            Player::White => -1.0,
        },
        Outcome::Draw => 0.0,
    };

    // JSON
    let mut game_json: Vec<Value> = vec![];
    for (board_vec, policy) in board_vecs.iter().zip(policies.iter()) {
        game_json.push(json!({
            "state": board_vec,
            "policy": policy,
            "value": value
        }));
    }
    std::fs::write(
        format!("games/{}.json", Uuid::new_v4()),
        serde_json::to_string_pretty(&game_json).unwrap(),
    )
    .unwrap();
}

pub fn self_play(n_games: usize) {
    let size: usize = 8;
    let n_in_a_row: usize = 5;
    let n_mcts_simulations = 400;

    let total_elapsed_s: f32 = (0..n_games)
        .collect::<Vec<usize>>()
        .par_iter()
        .map(|i| {
            let now = Instant::now();
            self_play_single_game(size, n_in_a_row, n_mcts_simulations);
            let elapsed_s = now.elapsed().as_secs_f32();
            println!("Seconds per game: {}", elapsed_s);
            elapsed_s
        })
        .sum();

    println!(
        "Average seconds per game: {}",
        total_elapsed_s / n_games as f32
    )
}

pub fn ai_vs_ai_single(
    size: usize,
    n_in_a_row: usize,
    n_mcts_simulations: usize,
    new_player: Player,
) -> Outcome {
    let old_model = get_torchjit_model("old.pt");
    let new_model = get_torchjit_model("new.pt");
    let mut board = Board::new(size, n_in_a_row);

    while !board.is_game_over() {
        let mut mcts = MCTS::new(&board, n_mcts_simulations);

        let action = if board.turn == new_player {
            mcts.get_best_action(&new_model, false)
        } else {
            mcts.get_best_action(&old_model, false)
        };

        board.make_action(action).ok();
    }

    board.outcome.expect("Game over should have an outcome.")
}

pub fn ai_vs_ai(size: usize, n_in_a_row: usize, n_mcts_simulations: usize) {
    let n_games = 400;

    let new_wins: Vec<f32> = (0..n_games)
        .collect::<Vec<usize>>()
        .par_iter()
        .map(|i| {
            let new_player = if i % 2 == 0 {
                Player::Black
            } else {
                Player::White
            };
            let outcome = ai_vs_ai_single(size, n_in_a_row, n_mcts_simulations, new_player);

            if let Outcome::Winner(winner) = outcome {
                if winner == new_player {
                    1.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        })
        .collect();

    let n_games_played = new_wins.len(); // Sometimes par_iter gives less than n_games?
    let new_wins_ratio: f32 = new_wins.iter().sum::<f32>() / n_games_played as f32;

    // println!("Old wins: {}", old_wins);
    // println!("New wins: {}", new_wins);
    // println!("Draws: {}", draws);
    println!("New wins ratio: {}", new_wins_ratio);
}
