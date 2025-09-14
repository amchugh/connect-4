
connect4-macos-arm64: .FORCE
	cargo build --release --target aarch64-apple-darwin
	cp target/aarch64-apple-darwin/release/connect4 connect4-macos-arm64

.PHONY: original-board-benchmark
original-board-benchmark:
	git stash push
	git switch original-board-benchmark
	cargo bench --bench board_bench -- --save-baseline original-board
	git switch -
	git stash pop || echo "Stash pop failed. This is expected if there were no changes to stash"

.PHONY: FORCE
.FORCE:
