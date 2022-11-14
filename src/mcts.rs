use std::iter::zip;
use std::time::Instant;

use tch;

use rand::prelude::*;
use rand_distr::Dirichlet;

use crate::board::{show, Action, Board, Outcome, Player};
use crate::utils::{
    get_random_action, get_torchjit_model, get_torchjit_policy_value, sample_from_weights,
};

const SQRT_TWO: f32 = 1.41421356237;
const C_BASE: f32 = 19652.0;
const C_INIT: f32 = 1.25;
const DIRICHLET_ALPHA: f32 = 0.3;
const DIRICHLET_EPSILON: f32 = 0.25;

pub fn rollout(board: &mut Board) -> Outcome {
    while !board.is_game_over() {
        let random_action = get_random_action(&board.legal_actions());
        board.make_action(random_action).expect(
            "Randomly selected action from the legal actions should not result in an error.",
        );
    }

    board
        .outcome
        .expect("The game is over and should have an outcome.")
}

#[derive(Debug)]
pub struct Node {
    action: Option<Action>,
    children: Vec<Node>,
    total_value: f32,
    prior: f32,
    visit_count: usize,
    turn: Player,
}

impl Node {
    pub fn new(action: Option<Action>, turn: Player, prior: f32) -> Self {
        Node {
            action,
            turn,
            prior,
            children: Vec::new(),
            total_value: 0.0,
            visit_count: 0,
        }
    }

    pub fn value(&self) -> f32 {
        if self.visit_count == 0 {
            return 0.0;
        }

        self.total_value / self.visit_count as f32
    }

    pub fn ucb(&self, parent_visit_count: usize) -> f32 {
        let Q_s = self.value();
        let C_s = f32::log10((1.0 + parent_visit_count as f32 + C_BASE) / C_BASE) + C_INIT;
        let U_s =
            C_s * self.prior * f32::sqrt(parent_visit_count as f32) / (1 + self.visit_count) as f32;

        Q_s + U_s
    }

    /// The output of the neural network is always from Black's perspective
    pub fn update(&mut self, value: f32) {
        match self.turn {
            Player::White => self.total_value += value,
            Player::Black => self.total_value -= value,
        }

        self.visit_count += 1;
    }

    pub fn get_best_child(&mut self) -> Option<&mut Node> {
        let mut best_score: f32 = f32::NEG_INFINITY;
        let mut best_child: Option<&mut Node> = None;

        for child in &mut self.children {
            let child_score = child.ucb(self.visit_count);
            if child_score > best_score {
                best_score = child_score;
                best_child = Some(child);
            }
        }

        best_child
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

pub struct MCTS {
    pub root: Node,
    pub board: Board,
    pub n_iterations: usize,
}

impl MCTS {
    pub fn new(board: &Board, n_iterations: usize) -> Self {
        let root = Node::new(None, board.turn, 0.0);
        let board = board.clone();
        Self {
            root,
            board,
            n_iterations,
        }
    }

    pub fn iteration(&mut self, board: &mut Board, model: &tch::CModule) {
        let mut parents_pointers: Vec<*mut Node> = Vec::new();

        // Selection
        let mut node = &mut self.root;
        parents_pointers.push(node);

        while !node.is_leaf() {
            node = node.get_best_child().unwrap();
            board.make_action(node.action.unwrap()).ok();
            parents_pointers.push(node);
        }

        // Expansion
        let value = expand(&mut node, board, &model);

        // Backpropagate
        for parent_pointer in parents_pointers.iter().rev() {
            let parent = unsafe { parent_pointer.as_mut().unwrap() };
            parent.update(value);
        }
    }

    pub fn get_best_action(&mut self, model: &tch::CModule, exploratory_play: bool) -> Action {
        let _ = expand(&mut self.root, &mut self.board.clone(), &model);
        inject_exploration_noise(&mut self.root);

        for i in 0..self.n_iterations {
            let mut board = self.board.clone();
            self.iteration(&mut board, &model);
        }

        let action = if exploratory_play {
            // Sample
            let children_probabilities = self
                .root
                .children
                .iter()
                .map(|c| c.visit_count as f32 / self.root.visit_count as f32)
                .collect();

            let child_index = sample_from_weights(&children_probabilities);
            let chosen_child = &self.root.children[child_index];
            chosen_child.action.expect("Child should have an action")
        } else {
            // Deterministic
            let mut chosen_child = &self.root.children[0];
            for child in &self.root.children {
                // println!(
                //     "{:?} -> {} {} {}",
                //     child.action.unwrap(),
                //     child.visit_count,
                //     child.total_value,
                //     child.ucb(self.root.visit_count),
                // );
                if child.visit_count > chosen_child.visit_count {
                    chosen_child = child;
                }
            }
            chosen_child.action.expect("Child should have an action")
        };

        // println!("ROOT STATS:");
        // println!("{} {}", self.root.visit_count, self.root.total_value);

        action
    }

    pub fn get_policy(&self) -> Vec<Vec<f32>> {
        let mut policy = vec![vec![0f32; self.board.size]; self.board.size];

        for child in &self.root.children {
            let [row_index, col_index] = child.action.expect("Child nodes should have an action.");
            let p = child.visit_count as f32 / self.n_iterations as f32;
            policy[row_index][col_index] = p;
        }

        policy
    }

    pub fn get_flat_policy(&self) -> Vec<f32> {
        let mut flat_policy = vec![0f32; self.board.size * self.board.size];

        for child in &self.root.children {
            let [row_index, col_index] = child.action.expect("Child nodes should have an action.");
            let p = child.visit_count as f32 / self.n_iterations as f32;
            flat_policy[row_index * self.board.size + col_index] = p;
        }

        flat_policy
    }
}

pub fn expand(node: &mut Node, board: &mut Board, model: &tch::CModule) -> f32 {
    let value = if !board.is_game_over() {
        let (policies, value) = get_torchjit_policy_value(&model, &board.to_flat_tensor());
        let legal_actions = board.legal_actions();
        for &action in legal_actions {
            let prior = policies[board.action_to_flat_index(&action)];
            let child = Node::new(Some(action), node.turn.opposite(), prior);
            node.children.push(child);
        }
        value
    } else {
        match board.outcome.expect("Just checked is_some().") {
            Outcome::Winner(Player::Black) => 1.0,
            Outcome::Winner(Player::White) => -1.0,
            Outcome::Draw => 0.0,
        }
    };

    value
}

pub fn inject_exploration_noise(root: &mut Node) {
    if root.children.len() < 2 {
        return;
    }

    let dirichlet = Dirichlet::new(&vec![DIRICHLET_ALPHA; root.children.len()]).unwrap();
    let samples = dirichlet.sample(&mut rand::thread_rng());

    for (child, noise) in zip(&mut root.children, samples) {
        child.prior = (1.0 - DIRICHLET_EPSILON) * child.prior + DIRICHLET_EPSILON * noise;
    }
}

pub fn test_basics() {
    let model = get_torchjit_model("old.pt");
    let mut board = Board::new(3, 3);

    for move_string in vec!["B2", "A2", "C3", "A1", "A3", "B3", "C1"].iter() {
        let action = board
            .parse_string_to_action(&String::from(*move_string))
            .unwrap();
        // println!("{} -> {:?}", move_string, action);
        board.make_action(action).ok();
        show(&board);
    }

    assert!(board.is_game_over());

    board.make_action([2, 1]).ok();
    show(&board);
}

pub fn test_mcts_black_wins() {
    /*
        3 X O X
        2 O X .
        1 O . .
          A B C
    */
    let model = get_torchjit_model("old.pt");
    let mut board = Board::new(3, 3);

    for move_string in vec!["B2", "A2", "C3", "A1", "A3", "B3"].iter() {
        let action = board
            .parse_string_to_action(&String::from(*move_string))
            .unwrap();
        board.make_action(action).ok();
    }
    show(&board);

    let mut mcts = MCTS::new(&board, 1_000);
    let best_action = mcts.get_best_action(&model, false);

    dbg!(&best_action);
    board.make_action(best_action).ok();
    show(&board);
}

pub fn test_mcts_white_wins() {
    /*
       3 . . .
       2 X O .
       1 X O X
         A B C
    */
    let mut board = Board::new(3, 3);
    let model = get_torchjit_model("old.pt");

    for move_string in vec!["A1", "B1", "A2", "B2", "C1"].iter() {
        let action = board
            .parse_string_to_action(&String::from(*move_string))
            .unwrap();
        board.make_action(action).ok();
    }
    show(&board);

    let mut mcts = MCTS::new(&board, 1_000);
    let best_action = mcts.get_best_action(&model, false);

    dbg!(&best_action);
    board.make_action(best_action).ok();
    show(&board);
}

pub fn benchmark() {
    let n_iterations = 400;
    let board = Board::new(3, 3);
    let model = get_torchjit_model("old.pt");
    let mut mcts = MCTS::new(&board, n_iterations);
    let now = Instant::now();

    mcts.get_best_action(&model, false);

    let elapsed_s = now.elapsed().as_secs_f32();
    println!(
        "Iterations per second: {}",
        (n_iterations as f32 / elapsed_s) as usize
    );
    println!("{} seconds per {} iterations", elapsed_s, n_iterations);
}
