use indexmap::IndexSet;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::Rng;

extern crate tch;

use crate::board::Action;

pub fn get_random_action(legal_moves: &IndexSet<Action>) -> Action {
    let random_index = rand::thread_rng().gen_range(0..legal_moves.len());

    *legal_moves
        .get_index(random_index)
        .expect("The random index should be in the IndexSet.")
}

pub fn get_torchjit_model(path: &str) -> tch::CModule {
    tch::CModule::load(path).expect("Should be able to load the model")
}

pub fn get_torchjit_policy_value(
    model: &tch::CModule,
    board_tensor: &tch::Tensor,
) -> (Vec<f32>, f32) {
    let outputs = model
        .forward_ts(&[board_tensor])
        .expect("Should return a tensor");

    let outputs: Vec<f32> = outputs.get(0).into();

    let policy_logits = outputs[0..outputs.len() - 1].to_vec();
    let policies = softmax(policy_logits);
    let value = outputs[outputs.len() - 1];

    (policies, value)
}

pub fn softmax(logits: Vec<f32>) -> Vec<f32> {
    let max = logits.iter().fold(f32::NEG_INFINITY, |m, v| m.max(*v));
    let numerator: Vec<f32> = logits.iter().map(|v| (v - max).exp()).collect();
    let denominator: f32 = numerator.iter().sum();
    let softmax = numerator.iter().map(|v| v / denominator).collect();

    softmax
}

pub fn sample_from_weights(weights: &Vec<f32>) -> usize {
    let dist = WeightedIndex::new(weights).unwrap();
    let mut rng = thread_rng();
    dist.sample(&mut rng)
}
