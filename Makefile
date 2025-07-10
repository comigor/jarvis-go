
# Makefile for Rust workspace
.PHONY: build run test lint fmt

# Build all crates in debug mode
build:
	cargo build --workspace

# Run the main web server (jarvis-web crate)
run:
	cargo run -p jarvis-web --release

# Run the full test suite
test:
	cargo test --workspace

# Lint with clippy and rustfmt
lint:
	cargo clippy --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all
