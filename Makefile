# Oracle - Rust Code Inspector - Extended Makefile

.PHONY: all build release run clean test lint lint-fix typecheck fmt fmt-fix check dev-setup install publish-dry-run publish help

all: help

build:
	@echo "🔨 Building (debug)..."
	cargo build

release:
	@echo "📦 Building (release)..."
	cargo build --release

install:
	@echo "📥 Installing oracle (from current directory)..."
	cargo install --path .

publish-dry-run:
	@echo "🔍 Dry-run: would publish to crates.io..."
	cargo publish --dry-run

publish:
	@echo "📤 Publishing to crates.io..."
	cargo publish

run:
	@echo "🚀 Running Oracle..."
	cargo run

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

test:
	@echo "🧪 Running tests..."
	cargo test

lint:
	@echo "🧹 Running linter (clippy, check only)..."
	cargo clippy --all-targets --all-features -- -D warnings

lint-fix:
	@echo "🧹 Running linter (clippy, attempt to fix)..."
	cargo clippy --all-targets --all-features --fix --allow-dirty -- -D warnings || echo "Some lints could not be fixed automatically. Please review manually."

typecheck:
	@echo "📝 Type checking..."
	cargo check

fmt:
	@echo "🎨 Checking code format..."
	cargo fmt --all -- --check

fmt-fix:
	@echo "🎨 Fixing code format..."
	cargo fmt --all

check: fmt lint typecheck test

dev-setup:
	@echo "⚙️  Setting up development environment (installing Rust toolchain, components)..."
	rustup component add clippy rustfmt

help:
	@echo "Oracle - Rust Code Inspector (Extended)"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build        Build debug version"
	@echo "  release      Build optimized release"
	@echo "  install      Install binary (cargo install --path .)"
	@echo "  publish-dry-run  Check crate for publish (no upload)"
	@echo "  publish      Publish to crates.io (requires login)"
	@echo "  run          Run Oracle"
	@echo "  clean        Remove build artifacts"
	@echo "  test         Run tests"
	@echo "  lint         Lint with clippy (does not fix)"
	@echo "  lint-fix     Attempt to automatically fix lints (clippy --fix)"
	@echo "  typecheck    Typecheck the code"
	@echo "  fmt          Check code format"
	@echo "  fmt-fix      Fix code format"
	@echo "  check        Format + Lint + Typecheck + Test"
	@echo "  dev-setup    Install required Rust components"
	@echo "  help         Show this help message"
