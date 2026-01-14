.PHONY: build lint test clean release docs-serve bench check-file-sizes

# Default target
all: build

# Build debug binary
build:
	cargo build

# Build release binary
release:
	cargo build --release

# Check file sizes
check-file-sizes:
	@echo "Checking file sizes..."
	@find src -name "*.rs" -exec sh -c 'lines=$$(wc -l < "$$1"); if [ $$lines -gt 500 ]; then echo "FAIL: $$1 has $$lines lines (max 500)"; exit 1; elif [ $$lines -gt 300 ]; then echo "WARN: $$1 has $$lines lines (target 300)"; fi' _ {} \; || exit 1
	@echo "PASS: All files under 500 lines"

# Run clippy linter
lint: check-file-sizes
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
