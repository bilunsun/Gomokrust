use indexmap::IndexSet;
use rand::Rng;

pub fn get_random_action(legal_moves: &IndexSet<usize>) -> usize {
    let random_index = rand::thread_rng().gen_range(0..legal_moves.len());

    *legal_moves
        .get_index(random_index)
        .expect("The random index should be in the IndexSet.")
}
