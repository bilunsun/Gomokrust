mod board;
mod game;
mod mcts;
mod utils;

fn main() {
    // game::play_random_game();
    game::benchmark();
    // game::check_stats();
    // game::play_game();
    // mcts::test_mcts_black_wins();
    // mcts::test_mcts_white_wins();
    // mcts::benchmark();
    // game::play_game_against_mcts();

    // println!("Random vs MCTS");
    // game::random_against_mcts();

    // println!("");

    // println!("Random vs Random");
    // game::random_against_random();

    let mut base_board = board::BaseBoard::new(6);

    base_board.set(board::Player::Black, (0, 0));
    println!("{:?}", base_board);
}
