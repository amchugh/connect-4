use crate::board::{Board, Piece};
use rand::seq::SliceRandom;
use std::cell::RefCell;

pub trait Strategy: Clone + std::fmt::Display {
    fn play(&self, board: &Board) -> Option<usize>;
}

#[derive(Clone)]
pub struct RandomStrategy {
    rng: RefCell<rand::rngs::ThreadRng>,
}

impl Default for RandomStrategy {
    fn default() -> Self {
        RandomStrategy {
            rng: RefCell::new(rand::rng()),
        }
    }
}

impl std::fmt::Display for RandomStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RandomStrategy")
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
pub struct TriesToWin<S: Strategy> {
    fallback: Box<S>,
    piece: Piece,
}

impl<S: Strategy> TriesToWin<S> {
    pub fn new(fallback: S, piece: Piece) -> Self {
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
            let mut test_board = *board;
            test_board.place(col, self.piece);
            if test_board.has_winner() == Some(self.piece) {
                return Some(col);
            }
        }

        // If we are going to lose, don't lose to it.
        let opponent = self.piece.opponent();
        for col in board.valid_moves() {
            let mut test_board = *board;
            test_board.place(col, opponent);
            if test_board.has_winner() == Some(opponent) {
                return Some(col);
            }
        }

        // Otherwise, do the fallback strategy
        self.fallback.play(board)
    }
}

impl<S: Strategy> std::fmt::Display for TriesToWin<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TriesToWin => {}", self.fallback)
    }
}

#[derive(Clone)]
pub struct Setup<S: Strategy> {
    fallback: Box<S>,
    piece: Piece,
}

impl<S: Strategy> Setup<S> {
    pub fn new(fallback: S, piece: Piece) -> Self {
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
            let mut test_board = *board;
            test_board.place(col, self.piece);
            if test_board.has_winner() == Some(self.piece) {
                return Some(col);
            }
            // Now, let's look at another move and see if it would win
            for second_move in test_board.valid_moves() {
                let mut second_move_board = test_board;
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

impl<S: Strategy> std::fmt::Display for Setup<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Setup => {}", self.fallback)
    }
}

#[derive(Clone)]
pub struct ThreeInARow<S: Strategy> {
    piece: Piece,
    fallback: Box<S>,
    rng: RefCell<rand::rngs::ThreadRng>,
}

impl<S: Strategy> ThreeInARow<S> {
    pub fn new(fallback: S, piece: Piece, rng: RefCell<rand::rngs::ThreadRng>) -> Self {
        ThreeInARow {
            piece,
            fallback: Box::new(fallback),
            rng,
        }
    }
}

impl<S: Strategy> Strategy for ThreeInARow<S> {
    fn play(&self, board: &Board) -> Option<usize> {
        let mut best = 0;
        let mut best_moves = vec![];

        for col in board.valid_moves() {
            let mut test_board = *board;
            test_board.place(col, self.piece);
            let score = test_board.count_winning_opportunities(self.piece);
            if score > best {
                best = score;
                best_moves.clear();
                best_moves.push(col);
            } else if score == best {
                best_moves.push(col);
            }
        }

        // See if we found any with three-in-a-row
        if best == 0 {
            // Otherwise, do the fallback strategy
            self.fallback.play(board)
        } else {
            // Return a random move from the best moves
            best_moves.shuffle(&mut self.rng.borrow_mut());
            best_moves.pop()
        }
    }
}

impl<S: Strategy> std::fmt::Display for ThreeInARow<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ThreeInARow => {}", self.fallback)
    }
}
