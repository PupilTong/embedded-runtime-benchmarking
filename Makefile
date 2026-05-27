.PHONY: bench wasm wasm-threads check

bench:
	python3 scripts/bench.py

wasm:
	cargo build --release --target wasm32-wasip1

wasm-threads:
	RUSTFLAGS="--cfg wasip1_threads" cargo build --release --target wasm32-wasip1-threads

check:
	cargo check
	cargo check --target wasm32-wasip1
	RUSTFLAGS="--cfg wasip1_threads" cargo check --target wasm32-wasip1-threads
