mod board;
mod game;
mod mcts;
mod utils;

fn main() {
    // game::play_random_game();
    // game::benchmark();
    // game::check_stats();
    // game::play_game();
    mcts::test_mcts_black_wins();
    mcts::test_mcts_white_wins();
    game::play_game_against_mcts();
}
