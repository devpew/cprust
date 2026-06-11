# Contributing to cprust

Thank you for your interest in contributing!

## Development Setup

```bash
# Clone the repository
git clone https://github.com/devpew/cprust.git
cd cprust

# Build
cargo build

# Run tests
cargo test

# Run linter
cargo clippy -- -D warnings

# Check formatting
cargo fmt --check

# Run benchmarks
cargo bench
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Ensure all tests pass: `cargo test`
5. Ensure clippy passes: `cargo clippy -- -D warnings`
6. Format code: `cargo fmt`
7. Commit with a descriptive message
8. Push and open a Pull Request

## Coding Standards

- Follow `cargo fmt` formatting
- All code must pass `cargo clippy -- -D warnings`
- Write tests for new features
- Use descriptive commit messages
- Update documentation for user-facing changes

## Reporting Issues

- Use the GitHub issue tracker
- Include steps to reproduce
- Include your OS and Rust version
- Check existing issues before creating a new one

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
