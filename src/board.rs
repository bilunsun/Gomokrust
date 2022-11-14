extern crate indexmap;
use indexmap::IndexSet;

extern crate ndarray;
use ndarray::prelude::*;

use std::collections::{HashMap, HashSet};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn opposite(&self) -> Self {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Player::White => false,
            Player::Black => true,
        }
    }

    pub fn to_f32(&self) -> f32 {
        match self {
            Player::Black => 1.0,
            Player::White => 0.0,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum SquareState {
    Occupied(Player),
    Vacant,
}

#[derive(Copy, Clone, Debug)]
pub enum Outcome {
    Winner(Player),
    Draw,
}

pub type Action = [usize; 2];
type BaseBoardLocation = [usize; 2];

#[derive(Debug, Clone)]
pub struct BaseBoard {
    data: Array<SquareState, Ix2>,
}

impl BaseBoard {
    pub fn new(size: usize) -> Self {
        Self {
            data: Array::<SquareState, Ix2>::from_elem((size, size), SquareState::Vacant),
        }
    }

    pub fn set(&mut self, location: BaseBoardLocation, player: Player) {
        self.data[location] = SquareState::Occupied(player);
    }

    pub fn get(&self, location: BaseBoardLocation) -> &SquareState {
        &self.data[location]
    }

    pub fn is_occupied(&self, location: BaseBoardLocation) -> bool {
        *self.get(location) != SquareState::Vacant
    }

    pub fn is_occupied_by(&self, location: BaseBoardLocation, player: Player) -> bool {
        *self.get(location) == SquareState::Occupied(player)
    }

    pub fn reset(&mut self) {
        self.data.fill(SquareState::Vacant);
    }
}

pub struct Board {
    pub size: usize,
    pub n_in_a_row: usize,
    pub turn: Player,
    pub base_board: BaseBoard,
    pub outcome: Option<Outcome>,
    pub num_stones_placed: usize,
    legal_actions_indexset: IndexSet<Action>,
    action_to_check_indices: HashMap<Action, Vec<Vec<BaseBoardLocation>>>,
}

impl Board {
    /// Creates a new instance of Board.
    /// * `size` - The width and height of the board
    /// * `n_in_a_row` - The number of aligned pieces needed to win.
    ///
    /// e.g. size=3 and n_in_a_row=3 is TicTacToe
    /// e.g. size=15 and n_in_a_row=5 is Gomoku
    pub fn new(size: usize, n_in_a_row: usize) -> Self {
        assert!(size <= 26, "The maximum supported board size is 26.");
        assert!(n_in_a_row <= size, "n_in_a_row cannot be larger than size.");
        assert!(n_in_a_row > 1, "n_in_a_row must be at least 2.");

        let base_board_size = size + (n_in_a_row - 1) * 2;
        let base_board = BaseBoard::new(base_board_size);

        let legal_actions_indexset = IndexSet::with_capacity(size * size);
        let action_to_check_indices = HashMap::new();

        let mut board = Self {
            size,
            n_in_a_row,
            base_board,
            legal_actions_indexset,
            action_to_check_indices,
            turn: Player::Black,
            outcome: None,
            num_stones_placed: 0,
        };

        board.initialize_legal_actions_indexset();
        board.initialize_action_to_check_locations();
        board
    }

    pub fn make_action(&mut self, action: Action) -> Result<Action, ()> {
        if self.is_game_over() {
            panic!("Cannot make action as the game is already over.");
        }

        let base_board_location = self.action_to_base_board_location(action);
        // Cannot place a stone on an occupied square
        if self.base_board.is_occupied(base_board_location) {
            return Err(());
        }

        // Place stone
        self.base_board.set(base_board_location, self.turn);
        self.legal_actions_indexset.remove(&action);
        self.num_stones_placed += 1;

        // Check for an outcome
        // If no winner nor draw, switch the turn.
        self.outcome = self.check_outcome(action);
        if self.outcome.is_none() {
            self.turn = self.turn.opposite();
        }

        Ok(action)
    }

    pub fn parse_string_to_action(&self, string: &String) -> Result<Action, ()> {
        if string.len() < 2 {
            return Err(());
        }

        let row_string = (string[1..]).to_string();
        let col_string = (string[0..1]).to_string();

        let (row_names_hashmap, col_names_hashmap) = get_names_hashmaps(self.size);
        let row_index = row_names_hashmap.get(&row_string);
        let col_index = col_names_hashmap.get(&col_string);

        if row_index.is_none() || col_index.is_none() {
            return Err(());
        }

        let row_index = row_index.expect("Just checked is_none().");
        let col_index = col_index.expect("Just checked is_none().");
        let action = [*row_index, *col_index] as Action;

        Ok(action)
    }

    /// Creates and returns a HashSet of legal moves as strings, e.g. "A1".
    /// Can be used with `place_stone`.
    pub fn legal_moves_as_strings(&self) -> HashSet<String> {
        let (row_names, col_names) = get_row_col_names(self.size);
        let mut legal_moves_hashset: HashSet<String> =
            HashSet::with_capacity(self.legal_actions_indexset.len());

        for action in self.legal_actions_indexset.iter() {
            let [row_index, col_index] = *action;
            let legal_move = col_names[col_index].clone() + &row_names[row_index];
            legal_moves_hashset.insert(legal_move);
        }
        legal_moves_hashset
    }

    /// Returns a reference to `self.legal_moves_indices_indexset`
    pub fn legal_actions(&self) -> &IndexSet<Action> {
        &self.legal_actions_indexset
    }

    /// Returns whether the game has ended, based on `self.outcome`.
    pub fn is_game_over(&self) -> bool {
        self.outcome.is_some()
    }

    /// Checks whether the action made resulted in an Outcome.
    fn check_outcome(&self, action: Action) -> Option<Outcome> {
        let check_locations = self
            .action_to_check_indices
            .get(&action)
            .expect("These should be pre-computed.");

        if check_locations
            .iter()
            .any(|locations| self.locations_contain_win(locations))
        {
            return Some(Outcome::Winner(self.turn));
        }

        if self.num_stones_placed == self.size * self.size {
            return Some(Outcome::Draw);
        }

        None
    }

    /// Checks whether a list of BaseBoardLocations contain a winning condition
    /// by checking whether there are `n_in_a_row` occupied states
    fn locations_contain_win(&self, locations: &Vec<BaseBoardLocation>) -> bool {
        locations.windows(self.n_in_a_row).any(|w| {
            w.iter()
                .map(|location| self.base_board.is_occupied_by(*location, self.turn))
                .all(|x| x)
        })
    }

    /// Returns the size of the base board,
    /// which is the `size` with padding on either side.
    fn base_board_size(&self) -> usize {
        self.size + self.base_board_padding() * 2
    }

    /// Returns the padding on either side of the base board.
    fn base_board_padding(&self) -> usize {
        self.n_in_a_row - 1
    }

    /// Converts an Action to a BaseBoardLocation
    pub fn action_to_base_board_location(&self, action: Action) -> BaseBoardLocation {
        [
            action[0] + self.base_board_padding(),
            action[1] + self.base_board_padding(),
        ] as BaseBoardLocation
    }

    /// Converts an Action to a flat index
    pub fn action_to_flat_index(&self, action: &Action) -> usize {
        action[0] * self.size + action[1]
    }

    /// Converts a BaseBoardLocation to an Action
    fn base_board_location_to_action(&self, base_board_location: BaseBoardLocation) -> Action {
        [
            base_board_location[0] - self.base_board_padding(),
            base_board_location[1] - self.base_board_padding(),
        ] as Action
    }

    /// Resets the state of the board.
    pub fn reset(&mut self) {
        self.base_board.reset();
        self.turn = Player::Black;
        self.outcome = None;
        self.num_stones_placed = 0;
        self.initialize_legal_actions_indexset();
    }

    /// Initializes an IndexSet containing all legal moves
    /// by iterating through pairs of `row_index` and `col_index`,
    /// then converting them to an Action.
    fn initialize_legal_actions_indexset(&mut self) {
        self.legal_actions_indexset.clear();
        for row_index in 0..self.size {
            for col_index in 0..self.size {
                self.legal_actions_indexset
                    .insert([row_index, col_index] as Action);
            }
        }
    }

    /// Initializes the BaseBoardLocations to be checked for a winning condition for an action.
    fn initialize_action_to_check_locations(&mut self) {
        self.action_to_check_indices = HashMap::new();

        for row_index in 0..self.size {
            for col_index in 0..self.size {
                let action = [row_index, col_index] as Action;

                let mut horizontal: Vec<BaseBoardLocation> = Vec::new();
                let mut vertical: Vec<BaseBoardLocation> = Vec::new();
                let mut forward_slash: Vec<BaseBoardLocation> = Vec::new();
                let mut backward_slash: Vec<BaseBoardLocation> = Vec::new();

                for offset in -(self.base_board_padding() as i32)..=self.base_board_padding() as i32
                {
                    horizontal.push(self.action_to_base_board_location([
                        row_index,
                        (col_index as i32 + offset) as usize,
                    ]
                        as Action));

                    vertical.push(self.action_to_base_board_location([
                        (row_index as i32 + offset) as usize,
                        col_index,
                    ]
                        as Action));

                    forward_slash.push(self.action_to_base_board_location([
                        (row_index as i32 - offset) as usize,
                        (col_index as i32 + offset) as usize,
                    ]
                        as Action));

                    backward_slash.push(self.action_to_base_board_location([
                        (row_index as i32 - offset) as usize,
                        (col_index as i32 - offset) as usize,
                    ]
                        as Action));
                }

                let mut check_indices: Vec<Vec<BaseBoardLocation>> = vec![];
                check_indices.push(horizontal);
                check_indices.push(vertical);
                check_indices.push(forward_slash);
                check_indices.push(backward_slash);
                self.action_to_check_indices.insert(action, check_indices);
            }
        }
    }

    pub fn to_vec(&self) -> Vec<Vec<Vec<f32>>> {
        let board_slice = self.base_board.data.slice(s![
            self.n_in_a_row - 1..self.size + self.base_board_padding(),
            self.n_in_a_row - 1..self.size + self.base_board_padding()
        ]);

        let mut board_vec = vec![vec![vec![0f32; self.size]; self.size]; 2];

        // Set the pieces
        for ((row_index, col_index), square_state) in board_slice.indexed_iter() {
            match square_state {
                SquareState::Occupied(turn) => match turn {
                    Player::Black => board_vec[0][row_index][col_index] = 1.0,
                    Player::White => board_vec[1][row_index][col_index] = 1.0,
                },
                _ => (),
            }
        }

        // Set the turn
        let turn_plane = vec![vec![self.turn.to_f32(); self.size]; self.size];
        board_vec.push(turn_plane);

        board_vec
    }

    pub fn to_array(&self) -> Array3<f32> {
        let board_slice = self.base_board.data.slice(s![
            self.n_in_a_row - 1..self.size + self.base_board_padding(),
            self.n_in_a_row - 1..self.size + self.base_board_padding()
        ]);

        let mut board_array = Array3::<f32>::zeros((3, self.size, self.size));

        // Set the pieces
        for ((row_index, col_index), square_state) in board_slice.indexed_iter() {
            match square_state {
                SquareState::Occupied(turn) => match turn {
                    Player::Black => board_array[[0, row_index, col_index]] = 1.0,
                    Player::White => board_array[[1, row_index, col_index]] = 1.0,
                },
                _ => (),
            }
        }

        // Set the turn
        board_array
            .slice_mut(s![2, .., ..])
            .fill(self.turn.to_f32());

        board_array
    }

    pub fn to_flat_array(&self) -> Array1<f32> {
        let board_slice = self.base_board.data.slice(s![
            self.n_in_a_row - 1..self.size + self.base_board_padding(),
            self.n_in_a_row - 1..self.size + self.base_board_padding()
        ]);

        let mut board_flat_array = Array1::<f32>::zeros(self.size * self.size + 1);

        // Set the pieces
        for ((row_index, col_index), square_state) in board_slice.indexed_iter() {
            let index = row_index * self.size + col_index;
            match square_state {
                SquareState::Occupied(player) => board_flat_array[index] = player.to_f32(),
                _ => (),
            }
        }

        board_flat_array[self.size * self.size] = self.turn.to_f32();

        board_flat_array
    }

    pub fn to_flat_vec(&self) -> Vec<f32> {
        let board_flat_array = self.to_flat_array();
        board_flat_array.iter().map(|i| *i).collect()
    }

    pub fn to_tensor(&self) -> tch::Tensor {
        let board_array = self.to_array();
        let board_tensor = tch::Tensor::try_from(board_array)
            .unwrap()
            // .to_device(tch::Device::Cuda(0))
            .reshape(&[1, 3, self.size as i64, self.size as i64]);

        board_tensor
    }

    pub fn to_flat_tensor(&self) -> tch::Tensor {
        let board_flat_array = self.to_flat_array();
        let board_tensor = tch::Tensor::try_from(board_flat_array)
            .unwrap()
            // .to_device(tch::Device::Cuda(0))
            .reshape(&[1, (self.size * self.size + 1) as i64]);

        board_tensor
    }
}

impl Clone for Board {
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            n_in_a_row: self.n_in_a_row,
            base_board: self.base_board.clone(),
            legal_actions_indexset: self.legal_actions_indexset.clone(),
            action_to_check_indices: self.action_to_check_indices.clone(),
            turn: self.turn,
            outcome: self.outcome,
            num_stones_placed: self.num_stones_placed,
        }
    }
}

fn get_row_col_names(size: usize) -> (Vec<String>, Vec<String>) {
    let row_names: Vec<String> = (1..=size as u32).map(|c| c.to_string()).collect();
    let col_names: Vec<String> = (b'A'..=b'Z')
        .filter(|c| c - b'A' < size as u8)
        .map(|c| (c as char).to_string())
        .collect();

    (row_names, col_names)
}

fn get_names_hashmaps(size: usize) -> (HashMap<String, usize>, HashMap<String, usize>) {
    let (row_names, col_names) = get_row_col_names(size);
    let mut row_names_hashmap = HashMap::with_capacity(size);
    let mut col_names_hashmap = HashMap::with_capacity(size);

    for (i, n) in row_names.iter().rev().enumerate() {
        row_names_hashmap.insert(n.clone(), i);
    }
    for (i, n) in col_names.iter().enumerate() {
        col_names_hashmap.insert(n.clone(), i);
    }

    (row_names_hashmap, col_names_hashmap)
}

pub fn show(board: &Board) {
    let mut board_string = String::new();
    let (row_names, col_names) = get_row_col_names(board.size);

    let padded_row_names: Vec<String> = row_names
        .iter()
        .rev()
        .map(|n| {
            let mut padded_name = String::from(if n.len() == 1 { " " } else { "" });
            padded_name.push_str(n);
            padded_name
        })
        .collect();

    for row_index in 0..board.size {
        let mut row_string = padded_row_names[row_index].clone();
        row_string.push_str(" ");

        for col_index in 0..board.size {
            let action = board.action_to_base_board_location([row_index, col_index] as Action);

            match board.base_board.get(action) {
                SquareState::Occupied(Player::Black) => row_string.push_str("X "),
                SquareState::Occupied(Player::White) => row_string.push_str("O "),
                SquareState::Vacant => row_string.push_str(". "),
            }
        }
        board_string.push_str(&row_string);
        board_string.push_str("\n");
    }

    board_string.push_str("   ");
    board_string.push_str(&col_names.join(" "));
    board_string.push_str("\n");

    println!("{board_string}");
}
