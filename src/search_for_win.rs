use std::{cell::RefCell, collections::HashMap};

use crate::{
    board::{Board, Piece},
    strategy::StrategyDecider,
    strategy_cache::StrategyCacheStats,
};

struct SearchForWinCacheEntry {
    /// Store the depth we used when we calculated. If we arrive at this entry and don't know the result,
    /// but are willing to search deeper, we should do so.
    depth_searched_at: usize,
    /// Is this a good board for us?? None if we do not know because we bottomed out.
    forced_win: Option<bool>,
}

/// Strategy that searches for an unstoppable move with a given depth, but also
/// uses a cache so it runs in a reasonable time.
pub struct SearchForWinCache {
    piece: Piece,
    depth: usize,
    cache: RefCell<HashMap<Board, SearchForWinCacheEntry>>,
    stats: RefCell<StrategyCacheStats>,
}

impl SearchForWinCache {
    pub fn new(piece: Piece, depth: usize) -> Self {
        Self {
            piece,
            depth,
            cache: RefCell::new(HashMap::new()),
            stats: RefCell::new(StrategyCacheStats::default()),
        }
    }

    #[allow(unused)]
    pub fn get_stats(&self) -> StrategyCacheStats {
        let mut partial = *self.stats.borrow();
        partial.entries = self.cache.borrow().len();
        partial
    }

    /// Same scemantics as the other SearchForWin
    fn has_guaranteed_win(&self, prior: &Board, depth: usize, move_to_test: usize) -> Option<bool> {
        // This searches vertically... it might be faster to search horizontally
        // todo:: consider using a stack here instead and get rid of recursion

        assert!(prior.next_player() == self.piece); // make sure I don't fuck it up
        let board = prior.place(move_to_test, self.piece);

        // ------------------------------------------------------------
        // For these two, I'm guessing that they're
        // faster than a hashmap lookup.

        // If we've won, we've won.
        if board.has_winner() == Some(self.piece) {
            return Some(true);
        }

        // Otherwise, if this is our search depth, we can't guarantee a win
        if depth == 0 {
            // We don't know the result.
            return None;
        }

        // ------------------------------------------------------------

        // Here's where the magic is:

        // First, the cache lookup
        if let Some(entry) = self.cache.borrow().get(&board) {
            self.stats.borrow_mut().hits += 1;
            // Ok, first let's check if we found a solution:
            if entry.forced_win == Some(true) {
                // Yay! we would win!
                return Some(true);
            }
            if entry.forced_win == Some(false) {
                // Oh no... we would not win.
                return Some(false);
            }
            debug_assert!(entry.forced_win.is_none());
            // Ok, so we don't know the result yet.
            // We'll need to search deeper. Are we allowed to?
            if entry.depth_searched_at >= depth {
                // Ok... we know we won't make any progress here. Short circuit!
                return None;
            }
            // Otherwise, allow us to fall through!
        } else {
            self.stats.borrow_mut().misses += 1;
        }

        // Look at all of the possible ways the enemy could respond
        let enemy_moves = board.all_future_boards(self.piece.opponent());

        for enemy_board in enemy_moves {
            // If the enemy has won, we've obviously lost!
            if enemy_board.has_winner() == Some(self.piece.opponent()) {
                return Some(false);
            }
            let responses = enemy_board.valid_moves();
            let mut found_winning_response = false;
            for col in responses {
                let res = self.has_guaranteed_win(&enemy_board, depth - 1, col);
                // If we hit the search depth at any point, we need to abort.
                if res.is_none() {
                    // Let's cache that we couldn't quite find it.
                    let old = self.cache.borrow_mut().insert(
                        board,
                        SearchForWinCacheEntry {
                            depth_searched_at: depth,
                            forced_win: None,
                        },
                    );
                    if let Some(old) = old {
                        // Let's double check that we didn't already know the answer and that the depth was lower.
                        assert!(old.depth_searched_at < depth);
                        assert!(old.forced_win.is_none());
                    }
                    return None;
                }

                if res == Some(true) {
                    found_winning_response = true;
                    break;
                }
            }

            // So if we did not find a winning response, the enemy has a way out.
            if !found_winning_response {
                // Cache this value as well.
                self.cache.borrow_mut().insert(
                    board,
                    SearchForWinCacheEntry {
                        depth_searched_at: 0, // The depth doesn't matter here, we know the opponent has a way out.
                        forced_win: Some(false),
                    },
                );
                return Some(false);
            }
        }

        // If we are here, the following are true:
        // 1. At no point we bottomed out. This means forced win is possible in depth-1 moves.
        // 2. The opponent cannot win if we play perfectly for the next depth moves.
        // This means we 100% win in the next `depth` moves if we play `move_to_test`.
        // Cache that and return.
        self.cache.borrow_mut().insert(
            board,
            SearchForWinCacheEntry {
                depth_searched_at: 0, // The depth doesn't matter here, we know we're winning and don't care how long it takes.
                forced_win: Some(true),
            },
        );

        Some(true)
    }
}

impl StrategyDecider for SearchForWinCache {
    fn choose(&self, board: &Board, options: &[usize]) -> Option<usize> {
        for col in options {
            if self.has_guaranteed_win(board, self.depth, *col) == Some(true) {
                return Some(*col);
            }
        }
        None
    }

    fn name(&self) -> &'static str {
        "SearchForWinCache"
    }
}
