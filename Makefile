.PHONY: build lint test clean release docs-serve bench

# Default target
all: build

# Build debug binary
build:
	cargo build

# Build release binary
release:
	cargo build --release

# Run clippy linter
lint:
	cargo clippy -- -D warnings

# Run test suite
test: release
	./target/release/vhp test

# Run tests in verbose mode
test-verbose: release
	./target/release/vhp test -v

# Clean build artifacts
clean:
	cargo clean

# Run performance benchmarks
bench: release
	python3 run_benchmarks.py

# Serve documentation locally
docs-serve:
	cd docs && jekyll serve --livereload
