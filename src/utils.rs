use indexmap::IndexSet;
use rand::Rng;

extern crate tch;

use crate::board::Action;

pub fn get_random_action(legal_moves: &IndexSet<Action>) -> Action {
    let random_index = rand::thread_rng().gen_range(0..legal_moves.len());

    *legal_moves
        .get_index(random_index)
        .expect("The random index should be in the IndexSet.")
}

pub fn get_torchjit_model() -> tch::CModule {
    tch::CModule::load("model.pt").expect("Should be able to load the model")
}

pub fn get_torchjit_policy_value(
    model: &tch::CModule,
    board_tensor: &tch::Tensor,
) -> (Vec<f32>, f32) {
    let outputs = model
        .forward_ts(&[board_tensor])
        .expect("Should return a tensor");

    let outputs: Vec<f32> = outputs.get(0).into();

    let policies = outputs[0..outputs.len() - 1].to_vec();
    let value = outputs[outputs.len() - 1];

    (policies, value)
}
