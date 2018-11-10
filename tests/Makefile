tester:
	cd ..; cargo build --release
	

docker:
	cd ..; CROSS_COMPILE=x86_64-linux-musl- cargo build --target=x86_64-unknown-linux-musl --release --features tester
	cd ..; mv target/x86_64-unknown-linux-musl/release/laminar-tester scripts/laminar-tester
	docker build -t zerosubnet/laminar .
	docker push zerosubnet/laminar