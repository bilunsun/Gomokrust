use std::collections::{HashMap, HashSet};

const VACANT: bool = false;
const OCCUPIED: bool = true;

enum Player {
    Black = 0,
    White = 1,
}

pub struct Board {
    size: usize,
    turn: Player,
    black_stones: Vec<bool>,
    white_stones: Vec<bool>,
    row_names: Vec<String>,
    col_names: Vec<String>,
    row_names_hashmap: HashMap<String, usize>,
    col_names_hashmap: HashMap<String, usize>,
}

impl Board {
    pub fn new(size: usize) -> Self {
        assert!(size <= 26, "The maximum supported board size is 26.");

        let black_stones = vec![VACANT; size * size];
        let white_stones = vec![VACANT; size * size];
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
            black_stones,
            white_stones,
            row_names,
            col_names,
            row_names_hashmap,
            col_names_hashmap,
            turn: Player::Black,
        }
    }

    pub fn place_stone(&mut self, square_string: &String) -> Option<(usize, usize)> {
        let row_string = (square_string[1..]).to_string();
        let col_string = (square_string[0..1]).to_string();
        let row_index = self.row_names_hashmap.get(&row_string);
        let col_index = self.col_names_hashmap.get(&col_string);

        if row_index.is_some() && col_index.is_some() {
            let row_index = row_index.unwrap();
            let col_index = col_index.unwrap();
            let index = self.row_col_to_flat_index(*row_index, *col_index);

            match self.turn {
                Player::Black => {
                    self.black_stones[index] = OCCUPIED;
                    self.turn = Player::White
                }
                Player::White => {
                    self.white_stones[index] = OCCUPIED;
                    self.turn = Player::Black
                }
            }

            Some((*row_index, *col_index))
        } else {
            None
        }
    }

    pub fn legal_moves(&self) -> HashSet<String> {
        let mut legal_moves_hashset = HashSet::new();
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
    fn row_col_to_flat_index(&self, row_index: usize, col_index: usize) -> usize {
        row_index * self.size + col_index
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

        println!("{board_string}");
    }
}
