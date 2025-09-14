use connect4::Board;
use criterion::{Criterion, criterion_group, criterion_main};

const TEST_BOARDS: [&str; 7] = [
    "!////RR B/BB R",
    "!///  RB/ RRR/ BBBR",
    "!///   B/  RRR/B BRB",
    "!/B  BB/R  RR/B BBR/B RRR R/B BRB R",
    "!B BRB R/B RBB R/R BRR B/B BBR R/B RRR R/B BRB R",
    "!B BRB R/B RBB R/R BRR B/BRBBR R/BBRRR R/BRBRB R",
    "!/////",
];

fn bench_basic_operations(c: &mut Criterion) {
    // Create our test boards. We don't care how long
    // this operation takes as it's only used for testing.
    let boards: Vec<Board> = TEST_BOARDS.into_iter().map(Board::from).collect();

    c.bench_function("is terminal", |b| {
        b.iter(|| {
            for board in &boards {
                board.is_terminal();
            }
        })
    });

    c.bench_function("has winner", |b| {
        b.iter(|| {
            for board in &boards {
                board.has_winner();
            }
        })
    });

    c.bench_function("next states", |b| {
        b.iter(|| {
            for board in &boards {
                board.next_states();
            }
        })
    });

    c.bench_function("get next player", |b| {
        b.iter(|| {
            for board in &boards {
                board.next_player();
            }
        })
    });

    // For each board, let's find the non-terminal states and come up with a list of all possible moves
    let moves: Vec<_> = boards
        .iter()
        .filter_map(|board| {
            if board.is_terminal() {
                return None;
            }
            Some((board, board.valid_moves(), board.next_player()))
        })
        .collect();

    c.bench_function("place piece", |b| {
        b.iter(|| {
            for (board, moves, piece) in &moves {
                for m in moves {
                    board.place(*m, *piece);
                }
            }
        })
    });
}

fn board_graph_opertaions(c: &mut Criterion) {
    let boards: Vec<Board> = TEST_BOARDS.into_iter().map(Board::from).collect();

    c.bench_function("prior states", |b| {
        b.iter(|| {
            for board in &boards {
                board.prior_states();
            }
        })
    });
}

criterion_group!(benches, bench_basic_operations, board_graph_opertaions);

criterion_main!(benches);
