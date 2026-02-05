# Contributing to git-nexus

Thank you for your interest in contributing to git-nexus! We welcome contributions of all kinds.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork**:
   ```bash
   git clone https://github.com/yourusername/git-nexus.git
   cd git-nexus
   ```
3. **Build the project**:
   ```bash
   make build
   # or
   cargo build
   ```

## Development Workflow

### Making Changes

1. **Create a new branch**:
   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

2. **Make your changes** and test them:
   ```bash
   make dev          # Quick test run
   make test         # Run test suite
   make check        # Check for errors
   ```

3. **Format your code**:
   ```bash
   make fmt
   ```

4. **Run lints**:
   ```bash
   make clippy
   ```

5. **Commit your changes**:
   ```bash
   git add .
   git commit -m "Brief description of your changes"
   ```

### Code Style

- Follow Rust standard formatting (enforced by `cargo fmt`)
- Run `cargo clippy` and fix any warnings
- Write clear, descriptive commit messages
- Add comments for complex logic

### Testing

Before submitting a pull request:

```bash
make test      # Run all tests
make clippy    # Ensure no lint warnings
make fmt       # Format code
cargo build --release  # Ensure it builds
```

Test your changes with real repositories:
```bash
./target/release/git-nexus ~/projects -v
./target/release/git-nexus . --json
./target/release/git-nexus . --filter dirty
```

## Pull Request Process

1. **Update documentation** if you're adding new features
2. **Add tests** for new functionality
3. **Ensure all tests pass**
4. **Update README.md** if needed
5. **Create a Pull Request** with:
   - Clear description of changes
   - Why the change is needed
   - How it was tested
   - Any breaking changes

## Project Structure

```
git-nexus/
├── src/
│   └── main.rs           # Main application code
├── Cargo.toml            # Rust dependencies
├── Makefile              # Build automation
├── install.sh            # Installation script
├── uninstall.sh          # Uninstallation script
└── README.md             # Documentation
```

## Feature Ideas

Looking for something to work on? Consider:

- [ ] Configuration file support (.git-nexus.toml)
- [ ] Interactive mode with TUI
- [ ] Git hooks detection and display
- [ ] Performance benchmarking suite
- [ ] Support for git worktrees
- [ ] Remote tracking branch comparison
- [ ] Integration with GitHub/GitLab APIs
- [ ] Watch mode for continuous monitoring
- [ ] Export to different formats (CSV, HTML)
- [ ] Plugin/extension system

## Reporting Bugs

When reporting bugs, please include:

1. **Version**: Run `git-nexus --help` to see version info
2. **OS and environment**: Linux, macOS, Windows, etc.
3. **Steps to reproduce**: Clear steps to trigger the bug
4. **Expected behavior**: What you expected to happen
5. **Actual behavior**: What actually happened
6. **Logs/output**: Any error messages or relevant output

## Questions?

Feel free to open an issue for:
- Feature requests
- Bug reports
- Documentation improvements
- General questions

## License

By contributing, you agree that your contributions will be licensed under the same MIT License that covers the project.
