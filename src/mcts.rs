use std::collections::HashMap;
use std::time::Instant;

use tch;

use crate::board::{Action, Board, Outcome, Player};
use crate::utils::{get_random_action, get_torchjit_model, get_torchjit_policy_value};

const SQRT_TWO: f32 = 1.41421356237;

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
    value: f32,
    visit_count: usize,
    turn: Player,
    board_tensor: tch::Tensor,
}

impl Node {
    pub fn new(action: Option<Action>, turn: Player, board_tensor: tch::Tensor) -> Self {
        Node {
            action,
            turn,
            board_tensor,
            children: Vec::new(),
            value: 0.0,
            visit_count: 0,
        }
    }

    pub fn uct(&self, parent_visit_count: usize) -> f32 {
        if self.visit_count == 0 {
            return f32::INFINITY;
        }
        let exploitation_term = self.value / (self.visit_count as f32);
        let exploration_term =
            SQRT_TWO * f32::sqrt(f32::ln(parent_visit_count as f32) / (self.visit_count as f32));
        exploitation_term + exploration_term
    }

    pub fn update_with_outcome(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Winner(winner) => {
                if winner != self.turn {
                    self.value += 1.0
                } else {
                    self.value -= 1.0;
                }
            }
            _ => (),
        };

        self.visit_count += 1;
    }

    /// The output of the neural network is always from Black's perspective
    pub fn update(&mut self, value: f32) {
        match self.turn {
            Player::White => self.value += value,
            Player::Black => self.value -= value,
        }

        self.visit_count += 1;
    }

    pub fn get_best_child(&mut self) -> Option<&mut Node> {
        let mut best_score: f32 = f32::NEG_INFINITY;
        let mut best_child: Option<&mut Node> = None;

        for child in &mut self.children {
            let child_score = child.uct(self.visit_count);
            if child_score > best_score {
                best_score = child_score;
                best_child = Some(child);
            }
        }

        best_child
    }

    pub fn expand(&mut self, board: &Board) -> Option<&mut Node> {
        if board.is_game_over() {
            return None;
        }

        // Create a child node for all untried moves
        let legal_actions = board.legal_actions();
        for action in legal_actions {
            let child = Node::new(Some(*action), self.turn.opposite(), board.to_flat_tensor());
            self.children.push(child);
        }

        Some(&mut self.children[0])
    }
}

pub struct MCTS {
    pub root: Node,
    pub board: Board,
    pub n_iterations: usize,
}

impl MCTS {
    pub fn new(board: &Board, n_iterations: usize) -> Self {
        let root = Node::new(None, board.turn, board.to_flat_tensor());
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
        while !node.children.is_empty() {
            parents_pointers.push(node);
            node = node.get_best_child().unwrap();
            board.make_action(node.action.unwrap()).ok();
        }

        // Expansion
        let value = match node.expand(board) {
            Some(_) => {
                let (policies, value) = get_torchjit_policy_value(&model, &node.board_tensor); // Rollout
                value
            }
            None => {
                let outcome = board
                    .outcome
                    .expect("Terminal state should have an outcome.");

                match outcome {
                    Outcome::Winner(Player::Black) => 1.0,
                    Outcome::Winner(Player::White) => -1.0,
                    Outcome::Draw => 0.0,
                }
            }
        };

        // Backpropagate
        node.update(value);
        for parent_pointer in parents_pointers.iter().rev() {
            let parent = unsafe { parent_pointer.as_mut().unwrap() };
            parent.update(value);
        }
    }

    pub fn get_best_action(&mut self, model: &tch::CModule) -> Action {
        for i in 0..self.n_iterations {
            let mut board = self.board.clone();
            self.iteration(&mut board, &model);
        }

        let best_child = self
            .root
            .get_best_child()
            .expect("Root node should have children.");
        best_child.action.expect("Child should have action")
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

// pub fn test_mcts_black_wins() {
//     /*
//         3 X O X
//         2 O X .
//         1 O . .
//           A B C
//     */
//     let mut board = Board::new(3, 3);

//     for move_string in vec!["B2", "A2", "C3", "A1", "A3", "B3"].iter() {
//         let action = board
//             .parse_string_to_action(&String::from(*move_string))
//             .unwrap();
//         board.make_action(action).ok();
//     }

//     let mut mcts = MCTS::new(&board, 1_000);
//     let best_action = mcts.get_best_action();

//     dbg!(&best_action);

//     // assert!(best_action == 18);
// }

// pub fn test_mcts_white_wins() {
//     /*
//        3 . . .
//        2 X O .
//        1 X O X
//          A B C
//     */
//     let mut board = Board::new(3, 3);

//     for move_string in vec!["A1", "B1", "A2", "B2", "C1"].iter() {
//         let action = board
//             .parse_string_to_action(&String::from(*move_string))
//             .unwrap();
//         board.make_action(action).ok();
//     }

//     let mut mcts = MCTS::new(&board, 1_000);
//     let best_action = mcts.get_best_action();

//     dbg!(&best_action);
//     // assert!(best_action == 31);
// }

// pub fn benchmark() {
//     let n_iterations = 1_600;
//     let board = Board::new(15, 5);
//     let mut mcts = MCTS::new(&board, n_iterations);
//     let now = Instant::now();

//     mcts.get_best_action();

//     let elapsed_s = now.elapsed().as_secs_f32();
//     println!(
//         "Iterations per second: {}",
//         (n_iterations as f32 / elapsed_s) as usize
//     );
//     println!("{} seconds per {} iterations", elapsed_s, n_iterations);
// }
