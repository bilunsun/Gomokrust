mod board;
mod game;
mod mcts;
mod utils;

fn main() {
    // game::play_random_game();
    game::benchmark();
    // game::play_game();
    // mcts::test_mcts_black_wins();
    // mcts::test_mcts_white_wins();
    mcts::benchmark();
    // game::play_game_against_mcts();

    println!("Random vs MCTS");
    game::random_against_mcts();

    println!("");

    println!("Random vs Random");
    game::random_against_random();

    let mut board = board::Board::new(3, 3);

    for move_string in vec!["B2", "A2", "C3", "A1", "A3", "B3"].iter() {
        let action = board
            .parse_string_to_action(&String::from(*move_string))
            .unwrap();
        board.make_action(action).ok();
    }
    board::show(&board);
    utils::board_to_repr(&board);
}
