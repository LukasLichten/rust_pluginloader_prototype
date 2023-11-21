all: plugin
	cargo run
	
plugin:
	cd test_plugin && cargo build --release
	cd second_plugin && cargo build --release
	cp target/release/libtest_plugin.so lib/
	cp target/release/libsecond_plugin.so lib/

release: plugin
	cargo build --release