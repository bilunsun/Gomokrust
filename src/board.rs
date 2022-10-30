use indexmap::IndexSet;
use rand::Rng;
use std::collections::HashMap;

const VACANT: bool = false;
const OCCUPIED: bool = true;

#[derive(Copy, Clone)]
pub enum Player {
    Black = 0,
    White = 1,
}

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
    pub half_move_count: usize,
}

impl Board {
    pub fn new(size: usize, n_in_a_row: usize) -> Self {
        assert!(size <= 26, "The maximum supported board size is 26.");
        assert!(n_in_a_row <= size, "n_in_a_row cannot be larger than size");

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

        Self {
            size,
            n_in_a_row,
            black_stones,
            white_stones,
            row_names,
            col_names,
            row_names_hashmap,
            col_names_hashmap,
            turn: Player::Black,
            outcome: None,
            half_move_count: 0,
        }
    }

    pub fn place_stone(&mut self, square_string: &String) -> Option<(usize, usize)> {
        if square_string.len() < 2 {
            return None;
        }
        let row_string = (square_string[1..]).to_string();
        let col_string = (square_string[0..1]).to_string();
        let row_index = self.row_names_hashmap.get(&row_string);
        let col_index = self.col_names_hashmap.get(&col_string);

        if row_index.is_some() && col_index.is_some() {
            let row_index = row_index.unwrap();
            let col_index = col_index.unwrap();
            let index = self.row_col_to_flat_index(*row_index, *col_index);

            // Cannot place a stone on an occupied square
            if self.black_stones[index] == OCCUPIED || self.white_stones[index] == OCCUPIED {
                return None;
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

            // Check for an outcome
            // If no winner nor draw, switch the turn.
            self.outcome = self.check_outcome(index);
            if self.outcome.is_none() {
                match self.turn {
                    Player::Black => self.turn = Player::White,
                    Player::White => self.turn = Player::Black,
                }

                self.half_move_count += 1;
            }

            Some((*row_index, *col_index))
        } else {
            None
        }
    }

    pub fn place_stone_at_random(&mut self) -> Option<(usize, usize)> {
        let legal_moves = self.legal_moves();
        let square_string_index = rand::thread_rng().gen_range(0..legal_moves.len());
        let random_square_string = legal_moves.get_index(square_string_index).unwrap();
        // println!("Chosen move: {random_square_string}");
        self.place_stone(&random_square_string)
    }

    pub fn legal_moves(&self) -> IndexSet<String> {
        let mut legal_moves_hashset = IndexSet::new();
        for row_index in 0..self.size {
            for col_index in 0..self.size {
                let index = self.row_col_to_flat_index(row_index, col_index);
                if (self.black_stones[index] && self.white_stones[index]) == VACANT {
                    let legal_move = self.col_names[col_index].clone() + &self.row_names[row_index];
                    legal_moves_hashset.insert(legal_move);
                }
            }
        }
        legal_moves_hashset
    }

    pub fn is_game_over(&self) -> bool {
        self.outcome.is_some()
    }

    fn check_outcome(&self, flat_index: usize) -> Option<Outcome> {
        let horizontal_indices: Vec<usize> =
            (flat_index - self.n_in_a_row + 1..flat_index + self.n_in_a_row).collect();
        if self.n_in_a_row_in_indices(&horizontal_indices) {
            return Some(Outcome::Winner(self.turn));
        }

        let vertical_indices: Vec<usize> = (flat_index
            - self.base_board_padding() * self.base_board_size()
            ..=flat_index + self.base_board_padding() * self.base_board_size())
            .step_by(self.base_board_size())
            .collect();
        if self.n_in_a_row_in_indices(&vertical_indices) {
            return Some(Outcome::Winner(self.turn));
        }

        let diagonal_offsets: Vec<i32> =
            (-(self.base_board_padding() as i32)..=self.base_board_padding() as i32).collect();
        let forward_slash_indices: Vec<usize> = vertical_indices
            .iter()
            .zip(diagonal_offsets.iter())
            .map(|(i, offset)| (*i as i32 - offset) as usize)
            .collect();
        if self.n_in_a_row_in_indices(&forward_slash_indices) {
            return Some(Outcome::Winner(self.turn));
        }
        let back_slash_indices: Vec<usize> = vertical_indices
            .iter()
            .zip(diagonal_offsets.iter())
            .map(|(i, offset)| (*i as i32 + offset) as usize)
            .collect();
        if self.n_in_a_row_in_indices(&back_slash_indices) {
            return Some(Outcome::Winner(self.turn));
        }

        if self.half_move_count == self.size * self.size {
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

    // pub fn show_debug(&self) {
    //     let mut board_string = String::new();
    //     let mut padded_row_names: Vec<String> = self
    //         .row_names
    //         .iter()
    //         .map(|n| {
    //             let mut padded_name = String::from(if n.len() == 1 { " " } else { "" });
    //             padded_name.push_str(n);
    //             padded_name
    //         })
    //         .collect();

    //     let base_board_row_pad_names = (0..self.n_in_a_row)
    //         .collect::Vec<usize>()
    //         .iter()
    //         .map(|| String::from(". "))
    //         .collect();
    //     padded_row_names.append(base_board_row_pad_names);

    //     for row_index in (0..self.size).rev() {
    //         let mut row_string = padded_row_names[row_index].clone();
    //         row_string.push_str(" ");

    //         for col_index in 0..self.size {
    //             let index = self.row_col_to_flat_index(row_index, col_index);

    //             if self.black_stones[index] {
    //                 row_string.push_str("X ");
    //             } else if self.white_stones[index] {
    //                 row_string.push_str("O ");
    //             } else {
    //                 row_string.push_str(". ");
    //             }
    //         }
    //         board_string.push_str(&row_string);
    //         board_string.push_str("\n");
    //     }

    //     board_string.push_str("   ");
    //     board_string.push_str(&self.col_names.join(" "));

    //     println!("{board_string}");
    // }
}
