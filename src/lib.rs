pub mod board;
// Re-export so it can be used as `use connect4::Board` instead of `use connect4::board::Board`
pub use board::{Board, Piece};
