build:
	sudo apt-get install pkg-config libssl-dev
	cargo b

final:
	cargo test

docs:
	cargo doc --no-deps