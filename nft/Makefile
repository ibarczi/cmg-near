.ONESHELL:
.PHONY: lint test

clean:
	@cargo clean

lint:
	@cargo clippy

test:
	@cargo test -- --nocapture

coverage:
	@cargo tarpaulin --target-dir target-tarpaulin
