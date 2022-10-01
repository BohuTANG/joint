lint:
	cargo fmt --all
	cargo clippy --workspace --all-targets -- -D warnings

clean:
	cargo clean
