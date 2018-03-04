release:
	cargo build --release
	strip target/release/notedigest

doc:
	cargo rustdoc -- --document-private-items
