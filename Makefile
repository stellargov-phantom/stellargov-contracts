# Makefile — Soroban Smart Contract Build System
# Modeled after Phoenix Protocol Group standards

.PHONY: build test clean fmt check

# Default target
all: build test

# Build all contracts to WASM
build:
	@echo "🔨 Building all contracts..."
	cargo build --release --target wasm32-unknown-unknown
	@echo "✅ Build complete. WASM artifacts in target/wasm32-unknown-unknown/release/"

# Run all unit tests
test:
	@echo "🧪 Running test suite..."
	cargo test
	@echo "✅ All tests passed."

# Format all Rust source files
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all
	@echo "✅ Formatting complete."

# Run clippy linter
check:
	@echo "🔍 Running clippy linter..."
	cargo clippy --all-targets -- -D warnings
	@echo "✅ Clippy passed."

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean complete."

# Run full CI pipeline locally
ci: fmt check test build
	@echo "🎉 Full CI pipeline passed locally!"
