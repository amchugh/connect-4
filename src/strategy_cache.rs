use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, RwLock},
};

use rand::seq::IndexedRandom;

use crate::{
    board::Board,
    strategy::{Connect4AI, StrategyStack},
};

type BoardCache = HashMap<Board, Vec<usize>>;

pub struct StrategyCache {
    stack: StrategyStack,
    cache: RefCell<Arc<RwLock<BoardCache>>>,
    rng: RefCell<rand::rngs::ThreadRng>,
}

impl StrategyCache {
    pub fn new(stack: StrategyStack) -> Self {
        Self {
            stack,
            cache: RefCell::new(Arc::new(RwLock::new(HashMap::new()))),
            rng: RefCell::new(rand::rng()),
        }
    }
}

impl std::fmt::Display for StrategyCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CACHED({}))", self.stack)
    }
}

impl Connect4AI for StrategyCache {
    fn play(&self, board: &Board) -> Option<usize> {
        // See if we have this cached
        if let Some(result) = self.cache.borrow().read().unwrap().get(board) {
            result.choose(&mut self.rng.borrow_mut()).copied()
        } else {
            let result = self.stack.evaluate_options(board);
            let choice = result.choose(&mut self.rng.borrow_mut()).copied();
            self.cache
                .borrow_mut()
                .write()
                .unwrap()
                .insert(*board, result);
            choice
        }
    }
}
