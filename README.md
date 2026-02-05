# git-nexus 🚀

A blazing fast multi-repository scanner for developers who juggle multiple projects. Get a god-mode view of your development workspace without the lag of checking repositories one by one.

## Features

### 🚀 Blazing Fast Workspace Scanning
- **Rust-Powered Engine**: Built on standard `std::fs` and `walkdir` for millisecond-latency scans, even across thousands of directories
- **Parallel Processing**: Uses Rayon for concurrent repository analysis
- **Smart Filtering**: Automatically detects and ignores dependency black holes like `node_modules`, `target`, `venv`, and `.build` to keep scans instant
- **Configurable Depth**: Control how deep the rabbit hole goes with customizable directory traversal limits

### 🚦 Live Status Telemetry
- **Instant Health Check**: Immediately see if a repo is `CLEAN` or `DIRTY` (uncommitted changes)
- **Branch Display**: Shows current branch name with special handling for detached HEAD and unborn branches
- **Divergence Tracking**: Visual indicators show if you are Ahead (↑) or Behind (↓) your remote branch
- **Color-Coded Output**: Scannable terminal UI—Green for safe, Red for attention, Yellow for sync required

### 📊 Detailed Information (Verbose Mode)
- **Last Commit Info**: See commit hash, author, timestamp, and message
- **Stash Count**: Shows how many stashes you have (`📦`)
- **File Counts**: 
  - `~N` - Modified/staged files (yellow)
  - `+N` - Untracked files (cyan)

### 🎯 Flexible Filtering & Sorting
- **Filter by Status**: Show only clean, dirty, ahead, or behind repositories
- **Sort Options**: Sort by path, status, or branch name
- **JSON Output**: Machine-readable output for scripting and automation

### 🎨 Interactive & Advanced Modes
- **TUI Mode**: Beautiful terminal UI with keyboard navigation
- **Watch Mode**: Real-time monitoring with automatic rescanning
- **Configuration Files**: Customize behavior with `.git-nexus.toml`
- **Export**: Generate HTML reports or CSV files
- **Git Hooks Detection**: See which repos have active hooks
- **GitHub Integration**: Display issues/PRs (coming soon)

### 🛠️ Developer Experience
- **Zero-Config Defaults**: Works out of the box in your current directory (`.`), or specify a custom root path
- **Single Binary**: No Python environments or Node dependencies to manage—just one binary

## Installation

### Quick Install (Recommended)

**Using the install script:**
```bash
git clone https://github.com/yourusername/git-nexus.git
cd git-nexus
./install.sh
```

**Using Make:**
```bash
git clone https://github.com/yourusername/git-nexus.git
cd git-nexus
make install
```

This will:
1. Build the project in release mode
2. Install the binary to `~/.local/bin/git-nexus`
3. Make it executable

**Custom Install Location:**
```bash
# Using install script
INSTALL_DIR=/usr/local/bin ./install.sh

# Using Make
make install INSTALL_DIR=/usr/local/bin
```

### From Source (Manual)

```bash
git clone https://github.com/yourusername/git-nexus.git
cd git-nexus
cargo build --release
sudo cp target/release/git-nexus /usr/local/bin/
```

### Uninstall

**Using the uninstall script:**
```bash
./uninstall.sh
```

**Using Make:**
```bash
make uninstall
```

**Manual:**
```bash
rm ~/.local/bin/git-nexus
# or
sudo rm /usr/local/bin/git-nexus
```

### Verify Installation

```bash
git-nexus --help
```

## Development

### Build
```bash
make build
# or
cargo build --release
```

### Run
```bash
make run          # Run in current directory
make run-verbose  # Run with verbose output
make run-json     # Run with JSON output
```

### Testing
```bash
make test
# or
cargo test
```

### Other Commands
```bash
make help    # Show all available commands
make check   # Check for errors
make fmt     # Format code
make clippy  # Run lints
make clean   # Clean build artifacts
```

## Usage

### Basic Usage
```bash
# Scan current directory
git-nexus

# Scan specific directory
git-nexus ~/projects

# Scan with custom depth
git-nexus ~/projects -d 5
```

### Verbose Output
```bash
# Show detailed information including last commit, stash count, and file counts
git-nexus -v
```

Output example:
```
📁 ./my-project (main) [DIRTY] ↑2 📦1 ~3 +2
   └─ a1b2c3d · John Doe · Added new feature
```

### Filtering
```bash
# Show only dirty repositories
git-nexus --filter dirty

# Show only repositories that are ahead of remote
git-nexus --filter ahead

# Show only clean repositories
git-nexus --filter clean
```

### Sorting
```bash
# Sort by path (default)
git-nexus --sort path

# Sort by status (clean/dirty)
git-nexus --sort status

# Sort by branch name
git-nexus --sort branch
```

### JSON Output
```bash
# Output as JSON
git-nexus --json

# JSON with verbose details
git-nexus --json -v
```

Example JSON output:
```json
[
  {
    "path": "./my-project",
    "is_clean": false,
    "ahead": 2,
    "behind": 0,
    "branch": "main",
    "stash_count": 1,
    "modified_count": 3,
    "untracked_count": 2,
    "last_commit": {
      "message": "Added new feature",
      "author": "John Doe",
      "timestamp": "2026-02-05 12:30:45",
      "hash": "a1b2c3d"
    }
  }
]
```

## Advanced Features

### 🎨 Interactive TUI Mode
Navigate your repositories with a beautiful terminal user interface.

```bash
git-nexus tui
```

**Controls:**
- `↑/k` - Move up
- `↓/j` - Move down  
- `Home/End` - Jump to first/last
- `q/Esc` - Quit

### 👁️ Watch Mode
Continuously monitor your workspace for git changes in real-time.

```bash
git-nexus watch
```

Watch mode will automatically rescan when it detects changes in any `.git` directory. Press `Ctrl+C` to exit.

### 📊 Export to HTML/CSV
Generate beautiful reports of your repository status.

**HTML Export:**
```bash
git-nexus export html -o report.html
```

Creates a styled HTML page with:
- Repository statistics dashboard
- Color-coded status indicators
- Last commit information
- Fully responsive design

**CSV Export:**
```bash
git-nexus export csv -o report.csv
```

Perfect for importing into spreadsheets or data analysis tools.

### ⚙️ Configuration File
Customize git-nexus behavior with a `.git-nexus.toml` file.

**Generate example config:**
```bash
git-nexus config
```

**Example configuration:**
```toml
scan_depth = 3
ignore_dirs = ["node_modules", "target", "venv", ".build"]

[display]
show_branch = true
show_colors = true
default_verbose = false

[github]
token = "your_github_token_here"
check_issues = true
check_prs = true
```

**Config file locations** (checked in order):
1. `./.git-nexus.toml` (current directory)
2. `~/.config/git-nexus/config.toml`
3. `~/.git-nexus.toml`

### 🪝 Git Hooks Detection
See which repositories have active git hooks.

```bash
git-nexus --show-hooks -v
```

Detects hooks like: `pre-commit`, `pre-push`, `post-commit`, `post-merge`, `commit-msg`, `prepare-commit-msg`

## Options

```
Usage: git-nexus [OPTIONS] [PATH] [COMMAND]

Commands:
  tui     Interactive TUI mode
  watch   Watch mode - continuously monitor for changes
  export  Export to HTML or CSV
  config  Generate example configuration file
  help    Print this message or the help of the given subcommand(s)

Arguments:
  [PATH]  Root directory to scan for repositories [default: .]

Options:
  -d, --depth <DEPTH>    Maximum directory traversal depth
  -j, --json             Output in JSON format
  -v, --verbose          Show verbose information
  -f, --filter <FILTER>  Filter repositories by status [possible values: clean, dirty, ahead, behind]
  -s, --sort <SORT>      Sort repositories by field [default: path] [possible values: path, status, branch]
      --show-hooks       Show git hooks information
      --show-github      Show GitHub info (requires token in config)
  -h, --help             Print help
  -V, --version          Print version
```

## Symbol Legend

- 📁 - Repository
- ✓ - Success/Found
- `[CLEAN]` - No uncommitted changes (green)
- `[DIRTY]` - Has uncommitted changes (red)
- `↑N` - N commits ahead of remote (yellow)
- `↓N` - N commits behind remote (red)
- `📦N` - N stashes (magenta, verbose mode)
- `~N` - N modified/staged files (yellow, verbose mode)
- `+N` - N untracked files (cyan, verbose mode)

## Building from Source

### Prerequisites
- Rust 1.93.0 or later
- Cargo

### Build
```bash
cargo build --release
```

### Run Tests
```bash
cargo test
```

## Performance

git-nexus is designed for speed:
- Parallel repository scanning using Rayon
- Smart directory filtering to skip irrelevant paths
- Minimal git operations per repository
- Zero allocations where possible

Typical performance: Scans 100+ repositories in under 1 second on modern hardware.

## License

MIT License - see LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Credits

Built with:
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [colored](https://github.com/mackwic/colored) - Terminal colors
- [git2](https://github.com/rust-lang/git2-rs) - Git operations
- [walkdir](https://github.com/BurntSushi/walkdir) - Directory traversal
- [rayon](https://github.com/rayon-rs/rayon) - Parallel processing
- [serde](https://github.com/serde-rs/serde) - JSON serialization
- [chrono](https://github.com/chronotope/chrono) - Date/time handling
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [notify](https://github.com/notify-rs/notify) - File system watching
- [csv](https://github.com/BurntSushi/rust-csv) - CSV reading and writing
- [toml](https://github.com/toml-rs/toml) - Configuration file parsing
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client for GitHub API
