init:
	rustup component add rustfmt

fmt:
	cargo fmt --verbose

debug:
	cargo run -- -d


