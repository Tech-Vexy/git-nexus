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

### 🛠️ Developer Experience
- **Zero-Config Defaults**: Works out of the box in your current directory (`.`), or specify a custom root path
- **Single Binary**: No Python environments or Node dependencies to manage—just one binary

## Installation

### From Source
```bash
git clone https://github.com/yourusername/git-nexus.git
cd git-nexus
cargo build --release
# Binary will be at ./target/release/git-nexus
```

Copy to your PATH:
```bash
sudo cp target/release/git-nexus /usr/local/bin/
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

## Options

```
Usage: git-nexus [OPTIONS] [PATH]

Arguments:
  [PATH]  Root directory to scan for repositories [default: .]

Options:
  -d, --depth <DEPTH>    Maximum directory traversal depth [default: 3]
  -j, --json             Output in JSON format
  -v, --verbose          Show verbose information (last commit, stash count, file counts)
  -f, --filter <FILTER>  Filter repositories by status [possible values: clean, dirty, ahead, behind]
  -s, --sort <SORT>      Sort repositories by field [default: path] [possible values: path, status, branch]
  -h, --help             Print help
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
