# Big Picture

A multiplayer party-style game where up to 8 players collaboratively evolve an image toward a hidden target goal through AI-driven transformations.

## 🎮 Game Overview

Players work together to transform a starting image into a goal image through a series of AI-generated modifications. Each player takes turns selecting from four transformation options, building toward the target composition.

## 🏗️ Architecture

- **Language**: Rust only (no JavaScript)
- **Client**: Godot 4.x with Rust bindings (GDExtension)
- **Server**: Axum REST API with tokio async runtime
- **Domain**: Pure Rust business logic

### Project Structure

```
drawme/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── domain/             # Core game logic & types
│   ├── server/             # REST API server
│   └── client/             # Godot GDExtension (Rust)
├── godot/                  # Godot project files (future)
├── spec/                   # Feature specifications (future)
└── tasks/                  # Implementation tasks
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (`rustup`)
- Godot 4.2+ (for client development)

### Build

```bash
# Build all crates
cargo build --all

# Run server
cargo run -p big-picture-server

# Build Godot extension
cargo build -p big-picture-client --release
```

### Run Tests

```bash
cargo test --all
```

## 📋 Development Workflow

Follow the protocols in:
- [1.planning.md](1.planning.md) - Feature specs and acceptance criteria
- [2.context.md](2.context.md) - Architecture and constraints
- [3.execution.md](3.execution.md) - Systematic building protocol
- [4.quality.md](4.quality.md) - Testing and validation
- [5.docs.md](5.docs.md) - Documentation protocol

## 🎯 Current Milestone

**Lobby Implementation**: Welcome screen → room creation → player join/leave → "All is in!" → game start

See [tasks/lobby-implementation.md](tasks/lobby-implementation.md) for detailed task breakdown.

## 📝 License

MIT OR Apache-2.0
