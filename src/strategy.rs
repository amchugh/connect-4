use crate::board::{Board, Piece};
use rand::seq::IndexedRandom;
use std::cell::RefCell;

pub trait Connect4AI: std::fmt::Display {
    fn play(&self, board: &Board) -> Option<usize>;
}

pub struct StrategyStack {
    strategies: Vec<Box<dyn StrategyLayer>>,
    rng: RefCell<rand::rngs::ThreadRng>,
}

impl StrategyStack {
    pub fn new(strategies: Vec<Box<dyn StrategyLayer>>) -> Self {
        StrategyStack {
            strategies,
            rng: RefCell::new(rand::rngs::ThreadRng::default()),
        }
    }

    pub fn evaluate_options(&self, board: &Board) -> Vec<usize> {
        let mut options = board.valid_moves();
        assert!(!options.is_empty());

        for strategy in &self.strategies {
            options = strategy.prune_from(board, &options);
            assert!(
                !options.is_empty(),
                "Strategy {} gave no options",
                strategy.name()
            );
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

pub trait StrategyLayer {
    fn prune_from(&self, board: &Board, options: &[usize]) -> Vec<usize>;
    fn name(&self) -> &'static str;
}

#[derive(Clone, Default)]
pub struct RandomStrategy {}

impl std::fmt::Display for RandomStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RandomStrategy")
    }
}

impl StrategyLayer for RandomStrategy {
    fn prune_from(&self, _: &Board, options: &[usize]) -> Vec<usize> {
        // Does nothing!
        Vec::from(options)
    }

    fn name(&self) -> &'static str {
        "RandomStrategy"
    }
}

pub struct TriesToWin {
    piece: Piece,
}

impl TriesToWin {
    pub fn new(piece: Piece) -> Self {
        TriesToWin { piece }
    }
}

impl StrategyLayer for TriesToWin {
    fn prune_from(&self, board: &Board, options: &[usize]) -> Vec<usize> {
        let winning_moves: Vec<usize> = options
            .iter()
            .copied()
            .filter(|col| {
                // If we could win, add it.
                let mut test_board = *board;
                test_board.place(*col, self.piece);
                if test_board.has_winner() == Some(self.piece) {
                    return true;
                }
                // If we would lose, add it to block
                let mut test_board = *board;
                test_board.place(*col, self.piece.opponent());
                if test_board.has_winner() == Some(self.piece.opponent()) {
                    return true;
                }
                false
            })
            .collect();

        if winning_moves.is_empty() {
            Vec::from(options)
        } else {
            winning_moves
        }
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

impl StrategyLayer for Setup {
    fn prune_from(&self, board: &Board, options: &[usize]) -> Vec<usize> {
        let setups: Vec<usize> = options
            .iter()
            .copied()
            .filter(|col| {
                let mut test_board = *board;
                test_board.place(*col, self.piece);
                if test_board.has_winner() == Some(self.piece) {
                    return true;
                }
                if !test_board.winning_moves(self.piece).is_empty() {
                    return true;
                }
                false
            })
            .collect();

        if setups.is_empty() {
            Vec::from(options)
        } else {
            setups
        }
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

        // If any move loses, we know we're going to lose :(
        // So just pick the first move that we were given
        if allowed.is_empty() {
            Vec::from(options)
        } else {
            allowed
        }
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

        // If any move loses, we know we're going to lose :(
        // So just pick the first move that we were given
        if allowed.is_empty() {
            Vec::from(options)
        } else {
            allowed
        }
    }

    fn name(&self) -> &'static str {
        "AvoidInescapableTraps"
    }
}
