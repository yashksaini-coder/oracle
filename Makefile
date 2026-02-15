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

clippy:
	@echo "🧹 Running strict clippy (pedantic, etc)..."
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
		-A clippy::similar_names \
		-A clippy::needless_raw_string_hashes \
		-A clippy::unreadable_literal \
		-A clippy::doc_markdown \
		-A clippy::redundant_closure_for_method_calls \
		-A clippy::unused_self \
		-A clippy::match_same_arms \
		-A clippy::wildcard_imports \
		-A clippy::return_self_not_must_use \
		-A clippy::needless_pass_by_value \
		-A clippy::ref_option \
		-A clippy::doc_link_with_quotes \
		-A clippy::case_sensitive_file_extension_comparisons \
		-A clippy::option_if_let_else \
		-A clippy::single_match \
		-A clippy::struct_field_names \
		-A clippy::needless_lifetimes \
		-A clippy::map_unwrap_or \
		-A clippy::match_wild_err_arm \
		-A clippy::if_same_then_else \
		-A clippy::range_plus_one \
		-A clippy::branches_sharing_code \
		-A clippy::manual_let_else \
		-A clippy::uninlined_format_args \
		-A clippy::stable_sort_primitive \
		-A clippy::struct_excessive_bools \
		-A clippy::match_wildcard_for_single_variants \
		-A clippy::elidable_lifetime_names \
		-A clippy::comparison_chain \
		-A clippy::if_not_else

clippy-beta:
	@echo "🧹 Running strict clippy (pedantic, etc) on beta toolchain..."
	cargo +beta clippy --all-targets --all-features -- \
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
		-A clippy::similar_names \
		-A clippy::needless_raw_string_hashes \
		-A clippy::unreadable_literal \
		-A clippy::doc_markdown \
		-A clippy::redundant_closure_for_method_calls \
		-A clippy::unused_self \
		-A clippy::match_same_arms \
		-A clippy::wildcard_imports \
		-A clippy::return_self_not_must_use \
		-A clippy::needless_pass_by_value \
		-A clippy::ref_option \
		-A clippy::doc_link_with_quotes \
		-A clippy::case_sensitive_file_extension_comparisons \
		-A clippy::option_if_let_else \
		-A clippy::single_match \
		-A clippy::struct_field_names \
		-A clippy::needless_lifetimes \
		-A clippy::map_unwrap_or \
		-A clippy::match_wild_err_arm \
		-A clippy::if_same_then_else \
		-A clippy::range_plus_one \
		-A clippy::branches_sharing_code \
		-A clippy::manual_let_else \
		-A clippy::uninlined_format_args \
		-A clippy::stable_sort_primitive \
		-A clippy::struct_excessive_bools \
		-A clippy::match_wildcard_for_single_variants \
		-A clippy::elidable_lifetime_names \
		-A clippy::comparison_chain \
		-A clippy::if_not_else

dev-setup:
	@echo "⚙️  Setting up development environment (installing Rust toolchain, components)..."
	rustup component add clippy rustfmt

machete:
	@echo "🔎 Checking for unused dependencies (using cargo-machete)..."
	cargo machete --with-metadata

help:
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
	@echo "  clippy       Run strict clippy (pedantic, etc)"
	@echo "  clippy-beta  Run strict clippy (pedantic, etc) on beta toolchain"
	@echo "  machete      Check for unused dependencies (cargo machete)"
	@echo "  help         Show this help message"
