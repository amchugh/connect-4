use crate::board::{Board, Piece};
use rand::seq::IndexedRandom;
use std::cell::RefCell;

pub trait Connect4AI: std::fmt::Display {
    fn play(&self, board: &Board) -> Option<usize>;
}

pub struct StrategyStack {
    strategies: Vec<Strategy>,
    rng: RefCell<rand::rngs::ThreadRng>,
}

impl StrategyStack {
    pub fn new(strategies: Vec<Strategy>) -> Self {
        StrategyStack {
            strategies,
            rng: RefCell::new(rand::rngs::ThreadRng::default()),
        }
    }

    pub fn evaluate_options(&self, board: &Board) -> Vec<usize> {
        let mut options = board.valid_moves();
        assert!(!options.is_empty());

        for strategy in &self.strategies {
            match strategy {
                Strategy::Layer(strategy_layer) => {
                    let new_options = strategy_layer.prune_from(board, &options);
                    if !new_options.is_empty() {
                        options = new_options
                    }
                }
                Strategy::Decision(strategy_decider) => {
                    if let Some(choice) = strategy_decider.choose(board, &options) {
                        // Short circuit!
                        return vec![choice];
                    }
                }
            }
        }

        options
    }
}

impl Connect4AI for StrategyStack {
    fn play(&self, board: &Board) -> Option<usize> {
        let moves = self.evaluate_options(board);
        moves.choose(&mut self.rng.borrow_mut()).copied()
    }
}

impl std::fmt::Display for StrategyStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StrategyStack(")?;
        for (i, strategy) in self.strategies.iter().enumerate() {
            if i > 0 {
                write!(f, " => ")?;
            }
            write!(f, "{}", strategy.name())?;
        }
        write!(f, ")")
    }
}

pub enum Strategy {
    Layer(Box<dyn StrategyLayer>),
    Decision(Box<dyn StrategyDecider>),
}

impl Strategy {
    pub fn name(&self) -> &'static str {
        match self {
            Strategy::Layer(layer) => layer.name(),
            Strategy::Decision(decider) => decider.name(),
        }
    }
}

pub trait StrategyDecider {
    fn choose(&self, board: &Board, options: &[usize]) -> Option<usize>;
    fn name(&self) -> &'static str;
}

pub trait StrategyLayer {
    fn prune_from(&self, board: &Board, options: &[usize]) -> Vec<usize>;
    fn name(&self) -> &'static str;
}

pub struct TriesToWin {
    piece: Piece,
}

impl TriesToWin {
    pub fn new(piece: Piece) -> Self {
        TriesToWin { piece }
    }
}

impl StrategyDecider for TriesToWin {
    fn choose(&self, board: &Board, options: &[usize]) -> Option<usize> {
        for col in options {
            // If we could win, add it.
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            if test_board.has_winner() == Some(self.piece) {
                return Some(*col);
            }
            // If we would lose, add it to block
            let mut test_board = *board;
            test_board.place(*col, self.piece.opponent());
            if test_board.has_winner() == Some(self.piece.opponent()) {
                return Some(*col);
            }
        }
        None
    }

    fn name(&self) -> &'static str {
        "TriesToWin"
    }
}

pub struct Setup {
    piece: Piece,
}

impl Setup {
    pub fn new(piece: Piece) -> Self {
        Setup { piece }
    }
}

impl StrategyDecider for Setup {
    fn choose(&self, board: &Board, options: &[usize]) -> Option<usize> {
        for col in options {
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            if test_board.has_winner() == Some(self.piece) {
                return Some(*col);
            }
            if !test_board.winning_moves(self.piece).is_empty() {
                return Some(*col);
            }
        }
        None
    }

    fn name(&self) -> &'static str {
        "Setup"
    }
}

pub struct ThreeInARow {
    piece: Piece,
}

impl ThreeInARow {
    pub fn new(piece: Piece) -> Self {
        ThreeInARow { piece }
    }
}

impl StrategyLayer for ThreeInARow {
    fn prune_from(&self, board: &Board, options: &[usize]) -> Vec<usize> {
        let mut best = 0;
        let mut best_moves = vec![];

        for col in options {
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            if test_board.has_winner() == Some(self.piece) {
                return vec![*col];
            }
            let score = test_board.count_winning_opportunities(self.piece);
            if score > best {
                best = score;
                best_moves.clear();
                best_moves.push(*col);
            } else if score == best {
                best_moves.push(*col);
            }
        }

        assert!(!best_moves.is_empty());
        best_moves
    }

    fn name(&self) -> &'static str {
        "ThreeInARow"
    }
}

/// Strategy that avoids placing pieces in columns that would allow the opponent to win on their next turn.
pub struct AvoidTraps {
    piece: Piece,
}

impl AvoidTraps {
    pub fn new(piece: Piece) -> Self {
        AvoidTraps { piece }
    }
}

impl StrategyLayer for AvoidTraps {
    fn prune_from(&self, board: &Board, options: &[usize]) -> Vec<usize> {
        // Disqualify columns that would allow the opponent to win on their next turn
        let mut allowed = Vec::with_capacity(options.len());

        for col in options {
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            // If this move wins, short-circuit
            if test_board.has_winner() == Some(self.piece) {
                allowed.push(*col);
                continue;
            }
            // No good if the opponent has a winning opportunity
            if !test_board.winning_moves(self.piece.opponent()).is_empty() {
                continue;
            }
            allowed.push(*col);
        }

        allowed
    }

    fn name(&self) -> &'static str {
        "AvoidTraps"
    }
}

/// Strategy that avoids placing anywhere the other player gets more than one three-in-a-row.
pub struct AvoidInescapableTraps {
    piece: Piece,
}

impl AvoidInescapableTraps {
    pub fn new(piece: Piece) -> Self {
        AvoidInescapableTraps { piece }
    }
}

impl StrategyLayer for AvoidInescapableTraps {
    fn prune_from(&self, board: &Board, options: &[usize]) -> Vec<usize> {
        // Disqualify columns that would allow the opponent to win on their next turn
        let mut allowed = Vec::with_capacity(options.len());

        'candidate_loop: for col in options {
            let mut test_board = *board;
            test_board.place(*col, self.piece);
            // If this move wins, short-circuit
            if test_board.has_winner() == Some(self.piece) {
                allowed.push(*col);
                continue;
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

        allowed
    }

    fn name(&self) -> &'static str {
        "AvoidInescapableTraps"
    }
}
