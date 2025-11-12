# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ssc-tui` is a Terminal User Interface (TUI) application written in Rust for querying and analyzing NASA SSC (Satellite Situation Center) data.

## Development Commands

### Docker Development Environment

Start and enter the development container:
```bash
docker compose up -d     # Start container in background
docker compose exec dev bash  # Enter the container
```

Or run directly:
```bash
docker compose run -it --rm dev  # Start container and get shell
```

Run the TUI application:
```bash
docker compose run -it --rm dev cargo run  # Run the TUI in Docker
```

Stop the container:
```bash
docker compose down      # Stop and remove containers
```

Rebuild after Dockerfile changes:
```bash
docker compose build     # Rebuild the container
docker compose up -d --build  # Rebuild and start
```

### Building (inside container or local)
```bash
cargo build              # Build the project
cargo build --release    # Build optimized release version
```

### Running (inside container or local)
```bash
cargo run                # Run the application
cargo run --release      # Run optimized version
```

### Testing (inside container or local)
```bash
cargo test               # Run all tests
cargo test <test_name>   # Run a specific test
cargo test -- --nocapture # Run tests with output visible
```

### Code Quality (inside container or local)
```bash
cargo fmt                # Format code
cargo clippy             # Run linter
cargo check              # Quick compile check without producing binary
```

## Git Workflow

This project follows a feature branch workflow:

### Creating a New Feature

```bash
# Create and switch to a new feature branch
git checkout -b feat/your-feature-name

# Make your changes, then stage them
git add -A

# Commit with a descriptive message
git commit -m "feat: description of your feature

Detailed explanation of what was implemented and why.

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

# Push the feature branch to GitHub
git push -u origin feat/your-feature-name
```

### Branch Naming Convention

- `feat/` - New features (e.g., `feat/nasa-ssc-tui-implementation`)
- `fix/` - Bug fixes (e.g., `fix/api-timeout-handling`)
- `docs/` - Documentation changes (e.g., `docs/update-readme`)
- `refactor/` - Code refactoring (e.g., `refactor/ssc-client-structure`)
- `test/` - Test additions or updates (e.g., `test/api-integration`)

### Commit Message Format

Follow conventional commits:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

**Example:**
```
feat: add satellite trajectory visualization

Implement orbital trajectory plotting for selected satellites using
the NASA SSC location API.

- Add trajectory data fetching from /locations endpoint
- Implement 3D coordinate transformation
- Add visualization rendering in Analysis tab

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### Creating Pull Requests

After pushing your feature branch, create a pull request on GitHub:

```bash
# The push command will output a URL like:
# https://github.com/Sheepybloke2-0/ssc-tui/pull/new/feat/your-feature-name

# Visit that URL or go to GitHub and create the PR manually
```

### Checking Status

```bash
git status                          # Check current changes
git diff                            # View unstaged changes
git diff --staged                   # View staged changes
git log --oneline -5                # View recent commits
git branch                          # List local branches
git branch -r                       # List remote branches
```

## Project Structure

The project is organized into modules:

- `src/main.rs` - Application entry point, terminal setup/cleanup, event loop, and App state
- `src/ui.rs` - UI rendering logic using ratatui (tabs, panels, widgets)
- `src/ssc.rs` - NASA SSC API client and data structures

### Architecture

**App State (`main.rs`):**
- `App` struct holds all application state (current tab, query results, status messages)
- Event loop polls for keyboard input and re-renders UI
- Terminal setup/teardown handled with crossterm

**UI Rendering (`ui.rs`):**
- Tab-based interface: Query, Satellites, Analysis, Help
- Each tab has its own rendering function
- Layout uses vertical split: header (tabs) → content → status bar

**SSC Client (`ssc.rs`):**
- `SscClient` for NASA SSC API interactions using XML schema
- Environment-based configuration via `.env` file (API_KEY optional)
- In-memory HashMap cache for fast satellite lookups
- Data structures: `Observatory`, `SatellitePosition`, `LocationQuery`, `ConjunctionEvent`
- Implemented functions:
  - `get_satellites()` - Fetches 300+ satellites from NASA SSC API (cached)
  - `get_satellite_info(id)` - Quick lookup by satellite ID
  - `search_satellites(query)` - Search by name (partial match)
  - `cached_count()` - Get number of cached satellites
- Functions to implement:
  - `query_locations()` - Get satellite positions for time range
  - `get_trajectory()` - Get orbital trajectory data
  - `find_conjunctions()` - Find close approaches between satellites
  - `get_ground_track()` - Calculate ground coverage

### NASA SSC API

**Base URL:** `https://sscweb.gsfc.nasa.gov/WS/sscr/2`

**Get Observatories Endpoint:** `/observatories`
- Returns XML list of 300+ satellites
- Fields: ID, Name, Resolution (seconds), StartTime, EndTime, ResourceId
- Response format: XML (ObservatoryResponse schema)
- See `docs/NASA_SSC_XML_Schema.md` for complete API documentation

**Environment Configuration:**
- Create `.env` file with `API_KEY=<your_key>` (optional)
- See `.env.example` for template

The API provides spacecraft orbit data in various coordinate systems (GEO, GSE, GSM, etc.).
