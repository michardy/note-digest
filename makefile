release:
	cargo build --release
	strip target/release/notedigest

doc:
	cargo rustdoc -- --document-private-items

install:
	cp target/release/notedigest /bin/notedigest
