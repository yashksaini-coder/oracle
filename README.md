# 🔮 Oracle

**A blazing-fast Rust code inspector for the terminal**

<div align="center">

[![CI Badge]][CI] [![License Badge]][License] [![Rust Badge]][Rust]

[Installation](#installation) · [Usage](#usage) · [Keyboard Shortcuts](#keyboard-shortcuts) · [Contributing](#contributing)

</div>

---

Oracle is a terminal-based application for exploring Rust codebases. It parses your Rust source files and provides an interactive interface to browse functions, structs, enums, traits, and more — all without leaving your terminal.

Built with [Ratatui](https://ratatui.rs) for a smooth, responsive TUI experience.

[![Built With Ratatui](https://img.shields.io/badge/Built_With_Ratatui-000?logo=ratatui&logoColor=fff)](https://ratatui.rs/)

## ✨ Features

- **📦 Code Analysis** — Parses Rust source files using `syn`:
  - Functions (parameters, return types, async/const/unsafe)
  - Structs (fields, derives, generics)
  - Enums (variants with all field types)
  - Traits (methods, associated types, supertraits)
  - Impl blocks (inherent and trait implementations)
  - Modules, Type aliases, Constants, Statics

- **🔍 Smart Search** — Fuzzy matching with real-time filtering
- **📋 Dependency Analysis** — Visualize `Cargo.toml` dependencies
- **🎨 Multiple Themes** — Default Dark, Nord, Catppuccin Mocha, Dracula
- **⚡ Smooth Animations** — Selection highlights, tab transitions
- **⌨️ Vim-style Navigation** — `j/k` for movement, `/` for search

## 📦 Installation

### From crates.io (recommended)

With Rust and Cargo installed:

```bash
cargo install oracle
```

### From source

```bash
git clone https://github.com/yashksaini-coder/oracle.git
cd oracle
cargo install --path .
# or: make install
```

### Pre-built binaries

See [Releases](https://github.com/yashksaini-coder/oracle/releases) for Linux (x86_64), macOS (x86_64, Apple Silicon), and Windows (x86_64) binaries.

## 🚀 Usage

```bash
# Analyze current directory
oracle

# Analyze specific project
oracle /path/to/rust/project

# Analyze a single file
oracle /path/to/file.rs
```

## ⌨️ Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between panels |
| `↑/↓` or `j/k` | Navigate list / Scroll inspector |
| `Enter` / `→` / `l` | View item details |
| `←` / `h` | Back to list |
| `/` | Focus search |
| `1-4` | Switch tabs (Types/Functions/Modules/Dependencies) |
| `g` / `G` | Jump to first/last item |
| `PgUp` / `PgDn` | Page navigation |
| `?` | Show help |
| `q` / `Esc` | Quit |

## 🛠️ Development

```bash
# Install development tools
make dev-setup

# Full check (format, lint, typecheck)
make check

# Run linter
make lint

# Run tests
make test

# Build release
make release

# See all commands
make help
```

## 📤 Releasing (maintainers)

- **Publish to crates.io** (after `cargo login`): `make publish-dry-run` then `make publish`.
- **GitHub Release**: Push a version tag (e.g. `v0.1.0`). The [release workflow](.github/workflows/release.yml) builds binaries for Linux, macOS (Intel + Apple Silicon), and Windows and creates a release. Optionally set `CARGO_REGISTRY_TOKEN` in repo secrets to auto-publish to crates.io on tag push.

## 🏗️ Architecture

```
src/
├── main.rs           # Entry point, event loop
├── lib.rs            # Library exports
├── app/              # Application state
├── analyzer/         # Rust code parsing (syn-based)
│   ├── parser.rs     # Source file analyzer
│   ├── dependency.rs # Cargo.toml analyzer
│   └── types.rs      # Type definitions
├── config/           # Settings and configuration
├── error/            # Error types
├── ui/               # TUI components
│   ├── animation.rs  # Smooth transitions
│   ├── app.rs        # Main widget
│   ├── inspector.rs  # Detail panel
│   ├── search.rs     # Search bar + completion
│   ├── theme/        # Color themes
│   └── components/   # Reusable widgets
└── utils/            # Helpers and utilities
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feat/amazing-feature`)
3. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/)
4. Push to the branch (`git push origin feat/amazing-feature`)
5. Open a Pull Request

### Commit Convention

We use [Conventional Commits](https://www.conventionalcommits.org/). Examples:

- `feat: add new search feature`
- `fix: correct parsing error`
- `docs: update README`
- `refactor!: change API structure` (breaking change)

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

<div align="center">

*Reverse-engineered from the Python [shira](shira/) TUI explorer for the Rust ecosystem.*

</div>

<!-- Badges -->
[CI Badge]: https://img.shields.io/github/actions/workflow/status/yashksaini-coder/oracle/ci.yml?style=flat-square&logo=github&label=CI
[CI]: https://github.com/yashksaini-coder/oracle/actions/workflows/ci.yml
[License Badge]: https://img.shields.io/badge/license-MIT-blue?style=flat-square
[License]: ./LICENSE
[Rust Badge]: https://img.shields.io/badge/rust-1.75+-orange?style=flat-square&logo=rust
