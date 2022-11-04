use rand::Rng;
use std::rc::Rc;
use std::time::Instant;

use crate::board::{Action, Board, Outcome, Player};
use crate::utils::get_random_action;

type Reward = f32;

// pub fn outcome_to_reward(outcome: Outcome, turn: Player) -> Reward {
//     match outcome {
//         Outcome::Winner(winner) => match winner {
//             turn => 1.0,
//             _ => -1.0,
//         },
//         Outcome::Draw => 0.0,
//     }
// }

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
pub enum State {
    Terminal,
    Expandable,
}

#[derive(Debug)]
pub struct Node {
    action: Option<Action>,
    children: Vec<Node>,
    state: State,
    value: f32,
    visit_count: usize,
    turn: Player,
}

impl Node {
    pub fn new(action: Option<Action>, turn: Player) -> Self {
        Node {
            action,
            turn,
            children: Vec::new(),
            value: 0.0,
            state: State::Expandable,
            visit_count: 0,
        }
    }

    pub fn get_best_child(&mut self) -> Option<&mut Node> {
        let mut best_score: f32 = f32::NEG_INFINITY;
        let mut best_child: Option<&mut Node> = None;

        for child in &mut self.children {
            let score = 0.0;
            if score > best_score {
                best_score = score;
                best_child = Some(child);
            }
        }

        best_child
    }

    pub fn expand(&mut self, board: &Board) {
        if board.is_game_over() {
            self.state = State::Terminal;
            return;
        }

        // Create a child node for all untried moves
        let legal_actions = board.legal_actions();
        for action in legal_actions {
            let child = Node::new(Some(*action), self.turn.opposite());
            self.children.push(child);
        }

        // // Randomly select a child node
        // let random_index = rand::thread_rng().gen_range(0..self.children.len());
        // Some(&mut self.children[random_index])
    }

    pub fn iteration(&mut self, board: &mut Board) {
        // let mut parents: Vec<Rc<&mut Node>> = Vec::new();
        let mut parents_pointers: Vec<*mut Node> = Vec::new();

        // Selection
        let mut node = self;
        while !node.children.is_empty() {
            // parents.push(Rc::clone(&Rc::new(node)));
            parents_pointers.push(node);
            node = node.get_best_child().unwrap();
        }

        // Expansion
        node.expand(board);

        // Rollout
        let outcome = rollout(board);
        let reward = 1.0;
        // let reward = match outcome {
        //     Outcome::Winner(winner) => {
        //         if winner == self.turn {
        //             1
        //         } else {
        //             -1
        //         }
        //     }
        //     Outcome::Draw => 0,
        // };

        // Backpropagate
        node.visit_count += 1;
        node.value += reward;
        for parent_pointer in parents_pointers.iter().rev() {
            let mut parent = unsafe { parent_pointer.as_mut().unwrap() };
            parent.visit_count += 1;
            parent.value += reward;
            // dbg!(&parent.visit_count);
        }
    }
}

pub struct MCTS {
    root: Node,
    board: Board,
}

impl MCTS {
    pub fn new(board: &Board) -> Self {
        let root = Node::new(None, board.turn);
        let board = board.clone();
        Self { root, board }
    }

    pub fn get_best_action(&mut self, n_iterations: usize) {
        for i in 0..n_iterations {
            let mut board = self.board.clone();
            self.root.iteration(&mut board);
            // self.root.iteration(&mut self.board);
        }
        // for child in &self.root.children {
        //     dbg!(&child);
        // }
    }
}

pub fn test_mcts() {
    let mut board = Board::new(10, 5);
    println!("{board}");

    for move_string in vec!["B2", "A2", "C3", "A1", "A3", "B3"].iter() {
        let action = board
            .parse_string_to_action(&String::from(*move_string))
            .unwrap();
        board.make_action(action).ok();
    }
    println!("{board}");

    let mut mcts = MCTS::new(&board);
    let now = Instant::now();
    let best_action = mcts.get_best_action(1_000);

    // let n_iterations = 1_000;
    // let mut mcts = MCTS::new(&board);
    // let now = Instant::now();
    // let best_action = mcts.get_best_action(1_000);

    // let elapsed_s = now.elapsed().as_secs_f32();
    // println!(
    //     "Iterations per second: {}",
    //     (n_iterations as f32 / elapsed_s) as usize
    // );
}
