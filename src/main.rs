use anyhow::{Context, Result};
use colorize::AnsiColor;
use rand::seq::SliceRandom;
use std::{cell::RefCell, time::Instant};

const ROWS: usize = 6;
const COLUMNS: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Piece {
    Empty,
    Red,
    Blue,
}

impl Piece {
    fn opponent(&self) -> Piece {
        match self {
            Piece::Empty => panic!("Cannot get opponent of empty piece"),
            Piece::Red => Piece::Blue,
            Piece::Blue => Piece::Red,
        }
    }
}

#[derive(Clone)]
struct Board {
    state: [[Piece; COLUMNS]; ROWS],
}

impl Board {
    fn new() -> Self {
        Board {
            state: [[Piece::Empty; COLUMNS]; ROWS],
        }
    }

    fn place(&mut self, column: usize, piece: Piece) {
        assert!((0..COLUMNS).contains(&column));
        // Ensure that column isn't full
        assert!(self.state[0][column] == Piece::Empty);
        // Add it as high up as possible
        for row in (0..ROWS).rev() {
            if self.state[row][column] == Piece::Empty {
                self.state[row][column] = piece;
                return;
            }
        }
    }

    fn valid_moves(&self) -> Vec<usize> {
        let mut moves = Vec::new();
        for col in 0..COLUMNS {
            if self.state[0][col] == Piece::Empty {
                moves.push(col);
            }
        }
        moves
    }

    fn has_winner(&self) -> Option<Piece> {
        // Check rows
        for row in &self.state {
            for col in 0..COLUMNS - 3 {
                if row[col] == Piece::Red
                    && row[col + 1] == Piece::Red
                    && row[col + 2] == Piece::Red
                    && row[col + 3] == Piece::Red
                {
                    return Some(Piece::Red);
                }
                if row[col] == Piece::Blue
                    && row[col + 1] == Piece::Blue
                    && row[col + 2] == Piece::Blue
                    && row[col + 3] == Piece::Blue
                {
                    return Some(Piece::Blue);
                }
            }
        }

        // Check columns
        for col in 0..COLUMNS {
            for row in 0..ROWS - 3 {
                if self.state[row][col] == Piece::Red
                    && self.state[row + 1][col] == Piece::Red
                    && self.state[row + 2][col] == Piece::Red
                    && self.state[row + 3][col] == Piece::Red
                {
                    return Some(Piece::Red);
                }
                if self.state[row][col] == Piece::Blue
                    && self.state[row + 1][col] == Piece::Blue
                    && self.state[row + 2][col] == Piece::Blue
                    && self.state[row + 3][col] == Piece::Blue
                {
                    return Some(Piece::Blue);
                }
            }
        }

        // Check diagonals
        for row in 0..ROWS - 3 {
            for col in 0..COLUMNS - 3 {
                if self.state[row][col] == Piece::Red
                    && self.state[row + 1][col + 1] == Piece::Red
                    && self.state[row + 2][col + 2] == Piece::Red
                    && self.state[row + 3][col + 3] == Piece::Red
                {
                    return Some(Piece::Red);
                }
                if self.state[row][col] == Piece::Blue
                    && self.state[row + 1][col + 1] == Piece::Blue
                    && self.state[row + 2][col + 2] == Piece::Blue
                    && self.state[row + 3][col + 3] == Piece::Blue
                {
                    return Some(Piece::Blue);
                }
            }
        }

        for row in 0..ROWS - 3 {
            for col in 3..COLUMNS {
                if self.state[row][col] == Piece::Red
                    && self.state[row + 1][col - 1] == Piece::Red
                    && self.state[row + 2][col - 2] == Piece::Red
                    && self.state[row + 3][col - 3] == Piece::Red
                {
                    return Some(Piece::Red);
                }
                if self.state[row][col] == Piece::Blue
                    && self.state[row + 1][col - 1] == Piece::Blue
                    && self.state[row + 2][col - 2] == Piece::Blue
                    && self.state[row + 3][col - 3] == Piece::Blue
                {
                    return Some(Piece::Blue);
                }
            }
        }

        None
    }
}

// ------------------------------

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Piece::Empty => write!(f, "{}", "[ ]".black()),
            Piece::Red => write!(f, "{}", "[R]".b_redb()),
            Piece::Blue => write!(f, "{}", "[B]".b_blueb()),
        }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.state {
            for col in row {
                write!(f, "{} ", col)?;
            }
            writeln!(f)?;
        }
        // See if there is a winner
        match self.has_winner() {
            Some(Piece::Red) => write!(f, "Red wins!"),
            Some(Piece::Blue) => write!(f, "Blue wins!"),
            Some(_) => panic!("Unexpected winner"),
            None => write!(f, "No winner yet"),
        }
    }
}

trait Strategy {
    fn play(&self, board: &Board) -> Option<usize>;
}

#[derive(Clone)]
struct RandomStrategy {
    rng: RefCell<rand::rngs::ThreadRng>,
}

impl Default for RandomStrategy {
    fn default() -> Self {
        RandomStrategy {
            rng: RefCell::new(rand::rng()),
        }
    }
}

impl Strategy for RandomStrategy {
    fn play(&self, board: &Board) -> Option<usize> {
        let mut moves = board.valid_moves();
        moves.shuffle(&mut self.rng.borrow_mut());
        moves.pop()
    }
}

#[derive(Clone)]
struct TriesToWin<S: Strategy> {
    fallback: Box<S>,
    piece: Piece,
}

impl<S: Strategy> TriesToWin<S> {
    fn new(fallback: S, piece: Piece) -> Self {
        TriesToWin {
            fallback: Box::new(fallback),
            piece,
        }
    }
}

impl<S: Strategy> Strategy for TriesToWin<S> {
    fn play(&self, board: &Board) -> Option<usize> {
        // If we could win, win.
        for col in board.valid_moves() {
            let mut board = board.clone();
            board.place(col, self.piece);
            if board.has_winner() == Some(self.piece) {
                return Some(col);
            }
        }

        // If we are going to lose, don't lose to it.
        let opponent = self.piece.opponent();
        for col in board.valid_moves() {
            let mut board = board.clone();
            board.place(col, opponent);
            if board.has_winner() == Some(opponent) {
                return Some(col);
            }
        }

        // Otherwise, do the fallback strategy
        self.fallback.play(board)
    }
}

#[derive(Clone)]
struct Setup<S: Strategy> {
    fallback: Box<S>,
    piece: Piece,
}

impl<S: Strategy> Setup<S> {
    fn new(fallback: S, piece: Piece) -> Self {
        Setup {
            fallback: Box::new(fallback),
            piece,
        }
    }
}

impl<S: Strategy> Strategy for Setup<S> {
    fn play(&self, board: &Board) -> Option<usize> {
        // We're going to pretend like we can place twice in a row.
        // If we can do that and win, let's do it.
        for col in board.valid_moves() {
            let mut test_board = board.clone();
            test_board.place(col, self.piece);
            if test_board.has_winner() == Some(self.piece) {
                return Some(col);
            }
            // Now, let's look at another move and see if it would win
            for second_move in test_board.valid_moves() {
                let mut second_move_board = test_board.clone();
                second_move_board.place(second_move, self.piece);
                if second_move_board.has_winner() == Some(self.piece) {
                    return Some(col);
                }
            }
        }

        // Otherwise, do the fallback strategy
        self.fallback.play(board)
    }
}

fn game<R: Strategy, B: Strategy>(red: R, blue: B) -> Option<Board> {
    let mut board = Board::new();
    loop {
        // Red plays, then blue.
        // If there's a winner or no moves left, leave
        if board.has_winner().is_some() || board.valid_moves().is_empty() {
            break;
        }
        let col = red.play(&board)?;
        board.place(col, Piece::Red);

        if board.has_winner().is_some() || board.valid_moves().is_empty() {
            break;
        }
        let col = blue.play(&board)?;
        board.place(col, Piece::Blue);
    }
    Some(board)
}

fn main() -> Result<()> {
    let red = Setup::new(
        TriesToWin::new(RandomStrategy::default(), Piece::Red),
        Piece::Red,
    );
    let blue = TriesToWin::new(RandomStrategy::default(), Piece::Blue);

    let mut red_wins = 0;
    let mut blue_wins = 0;
    let mut ties = 0;

    const GAMES: usize = 100_000;

    let start = Instant::now();
    for _ in 0..GAMES {
        let result = game(red.clone(), blue.clone()).context("Failed to play game")?;

        match result.has_winner() {
            Some(Piece::Red) => red_wins += 1,
            Some(Piece::Blue) => blue_wins += 1,
            Some(_) => panic!("Unexpected winner"),
            None => ties += 1,
        }
    }
    let duration = start.elapsed();

    println!("Result from 100k games (took {}ms):", duration.as_millis());
    println!("Red wins: {:.2}%", red_wins as f64 / GAMES as f64 * 100.0);
    println!("Blue wins: {:.2}%", blue_wins as f64 / GAMES as f64 * 100.0);
    println!("Ties: {:.2}%", ties as f64 / GAMES as f64 * 100.0);

    Ok(())
}
