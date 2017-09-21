release:
	cargo build --release
	strip target/release/notedigest

doc:
	cargo rustdoc -- --no-defaults --enable-commonmark --passes "collapse-docs" --passes "unindent-comments" -Z unstable-options
