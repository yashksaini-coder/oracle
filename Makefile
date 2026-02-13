# Oracle - Rust Code Inspector
# Makefile for development workflow

.PHONY: all build release run clean check lint fmt test doc install help

# Default target
all: help

# Build debug version
build:
	@echo "🔨 Building debug..."
	cargo build

# Build optimized release
release:
	@echo "📦 Building release..."
	cargo build --release

# Run in debug mode
run:
	@echo "🚀 Running Oracle..."
	cargo run

# Run release version
run-release:
	@echo "🚀 Running Oracle (release)..."
	cargo run --release

# Run on a specific project
run-project:
	@echo "🔍 Analyzing project..."
	@if [ -z "$(PROJECT)" ]; then \
		echo "Usage: make run-project PROJECT=/path/to/rust/project"; \
		exit 1; \
	fi
	cargo run --release -- $(PROJECT)

# Clean build artifacts
clean:
	@echo "🧹 Cleaning..."
	cargo clean

# Full check: format, lint, typecheck
check: fmt-check lint typecheck
	@echo "✅ All checks passed!"

# Type check without building
typecheck:
	@echo "🔎 Type checking..."
	cargo check --all-targets

# Run clippy linter with strict settings
lint:
	@echo "📎 Running Clippy..."
	cargo clippy --all-targets --all-features -- \
		-D warnings \
		-D clippy::all \
		-D clippy::pedantic \
		-A clippy::module_name_repetitions \
		-A clippy::must_use_candidate \
		-A clippy::missing_errors_doc \
		-A clippy::missing_panics_doc \
		-A clippy::too_many_lines \
		-A clippy::cast_possible_truncation \
		-A clippy::cast_precision_loss \
		-A clippy::cast_sign_loss \
		-A clippy::similar_names

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all

# Check formatting without modifying
fmt-check:
	@echo "🎨 Checking format..."
	cargo fmt --all -- --check

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test --all-features

# Run tests with output
test-verbose:
	@echo "🧪 Running tests (verbose)..."
	cargo test --all-features -- --nocapture

# Generate documentation
doc:
	@echo "📚 Generating docs..."
	cargo doc --no-deps --open

# Install to cargo bin
install:
	@echo "📥 Installing Oracle..."
	cargo install --path .

# Uninstall from cargo bin
uninstall:
	@echo "🗑️ Uninstalling Oracle..."
	cargo uninstall oracle

# Watch for changes and rebuild
watch:
	@echo "👁️ Watching for changes..."
	cargo watch -x build

# Watch and run tests
watch-test:
	@echo "👁️ Watching tests..."
	cargo watch -x test

# Analyze binary size
size:
	@echo "📊 Binary size analysis..."
	@if [ -f target/release/oracle ]; then \
		ls -lh target/release/oracle; \
		echo "---"; \
		size target/release/oracle 2>/dev/null || true; \
	else \
		echo "Build release first: make release"; \
	fi

# Check for outdated dependencies
outdated:
	@echo "📦 Checking for outdated dependencies..."
	cargo outdated || echo "Install with: cargo install cargo-outdated"

# Audit dependencies for security issues
audit:
	@echo "🔒 Auditing dependencies..."
	cargo audit || echo "Install with: cargo install cargo-audit"

# Full CI pipeline
ci: fmt-check lint typecheck test
	@echo "✅ CI checks passed!"

# Development setup
dev-setup:
	@echo "🛠️ Setting up development environment..."
	rustup component add clippy rustfmt
	@echo "Installing optional tools (may fail if not available)..."
	-cargo install cargo-watch
	-cargo install cargo-outdated
	-cargo install cargo-audit
	@echo "✅ Development environment ready!"

# Show help
help:
	@echo "Oracle - Rust Code Inspector"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Build targets:"
	@echo "  build        Build debug version"
	@echo "  release      Build optimized release"
	@echo "  clean        Remove build artifacts"
	@echo ""
	@echo "Run targets:"
	@echo "  run          Run debug version"
	@echo "  run-release  Run release version"
	@echo "  run-project  Run on specific project (PROJECT=/path)"
	@echo ""
	@echo "Quality targets:"
	@echo "  check        Run all checks (fmt, lint, typecheck)"
	@echo "  lint         Run Clippy linter"
	@echo "  fmt          Format code"
	@echo "  fmt-check    Check formatting"
	@echo "  typecheck    Type check without building"
	@echo "  test         Run tests"
	@echo ""
	@echo "Other targets:"
	@echo "  doc          Generate and open documentation"
	@echo "  install      Install to ~/.cargo/bin"
	@echo "  watch        Watch and rebuild on changes"
	@echo "  size         Show binary size info"
	@echo "  outdated     Check for outdated deps"
	@echo "  audit        Security audit dependencies"
	@echo "  ci           Run full CI pipeline"
	@echo "  dev-setup    Install dev tools"
