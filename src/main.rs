mod board;
mod game;

fn main() {
    game::play_random_game();
    game::benchmark();
    game::check_stats();
    // game::play_game();
}
