use indexmap::IndexSet;
use rand::Rng;

extern crate serde_json;
use serde_json::{json, Value};

use crate::board::{Action, Board};
use crate::mcts::MCTS;

pub fn get_random_action(legal_moves: &IndexSet<usize>) -> Action {
    let random_index = rand::thread_rng().gen_range(0..legal_moves.len());

    *legal_moves
        .get_index(random_index)
        .expect("The random index should be in the IndexSet.")
}

// pub fn mcts_to_json(mcts: &MCTS) -> Value {
//     let board = &mcts.board;
//     json!({
//         "state": board_state,
//         "policy": policy,
//         "value": value
//     });
// }
