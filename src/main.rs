mod board;
mod strategy;

use anyhow::{Context, Result};
use board::{Board, COLUMNS, Piece};
use clap::Parser;
use console::Key;
use inquire::Select;
use scopeguard::defer;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::{
    thread,
    time::{Duration, Instant},
};
use strategy::{RandomStrategy, Setup, Strategy, TriesToWin};

use crate::board::ROWS;
use crate::strategy::ThreeInARow;

#[derive(Parser)]
#[command(name = "connect-4")]
#[command(about = "A Connect 4 game with AI strategies")]
struct Cli {
    /// Run AI simulation mode instead of interactive game
    #[arg(short, long)]
    sim: bool,
}

type S = Rc<dyn Strategy>;

fn game(red: &S, blue: &S) -> Option<Board> {
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

fn simulate_games(red: S, blue: S, games: usize) -> Result<(usize, usize, usize)> {
    let mut red_wins = 0;
    let mut blue_wins = 0;
    let mut ties = 0;

    for _ in 0..games {
        let result = game(&red, &blue).context("Failed to play game")?;

        match result.has_winner() {
            Some(Piece::Red) => red_wins += 1,
            Some(Piece::Blue) => blue_wins += 1,
            Some(_) => panic!("Unexpected winner"),
            None => ties += 1,
        }
    }

    Ok((red_wins, blue_wins, ties))
}

fn play_interactive() -> Result<()> {
    // Welcome:
    //
    // [ ] [ ] [ ] [ ] [ ] [ ] [ ]
    // [ ] [ ] [ ] [ ] [ ] [ ] [ ]
    // [ ] [ ] [ ] [ ] [ ] [ ] [ ]
    // [ ] [ ] [B] [ ] [ ] [ ] [ ]
    // [ ] [ ] [R] [ ] [ ] [ ] [ ]
    // [R] [ ] [B] [ ] [ ] [ ] [ ]
    //      ^
    // Pick your move
    //
    let mut term = console::Term::stdout();
    let mut board = Board::new();
    let mut selection = COLUMNS / 2;
    let ai = select_strategy(Piece::Blue)?;

    // Get a move
    // Get the AI response
    // Redraw the board
    // Is there a winner?
    // Repeat

    term.hide_cursor()?;
    let dropterm = term.clone();
    defer! {
        let _ = dropterm.show_cursor();
    };
    writeln!(term, "You are Red. You are playing against {}", ai)?;
    term.write_line("")?;

    writeln!(term, "{}", board)?;

    loop {
        'selection: loop {
            // Draw the selection
            writeln!(term, " {}", "    ".repeat(selection) + "^")?;
            write!(term, "Make your move")?;
            'key: loop {
                let key = term.read_key()?;
                match key {
                    Key::Unknown => anyhow::bail!("Problem"),
                    Key::Char('q') => anyhow::bail!("Quit!"),
                    Key::ArrowLeft | Key::Char('a') => {
                        selection = selection.saturating_sub(1);
                        break 'key;
                    }
                    Key::ArrowRight | Key::Char('d') => {
                        if selection < COLUMNS - 1 {
                            selection += 1;
                        }
                        break 'key;
                    }
                    Key::Enter => {
                        break 'selection;
                    }
                    _ => {}
                }
            }
            term.clear_last_lines(1)?;
        }

        // Make the move
        board.place(selection, Piece::Red);

        // Update the board display
        term.clear_line()?;
        term.clear_last_lines(ROWS + 2)?;
        write!(term, "\n{}\n\n", board)?;

        // Is the game over?
        if let Some(winner) = board.has_winner() {
            match winner {
                Piece::Red => writeln!(term, "Red wins.")?,
                Piece::Blue => writeln!(term, "Blue wins.")?,
                Piece::Empty => unreachable!(),
            }
            return Ok(());
        }

        if board.valid_moves().is_empty() {
            writeln!(term, "Tie.")?;
            return Ok(());
        }

        write!(term, "AI is thinking...")?;

        thread::sleep(Duration::from_millis(500));
        // Make the AI move
        let ai_move = ai.play(&board).context("Failed to get AI move");
        board.place(ai_move?, Piece::Blue);

        // Update the board display
        term.clear_line()?;
        term.clear_last_lines(ROWS + 2)?;
        writeln!(term, "\n{}", board)?;

        // Is the game over?
        if let Some(winner) = board.has_winner() {
            match winner {
                Piece::Red => writeln!(
                    term,
                    "Red wins after {} moves.",
                    board.get_num_pieces_played()
                )?,
                Piece::Blue => writeln!(
                    term,
                    "Blue wins after {} moves.",
                    board.get_num_pieces_played()
                )?,
                Piece::Empty => unreachable!(),
            }
            term.show_cursor()?;
            return Ok(());
        }

        if board.valid_moves().is_empty() {
            writeln!(term, "Tie.")?;
            term.show_cursor()?;
            return Ok(());
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.sim {
        // Run AI vs AI simulation
        return run_simulation();
    }

    // Default behavior: interactive mode
    play_interactive()
}

fn select_strategy(piece: Piece) -> Result<S> {
    let strategies: Vec<S> = vec![
        Rc::new(TriesToWin::new(
            ThreeInARow::new(
                Setup::new(RandomStrategy::default(), piece),
                piece,
                RefCell::new(rand::rng()),
            ),
            piece,
        )),
        Rc::new(TriesToWin::new(
            ThreeInARow::new(RandomStrategy::default(), piece, RefCell::new(rand::rng())),
            piece,
        )),
        Rc::new(TriesToWin::new(
            Setup::new(RandomStrategy::default(), piece),
            piece,
        )),
        Rc::new(TriesToWin::new(RandomStrategy::default(), piece)),
        Rc::new(RandomStrategy::default()),
    ];
    Ok(Select::new(
        &format!("Select a strategy for {}", piece.name()),
        strategies,
    )
    .prompt()?)
}

fn run_simulation() -> Result<()> {
    let red = select_strategy(Piece::Red)?;
    let blue = select_strategy(Piece::Blue)?;

    const GAMES: usize = if cfg!(debug_assertions) { 100 } else { 100_000 };

    let start = Instant::now();
    let (red_wins, blue_wins, ties) = simulate_games(red, blue, GAMES)?;
    let duration = start.elapsed();

    println!(
        "Result from {} games (took {}ms):",
        GAMES,
        duration.as_millis()
    );

    println!("Red wins:  {:.2}%", red_wins as f64 / GAMES as f64 * 100.0);
    println!("Blue wins: {:.2}%", blue_wins as f64 / GAMES as f64 * 100.0);
    println!("Ties:      {:.2}%", ties as f64 / GAMES as f64 * 100.0);

    Ok(())
}
