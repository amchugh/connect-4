use std::{fmt, hint::unreachable_unchecked};

pub const ROWS: usize = 6;
pub const COLUMNS: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Piece {
    Empty,
    Red,
    Blue,
}

impl Piece {
    #[inline]
    pub fn opponent(&self) -> Piece {
        match self {
            Piece::Empty => panic!("Cannot get opponent of empty piece"),
            Piece::Red => Piece::Blue,
            Piece::Blue => Piece::Red,
        }
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Piece::Red => "Red",
            Piece::Blue => "Blue",
            Piece::Empty => panic!("Why are we trying to get the color of Empty?"),
        }
    }
}

///
/// The board is 6 rows by 7 columns in size.
///
/// Every column is represented with 9 bits.
///
/// Bits 0-2 store a 3-bit number encoding the height of the current column.
/// Note that 7 is never used, so this isn’t the most efficient packing.
///
/// Bits 3-8 store the piece data. A zero represents a red piece while a
/// one represents a yellow piece. Only the first N bits determined by the
/// first 3 bits are valid. The rest is padded with 0s to keep implementation
/// clean. Again, not the most efficient packing but the next breakpoint (32b)
/// is so far away.
///
/// Seven columns of 9 bits gives 63b representation, meaning you can pack
/// any* board in one 64b integer.
///
/// 0: 76543210 -- unused,
/// 1: 76543210
/// 2: 76543210
/// 3: 76543210
/// 4: 76543210
/// 5: 76543210 -- [10] -> last of column 2 data, [432] -> column 3 height, ... etc.
/// 6: 76543210 -- [0] -> last bit of column 1 data, [321] -> column 2 height, [7654] -> column 2 data
/// 7: 76543210 -- [210] -> column 1 height, [76543] -> first 5 bits of column 1 data
///
///
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Board(u64);

type BoardArray = [[Piece; COLUMNS]; ROWS];

impl Board {
    pub const EMPTY: Board = Board(0);

    #[inline]
    pub fn new() -> Self {
        Board::EMPTY
    }

    fn from_array(arr: BoardArray) -> Self {
        let mut board = Board::EMPTY;
        for column in 0..COLUMNS {
            let mut height = 0;
            // We will end with setting the column height
            // Allow the range loop so that the compiler can unroll this.
            #[allow(clippy::needless_range_loop)]
            for row in 0..ROWS {
                let row_idx = ROWS - row - 1;
                let piece = arr[row_idx][column];
                match piece {
                    Piece::Empty => break,
                    Piece::Red => {
                        // Don't need to do anything as they are by-default red.
                        debug_assert!(board.get_raw(column, row) == Piece::Red, "{board}");
                    }
                    Piece::Blue => {
                        // Need to set that piece blue
                        board.set_blue(column, row);
                    }
                }
                height += 1;
            }
            // Set the column height
            board.set_column_height(column, height);
        }
        board
    }

    #[inline]
    fn column_height(&self, column: usize) -> usize {
        debug_assert!(column < COLUMNS, "Column must be on the board");

        const MASK: u64 = 0b111; // Column height is 3 bits
        let value = self.0 >> (column * 9);
        (value & MASK) as usize
    }

    #[inline]
    fn to_array(self) -> BoardArray {
        let mut arr = [[Piece::Empty; COLUMNS]; ROWS];
        for column in 0..COLUMNS {
            let height = self.column_height(column);
            for row in 0..height {
                let row_idx = ROWS - row - 1;
                arr[row_idx][column] = self.get_checked(column, row);
            }
        }
        arr
    }

    #[inline]
    fn get_raw(&self, column: usize, row: usize) -> Piece {
        const COLUMN_HEIGHT_OFFSET: usize = 3;
        let value = self.0 >> ((column * 9) + row + COLUMN_HEIGHT_OFFSET);
        match value & 0b1 {
            0 => Piece::Red,
            1 => Piece::Blue,
            // Obviously this value can only be 0 or 1.
            _ => unsafe { unreachable_unchecked() },
        }
    }

    #[inline]
    fn get_checked(&self, column: usize, row: usize) -> Piece {
        let height = self.column_height(column);
        if height <= row {
            Piece::Empty
        } else {
            self.get_raw(column, row)
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

        let mut board_array = [[Piece::Empty; COLUMNS]; ROWS];

        for (row, line) in lines.iter().enumerate() {
            assert!(
                line.len() <= COLUMNS,
                "Invalid number of columns, max {}, got {}",
                COLUMNS,
                line.len()
            );
            for (col, c) in line.chars().enumerate() {
                match c {
                    ' ' => board_array[row][col] = Piece::Empty,
                    'R' => {
                        board_array[row][col] = Piece::Red;
                    }
                    'B' => {
                        board_array[row][col] = Piece::Blue;
                    }
                    _ => panic!("Invalid character"),
                }
            }
        }

        // As a debug measure, make sure the board is balanced
        #[cfg(debug_assertions)]
        {
            let mut red_played = 0;
            let mut blue_played = 0;
            for row in board_array {
                for piece in row {
                    match piece {
                        Piece::Red => red_played += 1,
                        Piece::Blue => blue_played += 1,
                        _ => {}
                    }
                }
            }
            debug_assert!(red_played == blue_played || red_played == blue_played + 1);
        }

        Board::from_array(board_array)
    }

    pub fn short_string(&self) -> String {
        let mut s = String::with_capacity((ROWS + 1) * COLUMNS + 1);
        s.push('!');
        let repr = self.to_array();
        for (idx, row) in repr.into_iter().enumerate() {
            let mut leading_spaces = 0;
            for piece in row {
                match piece {
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
            if idx < ROWS - 1 {
                s.push('/');
            }
        }
        s
    }

    #[inline]
    fn set_blue(&mut self, column: usize, height: usize) {
        debug_assert!(column < COLUMNS, "Column must be on the board");
        debug_assert!(height < ROWS, "Cannot overfill a column");

        // We need to set this to a 1.
        let placed_value = 1 << ((column * 9) + 3 + height);
        self.0 |= placed_value;
    }

    #[inline]
    fn set_column_height(&mut self, column: usize, height: usize) {
        debug_assert!(column < COLUMNS, "Column must be on the board");
        debug_assert!(height <= ROWS, "Cannot overfill a column");
        // Create the mask to remove the current height. We will then OR it in.
        let mask = 0b111 << (column * 9);
        let height_placed = (height as u64) << (column * 9);
        let value = (self.0 & !mask) | height_placed;
        self.0 = value;
    }

    #[inline]
    pub fn with_place(&mut self, column: usize, piece: Piece) {
        debug_assert!(
            piece != Piece::Empty,
            "Should never try and place an empty piece"
        );
        debug_assert!(column < COLUMNS, "Column must be on the board");

        let height = self.column_height(column);
        debug_assert!(height < ROWS - 1, "Column is full");

        // Need to increment the column height
        self.set_column_height(column, height + 1);

        // Need to set the piece to the correct value
        match piece {
            Piece::Red => {
                // We need to set this to a 0... but by definition it should be 0 already.
                debug_assert!(self.get_checked(column, height) == Piece::Red);
            }
            Piece::Blue => {
                self.set_blue(column, height);
            }
            Piece::Empty => unreachable!(),
        }
    }

    pub fn place(&self, column: usize, piece: Piece) -> Board {
        let mut next_state = *self;
        next_state.with_place(column, piece);
        next_state
    }

    pub fn next_player(&self) -> Piece {
        // This is a bit expensive to calculate...
        let mut red_pieces = 0;
        let mut blue_pieces = 0;
        for column in 0..COLUMNS {
            let height = self.column_height(column);
            if height == 0 {
                continue;
            }
            let column_data_mask = 0b111111 >> (6 - height);
            let column_data = (self.0 >> (3 + column * 9)) & column_data_mask;
            let ones = column_data.count_ones();
            blue_pieces += ones;
            red_pieces += (height as u32) - ones;
        }
        assert!(
            red_pieces == blue_pieces || red_pieces == blue_pieces + 1,
            "Should only ever differ by one"
        );
        if red_pieces == blue_pieces {
            Piece::Red
        } else {
            Piece::Blue
        }
    }

    pub fn num_pieces_played(&self) -> usize {
        let mut pieces_played = 0;
        for column in 0..COLUMNS {
            let height = self.column_height(column);
            pieces_played += height;
        }
        pieces_played
    }

    pub fn valid_moves(&self) -> Vec<usize> {
        let mut moves = Vec::with_capacity(COLUMNS);
        for column in 0..COLUMNS {
            if self.column_height(column) < ROWS - 1 {
                moves.push(column);
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
        self.all_future_boards(self.next_player())
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
        let repr = self.to_array();

        // Check horizontal opportunities
        for row in repr.into_iter() {
            for col in 0..COLUMNS - 3 {
                let positions = [row[col], row[col + 1], row[col + 2], row[col + 3]];
                if self.is_winning_opportunity(&positions, piece) {
                    count += 1;
                }
            }
        }

        // Check vertical opportunities
        for row in 0..ROWS - 3 {
            for col in 0..COLUMNS {
                let positions = [
                    repr[row][col],
                    repr[row + 1][col],
                    repr[row + 2][col],
                    repr[row + 3][col],
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
                    repr[row][col],
                    repr[row - 1][col + 1],
                    repr[row - 2][col + 2],
                    repr[row - 3][col + 3],
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
                    repr[row][col],
                    repr[row + 1][col + 1],
                    repr[row + 2][col + 2],
                    repr[row + 3][col + 3],
                ];
                if self.is_winning_opportunity(&positions, piece) {
                    count += 1;
                }
            }
        }

        count
    }

    fn check_rows(&self) -> Option<Piece> {
        let repr = self.to_array();
        for row in &repr {
            if let Some(winner) = self.check_line_in_array(row) {
                return Some(winner);
            }
        }
        None
    }

    fn check_columns(&self) -> Option<Piece> {
        let repr = self.to_array();
        for col in 0..COLUMNS {
            for row in 0..ROWS - 3 {
                let pieces = [
                    repr[row][col],
                    repr[row + 1][col],
                    repr[row + 2][col],
                    repr[row + 3][col],
                ];
                if let Some(winner) = self.check_four_pieces(&pieces) {
                    return Some(winner);
                }
            }
        }
        None
    }

    fn check_diagonals(&self) -> Option<Piece> {
        let repr = self.to_array();
        // Positive slope diagonals (bottom-left to top-right)
        for row in 3..ROWS {
            for col in 0..COLUMNS - 3 {
                let pieces = [
                    repr[row][col],
                    repr[row - 1][col + 1],
                    repr[row - 2][col + 2],
                    repr[row - 3][col + 3],
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
                    repr[row][col],
                    repr[row + 1][col + 1],
                    repr[row + 2][col + 2],
                    repr[row + 3][col + 3],
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
        let repr = self.to_array();
        for (idx, row) in repr.into_iter().enumerate() {
            for col in row {
                write!(f, "{} ", col)?;
            }
            if idx != ROWS - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
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
    fn to_from_array() {
        let mut board = Board::new();
        assert_eq!(Board::from_array(board.to_array()), board);

        board.with_place(0, Piece::Red);
        board.with_place(1, Piece::Blue);
        board.with_place(2, Piece::Red);
        assert_eq!(Board::from_array(board.to_array()), board);

        board.with_place(0, Piece::Blue);
        board.with_place(1, Piece::Red);
        board.with_place(2, Piece::Blue);
        assert_eq!(Board::from_array(board.to_array()), board);

        board.with_place(0, Piece::Blue);
        board.with_place(6, Piece::Red);
        board.with_place(0, Piece::Blue);
        board.with_place(6, Piece::Red);
        board.with_place(0, Piece::Blue);
        board.with_place(6, Piece::Red);
        assert!(board.is_terminal());
        assert_eq!(Board::from_array(board.to_array()), board);
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
