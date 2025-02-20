.PHONY: all run test fmt lint clean

all: test fmt lint

run:
	cargo run --release

test:
	cargo test --all-features

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings

clean:
	cargo clean