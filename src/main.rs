mod board;
mod game;
mod mcts;
mod utils;

use rand::prelude::*;
use rand_distr::Dirichlet;

use std::time::Instant;

fn main() {
    // game::play_random_game();
    // game::benchmark();
    // // game::play_game();
    // mcts::test_mcts_black_wins();
    // mcts::test_basics();
    // mcts::test_mcts_white_wins();
    // mcts::benchmark();
    // game::play_game_against_mcts();

    // println!("Random vs MCTS");
    // game::random_against_mcts();

    // println!("");

    // println!("Random vs Random");
    // game::random_against_random();

    // game::self_play_single_game(5, 5, 400);
    // let now = Instant::now();
    game::self_play(5_000);

    // game::ai_vs_ai(8, 5, 100);

    // let elapsed = now.elapsed().as_secs_f32();
    // println!("TOTAL {}s", elapsed);
    // let board = board::Board::new(10, 5);
    // let model = utils::get_torchjit_model();
    // utils::get_torchjit_policy_value(&model, &board.to_flat_tensor());

    // let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    // let policies = utils::softmax(values);
    // dbg!(&policies);

    // let dirichlet = Dirichlet::new(&vec![0.03; 10]).unwrap();
    // let samples = dirichlet.sample(&mut rand::thread_rng());
    // dbg!(&samples);
}
