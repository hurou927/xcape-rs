init:
	rustup component add rustfmt

fmt:
	cargo fmt --verbose

debug:
	cargo run -- -d -e 64=65 #alt=space


