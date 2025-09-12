use std::{
    cell::RefCell,
    collections::HashMap,
    ops::Add,
    sync::{Arc, Mutex, RwLock},
};

use rand::seq::IndexedRandom;

use crate::{
    board::Board,
    strategy::{Connect4AI, StrategyStack},
};

type BoardCache = HashMap<Board, Vec<usize>>;

pub struct StrategyCacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: usize,
}

impl Add for StrategyCacheStats {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            hits: self.hits + other.hits,
            misses: self.misses + other.misses,
            entries: self.entries + other.entries,
        }
    }
}

impl std::fmt::Display for StrategyCacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(f, "Hits:    {:<10}", self.hits)?;
        writeln!(f, "Misses:  {:<10}", self.misses)?;
        writeln!(f, "Entries: {:<10}", self.entries)
    }
}

pub struct StrategyCache {
    stack: StrategyStack,
    cache: Arc<RwLock<BoardCache>>,
    rng: RefCell<rand::rngs::ThreadRng>,
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl StrategyCache {
    pub fn new(stack: StrategyStack) -> Self {
        Self {
            stack,
            cache: Arc::new(RwLock::new(HashMap::new())),
            rng: RefCell::new(rand::rng()),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    pub fn cache_stats(&self) -> StrategyCacheStats {
        let cache = self.cache.read().unwrap();
        StrategyCacheStats {
            hits: *self.hits.lock().unwrap(),
            misses: *self.misses.lock().unwrap(),
            entries: cache.len(),
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
        if let Some(result) = self.cache.read().unwrap().get(board) {
            *self.hits.lock().unwrap() += 1;
            result.choose(&mut self.rng.borrow_mut()).copied()
        } else {
            let result = self.stack.evaluate_options(board);
            let choice = result.choose(&mut self.rng.borrow_mut()).copied();
            self.cache.write().unwrap().insert(*board, result);
            *self.misses.lock().unwrap() += 1;
            choice
        }
    }
}
