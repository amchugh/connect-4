mod board;
mod strategy;
mod strategy_cache;

use anyhow::{Context, Result};
use board::{Board, COLUMNS, Piece};
use clap::Parser;
use console::{Key, Term};
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};
use scopeguard::defer;
use std::io::Write;
use std::{
    thread,
    time::{Duration, Instant},
};
use strategy::{RandomStrategy, Setup, StrategyLayer, TriesToWin};

use crate::board::ROWS;
use crate::strategy::{
    AvoidInescapableTraps, AvoidTraps, Connect4AI, Strategy, StrategyDecider, StrategyStack,
    ThreeInARow,
};
use crate::strategy_cache::StrategyCache;

#[derive(Parser)]
#[command(name = "connect-4")]
#[command(about = "A Connect 4 game with AI strategies")]
#[command(version)]
struct Cli {
    /// Run AI simulation mode instead of interactive game
    #[arg(short, long)]
    sim: bool,

    /// How many iterations should be ran in a simulation
    /// Default: 100,000
    #[arg(short, long)]
    iterations: Option<usize>,

    /// Should we cache strategy decisions
    #[arg(short = 'c', long = "cache")]
    use_cache: bool,
}

fn game(red: &dyn Connect4AI, blue: &dyn Connect4AI) -> Option<Board> {
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

fn simulate_games(
    red: &dyn Connect4AI,
    blue: &dyn Connect4AI,
    games: usize,
) -> Result<(usize, usize, usize)> {
    let mut red_wins = 0;
    let mut blue_wins = 0;
    let mut ties = 0;

    println!("Running with strategies:\nRed:  {red}\nBlue: {blue}",);

    let pb = ProgressBar::new(games as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{eta_precise} => {elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap(),
    );
    pb.set_message("Simulating games...");

    for _ in 0..games {
        let result = game(red, blue).unwrap();

        match result.has_winner() {
            Some(Piece::Red) => red_wins += 1,
            Some(Piece::Blue) => blue_wins += 1,
            Some(_) => panic!("Unexpected winner"),
            None => ties += 1,
        }

        pb.inc(1);
    }
    pb.finish_and_clear();

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
    let ai = build_strategy_stack(Piece::Blue, &term)?;

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
        const GAMES: usize = if cfg!(debug_assertions) { 100 } else { 100_000 };
        let games = cli.iterations.unwrap_or(GAMES);
        return run_simulation(games, cli.use_cache);
    }

    // Default behavior: interactive mode
    play_interactive()
}

fn build_strategy_stack(piece: Piece, term: &Term) -> Result<StrategyStack> {
    let mut stack = vec![];

    term.write_line(&format!("Build a strategy stack for {}. Every layer in the stack filters the possible moves. The AI will pick randomly from possible moves at the end.", piece.name()))?;

    enum Option {
        Done,
        Layer(Box<dyn StrategyLayer>),
        Decider(Box<dyn StrategyDecider>),
    }

    impl std::fmt::Display for Option {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Option::Done => write!(f, "Done"),
                Option::Layer(x) => write!(f, "Decider: {}", x.name()),
                Option::Decider(x) => write!(f, "Filter Layer: {}", x.name()),
            }
        }
    }

    loop {
        let strategies: Vec<Option> = vec![
            Option::Done,
            Option::Layer(Box::new(AvoidInescapableTraps::new(piece))),
            Option::Layer(Box::new(AvoidTraps::new(piece))),
            Option::Layer(Box::new(ThreeInARow::new(piece))),
            Option::Decider(Box::new(Setup::new(piece))),
            Option::Decider(Box::new(TriesToWin::new(piece))),
        ];

        let choice = Select::new()
            .default(0)
            .with_prompt("Select a strategy")
            .items(&strategies)
            .interact_on(term)
            .unwrap();

        match strategies.into_iter().nth(choice).unwrap() {
            Option::Done => break,
            Option::Layer(strat) => stack.push(Strategy::layer(strat)),
            Option::Decider(strat) => stack.push(Strategy::Decision(strat)),
        }
    }

    // Clear the lines that we've added
    term.clear_last_lines(stack.len() + 2)?;

    let stack = StrategyStack::new(stack);
    Ok(stack)
}

fn run_simulation(iterations: usize, use_cache: bool) -> Result<()> {
    let term = console::Term::stdout();

    let red: Box<dyn Connect4AI>;
    let blue: Box<dyn Connect4AI>;

    if use_cache {
        // Let's use caching for red and blue strategies so they run faster!
        red = Box::new(StrategyCache::new(build_strategy_stack(Piece::Red, &term)?));
        blue = Box::new(StrategyCache::new(build_strategy_stack(
            Piece::Blue,
            &term,
        )?));
    } else {
        red = Box::new(build_strategy_stack(Piece::Red, &term)?);
        blue = Box::new(build_strategy_stack(Piece::Blue, &term)?);
    }

    let start = Instant::now();
    let (red_wins, blue_wins, ties) = simulate_games(red.as_ref(), blue.as_ref(), iterations)?;
    let duration = start.elapsed();

    println!(
        "Result from {} games (took {}ms):",
        iterations,
        duration.as_millis()
    );

    println!(
        "Red wins:  {:.2}%",
        red_wins as f64 / iterations as f64 * 100.0
    );
    println!(
        "Blue wins: {:.2}%",
        blue_wins as f64 / iterations as f64 * 100.0
    );
    println!("Ties:      {:.2}%", ties as f64 / iterations as f64 * 100.0);

    Ok(())
}
