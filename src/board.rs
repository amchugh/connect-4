use std::fmt;

pub const ROWS: usize = 6;
pub const COLUMNS: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    Empty,
    Red,
    Blue,
}

impl Piece {
    pub fn opponent(&self) -> Piece {
        match self {
            Piece::Empty => panic!("Cannot get opponent of empty piece"),
            Piece::Red => Piece::Blue,
            Piece::Blue => Piece::Red,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Board {
    state: [[Piece; COLUMNS]; ROWS],
}

impl Board {
    pub fn new() -> Self {
        Board {
            state: [[Piece::Empty; COLUMNS]; ROWS],
        }
    }

    pub fn place(&mut self, column: usize, piece: Piece) {
        debug_assert!((0..COLUMNS).contains(&column));
        debug_assert!(self.state[0][column] == Piece::Empty);

        for row in (0..ROWS).rev() {
            if self.state[row][column] == Piece::Empty {
                self.state[row][column] = piece;
                return;
            }
        }
    }

    pub fn valid_moves(&self) -> Vec<usize> {
        let mut moves = Vec::with_capacity(COLUMNS);
        for col in 0..COLUMNS {
            if self.state[0][col] == Piece::Empty {
                moves.push(col);
            }
        }
        moves
    }

    pub fn has_winner(&self) -> Option<Piece> {
        self.check_rows()
            .or_else(|| self.check_columns())
            .or_else(|| self.check_diagonals())
    }

    fn check_rows(&self) -> Option<Piece> {
        for row in &self.state {
            if let Some(winner) = self.check_line_in_array(row) {
                return Some(winner);
            }
        }
        None
    }

    fn check_columns(&self) -> Option<Piece> {
        for col in 0..COLUMNS {
            for row in 0..ROWS - 3 {
                let pieces = [
                    self.state[row][col],
                    self.state[row + 1][col],
                    self.state[row + 2][col],
                    self.state[row + 3][col],
                ];
                if let Some(winner) = self.check_four_pieces(&pieces) {
                    return Some(winner);
                }
            }
        }
        None
    }

    fn check_diagonals(&self) -> Option<Piece> {
        // Positive slope diagonals (bottom-left to top-right)
        for row in 3..ROWS {
            for col in 0..COLUMNS - 3 {
                let pieces = [
                    self.state[row][col],
                    self.state[row - 1][col + 1],
                    self.state[row - 2][col + 2],
                    self.state[row - 3][col + 3],
                ];
                if let Some(winner) = self.check_four_pieces(&pieces) {
                    return Some(winner);
                }
            }
        }

        // Negative slope diagonals (top-left to bottom-right)
        for row in 0..ROWS - 3 {
            for col in 0..COLUMNS - 3 {
                let pieces = [
                    self.state[row][col],
                    self.state[row + 1][col + 1],
                    self.state[row + 2][col + 2],
                    self.state[row + 3][col + 3],
                ];
                if let Some(winner) = self.check_four_pieces(&pieces) {
                    return Some(winner);
                }
            }
        }

        None
    }

    fn check_line_in_array(&self, row: &[Piece; COLUMNS]) -> Option<Piece> {
        for col in 0..COLUMNS - 3 {
            let pieces = [row[col], row[col + 1], row[col + 2], row[col + 3]];
            if let Some(winner) = self.check_four_pieces(&pieces) {
                return Some(winner);
            }
        }
        None
    }

    fn check_four_pieces(&self, pieces: &[Piece; 4]) -> Option<Piece> {
        if pieces[0] != Piece::Empty
            && pieces[0] == pieces[1]
            && pieces[1] == pieces[2]
            && pieces[2] == pieces[3]
        {
            Some(pieces[0])
        } else {
            None
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use colorize::AnsiColor;
        match self {
            Piece::Empty => write!(f, "{}", "[ ]".black()),
            Piece::Red => write!(f, "{}", "[R]".b_redb()),
            Piece::Blue => write!(f, "{}", "[B]".b_blueb()),
        }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.state {
            for col in row {
                write!(f, "{} ", col)?;
            }
            writeln!(f)?;
        }
        match self.has_winner() {
            Some(Piece::Red) => write!(f, "Red wins!"),
            Some(Piece::Blue) => write!(f, "Blue wins!"),
            Some(Piece::Empty) => unreachable!("Empty piece cannot win"),
            None => write!(f, "No winner yet"),
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}
