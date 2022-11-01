extern crate indexmap;
use indexmap::IndexSet;

use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Player {
    Black,
    White,
}

#[derive(PartialEq, Eq, Clone)]
pub enum SquareState {
    Occupied(Player),
    Vacant,
}

#[derive(Copy, Clone)]
pub enum Outcome {
    Winner(Player),
    Draw,
}

pub struct Board {
    pub size: usize,
    pub n_in_a_row: usize,
    pub turn: Player,
    pub square_states: Vec<SquareState>,
    pub row_names: Vec<String>,
    pub col_names: Vec<String>,
    pub row_names_hashmap: HashMap<String, usize>,
    pub col_names_hashmap: HashMap<String, usize>,
    pub outcome: Option<Outcome>,
    pub num_stones_placed: usize,
    legal_moves_indices_indexset: IndexSet<usize>,
    flat_index_to_check_indices: HashMap<usize, Vec<Vec<usize>>>,
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
        let square_states = vec![SquareState::Vacant; base_board_size * base_board_size];

        let row_names = Vec::with_capacity(size);
        let col_names = Vec::with_capacity(size);
        let row_names_hashmap = HashMap::with_capacity(size);
        let col_names_hashmap = HashMap::with_capacity(size);
        let legal_moves_indices_indexset = IndexSet::with_capacity(size * size);
        let flat_index_to_check_indices = HashMap::new();

        let mut board = Self {
            size,
            n_in_a_row,
            square_states,
            row_names,
            col_names,
            row_names_hashmap,
            col_names_hashmap,
            legal_moves_indices_indexset,
            flat_index_to_check_indices,
            turn: Player::Black,
            outcome: None,
            num_stones_placed: 0,
        };

        board.initialize_legal_moves_indexset();
        board.initialize_flat_index_to_check_indices();
        board.initialize_names_and_hashmaps();
        board
    }

    /// Tries to place a stone at the location specified by a string.
    ///
    /// * `square_string` - The target location, e.g. "A1".
    pub fn place_stone(&mut self, square_string: &String) -> Result<(), ()> {
        if square_string.len() < 2 {
            return Err(());
        }

        let row_string = (square_string[1..]).to_string();
        let col_string = (square_string[0..1]).to_string();
        let row_index = self.row_names_hashmap.get(&row_string);
        let col_index = self.col_names_hashmap.get(&col_string);

        if row_index.is_some() && col_index.is_some() {
            let row_index = row_index.unwrap();
            let col_index = col_index.unwrap();
            let index = self.row_col_to_flat_index(*row_index, *col_index);

            self.place_stone_at_index(index)?;
            return Ok(());
        }

        return Err(());
    }

    /// Tries to place a stone at a specific index.
    /// Checks whether the target square is occupied or vacant.
    /// Updates the state of:
    ///     - `self.square_states`
    ///     - `self.legal_moves_indices_indexset`
    ///     - `self.num_stones_placed`
    ///     - `self.outcome`
    ///     - `self.turn`
    ///
    /// * `index` - Index of `self.square_states`
    pub fn place_stone_at_index(&mut self, index: usize) -> Result<(), ()> {
        // Cannot place a stone on an occupied square
        if let SquareState::Occupied(_) = self.square_states[index] {
            return Err(());
        }

        // Place stone
        self.square_states[index] = SquareState::Occupied(self.turn);
        self.legal_moves_indices_indexset.remove(&index);
        self.num_stones_placed += 1;

        // Check for an outcome
        // If no winner nor draw, switch the turn.
        self.outcome = self.check_outcome(index);
        if self.outcome.is_none() {
            match self.turn {
                Player::Black => self.turn = Player::White,
                Player::White => self.turn = Player::Black,
            }
        }

        Ok(())
    }

    /// Creates and returns a HashSet of legal moves as strings, e.g. "A1".
    /// Can be used with `place_stone`.
    pub fn legal_moves_as_strings(&self) -> HashSet<String> {
        let mut legal_moves_hashset: HashSet<String> =
            HashSet::with_capacity(self.legal_moves_indices_indexset.len());

        for i in self.legal_moves_indices_indexset.iter() {
            let (row_index, col_index) = self.flat_index_to_row_col(*i);
            let legal_move = self.col_names[col_index].clone() + &self.row_names[row_index];
            legal_moves_hashset.insert(legal_move);
        }
        legal_moves_hashset
    }

    /// Returns a reference to `self.legal_moves_indices_indexset`
    pub fn legal_moves(&self) -> &IndexSet<usize> {
        &self.legal_moves_indices_indexset
    }

    /// Returns whether the game has ended, based on `self.outcome`.
    pub fn is_game_over(&self) -> bool {
        self.outcome.is_some()
    }

    /// Checks whether the move made resulted in an Outcome.
    ///
    /// * `flat_index` - The index where the stone was just placed.
    fn check_outcome(&self, flat_index: usize) -> Option<Outcome> {
        let horizontal_vertical_diagonal_indices = self
            .flat_index_to_check_indices
            .get(&flat_index)
            .expect("These should be pre-computed.");

        if horizontal_vertical_diagonal_indices
            .iter()
            .any(|indices| self.indices_contain_win(indices))
        {
            return Some(Outcome::Winner(self.turn));
        }

        if self.num_stones_placed == self.size * self.size {
            return Some(Outcome::Draw);
        }

        None
    }

    /// Checks whether a list of indices contain a winning condition
    /// by checking whether there are `n_in_a_row` square states
    /// that are occupied
    fn indices_contain_win(&self, indices: &Vec<usize>) -> bool {
        indices.windows(self.n_in_a_row).any(|w| {
            w.iter()
                .map(|i| self.square_states[*i] == SquareState::Occupied(self.turn))
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

    /// Converts from `row_index` and `col_index` to a flat index
    /// to index into `self.square_states`.
    ///
    /// * `row_index` and `col_index` - The index pair to be converted
    ///     into a flat index.
    fn row_col_to_flat_index(&self, row_index: usize, col_index: usize) -> usize {
        let row_index = row_index + self.base_board_padding();
        let col_index = col_index + self.base_board_padding();
        row_index * self.base_board_size() + col_index
    }

    /// Converts from a flat index to a `row_index` and `col_index`.
    /// Opposite of `row_col_to_flat_index`.
    ///
    /// * `index` - The index to be converted into `row_index` and `col_index`
    fn flat_index_to_row_col(&self, index: usize) -> (usize, usize) {
        let row_index = index / self.base_board_size();
        let col_index = index % self.base_board_size();

        (
            row_index - self.base_board_padding(),
            col_index - self.base_board_padding(),
        )
    }

    /// Resets the state of the board.
    pub fn reset(&mut self) {
        self.square_states.fill(SquareState::Vacant);
        self.turn = Player::Black;
        self.outcome = None;
        self.num_stones_placed = 0;
        self.initialize_legal_moves_indexset();
    }

    /// Initializes an IndexSet containing all legal moves
    /// by iterating through pairs of `row_index` and `col_index`,
    /// then converting them to a flat index.
    fn initialize_legal_moves_indexset(&mut self) {
        self.legal_moves_indices_indexset.clear();
        for row_index in 0..self.size {
            for col_index in 0..self.size {
                let index = self.row_col_to_flat_index(row_index, col_index);
                self.legal_moves_indices_indexset.insert(index);
            }
        }
    }

    /// Initializes the indices to be checked for a winning condition.
    /// The indices are used by `indices_contain_win`.
    fn initialize_flat_index_to_check_indices(&mut self) {
        self.flat_index_to_check_indices = HashMap::new();
        for row_index in 0..self.size {
            for col_index in 0..self.size {
                let flat_index = self.row_col_to_flat_index(row_index, col_index);

                let horizontal_indices: Vec<usize> =
                    (flat_index - self.n_in_a_row + 1..flat_index + self.n_in_a_row).collect();

                let vertical_indices: Vec<usize> = (flat_index
                    - self.base_board_padding() * self.base_board_size()
                    ..=flat_index + self.base_board_padding() * self.base_board_size())
                    .step_by(self.base_board_size())
                    .collect();

                let diagonal_offsets: Vec<i32> = (-(self.base_board_padding() as i32)
                    ..=self.base_board_padding() as i32)
                    .collect();
                let forward_slash_indices: Vec<usize> = vertical_indices
                    .iter()
                    .zip(diagonal_offsets.iter())
                    .map(|(i, offset)| (*i as i32 - offset) as usize)
                    .collect();
                let back_slash_indices: Vec<usize> = vertical_indices
                    .iter()
                    .zip(diagonal_offsets.iter())
                    .map(|(i, offset)| (*i as i32 + offset) as usize)
                    .collect();

                let mut check_indices: Vec<Vec<usize>> = vec![];
                check_indices.push(horizontal_indices);
                check_indices.push(vertical_indices);
                check_indices.push(forward_slash_indices);
                check_indices.push(back_slash_indices);
                self.flat_index_to_check_indices
                    .insert(flat_index, check_indices);
            }
        }
    }

    /// Initializes the necessary Vec and HashMap objects for displaying the board.
    fn initialize_names_and_hashmaps(&mut self) {
        self.row_names = (1..=self.size as u32).map(|c| c.to_string()).collect();
        self.col_names = (b'A'..=b'Z')
            .filter(|c| c - b'A' < self.size as u8)
            .map(|c| (c as char).to_string())
            .collect();
        for (i, n) in self.row_names.iter().enumerate() {
            self.row_names_hashmap.insert(n.clone(), i);
        }
        for (i, n) in self.col_names.iter().enumerate() {
            self.col_names_hashmap.insert(n.clone(), i);
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut board_string = String::new();
        let padded_row_names: Vec<String> = self
            .row_names
            .iter()
            .map(|n| {
                let mut padded_name = String::from(if n.len() == 1 { " " } else { "" });
                padded_name.push_str(n);
                padded_name
            })
            .collect();

        for row_index in (0..self.size).rev() {
            let mut row_string = padded_row_names[row_index].clone();
            row_string.push_str(" ");

            for col_index in 0..self.size {
                let index = self.row_col_to_flat_index(row_index, col_index);

                match self.square_states[index] {
                    SquareState::Occupied(Player::Black) => row_string.push_str("X "),
                    SquareState::Occupied(Player::White) => row_string.push_str("O "),
                    SquareState::Vacant => row_string.push_str(". "),
                }
            }
            board_string.push_str(&row_string);
            board_string.push_str("\n");
        }

        board_string.push_str("   ");
        board_string.push_str(&self.col_names.join(" "));
        board_string.push_str("\n");

        write!(f, "{board_string}")
    }
}
