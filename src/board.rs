enum Player {
    White = 0,
    Black = 1,
}

pub struct Board {
    size: usize,
    board: Vec<Vec<bool>>,
    turn: Player,
    row_names: Vec<String>,
    col_names: Vec<String>,
}

impl Board {
    pub fn new(size: usize) -> Self {
        assert!(size <= 26, "The maximum supported board size is 15.");
        let mut board: Vec<Vec<bool>> = vec![vec![false; size]; size];
        let row_names: Vec<String> = (1..=size as u32).map(|c| c.to_string()).collect();
        let col_names: Vec<String> = (b'a'..=b'z')
            .filter(|c| c - b'a' <= size as u8)
            .map(|c| (c as char).to_string())
            .collect();

        Self {
            size,
            board,
            row_names,
            col_names,
            turn: Player::Black,
        }
    }
}
