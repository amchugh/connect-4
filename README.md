# Connect 4 🔴🔵

A fast Connect 4 game written in Rust with AI strategies and an interactive terminal interface.

## Features

- 🎮 **Interactive Mode**: Play against AI with intuitive arrow-key column selection
- 🤖 **Smart AI**: Multiple AI strategies including winning/blocking moves and setup tactics
- ⚡ **High Performance**: Runs 100,000 games in ~3.5 seconds (release mode)
- 🎨 **Beautiful Terminal UI**: Colorized game board with emoji feedback
- 📊 **AI Battle Simulation**: Watch different strategies compete

## Installation

### Pre-built Binary (macOS M-series)
Download the latest release from the [Releases page](../../releases).

### From Source
```bash
git clone https://github.com/YOUR_USERNAME/connect-4.git
cd connect-4
cargo install --path .
```

## Usage

### Interactive Mode (Human vs AI)
```bash
connect-4 --interactive
# or
connect-4 -i
```

Use arrow keys to select a column, press Enter to drop your piece. You play as Red, AI plays as Blue.

### AI Simulation Mode (Default)
```bash
connect-4
```

Runs AI vs AI battles to test strategy effectiveness:
- Debug builds: 100 games
- Release builds: 100,000 games

### Help
```bash
connect-4 --help
```

## AI Strategies

The game implements a composable strategy system:

- **RandomStrategy**: Plays random valid moves
- **TriesToWin**: Prioritizes winning moves, then blocking opponent wins, falls back to inner strategy
- **Setup**: Looks ahead to create winning opportunities

Example AI composition:
```rust
let smart_ai = Setup::new(
    TriesToWin::new(RandomStrategy::default(), Piece::Red),
    Piece::Red,
);
```

## Development

```bash
# Run in debug mode (100 games)
cargo run

# Run optimized (100,000 games)
cargo run --release

# Interactive mode
cargo run -- --interactive

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy
```

## Architecture

- `src/board.rs`: Game board logic and win detection
- `src/strategy.rs`: AI strategy implementations
- `src/main.rs`: Game loop and CLI interface

## License

MIT License - see LICENSE file for details.
