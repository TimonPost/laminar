tester:
	cargo build --release
	mv target/release/laminar-tester /usr/local/bin/laminar-tester
	chmod ugo+x /usr/local/bin/laminar-tester
