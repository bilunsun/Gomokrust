use indexmap::IndexSet;
use rand::Rng;
use std::collections::{HashMap, HashSet};

const VACANT: bool = false;
const OCCUPIED: bool = true;

#[derive(Copy, Clone)]
pub enum Player {
    Black,
    White,
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
    pub black_stones: Vec<bool>,
    pub white_stones: Vec<bool>,
    pub row_names: Vec<String>,
    pub col_names: Vec<String>,
    pub row_names_hashmap: HashMap<String, usize>,
    pub col_names_hashmap: HashMap<String, usize>,
    pub outcome: Option<Outcome>,
    pub num_stones_placed: usize,
    legal_moves_indices_hashset: IndexSet<usize>,
    flat_index_to_check_indices: HashMap<usize, Vec<Vec<usize>>>,
}

impl Board {
    pub fn new(size: usize, n_in_a_row: usize) -> Self {
        assert!(size <= 26, "The maximum supported board size is 26.");
        assert!(n_in_a_row <= size, "n_in_a_row cannot be larger than size.");
        assert!(n_in_a_row > 1, "n_in_a_row must be at least 2.");

        let _base_board_size = size + (n_in_a_row - 1) * 2;
        let black_stones = vec![VACANT; _base_board_size * _base_board_size];
        let white_stones = vec![VACANT; _base_board_size * _base_board_size];
        let row_names: Vec<String> = (1..=size as u32).map(|c| c.to_string()).collect();
        let col_names: Vec<String> = (b'A'..=b'Z')
            .filter(|c| c - b'A' < size as u8)
            .map(|c| (c as char).to_string())
            .collect();

        let mut row_names_hashmap = HashMap::new();
        for (i, n) in row_names.iter().enumerate() {
            row_names_hashmap.insert(n.clone(), i);
        }
        let mut col_names_hashmap = HashMap::new();
        for (i, n) in col_names.iter().enumerate() {
            col_names_hashmap.insert(n.clone(), i);
        }

        let legal_moves_indices_hashset: IndexSet<usize> = IndexSet::with_capacity(size * size);
        let flat_index_to_check_indices = HashMap::new();

        let mut board = Self {
            size,
            n_in_a_row,
            black_stones,
            white_stones,
            row_names,
            col_names,
            row_names_hashmap,
            col_names_hashmap,
            legal_moves_indices_hashset,
            flat_index_to_check_indices,
            turn: Player::Black,
            outcome: None,
            num_stones_placed: 0,
        };

        board.initialize_legal_moves_indexset();
        board.initialize_flat_index_to_check_indices();

        board
    }

    pub fn place_stone(&mut self, square_string: &String) -> Result<(), ()> {
        if square_string.len() < 2 {
            return Ok(());
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

    fn place_stone_at_index(&mut self, index: usize) -> Result<(), ()> {
        // Cannot place a stone on an occupied square
        if self.black_stones[index] == OCCUPIED || self.white_stones[index] == OCCUPIED {
            return Err(());
        }

        // Place stone
        match self.turn {
            Player::Black => {
                self.black_stones[index] = OCCUPIED;
            }
            Player::White => {
                self.white_stones[index] = OCCUPIED;
            }
        }
        self.legal_moves_indices_hashset.remove(&index);

        // Check for an outcome
        // If no winner nor draw, switch the turn.
        self.outcome = self.check_outcome(index);
        if self.outcome.is_none() {
            match self.turn {
                Player::Black => self.turn = Player::White,
                Player::White => self.turn = Player::Black,
            }

            self.num_stones_placed += 1;
        }

        Ok(())
    }

    pub fn place_stone_at_random(&mut self) -> Result<(), ()> {
        let random_index = rand::thread_rng().gen_range(0..self.legal_moves_indices_hashset.len());
        let random_square_index = self
            .legal_moves_indices_hashset
            .get_index(random_index)
            .unwrap();

        self.place_stone_at_index(*random_square_index)
    }

    pub fn legal_moves(&self) -> HashSet<String> {
        let mut legal_moves_hashset: HashSet<String> =
            HashSet::with_capacity(self.legal_moves_indices_hashset.len());

        for i in self.legal_moves_indices_hashset.iter() {
            let (row_index, col_index) = self.flat_index_to_row_col(*i);
            let legal_move = self.col_names[col_index].clone() + &self.row_names[row_index];
            legal_moves_hashset.insert(legal_move);
        }
        legal_moves_hashset
    }

    pub fn is_game_over(&self) -> bool {
        self.outcome.is_some()
    }

    fn check_outcome(&self, flat_index: usize) -> Option<Outcome> {
        let horizontal_vertical_diagonal_indices = self
            .flat_index_to_check_indices
            .get(&flat_index)
            .expect("These should be pre-computed.");

        if horizontal_vertical_diagonal_indices
            .iter()
            .map(|indices| self.n_in_a_row_in_indices(indices))
            .any(|x| x)
        {
            return Some(Outcome::Winner(self.turn));
        }
        if self.num_stones_placed == self.size * self.size {
            return Some(Outcome::Draw);
        }

        None
    }

    fn n_in_a_row_in_indices(&self, indices: &Vec<usize>) -> bool {
        for w in indices.windows(self.n_in_a_row) {
            let is_n_in_a_row: bool = w
                .iter()
                .map(|i| match self.turn {
                    Player::Black => self.black_stones[*i] == OCCUPIED,
                    Player::White => self.white_stones[*i] == OCCUPIED,
                })
                .all(|x| x);

            if is_n_in_a_row {
                return true;
            }
        }
        false
    }

    fn base_board_size(&self) -> usize {
        self.size + self.base_board_padding() * 2
    }

    fn base_board_padding(&self) -> usize {
        self.n_in_a_row - 1
    }

    fn row_col_to_flat_index(&self, row_index: usize, col_index: usize) -> usize {
        let row_index = row_index + self.base_board_padding();
        let col_index = col_index + self.base_board_padding();
        row_index * self.base_board_size() + col_index
    }

    fn flat_index_to_row_col(&self, index: usize) -> (usize, usize) {
        let row_index = index / self.base_board_size();
        let col_index = index % self.base_board_size();

        (
            row_index - self.base_board_padding(),
            col_index - self.base_board_padding(),
        )
    }

    pub fn reset(&mut self) {
        self.black_stones.fill(VACANT);
        self.white_stones.fill(VACANT);
        self.turn = Player::Black;
        self.outcome = None;
        self.num_stones_placed = 0;
        self.initialize_legal_moves_indexset();
    }

    fn initialize_legal_moves_indexset(&mut self) {
        self.legal_moves_indices_hashset.clear();
        for row_index in 0..self.size {
            for col_index in 0..self.size {
                let index = self.row_col_to_flat_index(row_index, col_index);
                self.legal_moves_indices_hashset.insert(index);
            }
        }
    }

    fn initialize_flat_index_to_check_indices(&mut self) {
        self.flat_index_to_check_indices = HashMap::new();
        for row_index in 0..self.size {
            for col_index in 0..self.size {
                let mut check_indices: Vec<Vec<usize>> = vec![];
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

                check_indices.push(horizontal_indices);
                check_indices.push(vertical_indices);
                check_indices.push(forward_slash_indices);
                check_indices.push(back_slash_indices);
                self.flat_index_to_check_indices
                    .insert(flat_index, check_indices);
            }
        }
    }

    pub fn show(&self) {
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

                if self.black_stones[index] {
                    row_string.push_str("X ");
                } else if self.white_stones[index] {
                    row_string.push_str("O ");
                } else {
                    row_string.push_str(". ");
                }
            }
            board_string.push_str(&row_string);
            board_string.push_str("\n");
        }

        board_string.push_str("   ");
        board_string.push_str(&self.col_names.join(" "));
        board_string.push_str("\n");

        println!("{board_string}");
    }
}
