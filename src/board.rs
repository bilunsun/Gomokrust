enum Player {
    White = 0,
    Black = 1,
}

struct BaseBoard {
    white_stones: Vec<bool>,
    black_stones: Vec<bool>,
}

pub struct Board {
    size: usize,
    base_board: BaseBoard,
    turn: Player,
    row_names: Vec<String>,
    col_names: Vec<String>,
}

impl BaseBoard {
    fn new(size: usize) -> Self {
        let mut white_stones = vec![false; size];
        let mut black_stones = vec![false; size];

        Self {
            white_stones,
            black_stones,
        }
    }
}

impl Board {
    pub fn new(size: usize) -> Self {
        assert!(size <= 26, "The maximum supported board size is 15.");
        let base_board: BaseBoard::new(size);
        let row_names: Vec<String> = (1..=size as u32).map(|c| c.to_string()).collect();
        let col_names: Vec<String> = (b'a'..=b'z')
            .filter(|c| c - b'a' <= size as u8)
            .map(|c| (c as char).to_string())
            .collect();

        Self {
            size,
            base_board,
            row_names,
            col_names,
            turn: Player::Black,
        }
    }
}
