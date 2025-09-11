use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{board::Board, strategy::Strategy};

pub struct StrategyCache {
    strategy: Rc<dyn Strategy>,
    cache: RefCell<HashMap<Board, Option<usize>>>,
}

impl StrategyCache {
    pub fn new(strategy: Rc<dyn Strategy>) -> Self {
        Self {
            strategy,
            cache: RefCell::new(HashMap::new()),
        }
    }
}

impl std::fmt::Display for StrategyCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StrategyCache({})", self.strategy)
    }
}

impl Strategy for StrategyCache {
    fn play(&self, board: &Board) -> Option<usize> {
        // See if we have this cached
        if let Some(result) = self.cache.borrow().get(board) {
            *result
        } else {
            let result = self.strategy.play(board);
            self.cache.borrow_mut().insert(*board, result);
            result
        }
    }

    fn select_from(&self, _: &crate::board::Board, _: &[usize]) -> Option<usize> {
        todo!(
            "Unsure how to optimially cache when options are limited. Maybe Strategy needs to split into two traits."
        )
    }
}
