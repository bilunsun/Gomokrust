mod board;
mod game;
mod mcts;
mod utils;

fn main() {
    // game::play_random_game();
    // game::benchmark();
    // // game::play_game();
    // // mcts::test_mcts_black_wins();
    // // mcts::test_mcts_white_wins();
    // mcts::benchmark();
    // // game::play_game_against_mcts();

    // println!("Random vs MCTS");
    // game::random_against_mcts();

    // println!("");

    // println!("Random vs Random");
    // game::random_against_random();

    // game::self_play();
    let board = board::Board::new(10, 5);
    utils::get_onnx_policy_value(&board);
}
