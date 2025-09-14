mod board;
mod search_for_win;
mod strategy;
mod strategy_cache;

use anyhow::{Context, Result};
use board::{Board, COLUMNS, Piece};
use clap::Parser;
use console::{Key, Term};
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::Write;
use std::{
    thread,
    time::{Duration, Instant},
};
use strategy::{Setup, StrategyLayer, TriesToWin};

use crate::board::ROWS;
use crate::search_for_win::SearchForWinCache;
use crate::strategy::{
    AvoidInescapableTraps, AvoidTraps, Connect4AI, SearchForWin, Strategy, StrategyDecider,
    StrategyStack, ThreeInARow,
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

fn game(red: &dyn Connect4AI, yellow: &dyn Connect4AI) -> Option<Board> {
    let mut board = Board::new();
    loop {
        // Red plays, then yellow.
        // If there's a winner or no moves left, leave
        if board.has_winner().is_some() || board.valid_moves().is_empty() {
            break;
        }
        let col = red.play(&board)?;
        board.with_place(col, Piece::Red);

        if board.has_winner().is_some() || board.valid_moves().is_empty() {
            break;
        }
        let col = yellow.play(&board)?;
        board.with_place(col, Piece::Yellow);
    }
    Some(board)
}

fn simulate_games(
    red: &dyn Connect4AI,
    yellow: &dyn Connect4AI,
    games: usize,
) -> Result<(usize, usize, usize)> {
    let mut red_wins = 0;
    let mut yellow_wins = 0;
    let mut ties = 0;

    println!("Running with strategies:\nRed:    {red}\nYellow: {yellow}",);

    let pb = ProgressBar::new(games as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{eta_precise} => {elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        )
        .unwrap(),
    );
    pb.set_message("Simulating games...");

    for _ in 0..games {
        let result = game(red, yellow).unwrap();

        match result.has_winner() {
            Some(Piece::Red) => red_wins += 1,
            Some(Piece::Yellow) => yellow_wins += 1,
            Some(_) => panic!("Unexpected winner"),
            None => ties += 1,
        }

        pb.inc(1);
    }
    pb.finish_and_clear();

    Ok((red_wins, yellow_wins, ties))
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
    let ai = build_strategy_stack(Piece::Yellow, &term)?;

    // Get a move
    // Get the AI response
    // Redraw the board
    // Is there a winner?
    // Repeat

    term.hide_cursor()?;
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
                    Key::Char('p') => {
                        term.clear_line()?;
                        term.clear_last_lines(ROWS + 2)?;
                        writeln!(term, "{}", &board.short_string())?;
                        write!(term, "\n{}\n", board)?;
                        continue 'selection;
                    }
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
        board.with_place(selection, Piece::Red);

        // Update the board display
        term.clear_line()?;
        term.clear_last_lines(ROWS + 2)?;
        write!(term, "\n{}\n\n", board)?;

        // Is the game over?
        if let Some(winner) = board.has_winner() {
            match winner {
                Piece::Red => {
                    writeln!(term, "Red wins after {} moves.", board.num_pieces_played())?
                }
                Piece::Yellow => writeln!(
                    term,
                    "Yellow wins after {} moves.",
                    board.num_pieces_played()
                )?,
                Piece::Empty => unreachable!(),
            }
            term.show_cursor()?;
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
        board.with_place(ai_move?, Piece::Yellow);

        // Update the board display
        term.clear_line()?;
        term.clear_last_lines(ROWS + 2)?;
        writeln!(term, "\n{}", board)?;

        // Is the game over?
        if let Some(winner) = board.has_winner() {
            match winner {
                Piece::Red => {
                    writeln!(term, "Red wins after {} moves.", board.num_pieces_played())?
                }
                Piece::Yellow => writeln!(
                    term,
                    "Yellow wins after {} moves.",
                    board.num_pieces_played()
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
                Option::Layer(x) => write!(f, "Filter Layer: {}", x.name()),
                Option::Decider(x) => write!(f, "Decider: {}", x.name()),
            }
        }
    }

    loop {
        let strategies: Vec<Option> = vec![
            Option::Done,
            Option::Decider(Box::new(SearchForWin::new(piece, 3))),
            Option::Decider(Box::new(SearchForWinCache::new(piece, 6))),
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
            Option::Layer(strat) => stack.push(Strategy::Layer(strat)),
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

    if use_cache {
        // Let's use caching for red and yellow strategies so they run faster!
        let red = Box::new(StrategyCache::new(build_strategy_stack(Piece::Red, &term)?));
        let yellow = Box::new(StrategyCache::new(build_strategy_stack(
            Piece::Yellow,
            &term,
        )?));

        let start = Instant::now();
        let (red_wins, yellow_wins, ties) =
            simulate_games(red.as_ref(), yellow.as_ref(), iterations)?;
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
            "Yellow wins: {:.2}%",
            yellow_wins as f64 / iterations as f64 * 100.0
        );
        println!("Ties:      {:.2}%", ties as f64 / iterations as f64 * 100.0);

        let red_cache_stats = red.cache_stats();
        let yellow_cache_stats = yellow.cache_stats();

        println!("Red cache:{}", &red_cache_stats);
        println!("Yellow cache:{}", &yellow_cache_stats);

        let cache_stats = red_cache_stats + yellow_cache_stats;
        println!("Overall cache stats:{}", &cache_stats);
    } else {
        let red = Box::new(build_strategy_stack(Piece::Red, &term)?);
        let yellow = Box::new(build_strategy_stack(Piece::Yellow, &term)?);

        let start = Instant::now();
        let (red_wins, yellow_wins, ties) =
            simulate_games(red.as_ref(), yellow.as_ref(), iterations)?;
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
            "Yellow wins: {:.2}%",
            yellow_wins as f64 / iterations as f64 * 100.0
        );
        println!("Ties:      {:.2}%", ties as f64 / iterations as f64 * 100.0);
    }

    Ok(())
}
