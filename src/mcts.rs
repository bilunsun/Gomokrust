use crate::board::{Action, Board, Outcome, Player};
use crate::utils::get_random_action;
use rand::Rng;

type Reward = f32;

pub fn outcome_to_reward(outcome: Outcome, turn: Player) -> Reward {
    match outcome {
        Outcome::Winner(winner) => match winner {
            turn => 1.0,
            _ => -1.0,
        },
        Outcome::Draw => 0.0,
    }
}

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
    // pub fn new(action: Option<Action>) -> Self {
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

    pub fn is_leaf(&self) -> bool {
        self.children.len() == 0
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

        // Randomly select a child node
        let random_index = rand::thread_rng().gen_range(0..self.children.len());
        Some(&mut self.children[random_index])
    }

    pub fn iteration(&mut self, board: &mut Board) {
        // Selection
        let mut node = self;
        while !node.is_leaf() {
            node = node.get_best_child().unwrap();
        }

        // Expansion
        match node.expand(board) {
            Some(node) => {}
            None => {}
        }
    }

    // pub fn iteration(&mut self, board: &mut Board) -> Reward {
    //     let reward = match self.state {
    //         State::Terminal => {
    //             let outcome = board.outcome.expect("Expandable node with state=State::Terminal means the game is over, and thus should have an outcome.");
    //             let reward = outcome_to_reward(outcome, self.turn);
    //             -reward
    //         }
    //         State::Expandable => {
    //             let child = self.expand(board);
    //             match child {
    //                 Some(child) => {
    //                     board
    //                         .make_action(child.action.expect("Child node should have an action."))
    //                         .expect("Child node action should be in legal moves.");

    //                     let outcome = rollout(board);
    //                     let reward = outcome_to_reward(outcome, child.turn);

    //                     child.visit_count += 1;
    //                     child.value += reward;
    //                     -reward
    //                 }
    //                 None => {
    //                     let outcome = board.outcome.expect("Expandable node with state=State::Terminal means the game is over, and thus should have an outcome.");
    //                     let reward = outcome_to_reward(outcome, self.turn);
    //                     -reward
    //                 }
    //             }
    //         }
    //     };
    //     self.visit_count += 1;
    //     self.value += reward;
    //     -reward
    // }
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

    pub fn get_best_action(&mut self) {
        for i in 0..10 {
            self.root.iteration(&mut self.board);
        }
        for child in &self.root.children {
            dbg!(&child);
        }
    }
}

pub fn test_mcts() {
    let mut board = Board::new(3, 3);
    println!("{board}");

    let action = board.parse_string_to_action(&String::from("B2")).unwrap();
    board.make_action(action).ok();
    println!("{board}");

    let action = board.parse_string_to_action(&String::from("A2")).unwrap();
    board.make_action(action).ok();
    println!("{board}");

    let action = board.parse_string_to_action(&String::from("C3")).unwrap();
    board.make_action(action).ok();
    println!("{board}");

    let action = board.parse_string_to_action(&String::from("A1")).unwrap();
    board.make_action(action).ok();
    println!("{board}");

    let action = board.parse_string_to_action(&String::from("A3")).unwrap();
    board.make_action(action).ok();
    println!("{board}");

    let action = board.parse_string_to_action(&String::from("B3")).unwrap();
    board.make_action(action).ok();
    println!("{board}");

    // let action = board.parse_string_to_action(&String::from("")).unwrap();
    // board.make_action(action).ok();
    // println!("{board}");
    let mut mcts = MCTS::new(&board);
    let best_action = mcts.get_best_action();

    println!("OK.");
}
