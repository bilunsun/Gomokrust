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
}

impl Board {
    pub fn new(size: usize) -> Self {
        assert!(size <= 26, "The maximum supported board size is 26.");

        let mut black_stones = vec![false; size * size];
        let mut white_stones = vec![false; size * size];
        let row_names: Vec<String> = (1..=size as u32).map(|c| c.to_string()).collect();
        let col_names: Vec<String> = (b'a'..=b'z')
            .filter(|c| c - b'a' < size as u8)
            .map(|c| (c as char).to_string())
            .collect();

        Self {
            size,
            black_stones,
            white_stones,
            row_names,
            col_names,
            turn: Player::Black,
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
                let index = row_index * self.size + col_index;
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
