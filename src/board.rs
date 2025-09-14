use std::fmt;

pub const ROWS: usize = 6;
pub const COLUMNS: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn name(&self) -> &'static str {
        match self {
            Piece::Red => "Red",
            Piece::Blue => "Blue",
            Piece::Empty => panic!("Why are we trying to get the color of Empty?"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Board {
    state: [[Piece; COLUMNS]; ROWS],
    pieces_played: usize,
    next_player: Piece,
}

impl std::cmp::PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        for row in 0..ROWS {
            for col in 0..COLUMNS {
                if self.state[row][col] != other.state[row][col] {
                    return false;
                }
            }
        }
        true
    }
}
impl std::cmp::Eq for Board {}

impl std::hash::Hash for Board {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.state.hash(state);
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            state: [[Piece::Empty; COLUMNS]; ROWS],
            pieces_played: 0,
            next_player: Piece::Red,
        }
    }

    #[allow(unused)]
    pub fn from(board: &str) -> Self {
        // Assumes the board is like the following:
        // "!///    B/    B/  BRRRR"
        assert!(board.starts_with("!"));
        let (_, board) = board.split_at(1);
        let lines: Vec<_> = board.split("/").collect();
        assert!(
            lines.len() == ROWS,
            "Wrong number of rows, expected {}, got {}",
            ROWS,
            lines.len()
        );

        let mut board = Board::new();
        let mut red_played = 0;
        let mut blue_played = 0;

        for (row, line) in lines.iter().enumerate() {
            assert!(
                line.len() <= COLUMNS,
                "Invalid number of columns, max {}, got {}",
                COLUMNS,
                line.len()
            );
            for (col, c) in line.chars().enumerate() {
                match c {
                    ' ' => board.state[row][col] = Piece::Empty,
                    'R' => {
                        board.state[row][col] = Piece::Red;
                        red_played += 1;
                    }
                    'B' => {
                        board.state[row][col] = Piece::Blue;
                        blue_played += 1;
                    }
                    _ => panic!("Invalid character"),
                }
            }
        }
        assert!(red_played == blue_played || red_played == blue_played + 1);
        board.next_player = if red_played > blue_played {
            Piece::Blue
        } else {
            Piece::Red
        };
        board.pieces_played = red_played + blue_played;
        board
    }

    pub fn short_string(&self) -> String {
        let mut s = String::with_capacity((ROWS + 1) * COLUMNS + 1);
        s.push('!');
        for row in 0..ROWS {
            let mut leading_spaces = 0;
            for col in 0..COLUMNS {
                match self.state[row][col] {
                    Piece::Empty => leading_spaces += 1,
                    Piece::Red => {
                        if leading_spaces > 0 {
                            for _ in 0..leading_spaces {
                                s.push(' ');
                            }
                            leading_spaces = 0;
                        }
                        s.push('R');
                    }
                    Piece::Blue => {
                        if leading_spaces > 0 {
                            for _ in 0..leading_spaces {
                                s.push(' ');
                            }
                            leading_spaces = 0;
                        }
                        s.push('B');
                    }
                }
            }
            if row < ROWS - 1 {
                s.push('/');
            }
        }
        s
    }

    fn _place(&mut self, column: usize, piece: Piece) {
        debug_assert!((0..COLUMNS).contains(&column));
        debug_assert!(self.state[0][column] == Piece::Empty);

        self.pieces_played += 1;
        for row in (0..ROWS).rev() {
            if self.state[row][column] == Piece::Empty {
                self.state[row][column] = piece;
                return;
            }
        }
        unreachable!("Column is full");
    }

    pub fn with_place(&mut self, column: usize, piece: Piece) {
        debug_assert!(piece == self.next_player);
        self.next_player = piece.opponent();
        self._place(column, piece);
    }

    pub fn place(&self, column: usize, piece: Piece) -> Board {
        let mut next_state = *self;
        next_state._place(column, piece);
        next_state.next_player = piece.opponent();
        next_state
    }

    pub fn next_player(&self) -> Piece {
        self.next_player
    }

    pub fn num_pieces_played(&self) -> usize {
        self.pieces_played
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

    #[allow(unused)]
    pub fn is_terminal(&self) -> bool {
        self.has_winner().is_some() || self.valid_moves().is_empty()
    }

    pub fn has_winner(&self) -> Option<Piece> {
        self.check_rows()
            .or_else(|| self.check_columns())
            .or_else(|| self.check_diagonals())
    }

    #[allow(unused)]
    pub fn next_states(&self) -> Vec<Self> {
        self.all_future_boards(self.next_player)
    }

    pub fn all_future_boards(&self, piece: Piece) -> Vec<Self> {
        self.valid_moves()
            .into_iter()
            .map(|col| self.place(col, piece))
            .collect()
    }

    /// Returns a vector of valid moves that would result in a win for the given piece.
    pub fn winning_moves(&self, piece: Piece) -> Vec<usize> {
        // Doesn't make sense to ask for winning moves if someone already won
        assert!(self.has_winner().is_none());
        let mut winning_moves = Vec::new();
        for m in self.valid_moves() {
            let mut next_board = *self;
            next_board.with_place(m, piece);
            if next_board.has_winner() == Some(piece) {
                winning_moves.push(m)
            }
        }
        winning_moves
    }

    /// Counts the number of potential four-in-a-row opportunities for the given piece.
    /// This includes patterns like "XXX_", "_XXX", "XX_X", "X_XX" where X is the piece
    /// and _ is an empty space that could be filled to create four-in-a-row.
    pub fn count_winning_opportunities(&self, piece: Piece) -> usize {
        // Don't know how to count winning opportunities with a winner
        assert!(self.has_winner().is_none());

        let mut count = 0;

        // Check horizontal opportunities
        for row in 0..ROWS {
            for col in 0..COLUMNS - 3 {
                let positions = [
                    self.state[row][col],
                    self.state[row][col + 1],
                    self.state[row][col + 2],
                    self.state[row][col + 3],
                ];
                if self.is_winning_opportunity(&positions, piece) {
                    count += 1;
                }
            }
        }

        // Check vertical opportunities
        for row in 0..ROWS - 3 {
            for col in 0..COLUMNS {
                let positions = [
                    self.state[row][col],
                    self.state[row + 1][col],
                    self.state[row + 2][col],
                    self.state[row + 3][col],
                ];
                if self.is_winning_opportunity(&positions, piece) {
                    count += 1;
                }
            }
        }

        // Check positive slope diagonals (bottom-left to top-right)
        for row in 3..ROWS {
            for col in 0..COLUMNS - 3 {
                let positions = [
                    self.state[row][col],
                    self.state[row - 1][col + 1],
                    self.state[row - 2][col + 2],
                    self.state[row - 3][col + 3],
                ];
                if self.is_winning_opportunity(&positions, piece) {
                    count += 1;
                }
            }
        }

        // Check negative slope diagonals (top-left to bottom-right)
        for row in 0..ROWS - 3 {
            for col in 0..COLUMNS - 3 {
                let positions = [
                    self.state[row][col],
                    self.state[row + 1][col + 1],
                    self.state[row + 2][col + 2],
                    self.state[row + 3][col + 3],
                ];
                if self.is_winning_opportunity(&positions, piece) {
                    count += 1;
                }
            }
        }

        count
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

    /// Checks if a four-position line has exactly three pieces of the given type
    /// and one empty space, making it a winning opportunity.
    fn is_winning_opportunity(&self, positions: &[Piece; 4], piece: Piece) -> bool {
        let piece_count = positions.iter().filter(|&&p| p == piece).count();
        let empty_count = positions.iter().filter(|&&p| p == Piece::Empty).count();
        let opponent_count = positions.iter().filter(|&&p| p == piece.opponent()).count();

        // Must have exactly 3 of our pieces, 1 empty space, and 0 opponent pieces
        piece_count == 3 && empty_count == 1 && opponent_count == 0
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
        for y in 0..ROWS {
            for col in &self.state[y] {
                write!(f, "{} ", col)?;
            }
            if y != ROWS - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
        // match self.has_winner() {
        //     Some(Piece::Red) => write!(f, "Red wins!"),
        //     Some(Piece::Blue) => write!(f, "Blue wins!"),
        //     Some(Piece::Empty) => unreachable!("Empty piece cannot win"),
        //     None => write!(f, "No winner yet"),
        // }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        let mut board1 = Board::new();
        let mut board2 = Board::new();
        assert_eq!(board1, board2);

        board1.with_place(0, Piece::Red);
        board2.with_place(0, Piece::Red);
        assert_eq!(board1, board2);

        board1.with_place(1, Piece::Blue);
        board2.with_place(2, Piece::Red);
        assert_ne!(board1, board2);

        // Order doesn't matter
        board1.with_place(2, Piece::Red);
        board2.with_place(1, Piece::Blue);
        assert_eq!(board1, board2);
    }

    #[test]
    fn test_count_winning_opportunities_empty_board() {
        let board = Board::new();
        assert_eq!(board.count_winning_opportunities(Piece::Red), 0);
        assert_eq!(board.count_winning_opportunities(Piece::Blue), 0);
    }

    #[test]
    fn test_count_winning_opportunities_horizontal() {
        let mut board = Board::new();
        // Place three red pieces horizontally: RRR_
        board.with_place(0, Piece::Red);
        board.with_place(1, Piece::Red);
        board.with_place(2, Piece::Red);

        // Should have 1 winning opportunity (can complete at column 3)
        assert_eq!(board.count_winning_opportunities(Piece::Red), 1);
        assert_eq!(board.count_winning_opportunities(Piece::Blue), 0);
    }

    #[test]
    fn test_count_winning_opportunities_horizontal_gap_in_middle() {
        let mut board = Board::new();
        // Place RR_R pattern
        board.with_place(0, Piece::Red);
        board.with_place(1, Piece::Red);
        board.with_place(3, Piece::Red);

        // Should have 1 winning opportunity (can complete at column 2)
        assert_eq!(board.count_winning_opportunities(Piece::Red), 1);
        assert_eq!(board.count_winning_opportunities(Piece::Blue), 0);
    }

    #[test]
    fn test_count_winning_opportunities_horizontal_gap_at_start() {
        let mut board = Board::new();
        // Place _RRR pattern
        board.with_place(1, Piece::Red);
        board.with_place(2, Piece::Red);
        board.with_place(3, Piece::Red);

        // This creates two overlapping opportunities:
        // _RRR (positions 0-3) and RRR_ (positions 1-4)
        assert_eq!(board.count_winning_opportunities(Piece::Red), 2);
        assert_eq!(board.count_winning_opportunities(Piece::Blue), 0);
    }

    #[test]
    fn test_count_winning_opportunities_vertical() {
        let mut board = Board::new();
        // Place three red pieces vertically in column 0
        board.with_place(0, Piece::Red);
        board.with_place(0, Piece::Red);
        board.with_place(0, Piece::Red);

        // Should have 1 winning opportunity (can complete by placing on top)
        assert_eq!(board.count_winning_opportunities(Piece::Red), 1);
        assert_eq!(board.count_winning_opportunities(Piece::Blue), 0);
    }

    #[test]
    fn test_count_winning_opportunities_diagonal_positive_slope() {
        let mut board = Board::new();
        // Create a diagonal pattern (bottom-left to top-right)
        // Place pieces to build up the diagonal
        board.with_place(0, Piece::Red); // Bottom of column 0

        board.with_place(1, Piece::Blue); // Bottom of column 1
        board.with_place(1, Piece::Red); // Second level of column 1

        board.with_place(2, Piece::Blue); // Bottom of column 2
        board.with_place(2, Piece::Blue); // Second level of column 2
        board.with_place(2, Piece::Red); // Third level of column 2

        // Now we have a diagonal RRR_ pattern, missing the top-right piece
        // Should have 1 winning opportunity
        assert_eq!(board.count_winning_opportunities(Piece::Red), 1);
    }

    #[test]
    fn test_count_winning_opportunities_diagonal_negative_slope() {
        let mut board = Board::new();
        // Create a diagonal pattern (top-left to bottom-right)
        // We need to build up the columns to the right heights

        // Column 0: need red at row 2 (third from top)
        board.with_place(0, Piece::Blue); // Row 5 (bottom)
        board.with_place(0, Piece::Blue); // Row 4
        board.with_place(0, Piece::Blue); // Row 3
        board.with_place(0, Piece::Red); // Row 2

        // Column 1: need red at row 3
        board.with_place(1, Piece::Blue); // Row 5
        board.with_place(1, Piece::Blue); // Row 4
        board.with_place(1, Piece::Red); // Row 3

        // Column 2: need red at row 4
        board.with_place(2, Piece::Blue); // Row 5
        board.with_place(2, Piece::Red); // Row 4

        // Column 3: needs to be empty at row 5 for the opportunity
        // Don't place anything in column 3

        // This should create a diagonal RRR_ pattern
        assert_eq!(board.count_winning_opportunities(Piece::Red), 1);
    }

    #[test]
    fn test_count_winning_opportunities_blocked_by_opponent() {
        let mut board = Board::new();
        // Place RRR but then block with opponent piece
        board.with_place(0, Piece::Red);
        board.with_place(1, Piece::Red);
        board.with_place(2, Piece::Red);
        board.with_place(3, Piece::Blue); // Block the winning opportunity

        // Should have 0 winning opportunities because opponent piece blocks
        assert_eq!(board.count_winning_opportunities(Piece::Red), 0);
        assert_eq!(board.count_winning_opportunities(Piece::Blue), 0);
    }

    #[test]
    fn test_count_winning_opportunities_multiple_opportunities() {
        let mut board = Board::new();
        // Create a simple case with clear multiple opportunities
        // Bottom row: RRR_
        board.with_place(0, Piece::Red);
        board.with_place(1, Piece::Red);
        board.with_place(2, Piece::Red);

        // Create a separate vertical opportunity in column 6
        board.with_place(6, Piece::Red);
        board.with_place(6, Piece::Red);
        board.with_place(6, Piece::Red);

        // Should have at least 2 opportunities: horizontal and vertical
        assert_eq!(board.count_winning_opportunities(Piece::Red), 2);
    }

    #[test]
    fn test_count_winning_opportunities_r_gap_rr_pattern() {
        let mut board = Board::new();
        // Create R_RR pattern
        board.with_place(0, Piece::Red);
        board.with_place(2, Piece::Red);
        board.with_place(3, Piece::Red);

        // Should have 1 winning opportunity (can complete at column 1)
        assert_eq!(board.count_winning_opportunities(Piece::Red), 1);
    }
}
