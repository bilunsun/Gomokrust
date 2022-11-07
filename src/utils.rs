use indexmap::IndexSet;
use rand::Rng;

extern crate serde_json;
use serde_json::{json, Value};

extern crate ndarray;
use ndarray::prelude::*;

extern crate itertools;
use itertools::izip;

use crate::board::{Action, Board};
use crate::mcts::MCTS;

pub fn get_random_action(legal_moves: &IndexSet<Action>) -> Action {
    let random_index = rand::thread_rng().gen_range(0..legal_moves.len());

    *legal_moves
        .get_index(random_index)
        .expect("The random index should be in the IndexSet.")
}

// pub fn game_to_json(
//     policies: Vec<Vec<f32>>,
//     value: f32,
//     actions: Vec<Action>,
//     board: &mut Board,
// ) -> Value {
//     board.reset();

//     // for
//     let board_repr = board.to_repr();

//     let json_data = json!({
//         "value": 1.0,
//         "policy": [1, 2, 3],
//         "state": board_repr,
//     });

//     println!("{}", json_data);
//     json_data
// }
