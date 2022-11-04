use std::time::Instant;

use crate::board::{Action, Board, Outcome, Player};
use crate::utils::get_random_action;

const SQRT_TWO: f32 = 1.41421356237;
// const SQRT_TWO: f32 = 0.1;

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

    pub fn uct(&self, parent_visit_count: usize) -> f32 {
        if self.visit_count == 0 {
            return f32::INFINITY;
        }
        let exploitation_term = self.value / (self.visit_count as f32);
        let exploration_term =
            SQRT_TWO * f32::sqrt(f32::ln(parent_visit_count as f32) / (self.visit_count as f32));
        exploitation_term + exploration_term
    }

    pub fn update(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Winner(winner) => {
                if winner != self.turn {
                    self.value += 1.0
                }
            }
            _ => (),
        };

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
            self.state = State::Terminal;
            return None;
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

        Some(&mut self.children[0])
    }

    pub fn iteration(&mut self, board: &mut Board) {
        let mut parents_pointers: Vec<*mut Node> = Vec::new();

        // Selection
        let mut node = self;
        while !node.children.is_empty() {
            parents_pointers.push(node);
            node = node.get_best_child().unwrap();
            board.make_action(node.action.unwrap()).ok();
        }

        // if node.action.unwrap() == 18 {
        //     assert!(board.is_game_over());
        // }
        // if node.action.is_none() {
        //     dbg!(&node);
        // }

        // Expansion
        let outcome = match node.expand(board) {
            Some(_) => rollout(board), // Rollout
            None => board
                .outcome
                .expect("Terminal state should have an outcome."),
        };

        // Backpropagate
        node.update(outcome);
        for parent_pointer in parents_pointers.iter().rev() {
            let parent = unsafe { parent_pointer.as_mut().unwrap() };
            parent.update(outcome);
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

    pub fn get_best_action(&mut self, n_iterations: usize) -> Action {
        for _ in 0..n_iterations {
            let mut board = self.board.clone();
            self.root.iteration(&mut board);
        }

        // // For debugging only
        // for child in &mut self.root.children {
        //     println!(
        //         "{} -> {}",
        //         child.action.unwrap(),
        //         child.uct(self.root.visit_count)
        //     );
        // }

        // dbg!(&self.root);

        let best_child = self
            .root
            .get_best_child()
            .expect("Root node should have children.");
        best_child.action.expect("Child should have action")
    }
}

pub fn test_mcts() {
    let mut board = Board::new(15, 5);

    for move_string in vec!["B2", "A2", "C3", "A1", "A3", "B3"].iter() {
        let action = board
            .parse_string_to_action(&String::from(*move_string))
            .unwrap();
        board.make_action(action).ok();
    }
    println!("{board}");

    // let mut mcts = MCTS::new(&board);
    // let best_action = mcts.get_best_action(1_000);

    // board.make_action(best_action).ok();
    // println!("{board}");

    let n_iterations = 50_000;
    let mut mcts = MCTS::new(&board);
    let now = Instant::now();
    mcts.get_best_action(n_iterations);

    let elapsed_s = now.elapsed().as_secs_f32();
    println!(
        "Iterations per second: {}",
        (n_iterations as f32 / elapsed_s) as usize
    );
}
