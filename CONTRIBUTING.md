# Contributing to GTS Rust

Thank you for your interest in contributing to the Global Type System (GTS) Rust implementation! This document provides guidelines and information for contributors.

## Quick Start

### Prerequisites

- **Git** for version control
- **Rust 1.70+** for building and running the implementation

### Development Setup

```bash
# Clone the repository
git clone https://github.com/globaltypesystem/gts-rust
cd gts-rust

# Build the project
cargo build

# Run tests
cargo test

# Running Code Coverage
# To measure test coverage, ensure you have [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) installed:

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --lib
```

### Repository Layout

```
gts-rust/
├── README.md                 # Main project documentation
├── CONTRIBUTING.md           # This file
├── LICENSE                   # License information
├── Cargo.toml                # Workspace configuration
├── gts/                      # Core GTS library
│   ├── src/                  # Library source code
│   └── tests/                # Integration tests
├── gts-cli/                  # Command-line interface
│   └── src/                  # CLI source code
└── examples/                 # Usage examples
```

## Development Workflow

### 1. Create a Feature Branch or fork the repository

```bash
git checkout -b feature/your-feature-name
```

Use descriptive branch names:
- `feature/add-query-filters`
- `fix/uuid-generation-edge-case`
- `docs/update-readme-examples`
- `test/wildcard-matching`

### 2. Make Your Changes

Follow the code style and patterns described below.

### 3. Validate Your Changes

```bash
# Run all tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

### 4. Commit Changes

Follow a structured commit message format:

```text
<type>(<module>): <description>
```

- `<type>`: change category (see table below)
- `<module>` (optional): the area touched (e.g., core, cli, parser)
- `<description>`: concise, imperative summary

Accepted commit types:

| Type       | Meaning                                                     |
|------------|-------------------------------------------------------------|
| feat       | New feature                                                 |
| fix        | Bug fixes                                                   |
| docs       | Documentation updates                                       |
| test       | Adding or modifying tests                                   |
| style      | Formatting changes (rustfmt, whitespace, etc.)              |
| refactor   | Code changes that neither fix bugs nor add features         |
| perf       | Performance improvements                                    |
| chore      | Misc tasks (tooling, dependencies)                          |
| breaking   | Backward incompatible changes                               |

Best practices:

- Keep the title concise (ideally <50 chars)
- Use imperative mood (e.g., "Fix bug", not "Fixed bug")
- Make commits atomic (one logical change per commit)
- Add details in the body when necessary (what/why, not how)
- For breaking changes, either use `breaking!:` or include a `BREAKING CHANGE:` footer

Examples:

```
feat(parser): Add support for wildcard queries
fix(core): Resolve UUID generation edge case
docs: Update README with CLI examples
test(query): Add tests for filter syntax
```

## Code Style

### Formatting

- Follow Rust standard formatting: `cargo fmt`
- Use 4 spaces for indentation
- Keep lines under 100 characters when reasonable

### Linting

- Run clippy and address all warnings: `cargo clippy`
- Fix all clippy warnings before submitting a PR

### Testing

- Add unit tests in the same file as the code (using `#[cfg(test)]` modules)
- Add integration tests in the `tests/` directory
- Ensure all tests pass before submitting a PR
- Write tests for new functionality
- Aim for high code coverage

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Run clippy: `cargo clippy`
7. Commit with descriptive messages (see commit message guidelines above)
8. Push to your fork
9. Open a Pull Request with a clear description

### PR Description Template

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
Describe the tests you ran and how to reproduce them

## Checklist
- [ ] Tests pass (`cargo test`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation is updated
```

## Feature Parity

This implementation maintains 100% feature parity with the Python reference implementation. When adding features:

1. Check the Python implementation first
2. Ensure behavior matches exactly
3. Update tests to verify compatibility
4. Document any intentional differences (e.g., Rust-specific optimizations)

## Questions?

Open an issue or discussion on GitHub.

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 License.
