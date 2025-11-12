# ssc-tui

A Terminal User Interface (TUI) in Rust to query and analyze NASA Satellite Situation Center (SSC) data.

## Features

- **Interactive Satellite Browser**: Browse 300+ satellites with real-time data from NASA SSC
- **Auto-scrolling List**: Smoothly scroll through satellites as you navigate
- **Multiple Sort Options**: Sort alphabetically (A-Z, Z-A) or by mission start date
- **VIM-style Navigation**: Use arrow keys or VIM keys (j/k) for navigation
- **Detailed Satellite Information**: View comprehensive satellite details in an overlay
- **Docker Support**: Containerized development environment

## Quick Start

```bash
# Clone the repository
git clone https://github.com/Sheepybloke2-0/ssc-tui.git
cd ssc-tui

# Run with Docker (recommended)
docker compose run -it --rm dev cargo run

# Or run locally (requires Rust 1.83+)
cargo run
```

## Usage

### Navigation

- **↑↓ / j/k** - Navigate through satellite list (auto-scrolls with selection)
- **Enter** - View detailed satellite information
- **s** - Cycle through sort orders (Name A-Z, Z-A, Start Date)
- **Esc / q** - Close detail overlay
- **h** - Show help message
- **q** - Quit application
- **r** - Refresh data

## Development

See [CLAUDE.md](CLAUDE.md) for detailed development instructions, git workflow, and project architecture.
