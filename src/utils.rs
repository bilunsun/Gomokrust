use indexmap::IndexSet;
use rand::Rng;

extern crate tract_core;
extern crate tract_onnx;
use tract_onnx::prelude::*;

use ndarray::{Array, Array3, Array4};

use crate::board::{Action, Board};

pub fn get_random_action(legal_moves: &IndexSet<Action>) -> Action {
    let random_index = rand::thread_rng().gen_range(0..legal_moves.len());

    *legal_moves
        .get_index(random_index)
        .expect("The random index should be in the IndexSet.")
}

pub fn get_onnx_policy_value(board: &Board) -> (usize, usize, f32) {
    let model = tract_onnx::onnx()
        .model_for_path("test.onnx")
        .expect("Should be able to load the model.")
        .into_runnable()
        .expect("Should be able to run the model.");

    let board_repr = board
        .to_repr()
        .into_iter()
        .flat_map(|s| s.into_iter().flatten().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let state_array = Array::from(board_repr).into_shape((1, 3, 10, 10)).unwrap();

    let state_tensor = Tensor::from(state_array);
    let inputs = tvec!(state_tensor);
    let outputs = model.run(inputs).unwrap();

    let max_row: usize = *outputs.get(0).unwrap().to_scalar::<i64>().unwrap() as usize;
    let max_col: usize = *outputs.get(1).unwrap().to_scalar::<i64>().unwrap() as usize;
    let value: f32 = *outputs.get(2).unwrap().to_scalar().unwrap();
    dbg!(&max_row);
    dbg!(&max_col);
    dbg!(&value);

    (max_row, max_col, value)
}
