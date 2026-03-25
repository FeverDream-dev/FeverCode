# CONTRIBUTING

Thank you for your interest in contributing to Fever Code!

## Code of Conduct

This project adheres to a code of conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to [contact@fevercode.org](mailto:contact@fevercode.org).

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check the existing issues for similar problems. When creating a bug report, include:

- Clear description of the problem
- Steps to reproduce
- Expected behavior
- Actual behavior
- Environment (OS, Rust version, Fever Code version)
- Relevant logs or error messages

### Suggesting Enhancements

Enhancement suggestions are welcome. When suggesting an enhancement:

- Use a clear and descriptive title
- Provide a detailed description
- Explain why this enhancement would be useful
- Provide examples if applicable

### Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Run formatter (`cargo fmt`)
6. Run linter (`cargo clippy -- -D warnings`)
7. Commit your changes (`git commit -m 'Add amazing feature'`)
8. Push to the branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

## Development Setup

```bash
# Clone repository
git clone https://github.com/FeverDream-dev/FeverCode.git
cd FeverCode

# Install dependencies
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- fever code
```

## Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Run `cargo clippy` before committing
- Keep functions focused and small
- Add doc comments for public APIs
- Write tests for new functionality

## Project Structure

```
fevercode/
├── crates/
│   ├── fever-core/       # Core orchestration
│   ├── fever-agent/     # Agent and roles
│   ├── fever-providers/ # LLM providers
│   ├── fever-tools/     # System tools
│   ├── fever-search/    # Search functionality
│   ├── fever-browser/   # Chrome MCP
│   ├── fever-tui/       # Terminal UI
│   ├── fever-config/    # Configuration
│   ├── fever-cli/       # CLI entry point
│   └── fever-release/   # Release management
├── docs/                 # Documentation
├── examples/             # Examples
└── install.sh            # Installer script
```

## Testing

- Write unit tests for new functions
- Write integration tests for new features
- Ensure all tests pass before submitting PR
- Aim for good test coverage

## Documentation

- Update README.md for user-facing changes
- Update ARCHITECTURE.md for structural changes
- Add doc comments to public APIs
- Document new configuration options

## Release Process

Releases are managed by maintainers:

1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Create git tag
4. Build release artifacts
5. Create GitHub release
6. Publish to crates.io (if applicable)

## Questions?

Feel free to open an issue with your question, or contact the maintainers directly.

Thank you for contributing!
