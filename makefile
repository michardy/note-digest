release:
	cargo build --release
	strip target/release/notedigest
