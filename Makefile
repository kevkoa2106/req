build: 
	cargo build --release
	cargo build --release --target x86_64-apple-darwin
	cargo build --release --target x86_64-pc-windows-gnu
	cargo zigbuild --release --target x86_64-unknown-linux-gnu
	cargo zigbuild --release --target aarch64-unknown-linux-gnu
	tar -czvf req-aarch64-apple-darwin.tar.gz -C target/release req
	tar -czvf req-x86_64-apple-darwin.tar.gz -C target/x86_64-apple-darwin/release/ req
	tar -czvf req-aarch64-unknown-linux-gnu.tar.gz -C target/aarch64-unknown-linux-gnu/release req
	tar -czvf req-x86_64-unknown-linux-gnu.tar.gz -C target/x86_64-unknown-linux-gnu/release req

sum: 
	shasum -a 256 req*