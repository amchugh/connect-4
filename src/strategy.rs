use crate::board::{Board, Piece};
use rand::seq::IndexedRandom;
use std::cell::RefCell;

pub trait Strategy: std::fmt::Display {
    fn play(&self, board: &Board) -> Option<usize> {
        let moves = board.valid_moves();
        self.select_from(board, &moves)
    }
    fn select_from(&self, board: &Board, options: &[usize]) -> Option<usize>;
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
    fn select_from(&self, _: &Board, options: &[usize]) -> Option<usize> {
        options.choose(&mut self.rng.borrow_mut()).cloned()
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
    fn select_from(&self, board: &Board, options: &[usize]) -> Option<usize> {
        // If we could win, win.
        for col in options {
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            if test_board.has_winner() == Some(self.piece) {
                return Some(*col);
            }
        }

        // If we are going to lose, don't lose to it.
        let opponent = self.piece.opponent();
        for col in options {
            let mut test_board = *board;
            test_board.place(*col, opponent);
            if test_board.has_winner() == Some(opponent) {
                return Some(*col);
            }
        }

        // Otherwise, do the fallback strategy
        self.fallback.select_from(board, options)
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
    fn select_from(&self, board: &Board, options: &[usize]) -> Option<usize> {
        // We're going to pretend like we can place twice in a row.
        // If we can do that and win, let's do it.
        for col in options {
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            if test_board.has_winner() == Some(self.piece) {
                return Some(*col);
            }
            // Now, let's look at another move and see if it would win
            for second_move in test_board.valid_moves() {
                let mut second_move_board = test_board;
                second_move_board.place(second_move, self.piece);
                if second_move_board.has_winner() == Some(self.piece) {
                    return Some(*col);
                }
            }
        }

        // Otherwise, do the fallback strategy
        self.fallback.select_from(board, options)
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
}

impl<S: Strategy> ThreeInARow<S> {
    pub fn new(fallback: S, piece: Piece) -> Self {
        ThreeInARow {
            piece,
            fallback: Box::new(fallback),
        }
    }
}

impl<S: Strategy> Strategy for ThreeInARow<S> {
    fn select_from(&self, board: &Board, options: &[usize]) -> Option<usize> {
        let mut best = 0;
        let mut best_moves = vec![];

        for col in options {
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            let score = test_board.count_winning_opportunities(self.piece);
            if score > best {
                best = score;
                best_moves.clear();
                best_moves.push(*col);
            } else if score == best {
                best_moves.push(*col);
            }
        }

        // Run the fallback strategy on the best moves
        self.fallback.select_from(board, &best_moves)
    }
}

impl<S: Strategy> std::fmt::Display for ThreeInARow<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ThreeInARow => {}", self.fallback)
    }
}

/// Strategy that avoids placing pieces in columns that would allow the opponent to win on their next turn.
pub struct AvoidTraps {
    piece: Piece,
    fallback: Box<dyn Strategy>,
}

impl AvoidTraps {
    pub fn new(fallback: Box<dyn Strategy>, piece: Piece) -> Self {
        AvoidTraps { piece, fallback }
    }
}

impl Strategy for AvoidTraps {
    fn select_from(&self, board: &Board, options: &[usize]) -> Option<usize> {
        // Disqualify columns that would allow the opponent to win on their next turn
        let mut allowed = Vec::with_capacity(options.len());

        'candidate_loop: for col in options {
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            // If this move wins, short-circuit
            if test_board.has_winner() == Some(self.piece) {
                return Some(*col);
            }
            // No good if the opponent has a winning opportunity
            if !test_board.winning_moves(self.piece.opponent()).is_empty() {
                continue 'candidate_loop;
            }
            allowed.push(*col);
        }

        // If any move loses, we know we're going to lose :(
        // So just pick the first move that we were given
        if allowed.is_empty() {
            options.first().cloned()
        } else {
            // Run the fallback strategy on the allowed moves
            self.fallback.select_from(board, &allowed)
        }
    }
}

impl std::fmt::Display for AvoidTraps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AvoidTraps => {}", self.fallback)
    }
}

/// Strategy that avoids placing anywhere the other player gets more than one three-in-a-row.
pub struct AvoidInescapableTraps {
    piece: Piece,
    fallback: Box<dyn Strategy>,
}

impl AvoidInescapableTraps {
    pub fn new(fallback: Box<dyn Strategy>, piece: Piece) -> Self {
        AvoidInescapableTraps { piece, fallback }
    }
}

impl Strategy for AvoidInescapableTraps {
    fn select_from(&self, board: &Board, options: &[usize]) -> Option<usize> {
        // Disqualify columns that would allow the opponent to win on their next turn
        let mut allowed = Vec::with_capacity(options.len());

        'candidate_loop: for col in options {
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            // If this move wins, short-circuit
            if test_board.has_winner() == Some(self.piece) {
                return Some(*col);
            }
            for next_col in test_board.valid_moves() {
                let mut next_board = test_board;
                next_board.place(next_col, self.piece.opponent());
                // If we've lost or have a losing position, don't take it.
                if test_board.has_winner() == Some(self.piece.opponent()) {
                    continue 'candidate_loop;
                }
                if test_board.winning_moves(self.piece.opponent()).len() > 1 {
                    continue 'candidate_loop;
                }
            }
            allowed.push(*col);
        }

        // If any move loses, we know we're going to lose :(
        // So just pick the first move that we were given
        if allowed.is_empty() {
            options.first().cloned()
        } else {
            // Run the fallback strategy on the allowed moves
            self.fallback.select_from(board, &allowed)
        }
    }
}

impl std::fmt::Display for AvoidInescapableTraps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AvoidInescapableTraps => {}", self.fallback)
    }
}
