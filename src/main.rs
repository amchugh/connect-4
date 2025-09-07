mod board;
mod strategy;

use anyhow::{Context, Result};
use board::{Board, Piece};
use std::time::Instant;
use strategy::{RandomStrategy, Setup, Strategy, TriesToWin};

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

fn simulate_games<R: Strategy, B: Strategy>(
    red: R,
    blue: B,
    games: usize,
) -> Result<(usize, usize, usize)> {
    let mut red_wins = 0;
    let mut blue_wins = 0;
    let mut ties = 0;

    for _ in 0..games {
        let result = game(red.clone(), blue.clone()).context("Failed to play game")?;

        match result.has_winner() {
            Some(Piece::Red) => red_wins += 1,
            Some(Piece::Blue) => blue_wins += 1,
            Some(_) => panic!("Unexpected winner"),
            None => ties += 1,
        }
    }

    Ok((red_wins, blue_wins, ties))
}

fn main() -> Result<()> {
    let red = Setup::new(
        TriesToWin::new(RandomStrategy::default(), Piece::Red),
        Piece::Red,
    );
    let blue = RandomStrategy::default();

    const GAMES: usize = if cfg!(debug_assertions) { 100 } else { 100_000 };

    let start = Instant::now();
    let (red_wins, blue_wins, ties) = simulate_games(red, blue, GAMES)?;
    let duration = start.elapsed();

    println!(
        "Result from {} games (took {}ms):",
        GAMES,
        duration.as_millis()
    );

    println!("Red wins: {:.2}%", red_wins as f64 / GAMES as f64 * 100.0);
    println!("Blue wins: {:.2}%", blue_wins as f64 / GAMES as f64 * 100.0);
    println!("Ties: {:.2}%", ties as f64 / GAMES as f64 * 100.0);

    Ok(())
}
