# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Connect 4 game implemented in Rust. The project features:
- A complete Connect 4 game engine with board state management
- Multiple AI strategies including random play, winning/blocking moves, and setup moves
- Strategy pattern implementation with composable AI behaviors
- Performance testing that runs 100,000 games to compare strategies

## Development Commands

**Build and run:**
```bash
cargo run
```

**Check code (fast compile check):**
```bash
cargo check
```

**Build optimized release:**
```bash
cargo build --release
```

**Run tests:**
```bash
cargo test
```

**Format code:**
```bash
cargo fmt
```

**Lint code:**
```bash
cargo clippy
```

## Code Architecture

### Core Components

**Board (`struct Board`):**
- Manages game state as a 2D array of pieces
- Handles piece placement with gravity simulation
- Implements win detection for horizontal, vertical, and diagonal connections

**Strategy Pattern:**
- `trait Strategy` defines AI behavior interface
- `RandomStrategy` - plays random valid moves
- `TriesToWin<S>` - wrapper that prioritizes winning/blocking moves, falls back to inner strategy
- `Setup<S>` - looks ahead two moves to create winning setups

**Game Loop (`game` function):**
- Alternates between red and yellow players
- Handles win detection and game termination
- Returns final board state

### Key Design Patterns

The codebase uses composition over inheritance through strategy wrappers. For example:
```rust
let red = Setup::new(
    TriesToWin::new(RandomStrategy::default(), Piece::Red),
    Piece::Red,
);
```

This creates an AI that first tries setup moves, then winning/blocking moves, then random moves as fallback.

## Dependencies

- `anyhow` - Error handling with context
- `colorize` - Terminal color output for game display
- `rand` - Random number generation for AI strategies
